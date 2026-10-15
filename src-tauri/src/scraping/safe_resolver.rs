//! Résolveur DNS de `reqwest` qui écarte les adresses non publiques.
//!
//! La validation de l'URL (`url_validation`) ne voit que le nom d'hôte : un
//! nom public peut résoudre vers 127.0.0.1 ou une IP privée. Ce résolveur
//! applique la même politique (`ip_policy`) aux adresses réellement
//! contactées, y compris pour chaque saut de redirection.

// Importation des modules standard
use std::io;
use std::net::SocketAddr;

// Importation des crates externes
use reqwest::dns::{Addrs, Name, Resolve, Resolving};

// Importation des modules internes
use crate::scraping::ip_policy::is_public_ip;

/// Résolveur DNS ne renvoyant que des adresses publiques.
pub struct PublicOnlyResolver;

impl Resolve for PublicOnlyResolver {
    fn resolve(&self, name: Name) -> Resolving {
        let host = name.as_str().to_string();
        Box::pin(async move {
            // Le port est ignoré par reqwest (il applique celui de l'URL) :
            // 0 sert seulement à satisfaire `lookup_host`.
            let resolved: Vec<SocketAddr> = tokio::net::lookup_host((host.as_str(), 0)).await?.collect();
            let public: Vec<SocketAddr> = resolved.into_iter().filter(|addr| is_public_ip(addr.ip())).collect();
            if public.is_empty() {
                let err = io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    format!("l'hôte {host} ne résout vers aucune adresse publique"),
                );
                return Err(Box::new(err) as Box<dyn std::error::Error + Send + Sync>);
            }
            Ok(Box::new(public.into_iter()) as Addrs)
        })
    }
}
