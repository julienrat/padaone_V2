//! Démarrer et arrêter le point d'accès, avec retour au Wi-Fi précédent.
//!
//! Démarrage :
//! 1. diagnostic (lecture seule) et choix de la carte ;
//! 2. suppression d'un ancien profil `padaone-ap` (s'il existe), création du nouveau ;
//! 3. `nmcli -w 30 connection up padaone-ap` ; en cas d'échec, on rallume le Wi-Fi précédent.
//!
//! « Garder l'adresse actuelle » donne au point d'accès l'adresse que le PC avait sur le
//! Wi-Fi de la salle : les pages déjà ouvertes se reconnectent seules après la bascule.

use serde::{Deserialize, Serialize};

use crate::diagnostic::{lire_carte, lire_droits, lire_interfaces, Diagnostic};
use crate::executeur::{ErreurReseau, Executeur};
use crate::{arguments_creation, Bande, ReglagesPointAcces, Securite, PROFIL};

/// Délai laissé à NetworkManager pour allumer ou éteindre le point d'accès, en secondes.
const ATTENTE_NMCLI: &str = "30";

fn args(a: &[&str]) -> Vec<String> {
    a.iter().map(|s| s.to_string()).collect()
}

/// Lit les cartes Wi-Fi et les droits. Uniquement des commandes en lecture seule.
pub fn diagnostiquer(ex: &dyn Executeur) -> Result<Diagnostic, ErreurReseau> {
    let statut =
        ex.nmcli(&args(&["-t", "-f", "DEVICE,TYPE,STATE,CONNECTION", "device", "status"]))?;
    let mut cartes = Vec::new();
    for i in lire_interfaces(&statut).iter().filter(|i| i.genre == "wifi") {
        let details = ex.nmcli(&args(&[
            "-t",
            "-f",
            "GENERAL.DRIVER,IP4.ADDRESS,WIFI-PROPERTIES",
            "device",
            "show",
            &i.nom,
        ]))?;
        cartes.push(lire_carte(i, &details));
    }
    let droits = lire_droits(&ex.nmcli(&args(&["-t", "general", "permissions"]))?);
    Ok(Diagnostic::new(cartes, droits).avec_portail(crate::portail::verifier_installation(ex)))
}

/// Adresse du PC sur le point d'accès.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ChoixAdresse {
    /// L'adresse actuelle de la carte (bascule sans coupure pour les pages ouvertes).
    #[default]
    Garder,
    /// 10.42.0.1/24, la valeur par défaut de NetworkManager.
    ParDefaut,
    /// Une adresse précise avec son masque, par exemple « 192.168.50.1/24 ».
    Fixe(String),
}

/// Demande de démarrage, telle que la console ou la ligne de commande la formule.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Demande {
    /// Carte à utiliser ; sans précision, la carte conseillée par le diagnostic.
    #[serde(default)]
    pub carte: Option<String>,
    pub nom_reseau: String,
    pub securite: Securite,
    #[serde(default = "bande_auto")]
    pub bande: Bande,
    #[serde(default)]
    pub canal: Option<u32>,
    #[serde(default)]
    pub adresse: ChoixAdresse,
}

fn bande_auto() -> Bande {
    Bande::Auto
}

/// Vérifie le nom, le mot de passe, le canal et l'adresse avant de toucher au réseau.
pub fn valider(d: &Demande) -> Result<(), ErreurReseau> {
    let nom = &d.nom_reseau;
    if nom.is_empty() || nom.len() > 32 || nom.chars().any(char::is_control) {
        return Err(ErreurReseau::NomReseau);
    }
    if let Securite::Wpa2 { mot_de_passe } = &d.securite {
        let n = mot_de_passe.len();
        if !(8..=63).contains(&n) || !mot_de_passe.chars().all(|c| c.is_ascii_graphic() || c == ' ')
        {
            return Err(ErreurReseau::MotDePasse);
        }
    }
    if let Some(canal) = d.canal {
        let ok = match d.bande {
            Bande::Ghz2_4 => (1..=13).contains(&canal),
            Bande::Ghz5 => CANAUX_5GHZ.contains(&canal),
            Bande::Auto => (1..=13).contains(&canal) || CANAUX_5GHZ.contains(&canal),
        };
        if !ok {
            return Err(ErreurReseau::Canal);
        }
    }
    if let ChoixAdresse::Fixe(a) = &d.adresse {
        if !adresse_valide(a) {
            return Err(ErreurReseau::Adresse);
        }
    }
    Ok(())
}

