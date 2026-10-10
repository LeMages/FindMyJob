//! Commandes Tauri de la fenêtre de capture : quand l'application ne peut
//! pas récupérer une offre elle-même (accès refusé, contenu chargé en
//! JavaScript), l'utilisateur ouvre la page dans une fenêtre secondaire,
//! accepte les cookies ou se connecte, puis capture le HTML rendu. Ce module
//! ne fait que capturer et transmettre : l'extraction est dans
//! `services::scraping::extract_from_html`.
//!
//! Sécurité : la fenêtre `capture` n'appartient à aucune capability, donc la
//! page distante n'a accès à aucune commande IPC (Tauri refuse tout appel
//! d'une origine distante sans capability `remote`). Le HTML est lu côté
//! Rust par `eval_with_callback`, sans IPC depuis la page.

// Importation des modules standard
use std::sync::{mpsc, Mutex};
use std::time::Duration;

// Importation des crates externes
use serde_json::Value;
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

// Importation des modules internes
use crate::app_state::AppContext;
use crate::error::AppError;
use crate::services;
use findmyjob_core::models::OfferExtraction;
use findmyjob_core::scraping::failure::ScrapingFailure;
use findmyjob_core::scraping::fetch::MAX_RESPONSE_BYTES;
use findmyjob_core::scraping::url_validation::validate_offer_url;

/// Label de la fenêtre de capture (absent de toute capability).
const CAPTURE_LABEL: &str = "capture";
/// Sous-dossier du profil WebView de la fenêtre de capture : la session
/// (cookies, connexion) y est conservée entre deux utilisations, isolée de
/// la fenêtre principale.
const CAPTURE_DATA_DIR: &str = "capture_webview";
/// Taille maximale du HTML capturé : la même que pour une page récupérée par
/// lien (5 Mo).
const MAX_CAPTURE_BYTES: usize = MAX_RESPONSE_BYTES;
/// Délai maximal de lecture du HTML dans la WebView.
const CAPTURE_TIMEOUT: Duration = Duration::from_secs(20);

/// Script de capture : renvoie le HTML rendu, ou un marqueur si la page est
/// trop lourde. Sous Windows, une exception JavaScript n'est pas remontée
/// par `eval_with_callback` : elle est attrapée et renvoyée explicitement.
fn capture_script() -> String {
    format!(
        "(() => {{ try {{ const html = document.documentElement.outerHTML; \
         return html.length > {MAX_CAPTURE_BYTES} ? {{ tooLarge: true }} : {{ html }}; }} \
         catch (e) {{ return {{ error: String(e) }}; }} }})()"
    )
}

/// Ouvre l'offre dans la fenêtre de capture (ou y navigue si elle est déjà
/// ouverte) et la met au premier plan.
///
/// # Arguments
///
/// * `app` - Handle de l'application Tauri.
/// * `url` - URL de l'offre, validée comme pour une récupération : seule une page
///   publique en http(s) est chargée. Indispensable ici : une URL locale
///   (ex. le serveur de développement) serait traitée par Tauri comme une
///   page de l'application, avec accès à l'IPC.
///
/// # Returns
///
/// `Ok(())` une fois la fenêtre ouverte, ou l'erreur de validation/création.
#[tauri::command]
pub async fn open_capture_window(app: AppHandle, url: String) -> Result<(), AppError> {
    let url = validate_offer_url(&url)?;

    if let Some(window) = app.get_webview_window(CAPTURE_LABEL) {
        window
            .navigate(url)
            .map_err(|e| window_error("naviguer vers l'offre", e))?;
        window.set_focus().map_err(|e| window_error("afficher la fenêtre", e))?;
        return Ok(());
    }

    let data_dir = app
        .path()
        .app_local_data_dir()
        .map_err(|e| window_error("localiser le dossier de session", e))?
        .join(CAPTURE_DATA_DIR);
    WebviewWindowBuilder::new(&app, CAPTURE_LABEL, WebviewUrl::External(url))
        .title("FindMyJob — page de l'offre")
        .inner_size(1100.0, 800.0)
        .data_directory(data_dir)
        .build()
        .map_err(|e| window_error("ouvrir la fenêtre", e))?;
    Ok(())
}

