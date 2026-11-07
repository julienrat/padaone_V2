//! Exécution des commandes réseau.
//!
//! - [`Systeme`] lance vraiment `nmcli` : à n'utiliser que sur un geste explicite de
//!   l'animateur (bouton de la console, sous-commande `padaone point-acces`).
//! - [`Simulation`] imite un poste à deux cartes, sans rien toucher : pour les essais,
//!   les démonstrations et les tests automatiques (`padaone --reseau-simule`).

use std::process::Command;
use std::sync::Mutex;
use std::time::Duration;

use thiserror::Error;

use crate::PROFIL;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ErreurReseau {
    #[error("nmcli est introuvable : NetworkManager est-il installé ?")]
    NmcliAbsent,
    #[error("échec de « nmcli {commande} » : {message}")]
    Echec { commande: String, message: String },
    #[error("aucune carte Wi-Fi de ce PC ne sait faire point d'accès")]
    AucuneCarte,
    #[error("la carte {0} n'existe pas")]
    CarteInconnue(String),
    #[error("la carte {0} ne sait pas faire point d'accès")]
    CarteSansPointAcces(String),
    #[error("le nom du réseau doit faire de 1 à 32 caractères, sans caractère de contrôle")]
    NomReseau,
    #[error(
        "le mot de passe doit faire de 8 à 63 caractères simples (lettres, chiffres, ponctuation)"
    )]
    MotDePasse,
    #[error("ce canal ne correspond pas à la bande choisie")]
    Canal,
    #[error("adresse invalide : il faut par exemple 10.42.0.1/24")]
    Adresse,
}

/// Ce dont le module réseau a besoin du système.
pub trait Executeur: Send + Sync {
    /// Lance `nmcli` avec ces arguments et renvoie sa sortie standard.
    fn nmcli(&self, arguments: &[String]) -> Result<String, ErreurReseau>;
    /// Lit un petit fichier texte (réglages, baux DHCP), s'il existe et s'il est lisible.
    fn lire(&self, chemin: &str) -> Option<String>;
    /// Sortie de `iw dev <carte> station dump` : appareils associés au point d'accès.
    fn stations(&self, carte: &str) -> Option<String>;
}

/// Le vrai système : `nmcli` de NetworkManager.
pub struct Systeme;

impl Executeur for Systeme {
    fn nmcli(&self, arguments: &[String]) -> Result<String, ErreurReseau> {
        let sortie = Command::new("nmcli")
            .args(arguments)
            // Sorties en anglais, stables pour l'analyse quelle que soit la langue du poste.
            .env("LC_ALL", "C")
            .output()
            .map_err(|_| ErreurReseau::NmcliAbsent)?;
        if sortie.status.success() {
            Ok(String::from_utf8_lossy(&sortie.stdout).into_owned())
        } else {
            Err(ErreurReseau::Echec {
                commande: resume(arguments),
                message: String::from_utf8_lossy(&sortie.stderr).trim().to_string(),
            })
        }
    }

    fn lire(&self, chemin: &str) -> Option<String> {
        std::fs::read_to_string(chemin).ok()
    }

    fn stations(&self, carte: &str) -> Option<String> {
        let sortie = Command::new("iw").args(["dev", carte, "station", "dump"]).output().ok()?;
        sortie.status.success().then(|| String::from_utf8_lossy(&sortie.stdout).into_owned())
    }
}

/// Début de la commande, sans les valeurs (le mot de passe ne doit jamais apparaître).
fn resume(arguments: &[String]) -> String {
    arguments.iter().filter(|a| !a.starts_with('-')).take(2).cloned().collect::<Vec<_>>().join(" ")
}

/// Un poste imaginaire : une carte Intel connectée au Wi-Fi de la salle et une clé USB libre.
pub struct Simulation {
    etat: Mutex<EtatSimule>,
    /// Pause imitant la durée d'un démarrage réel.
    pub delai: Duration,
}

struct EtatSimule {
    /// Carte sur laquelle `padaone-ap` est actif.
    actif: Option<String>,
    /// Carte visée par le profil créé.
    profil: Option<String>,
    /// Adresse demandée dans le profil (`ipv4.addresses`), sinon 10.42.0.1/24.
    adresse_profil: Option<String>,
    salle_connectee: bool,
}

pub const CARTE_INTERNE_SIMULEE: &str = "wlp0s20f3";
pub const CARTE_USB_SIMULEE: &str = "wlx00c0ca9e1b2f";
pub const WIFI_SALLE_SIMULE: &str = "Wi-Fi de la salle";

impl Default for Simulation {
    fn default() -> Self {
        Simulation {
            etat: Mutex::new(EtatSimule {
                actif: None,
                profil: None,
                adresse_profil: None,
                salle_connectee: true,
            }),
            delai: Duration::from_millis(800),
        }
    }
}

impl Simulation {
    /// Simulation instantanée (pour les tests automatiques).
    pub fn immediate() -> Self {
        Simulation { delai: Duration::ZERO, ..Default::default() }
    }

    fn etat(&self) -> std::sync::MutexGuard<'_, EtatSimule> {
        self.etat.lock().unwrap_or_else(|e| e.into_inner())
    }
}

