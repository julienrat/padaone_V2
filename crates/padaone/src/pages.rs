//! Pages web embarquées dans le binaire (dossier `web/`), sans aucun CDN.
//!
//! En compilation de développement, rust-embed relit les fichiers sur le disque :
//! il suffit de recharger la page après une modification du HTML, du CSS ou du JS.

use std::sync::Arc;

use axum::extract::State;
use axum::http::{header, HeaderMap, StatusCode, Uri};
use axum::response::{IntoResponse, Redirect, Response};
use rust_embed::RustEmbed;

use crate::etat::Partage;

#[derive(RustEmbed)]
#[folder = "web/"]
#[exclude = "*.md"]
struct Web;

/// Sert `/p`, `/ecran`, `/console` et leurs fichiers, ainsi que `/commun/…`.
/// Les tests de connexion des appareils et les noms inconnus passent d'abord par le portail captif.
pub async fn servir(State(partage): State<Arc<Partage>>, entetes: HeaderMap, uri: Uri) -> Response {
    let hote = entetes.get(header::HOST).and_then(|h| h.to_str().ok());
    if let Some(reponse) = crate::portail::repondre(&partage, hote, uri.path()) {
        return reponse;
    }
    let chemin = uri.path().trim_start_matches('/');
    let chemin = match chemin.trim_end_matches('/') {
        "" => return Redirect::temporary("/p").into_response(),
        interface @ ("p" | "ecran" | "console") => format!("{interface}/index.html"),
        autre => autre.to_string(),
    };
    match Web::get(&chemin) {
        Some(fichier) => {
            let type_mime = fichier.metadata.mimetype().to_string();
            let type_mime = if type_mime.starts_with("text/") {
                format!("{type_mime}; charset=utf-8")
            } else {
                type_mime
            };
            (
                [(header::CONTENT_TYPE, type_mime), (header::CACHE_CONTROL, "no-cache".into())],
                fichier.data,
            )
                .into_response()
        }
        None => (StatusCode::NOT_FOUND, "Page introuvable.").into_response(),
    }
}