/// Canaux 5 GHz autorisés en point d'accès en France sans détection radar (bande basse).
pub const CANAUX_5GHZ: [u32; 4] = [36, 40, 44, 48];

/// « 192.168.50.1/24 » : quatre octets et un masque de 8 à 30.
pub fn adresse_valide(a: &str) -> bool {
    let Some((ip, masque)) = a.split_once('/') else { return false };
    let masque_ok = masque.parse::<u8>().is_ok_and(|m| (8..=30).contains(&m));
    let octets: Vec<&str> = ip.split('.').collect();
    masque_ok && octets.len() == 4 && octets.iter().all(|o| o.parse::<u8>().is_ok())
}

/// Ce qui a été démarré, pour l'affichage et pour l'arrêt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PointAccesDemarre {
    pub carte: String,
    pub nom_reseau: String,
    pub securite: Securite,
    /// Adresse IPv4 du PC sur le point d'accès, sans masque.
    pub adresse: String,
    /// Connexion active sur la carte avant le démarrage, à rallumer à l'arrêt.
    pub connexion_precedente: Option<String>,
}

/// Démarre le point d'accès. **Modifie le réseau** : à n'appeler que sur un geste explicite.
pub fn demarrer(ex: &dyn Executeur, d: &Demande) -> Result<PointAccesDemarre, ErreurReseau> {
    valider(d)?;
    let diag = diagnostiquer(ex)?;
    let carte = match &d.carte {
        Some(nom) => diag
            .cartes
            .iter()
            .find(|c| &c.nom == nom)
            .ok_or_else(|| ErreurReseau::CarteInconnue(nom.clone()))?,
        None => diag.carte_conseillee().ok_or(ErreurReseau::AucuneCarte)?,
    };
    if !carte.point_acces {
        return Err(ErreurReseau::CarteSansPointAcces(carte.nom.clone()));
    }
    let precedente = carte.connexion.clone().filter(|c| c != PROFIL);
    let adresse = match &d.adresse {
        ChoixAdresse::Garder => carte.adresse.clone(),
        ChoixAdresse::ParDefaut => None,
        ChoixAdresse::Fixe(a) => Some(a.clone()),
    };
    // Un canal sans bande : la bande se déduit du canal (NetworkManager l'exige).
    let bande = match (d.bande, d.canal) {
        (Bande::Auto, Some(c)) if c <= 14 => Bande::Ghz2_4,
        (Bande::Auto, Some(_)) => Bande::Ghz5,
        (b, _) => b,
    };
    let reglages = ReglagesPointAcces {
        carte: Some(carte.nom.clone()),
        nom_reseau: d.nom_reseau.clone(),
        securite: d.securite.clone(),
        bande,
        canal: d.canal,
        adresse: adresse.clone(),
    };

    // Un ancien profil peut traîner (plantage, essai précédent) : on le remplace.
    let _ = ex.nmcli(&args(&["connection", "delete", PROFIL]));
    ex.nmcli(&arguments_creation(&reglages))?;
    if let Err(e) = ex.nmcli(&args(&["-w", ATTENTE_NMCLI, "connection", "up", PROFIL])) {
        if let Some(p) = &precedente {
            let _ = ex.nmcli(&args(&["-w", ATTENTE_NMCLI, "connection", "up", p]));
        }
        return Err(e);
    }

    // Adresse effective, relue sur la carte (repli : celle demandée, puis celle de NetworkManager).
    let relue = diagnostiquer(ex)
        .ok()
        .and_then(|d| d.cartes.into_iter().find(|c| c.nom == carte.nom))
        .and_then(|c| c.adresse);
    let adresse = relue.or(adresse).unwrap_or_else(|| "10.42.0.1/24".into());
    Ok(PointAccesDemarre {
        carte: carte.nom.clone(),
        nom_reseau: d.nom_reseau.clone(),
        securite: d.securite.clone(),
        adresse: adresse.split('/').next().unwrap_or("10.42.0.1").to_string(),
        connexion_precedente: precedente,
    })
}

/// Arrête le point d'accès et rallume la connexion précédente. **Modifie le réseau.**
/// Renvoie un avertissement si la connexion précédente n'a pas pu revenir.
pub fn arreter(
    ex: &dyn Executeur,
    precedente: Option<&str>,
) -> Result<Option<String>, ErreurReseau> {
    ex.nmcli(&args(&["-w", ATTENTE_NMCLI, "connection", "down", PROFIL]))?;
    match precedente {
        Some(p) => match ex.nmcli(&args(&["-w", ATTENTE_NMCLI, "connection", "up", p])) {
            Ok(_) => Ok(None),
            Err(e) => Ok(Some(format!("« {p} » n'a pas pu être rallumé : {e}"))),
        },
        None => Ok(None),
    }
}

