//! Portail captif : ce qu'il faut installer une fois avec les droits administrateur,
//! la vérification (en lecture seule) que c'est fait, et les réponses attendues par
//! les téléphones et ordinateurs quand ils testent la connexion.
//!
//! Padaone n'exécute jamais `sudo` : la sous-commande `padaone point-acces portail`
//! affiche les commandes, que l'animateur copie lui-même.

use serde::{Deserialize, Serialize};

use crate::dns::PORT_DNS;
use crate::executeur::Executeur;

/// Fichier de réglage lu par le dnsmasq des points d'accès NetworkManager.
pub const FICHIER_DNSMASQ: &str = "/etc/NetworkManager/dnsmasq-shared.d/padaone.conf";
/// Fichier de réglage système qui autorise le port 80 sans être administrateur.
pub const FICHIER_SYSCTL: &str = "/etc/sysctl.d/60-padaone.conf";
const PORT_NON_PRIVILEGIE: &str = "/proc/sys/net/ipv4/ip_unprivileged_port_start";

/// Que faire quand un appareil teste la connexion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ModePortail {
    /// L'appareil propose « Se connecter au réseau » et ouvre la page de la séance.
    #[default]
    Ouvrir,
    /// On répond comme si internet marchait (Android parlera de « connexion limitée »).
    Silencieux,
    /// Aucune réponse : l'appareil dira « sans internet ».
    Desactive,
}

/// Contenu du réglage dnsmasq : toutes les questions DNS du point d'accès vont à Padaone.
pub fn configuration_dnsmasq() -> String {
    format!(
        "# Padaone : sur le point d'accès, les noms sont résolus par Padaone (portail captif).\n\
         # Ce réglage vaut pour tous les partages de connexion de NetworkManager.\n\
         no-resolv\n\
         server=127.0.0.1#{PORT_DNS}\n"
    )
}

/// Contenu du réglage système : ports à partir de 80 ouverts aux programmes ordinaires.
pub fn configuration_sysctl() -> &'static str {
    "# Padaone : le portail captif répond sur le port 80 sans droits administrateur.\n\
     net.ipv4.ip_unprivileged_port_start=80\n"
}

/// État de l'installation unique du portail, vérifié en lecture seule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct InstallationPortail {
    pub dns: bool,
    pub port_80: bool,
}

pub fn verifier_installation(ex: &dyn Executeur) -> InstallationPortail {
    let dns = ex
        .lire(FICHIER_DNSMASQ)
        .is_some_and(|c| c.contains(&format!("server=127.0.0.1#{PORT_DNS}")));
    let port_80 = ex
        .lire(PORT_NON_PRIVILEGIE)
        .and_then(|c| c.trim().parse::<u16>().ok())
        .is_some_and(|p| p <= 80);
    InstallationPortail { dns, port_80 }
}

/// Une adresse de test de connexion et la réponse qui signifie « internet marche ».
pub struct Sonde {
    pub chemin: &'static str,
    /// `None` : réponse vide avec le code 204.
    pub reponse_ok: Option<&'static str>,
}

/// Adresses testées par Android, Apple, Windows et Firefox (spécification § 5).
pub const SONDES: &[Sonde] = &[
    Sonde { chemin: "/generate_204", reponse_ok: None },
    Sonde { chemin: "/gen_204", reponse_ok: None },
    Sonde {
        chemin: "/hotspot-detect.html",
        reponse_ok: Some("<HTML><HEAD><TITLE>Success</TITLE></HEAD><BODY>Success</BODY></HTML>"),
    },
    Sonde {
        chemin: "/library/test/success.html",
        reponse_ok: Some("<HTML><HEAD><TITLE>Success</TITLE></HEAD><BODY>Success</BODY></HTML>"),
    },
    Sonde { chemin: "/connecttest.txt", reponse_ok: Some("Microsoft Connect Test") },
    Sonde { chemin: "/ncsi.txt", reponse_ok: Some("Microsoft NCSI") },
    Sonde { chemin: "/success.txt", reponse_ok: Some("success\n") },
    Sonde {
        chemin: "/canonical.html",
        reponse_ok: Some(
            "<meta http-equiv=\"refresh\" content=\"0;url=https://support.mozilla.org/kb/captive-portal\"/>",
        ),
    },
];

/// Vrai si le nom d'hôte demandé désigne le PC (adresse IP, `localhost`, `padaone.lan`).
/// Sinon, c'est un nom quelconque que le DNS du portail a envoyé vers le PC.
pub fn hote_du_pc(hote: &str) -> bool {
    // « [::1]:8080 », « hote:8080 » ou une adresse IPv6 sans crochets.
    let sans_port = if let Some(reste) = hote.strip_prefix('[') {
        reste.split(']').next().unwrap_or_default()
    } else if hote.matches(':').count() == 1 {
        hote.split(':').next().unwrap_or_default()
    } else {
        hote
    };
    let h = sans_port.to_ascii_lowercase();
    h.is_empty()
        || h == "localhost"
        || h == crate::dns::NOM_SIMPLE
        || h.parse::<std::net::IpAddr>().is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::executeur::Simulation;

    #[test]
    fn hotes() {
        assert!(hote_du_pc("192.168.1.23:8080"));
        assert!(hote_du_pc("10.42.0.1"));
        assert!(hote_du_pc("padaone.lan"));
        assert!(hote_du_pc("localhost:8080"));
        assert!(!hote_du_pc("connectivitycheck.gstatic.com"));
        assert!(!hote_du_pc("captive.apple.com:80"));
    }

    #[test]
    fn reglage_dnsmasq_vers_padaone() {
        assert!(configuration_dnsmasq().contains("server=127.0.0.1#5380"));
        assert!(configuration_dnsmasq().contains("no-resolv"));
    }

    #[test]
    fn verification_en_simulation() {
        let i = verifier_installation(&Simulation::immediate());
        assert!(i.dns && i.port_80);
    }

    #[test]
    fn toutes_les_sondes_de_la_specification() {
        let chemins: Vec<&str> = SONDES.iter().map(|s| s.chemin).collect();
        for c in ["/generate_204", "/hotspot-detect.html", "/connecttest.txt", "/canonical.html"] {
            assert!(chemins.contains(&c));
        }
    }
}
