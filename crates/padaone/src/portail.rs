//! Portail captif côté serveur.
//!
//! - DNS : Padaone écoute **uniquement en local** (`127.0.0.1:5380`) les questions que le
//!   dnsmasq du point d'accès lui transmet, et répond par l'adresse du PC.
//! - HTTP : sur le port 80 (en plus du port principal), Padaone répond aux tests de
//!   connexion d'Android, d'Apple, de Windows et de Firefox selon le mode choisi, et
//!   renvoie vers la page de la séance toute adresse inconnue.
//!
//! Les deux demandent l'installation unique décrite par `padaone point-acces portail`.
//! Sans elle, Padaone fonctionne normalement : seuls les téléphones afficheront « sans internet ».

use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;

use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Router;
use padaone_net::dns::{self, PORT_DNS};
use padaone_net::portail::{hote_du_pc, SONDES};
use padaone_net::ModePortail;

use crate::etat::Partage;

/// Port des tests de connexion des appareils.
pub const PORT_HTTP: u16 = 80;
/// Délai entre deux tentatives d'écoute sur le port 80 (le réglage peut être fait après le lancement).
const REESSAI_PORT_80: Duration = Duration::from_secs(15);

/// Répond aux questions DNS transmises par dnsmasq, tant que Padaone tourne.
pub async fn servir_dns(partage: Arc<Partage>) {
    let socket = match tokio::net::UdpSocket::bind((Ipv4Addr::LOCALHOST, PORT_DNS)).await {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!("portail captif : DNS local indisponible sur le port {PORT_DNS} ({e})");
            return;
        }
    };
    partage.modifier_reseau(|r| r.point_acces.portail_dns = true);
    let mut tampon = [0u8; 1500];
    loop {
        let Ok((n, de)) = socket.recv_from(&mut tampon).await else { continue };
        let (ip, mode) = {
            let r = partage.reseau();
            (r.ip_portail(), r.point_acces.portail)
        };
        let Some(ip) = ip else { continue };
        if let Some(reponse) = dns::repondre(&tampon[..n], ip, mode != ModePortail::Desactive) {
            let _ = socket.send_to(&reponse, de).await;
        }
    }
}

/// Écoute le port 80 dès que le système le permet (nouvel essai toutes les 15 s).
pub async fn ecouter_port_80(partage: Arc<Partage>, app: Router) {
    let adresse = SocketAddr::from((Ipv4Addr::UNSPECIFIED, PORT_HTTP));
    let mut prevenu = false;
    loop {
        match tokio::net::TcpListener::bind(adresse).await {
            Ok(auditeur) => {
                tracing::info!("portail captif : Padaone répond sur le port 80");
                partage.modifier_reseau(|r| r.point_acces.portail_port_80 = true);
                if let Err(e) = axum::serve(auditeur, app.clone()).await {
                    tracing::warn!("portail captif : port 80 interrompu ({e})");
                }
                partage.modifier_reseau(|r| r.point_acces.portail_port_80 = false);
            }
            Err(e) => {
                if !prevenu {
                    tracing::info!(
                        "portail captif : port 80 indisponible ({e}) ; voir « padaone point-acces portail »"
                    );
                    prevenu = true;
                }
            }
        }
        tokio::time::sleep(REESSAI_PORT_80).await;
    }
}

/// Réponse du portail pour cette requête, ou `None` pour servir la page normalement.
pub fn repondre(partage: &Partage, hote: Option<&str>, chemin: &str) -> Option<Response> {
    let (mode, lien) = {
        let r = partage.reseau();
        (r.point_acces.portail, r.info.lien.clone())
    };
    if mode == ModePortail::Desactive {
        return None;
    }
    let vers_seance = || {
        let cible = format!("{lien}&portail=1");
        (
            StatusCode::FOUND,
            [(header::LOCATION, cible), (header::CACHE_CONTROL, "no-store".to_string())],
        )
            .into_response()
    };
    if let Some(sonde) = SONDES.iter().find(|s| s.chemin == chemin) {
        return Some(match (mode, sonde.reponse_ok) {
            (ModePortail::Silencieux, None) => StatusCode::NO_CONTENT.into_response(),
            (ModePortail::Silencieux, Some(texte)) => {
                let genre = if texte.starts_with('<') { "text/html" } else { "text/plain" };
                ([(header::CONTENT_TYPE, genre), (header::CACHE_CONTROL, "no-store")], texte)
                    .into_response()
            }
            _ => vers_seance(),
        });
    }
    // Un nom quelconque (que le DNS du portail a envoyé vers le PC) : direction la séance.
    match hote {
        Some(h) if !hote_du_pc(h) => Some(vers_seance()),
        _ => None,
    }
}
