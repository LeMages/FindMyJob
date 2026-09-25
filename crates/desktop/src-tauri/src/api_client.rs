//! Client HTTP vers l'API REST de l'agent (`findmyjob-agent`), qui remplace
//! l'accès direct à la base de données depuis le desktop.
//!
//! Distingue les réponses JSON (`get`, `post`, `put`) des réponses vides en
//! 200 (`post_no_content`, `put_no_content`, `delete`) : tenter de
//! désérialiser un corps vide en JSON échoue toujours côté serde, donc les
//! endpoints qui renvoient `()` ne doivent jamais passer par `resp.json()`.

// Importation des crates externes
use reqwest::{Client, Method, StatusCode};
use serde::{de::DeserializeOwned, Serialize};

// Importation des modules internes
use crate::error::AppError;

/// Client HTTP partagé, injecté comme état Tauri à la place de l'ancien pool
/// SQLite.
///
/// # Attributes
///
/// * `client` - Client `reqwest` réutilisé pour toutes les requêtes.
/// * `base_url` - URL de base de l'agent (ex. `http://localhost:3000`), sans `/` final.
/// * `token` - Jeton `Bearer` envoyé si l'agent exige une authentification.
#[derive(Clone)]
pub struct ApiClient {
    client: Client,
    base_url: String,
    token: Option<String>,
}

impl ApiClient {
    /// Construit un client pointant vers l'agent à `base_url`.
    ///
    /// # Arguments
    ///
    /// * `base_url` - URL de base de l'agent.
    /// * `token` - Jeton `Bearer` optionnel (absent si `API_TOKEN` n'est pas configuré).
    ///
    /// # Returns
    ///
    /// Un `ApiClient` prêt à l'emploi.
    pub fn new(base_url: &str, token: Option<String>) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
            token,
        }
    }

    /// Construit une requête vers `path`, avec le jeton `Bearer` si configuré.
    fn request(&self, method: Method, path: &str) -> reqwest::RequestBuilder {
        let url = format!("{}{}", self.base_url, path);
        let mut req = self.client.request(method, url);
        if let Some(token) = &self.token {
            req = req.bearer_auth(token);
        }
        req
    }

    /// `GET` désérialisant la réponse en JSON.
    pub async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T, AppError> {
        let resp = self
            .request(Method::GET, path)
            .send()
            .await
            .map_err(Self::network_error)?;
        self.handle_json(resp).await
    }

    /// `GET` avec des paramètres de requête (`?clé=valeur`), encodés par `reqwest`.
    pub async fn get_query<T: DeserializeOwned>(
        &self,
        path: &str,
        query: &[(&str, &str)],
    ) -> Result<T, AppError> {
        let resp = self
            .request(Method::GET, path)
            .query(query)
            .send()
            .await
            .map_err(Self::network_error)?;
        self.handle_json(resp).await
    }

    /// `GET` avec des paramètres de requête, retournant le corps brut en
    /// octets (fichiers binaires, ex. PDF) au lieu de le désérialiser.
    pub async fn get_bytes_query(&self, path: &str, query: &[(&str, &str)]) -> Result<Vec<u8>, AppError> {
        let resp = self
            .request(Method::GET, path)
            .query(query)
            .send()
            .await
            .map_err(Self::network_error)?;
        if !resp.status().is_success() {
            return Err(Self::extract_error(resp).await);
        }
        let bytes = resp.bytes().await.map_err(Self::network_error)?;
        Ok(bytes.to_vec())
    }

    /// `POST` avec un corps JSON, désérialisant la réponse en JSON.
    pub async fn post<T: DeserializeOwned, B: Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, AppError> {
        let resp = self
            .request(Method::POST, path)
            .json(body)
            .send()
            .await
            .map_err(Self::network_error)?;
        self.handle_json(resp).await
    }

    /// `PUT` avec un corps JSON, désérialisant la réponse en JSON.
    pub async fn put<T: DeserializeOwned, B: Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, AppError> {
        let resp = self
            .request(Method::PUT, path)
            .json(body)
            .send()
            .await
            .map_err(Self::network_error)?;
        self.handle_json(resp).await
    }

    /// `PATCH` avec un corps JSON, désérialisant la réponse en JSON.
    pub async fn patch<T: DeserializeOwned, B: Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, AppError> {
        let resp = self
            .request(Method::PATCH, path)
            .json(body)
            .send()
            .await
            .map_err(Self::network_error)?;
        self.handle_json(resp).await
    }

    /// `POST` avec un corps JSON, sans tenter de désérialiser une réponse
    /// (l'endpoint renvoie un corps vide en 200).
    pub async fn post_no_content<B: Serialize>(&self, path: &str, body: &B) -> Result<(), AppError> {
        let resp = self
            .request(Method::POST, path)
            .json(body)
            .send()
            .await
            .map_err(Self::network_error)?;
        self.handle_empty(resp).await
    }

    /// `PUT` avec un corps JSON, sans tenter de désérialiser une réponse
    /// (l'endpoint renvoie un corps vide en 200).
    pub async fn put_no_content<B: Serialize>(&self, path: &str, body: &B) -> Result<(), AppError> {
        let resp = self
            .request(Method::PUT, path)
            .json(body)
            .send()
            .await
            .map_err(Self::network_error)?;
        self.handle_empty(resp).await
    }

    /// `DELETE`, sans corps de réponse attendu.
    pub async fn delete(&self, path: &str) -> Result<(), AppError> {
        let resp = self
            .request(Method::DELETE, path)
            .send()
            .await
            .map_err(Self::network_error)?;
        self.handle_empty(resp).await
    }

    /// Vérifie le statut puis désérialise le corps en JSON.
    async fn handle_json<T: DeserializeOwned>(&self, resp: reqwest::Response) -> Result<T, AppError> {
        if resp.status().is_success() {
            resp.json().await.map_err(Self::network_error)
        } else {
            Err(Self::extract_error(resp).await)
        }
    }

    /// Vérifie le statut sans lire le corps (attendu vide en cas de succès).
    async fn handle_empty(&self, resp: reqwest::Response) -> Result<(), AppError> {
        if resp.status().is_success() {
            Ok(())
        } else {
            Err(Self::extract_error(resp).await)
        }
    }

    /// Reconstruit une `AppError` à partir d'une réponse en erreur.
    ///
    /// L'agent sérialise ses erreurs directement depuis `AppError`
    /// (`{"kind": ..., "message": ...}`), sauf le 401 du middleware
    /// d'authentification qui renvoie un `"kind":"Unauthorized"` ad-hoc, sans
    /// variante correspondante — traité à part.
    async fn extract_error(resp: reqwest::Response) -> AppError {
        let status = resp.status();
        match resp.json::<AppError>().await {
            Ok(err) => err,
            Err(_) if status == StatusCode::UNAUTHORIZED => {
                AppError::Config("Authentification agent invalide ou manquante.".to_string())
            }
            Err(e) => Self::network_error(e),
        }
    }

    /// Convertit une erreur réseau/désérialisation `reqwest` en `AppError::Config`.
    fn network_error(e: reqwest::Error) -> AppError {
        AppError::Config(format!("Erreur de communication avec l'agent : {e}"))
    }
}