/// Nombre d'appareils associés au point d'accès (`iw … station dump`), à défaut ceux
/// qui ont obtenu une adresse (baux DHCP de dnsmasq, souvent illisibles sans droits).
pub fn appareils(ex: &dyn Executeur, carte: &str) -> Option<usize> {
    if let Some(stations) = ex.stations(carte) {
        return Some(stations.lines().filter(|l| l.starts_with("Station ")).count());
    }
    let baux = ex.lire(&format!("/var/lib/NetworkManager/dnsmasq-{carte}.leases"))?;
    Some(baux.lines().filter(|l| !l.trim().is_empty()).count())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::executeur::{
        Simulation, CARTE_INTERNE_SIMULEE, CARTE_USB_SIMULEE, WIFI_SALLE_SIMULE,
    };

    fn simulation() -> Simulation {
        Simulation::immediate()
    }

    fn demande() -> Demande {
        Demande {
            carte: None,
            nom_reseau: "Padaone-Tilleul".into(),
            securite: Securite::Wpa2 { mot_de_passe: "tilleul-42".into() },
            bande: Bande::Auto,
            canal: None,
            adresse: ChoixAdresse::Garder,
        }
    }

    #[test]
    fn validation_avant_tout() {
        assert!(valider(&demande()).is_ok());
        let court =
            Demande { securite: Securite::Wpa2 { mot_de_passe: "court".into() }, ..demande() };
        assert_eq!(valider(&court), Err(ErreurReseau::MotDePasse));
        let accent = Demande {
            securite: Securite::Wpa2 { mot_de_passe: "tilleul-é42".into() },
            ..demande()
        };
        assert_eq!(valider(&accent), Err(ErreurReseau::MotDePasse));
        let long = Demande { nom_reseau: "x".repeat(33), ..demande() };
        assert_eq!(valider(&long), Err(ErreurReseau::NomReseau));
        let canal = Demande { bande: Bande::Ghz2_4, canal: Some(36), ..demande() };
        assert_eq!(valider(&canal), Err(ErreurReseau::Canal));
        let adresse = Demande { adresse: ChoixAdresse::Fixe("10.0.0.300/24".into()), ..demande() };
        assert_eq!(valider(&adresse), Err(ErreurReseau::Adresse));
    }

    #[test]
    fn adresses() {
        assert!(adresse_valide("192.168.50.1/24"));
        assert!(!adresse_valide("192.168.50.1"));
        assert!(!adresse_valide("192.168.50/24"));
        assert!(!adresse_valide("10.0.0.1/31"));
    }

    #[test]
    fn la_carte_libre_est_choisie_et_internet_reste() {
        let sim = simulation();
        let d = demarrer(&sim, &demande()).unwrap();
        assert_eq!(d.carte, CARTE_USB_SIMULEE);
        assert_eq!(d.connexion_precedente, None);
        let diag = diagnostiquer(&sim).unwrap();
        assert_eq!(diag.point_acces_actif.as_deref(), Some(CARTE_USB_SIMULEE));
        let interne = diag.cartes.iter().find(|c| c.nom == CARTE_INTERNE_SIMULEE).unwrap();
        assert_eq!(interne.connexion.as_deref(), Some(WIFI_SALLE_SIMULE));
    }

    #[test]
    fn sur_la_carte_interne_on_garde_l_adresse_puis_on_revient() {
        let sim = simulation();
        let d = demarrer(&sim, &Demande { carte: Some(CARTE_INTERNE_SIMULEE.into()), ..demande() })
            .unwrap();
        assert_eq!(d.connexion_precedente.as_deref(), Some(WIFI_SALLE_SIMULE));
        assert_eq!(d.adresse, "192.168.1.23", "l'adresse de la salle est gardée");
        assert_eq!(appareils(&sim, &d.carte), Some(12));
        assert_eq!(arreter(&sim, d.connexion_precedente.as_deref()), Ok(None));
        let diag = diagnostiquer(&sim).unwrap();
        assert_eq!(diag.point_acces_actif, None);
        assert_eq!(diag.cartes[0].connexion.as_deref(), Some(WIFI_SALLE_SIMULE));
    }

    #[test]
    fn carte_inconnue_refusee() {
        let sim = simulation();
        let d = Demande { carte: Some("wlan9".into()), ..demande() };
        assert_eq!(demarrer(&sim, &d), Err(ErreurReseau::CarteInconnue("wlan9".into())));
    }
}