/// Capture le HTML rendu de la fenêtre de capture, en extrait les champs,
/// puis ferme la fenêtre si l'extraction réussit.
///
/// # Arguments
///
/// * `app` - Handle de l'application Tauri.
/// * `ctx` - Contexte de l'application.
///
/// # Returns
///
/// Les champs extraits et l'étage utilisé, ou l'erreur typée d'extraction
/// (la fenêtre reste alors ouverte pour que l'utilisateur ajuste la page).
#[tauri::command]
pub async fn capture_offer_page(
    app: AppHandle,
    ctx: tauri::State<'_, AppContext>,
) -> Result<OfferExtraction, AppError> {
    let window = app.get_webview_window(CAPTURE_LABEL).ok_or_else(|| {
        AppError::Validation("La fenêtre de l'offre est fermée. Ouvre-la à nouveau.".to_string())
    })?;

    // L'URL courante peut différer de l'URL saisie (redirection, connexion) :
    // c'est elle qui est transmise, après la même validation.
    let page_url = window.url().map_err(|e| window_error("lire l'adresse de la page", e))?;
    let page_url = validate_offer_url(page_url.as_str())?;

    let html = read_rendered_html(window.clone()).await?;
    let extraction = services::scraping::extract_from_html(&ctx.state()?.pool, page_url.as_str(), &html).await?;

    window.close().map_err(|e| window_error("fermer la fenêtre", e))?;
    Ok(extraction)
}

/// Ferme la fenêtre de capture si elle est ouverte (retour à l'étape du
/// lien, saisie manuelle...). Sans effet si elle est déjà fermée.
///
/// # Arguments
///
/// * `app` - Handle de l'application Tauri.
///
/// # Returns
///
/// `Ok(())`, ou l'erreur de fermeture.
#[tauri::command]
pub async fn close_capture_window(app: AppHandle) -> Result<(), AppError> {
    match app.get_webview_window(CAPTURE_LABEL) {
        Some(window) => window.close().map_err(|e| window_error("fermer la fenêtre", e)),
        None => Ok(()),
    }
}

/// Lit `document.documentElement.outerHTML` dans la fenêtre, côté Rust.
async fn read_rendered_html(window: WebviewWindow) -> Result<String, AppError> {
    let (tx, rx) = mpsc::channel::<String>();
    // Le callback est `Fn` : l'expéditeur, consommé une seule fois, est
    // placé dans un `Mutex<Option<_>>`.
    let tx = Mutex::new(Some(tx));
    window
        .eval_with_callback(capture_script(), move |result| {
            let sender = tx.lock().ok().and_then(|mut slot| slot.take());
            if let Some(sender) = sender {
                // Le récepteur a pu abandonner après le délai : rien à faire.
                let _ = sender.send(result);
            }
        })
        .map_err(|e| window_error("lire la page", e))?;

    // `recv_timeout` est bloquant : il tourne hors du runtime asynchrone.
    let raw = tauri::async_runtime::spawn_blocking(move || rx.recv_timeout(CAPTURE_TIMEOUT))
        .await
        .map_err(|e| window_error("lire la page", e))?
        .map_err(|_| AppError::Config("La page n'a pas répondu à la capture. Réessaie.".to_string()))?;

    parse_capture_result(&raw)
}

/// Interprète le résultat JSON du script de capture.
fn parse_capture_result(raw: &str) -> Result<String, AppError> {
    let value: Value = serde_json::from_str(raw)
        .map_err(|e| AppError::Config(format!("Résultat de capture illisible : {e}")))?;
    if value.get("tooLarge").is_some() {
        return Err(AppError::Scraping(ScrapingFailure::TooLarge));
    }
    if let Some(error) = value.get("error").and_then(Value::as_str) {
        return Err(AppError::Config(format!("Impossible de lire la page : {error}")));
    }
    let html = value
        .get("html")
        .and_then(Value::as_str)
        .ok_or_else(|| AppError::Config("La capture n'a renvoyé aucun HTML.".to_string()))?;
    // `length` compte des unités UTF-16 côté JavaScript : on revérifie la
    // taille en octets, celle qui est limitée.
    if html.len() > MAX_CAPTURE_BYTES {
        return Err(AppError::Scraping(ScrapingFailure::TooLarge));
    }
    if html.trim().is_empty() {
        return Err(AppError::Scraping(ScrapingFailure::EmptyPage));
    }
    Ok(html.to_string())
}

/// Erreur d'opération sur la fenêtre de capture, avec son contexte.
fn window_error(action: &str, e: impl std::fmt::Display) -> AppError {
    AppError::Config(format!("Fenêtre de capture : impossible de {action} ({e})."))
}