impl Executeur for Simulation {
    fn nmcli(&self, a: &[String]) -> Result<String, ErreurReseau> {
        let a: Vec<&str> = a
            .iter()
            .map(String::as_str)
            .filter(|x| *x != "-w" && x.parse::<u32>().is_err())
            .collect();
        let mut e = self.etat();
        let connexion = |carte: &str, e: &EtatSimule| -> String {
            if e.actif.as_deref() == Some(carte) {
                PROFIL.to_string()
            } else if carte == CARTE_INTERNE_SIMULEE && e.salle_connectee {
                WIFI_SALLE_SIMULE.to_string()
            } else {
                String::new()
            }
        };
        match a.as_slice() {
            ["-t", "-f", _, "device", "status"] => {
                let etat = |c: &str| {
                    if connexion(c, &e).is_empty() {
                        "disconnected"
                    } else {
                        "connected"
                    }
                };
                Ok(format!(
                    "{i}:wifi:{ei}:{ci}\n{u}:wifi:{eu}:{cu}\nlo:loopback:connected (externally):lo\n",
                    i = CARTE_INTERNE_SIMULEE,
                    ei = etat(CARTE_INTERNE_SIMULEE),
                    ci = connexion(CARTE_INTERNE_SIMULEE, &e),
                    u = CARTE_USB_SIMULEE,
                    eu = etat(CARTE_USB_SIMULEE),
                    cu = connexion(CARTE_USB_SIMULEE, &e),
                ))
            }
            ["-t", "-f", _, "device", "show", carte] => {
                let interne = *carte == CARTE_INTERNE_SIMULEE;
                let adresse =
                    match (e.actif.as_deref() == Some(carte), interne && e.salle_connectee) {
                        (true, _) => format!(
                            "IP4.ADDRESS[1]:{}\n",
                            e.adresse_profil.as_deref().unwrap_or("10.42.0.1/24")
                        ),
                        (false, true) => "IP4.ADDRESS[1]:192.168.1.23/24\n".to_string(),
                        _ => String::new(),
                    };
                Ok(format!(
                    "GENERAL.DRIVER:{}\n{adresse}WIFI-PROPERTIES.AP:yes\nWIFI-PROPERTIES.2GHZ:yes\nWIFI-PROPERTIES.5GHZ:{}\n",
                    if interne { "iwlwifi" } else { "mt7601u" },
                    if interne { "yes" } else { "no" },
                ))
            }
            ["-t", "general", "permissions"] => {
                Ok("org.freedesktop.NetworkManager.network-control:yes\n\
                 org.freedesktop.NetworkManager.wifi.share.protected:yes\n\
                 org.freedesktop.NetworkManager.wifi.share.open:yes\n\
                 org.freedesktop.NetworkManager.settings.modify.system:yes\n"
                    .into())
            }
            ["connection", "delete", p] if *p == PROFIL => {
                if e.profil.take().is_some() {
                    Ok(String::new())
                } else {
                    Err(ErreurReseau::Echec {
                        commande: "connection delete".into(),
                        message: "unknown connection".into(),
                    })
                }
            }
            ["connection", "add", reste @ ..] => {
                let carte = reste.windows(2).find(|w| w[0] == "ifname").map(|w| w[1].to_string());
                e.profil = Some(carte.unwrap_or_else(|| CARTE_INTERNE_SIMULEE.into()));
                e.adresse_profil =
                    reste.windows(2).find(|w| w[0] == "ipv4.addresses").map(|w| w[1].to_string());
                Ok(String::new())
            }
            ["connection", "up", p] if *p == PROFIL => {
                drop(e);
                std::thread::sleep(self.delai);
                let mut e = self.etat();
                let carte = e.profil.clone().ok_or(ErreurReseau::Echec {
                    commande: "connection up".into(),
                    message: "unknown connection".into(),
                })?;
                if carte == CARTE_INTERNE_SIMULEE {
                    e.salle_connectee = false;
                }
                e.actif = Some(carte);
                Ok(String::new())
            }
            ["connection", "down", p] if *p == PROFIL => {
                e.actif = None;
                Ok(String::new())
            }
            ["connection", "up", autre] if *autre == WIFI_SALLE_SIMULE => {
                e.salle_connectee = true;
                Ok(String::new())
            }
            _ => Err(ErreurReseau::Echec {
                commande: resume(&a.iter().map(|s| s.to_string()).collect::<Vec<_>>()),
                message: "commande non simulée".into(),
            }),
        }
    }

    fn stations(&self, carte: &str) -> Option<String> {
        let e = self.etat();
        (e.actif.as_deref() == Some(carte)).then(|| {
            (1..=12)
                .map(|i| {
                    format!(
                        "Station aa:bb:cc:00:00:{i:02x} (on {carte})\n\tinactive time:\t10 ms\n"
                    )
                })
                .collect()
        })
    }

    fn lire(&self, chemin: &str) -> Option<String> {
        // Le poste simulé a fait l'installation unique du portail captif.
        if chemin == crate::portail::FICHIER_DNSMASQ {
            return Some(crate::portail::configuration_dnsmasq());
        }
        if chemin.ends_with("ip_unprivileged_port_start") {
            return Some("80\n".into());
        }
        let e = self.etat();
        let carte = e.actif.as_deref()?;
        if !chemin.ends_with(&format!("dnsmasq-{carte}.leases")) {
            return None;
        }
        Some(
            (1..=12)
                .map(|i| format!("0 aa:bb:cc:00:00:{i:02x} 10.42.0.{} telephone-{i} *\n", 10 + i))
                .collect(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn le_resume_ne_montre_jamais_le_mot_de_passe() {
        let args: Vec<String> = ["connection", "add", "wifi-sec.psk", "secret-42"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        assert_eq!(resume(&args), "connection add");
    }
}
