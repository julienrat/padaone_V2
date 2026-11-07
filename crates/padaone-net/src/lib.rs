//! Réseau de Padaone.
//!
//! Phase 0 : piloter NetworkManager avec `nmcli` pour créer et arrêter un
//! point d'accès Wi-Fi, et diagnostiquer les cartes (mode point d'accès,
//! bandes 2,4 et 5 GHz). Les fonctions qui construisent les commandes sont
//! pures et testées ; seule l'exécution touche au système.
//!
//! Règle : aucune commande qui modifie le réseau ne s'exécute pendant les
//! tests automatiques.

pub mod diagnostic;
pub mod dns;
pub mod executeur;
pub mod point_acces;
pub mod portail;
pub mod suggestion;

pub use diagnostic::Diagnostic;
pub use executeur::{ErreurReseau, Executeur, Simulation, Systeme};
pub use point_acces::{ChoixAdresse, Demande, PointAccesDemarre};
pub use portail::ModePortail;

use serde::{Deserialize, Serialize};

/// Nom du profil NetworkManager créé par Padaone.
pub const PROFIL: &str = "padaone-ap";

/// Sécurité du point d'accès. En JSON : `{"type": "ouvert"}` ou
/// `{"type": "wpa2", "mot_de_passe": "tilleul-42"}`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Securite {
    Ouvert,
    Wpa2 { mot_de_passe: String },
}

/// Bande radio demandée. En JSON : `"ghz2_4"`, `"ghz5"` ou `"auto"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Bande {
    /// 2,4 GHz (valeur NetworkManager « bg »), la plus compatible.
    Ghz2_4,
    /// 5 GHz (« a »), si la carte l'autorise en point d'accès.
    Ghz5,
    /// Laisser NetworkManager choisir.
    Auto,
}

/// Réglages du point d'accès, tels que l'écran « Point d'accès » les expose.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReglagesPointAcces {
    pub carte: Option<String>,
    pub nom_reseau: String,
    pub securite: Securite,
    pub bande: Bande,
    pub canal: Option<u32>,
    /// Adresse du PC avec masque, par exemple « 192.168.1.37/24 ».
    /// `None` : valeur par défaut de NetworkManager (10.42.0.1/24).
    pub adresse: Option<String>,
}

/// Arguments de `nmcli` pour créer le profil du point d'accès.
pub fn arguments_creation(r: &ReglagesPointAcces) -> Vec<String> {
    let mut a: Vec<String> = ["connection", "add", "type", "wifi", "con-name", PROFIL]
        .iter()
        .map(|s| s.to_string())
        .collect();
    if let Some(carte) = &r.carte {
        a.extend(["ifname".to_string(), carte.clone()]);
    }
    a.extend(
        ["autoconnect", "no", "ssid"].iter().map(|s| s.to_string()).chain([r.nom_reseau.clone()]),
    );
    a.extend(["802-11-wireless.mode".into(), "ap".into()]);
    match r.bande {
        Bande::Ghz2_4 => a.extend(["802-11-wireless.band".into(), "bg".into()]),
        Bande::Ghz5 => a.extend(["802-11-wireless.band".into(), "a".into()]),
        Bande::Auto => {}
    }
    if let Some(canal) = r.canal {
        a.extend(["802-11-wireless.channel".into(), canal.to_string()]);
    }
    a.extend(["ipv4.method".into(), "shared".into()]);
    if let Some(adresse) = &r.adresse {
        a.extend(["ipv4.addresses".into(), adresse.clone()]);
    }
    if let Securite::Wpa2 { mot_de_passe } = &r.securite {
        a.extend(
            [
                "wifi-sec.key-mgmt",
                "wpa-psk",
                "wifi-sec.proto",
                "rsn",
                "wifi-sec.pairwise",
                "ccmp",
                "wifi-sec.group",
                "ccmp",
                "wifi-sec.psk",
            ]
            .iter()
            .map(|s| s.to_string()),
        );
        a.push(mot_de_passe.clone());
    }
    a
}

/// Texte du QR code « Wi-Fi » lu par les appareils photo d'Android et d'iOS.
pub fn texte_qr_wifi(nom_reseau: &str, securite: &Securite) -> String {
    match securite {
        Securite::Ouvert => format!("WIFI:T:nopass;S:{};;", echapper(nom_reseau)),
        Securite::Wpa2 { mot_de_passe } => {
            format!("WIFI:T:WPA;S:{};P:{};;", echapper(nom_reseau), echapper(mot_de_passe))
        }
    }
}

/// Échappe les caractères réservés du format WIFI: (\ ; , : ").
fn echapper(texte: &str) -> String {
    let mut sortie = String::with_capacity(texte.len());
    for c in texte.chars() {
        if matches!(c, '\\' | ';' | ',' | ':' | '"') {
            sortie.push('\\');
        }
        sortie.push(c);
    }
    sortie
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reglages() -> ReglagesPointAcces {
        ReglagesPointAcces {
            carte: Some("wlx00c0ca".into()),
            nom_reseau: "Padaone-Tilleul".into(),
            securite: Securite::Wpa2 { mot_de_passe: "tilleul-42".into() },
            bande: Bande::Ghz2_4,
            canal: Some(11),
            adresse: Some("192.168.1.37/24".into()),
        }
    }

    #[test]
    fn commande_de_creation_complete() {
        let a = arguments_creation(&reglages()).join(" ");
        assert!(a.starts_with("connection add type wifi con-name padaone-ap ifname wlx00c0ca"));
        assert!(a.contains("802-11-wireless.mode ap"));
        assert!(a.contains("802-11-wireless.band bg"));
        assert!(a.contains("802-11-wireless.channel 11"));
        assert!(a.contains("ipv4.method shared ipv4.addresses 192.168.1.37/24"));
        assert!(a.ends_with("wifi-sec.psk tilleul-42"));
    }

    #[test]
    fn reseau_ouvert_sans_securite() {
        let mut r = reglages();
        r.securite = Securite::Ouvert;
        let a = arguments_creation(&r);
        assert!(!a.iter().any(|x| x.starts_with("wifi-sec")));
    }

    #[test]
    fn qr_wifi_echappe_les_caracteres_reserves() {
        let s = Securite::Wpa2 { mot_de_passe: "a;b:c".into() };
        assert_eq!(texte_qr_wifi("Salle, 2", &s), r"WIFI:T:WPA;S:Salle\, 2;P:a\;b\:c;;");
        assert_eq!(texte_qr_wifi("Libre", &Securite::Ouvert), "WIFI:T:nopass;S:Libre;;");
    }
}
