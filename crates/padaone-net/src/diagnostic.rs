//! Diagnostic des cartes Wi-Fi et des droits, à partir des sorties de `nmcli` en mode
//! « terse » (`-t`). Les fonctions d'analyse sont pures et testées sur des sorties réelles.
//!
//! Commandes lues (toutes en lecture seule) :
//! - `nmcli -t -f DEVICE,TYPE,STATE,CONNECTION device status`
//! - `nmcli -t -f GENERAL.DRIVER,IP4.ADDRESS,WIFI-PROPERTIES device show <carte>`
//! - `nmcli -t general permissions`

use serde::Serialize;

/// Découpe une ligne « terse » de nmcli sur les `:` non échappés (`\:` reste un deux-points).
pub fn decouper(ligne: &str) -> Vec<String> {
    let mut champs = vec![String::new()];
    let mut echappe = false;
    for c in ligne.chars() {
        match (echappe, c) {
            (true, _) => {
                if let Some(dernier) = champs.last_mut() {
                    dernier.push(c);
                }
                echappe = false;
            }
            (false, '\\') => echappe = true,
            (false, ':') => champs.push(String::new()),
            (false, _) => {
                if let Some(dernier) = champs.last_mut() {
                    dernier.push(c);
                }
            }
        }
    }
    champs
}

/// Une interface réseau vue par `nmcli device status`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Interface {
    pub nom: String,
    pub genre: String,
    pub etat: String,
    pub connexion: Option<String>,
}

pub fn lire_interfaces(sortie: &str) -> Vec<Interface> {
    sortie
        .lines()
        .filter(|l| !l.trim().is_empty())
        .filter_map(|l| {
            let c = decouper(l);
            if c.len() < 4 {
                return None;
            }
            Some(Interface {
                nom: c[0].clone(),
                genre: c[1].clone(),
                etat: c[2].clone(),
                connexion: Some(c[3].clone()).filter(|x| !x.is_empty()),
            })
        })
        .collect()
}

/// Capacités d'une carte Wi-Fi.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Carte {
    pub nom: String,
    pub pilote: Option<String>,
    /// État NetworkManager (« connected », « disconnected »…).
    pub etat: String,
    /// Connexion active sur la carte (le Wi-Fi de la salle, ou `padaone-ap`).
    pub connexion: Option<String>,
    /// Adresse IPv4 actuelle avec masque, par exemple « 192.168.1.23/24 ».
    pub adresse: Option<String>,
    pub point_acces: bool,
    pub ghz2: bool,
    pub ghz5: bool,
    /// Estimation lisible du nombre d'appareils supportés en point d'accès.
    pub capacite: String,
}

/// Complète une carte avec la sortie de `nmcli -t -f GENERAL.DRIVER,IP4.ADDRESS,WIFI-PROPERTIES device show`.
pub fn lire_carte(interface: &Interface, sortie: &str) -> Carte {
    let mut carte = Carte {
        nom: interface.nom.clone(),
        pilote: None,
        etat: interface.etat.clone(),
        connexion: interface.connexion.clone(),
        adresse: None,
        point_acces: false,
        ghz2: false,
        ghz5: false,
        capacite: String::new(),
    };
    for ligne in sortie.lines() {
        let Some((cle, valeur)) = ligne.split_once(':') else { continue };
        let oui = valeur.trim() == "yes";
        match cle {
            "GENERAL.DRIVER" => carte.pilote = Some(valeur.to_string()).filter(|v| !v.is_empty()),
            "WIFI-PROPERTIES.AP" => carte.point_acces = oui,
            "WIFI-PROPERTIES.2GHZ" => carte.ghz2 = oui,
            "WIFI-PROPERTIES.5GHZ" => carte.ghz5 = oui,
            c if c.starts_with("IP4.ADDRESS") && carte.adresse.is_none() => {
                carte.adresse = Some(valeur.to_string()).filter(|v| !v.is_empty());
            }
            _ => {}
        }
    }
    carte.capacite = capacite(carte.pilote.as_deref()).to_string();
    carte
}

/// Estimation prudente du nombre d'appareils en point d'accès, selon le pilote.
pub fn capacite(pilote: Option<&str>) -> &'static str {
    match pilote {
        Some("iwlwifi") => "8 à 14 appareils environ (carte Intel) : clé USB conseillée au-delà",
        Some(p) if p.starts_with("ath") => "15 à 30 appareils environ",
        Some(p) if p.starts_with("mt7") || p.starts_with("rt2") => "20 à 30 appareils environ",
        Some(p) if p.starts_with("rtl") || p.starts_with("r8") => "10 à 20 appareils environ",
        _ => "inconnue : à mesurer avant la séance",
    }
}

/// Réponse de NetworkManager pour une autorisation (polkit).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Droit {
    /// Autorisé sans mot de passe.
    Oui,
    /// Autorisé après saisie du mot de passe administrateur.
    Authentification,
    Non,
    Inconnu,
}

impl Droit {
    fn depuis(valeur: &str) -> Self {
        match valeur.trim() {
            "yes" => Droit::Oui,
            "auth" => Droit::Authentification,
            "no" => Droit::Non,
            _ => Droit::Inconnu,
        }
    }
}

/// Droits utiles pour créer et piloter le point d'accès.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Droits {
    /// Allumer et éteindre des connexions.
    pub controle_reseau: Droit,
    /// Partager une connexion en Wi-Fi protégé (WPA).
    pub partage_protege: Droit,
    /// Partager une connexion en Wi-Fi ouvert.
    pub partage_ouvert: Droit,
    /// Enregistrer un profil de connexion pour tout le système.
    pub modifier_systeme: Droit,
}

pub fn lire_droits(sortie: &str) -> Droits {
    let mut d = Droits {
        controle_reseau: Droit::Inconnu,
        partage_protege: Droit::Inconnu,
        partage_ouvert: Droit::Inconnu,
        modifier_systeme: Droit::Inconnu,
    };
    for ligne in sortie.lines() {
        let Some((cle, valeur)) = ligne.split_once(':') else { continue };
        let droit = Droit::depuis(valeur);
        match cle.trim() {
            "org.freedesktop.NetworkManager.network-control" => d.controle_reseau = droit,
            "org.freedesktop.NetworkManager.wifi.share.protected" => d.partage_protege = droit,
            "org.freedesktop.NetworkManager.wifi.share.open" => d.partage_ouvert = droit,
            "org.freedesktop.NetworkManager.settings.modify.system" => d.modifier_systeme = droit,
            _ => {}
        }
    }
    d
}

/// Une ligne du diagnostic, prête à afficher (« ✓ », « ! » ou « ✗ » + texte).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Constat {
    pub niveau: Niveau,
    pub texte: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Niveau {
    Bon,
    Attention,
    Bloquant,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Diagnostic {
    pub cartes: Vec<Carte>,
    pub droits: Droits,
    /// Carte sur laquelle le profil `padaone-ap` est actif, s'il l'est.
    pub point_acces_actif: Option<String>,
    /// Installation unique du portail captif (réglage DNS et port 80), si elle a été vérifiée.
    pub portail: Option<crate::portail::InstallationPortail>,
    pub constats: Vec<Constat>,
}

impl Diagnostic {
    /// Assemble le diagnostic et rédige les constats pour l'animateur.
    pub fn new(cartes: Vec<Carte>, droits: Droits) -> Self {
        let point_acces_actif = cartes
            .iter()
            .find(|c| c.connexion.as_deref() == Some(crate::PROFIL))
            .map(|c| c.nom.clone());
        let mut constats = Vec::new();
        let mut dire = |niveau, texte: String| constats.push(Constat { niveau, texte });

        let capables: Vec<&Carte> = cartes.iter().filter(|c| c.point_acces).collect();
        match capables.len() {
            0 if cartes.is_empty() => dire(Niveau::Bloquant, "Aucune carte Wi-Fi détectée.".into()),
            0 => dire(Niveau::Bloquant, "Aucune carte Wi-Fi ne sait faire point d'accès.".into()),
            _ => {
                for c in &capables {
                    let bandes = match (c.ghz2, c.ghz5) {
                        (true, true) => "2,4 et 5 GHz",
                        (true, false) => "2,4 GHz",
                        (false, true) => "5 GHz",
                        (false, false) => "bande inconnue",
                    };
                    dire(
                        Niveau::Bon,
                        format!("{} sait faire point d'accès ({bandes}) · {}", c.nom, c.capacite),
                    );
                }
            }
        }
        if let [seule] = capables.as_slice() {
            if let Some(cx) = seule.connexion.as_deref().filter(|cx| *cx != crate::PROFIL) {
                dire(
                    Niveau::Attention,
                    format!(
                        "Une seule carte : démarrer le point d'accès coupe « {cx} ». Le PC perd \
                         internet pendant la séance, sans conséquence pour Padaone."
                    ),
                );
            }
        }
        if let Some(p) = &point_acces_actif {
            dire(Niveau::Bon, format!("Le point d'accès Padaone est actif sur {p}."));
        }

        let droit = |d: Droit, quoi: &str| match d {
            Droit::Oui => (Niveau::Bon, format!("{quoi} : autorisé sans mot de passe.")),
            Droit::Authentification => (
                Niveau::Attention,
                format!(
                    "{quoi} : le mot de passe administrateur sera demandé (fenêtre du système)."
                ),
            ),
            Droit::Non => {
                (Niveau::Bloquant, format!("{quoi} : refusé. Voir « padaone point-acces droits »."))
            }
            Droit::Inconnu => (Niveau::Attention, format!("{quoi} : droit inconnu.")),
        };
        for (d, quoi) in [
            (droits.controle_reseau, "Piloter le réseau"),
            (droits.partage_protege, "Partager le Wi-Fi"),
            (droits.modifier_systeme, "Enregistrer le profil du point d'accès"),
        ] {
            let (niveau, texte) = droit(d, quoi);
            dire(niveau, texte);
        }
        Diagnostic { cartes, droits, point_acces_actif, portail: None, constats }
    }

    /// Ajoute l'état de l'installation du portail captif et les constats correspondants.
    pub fn avec_portail(mut self, i: crate::portail::InstallationPortail) -> Self {
        self.constats.push(if i.dns {
            Constat { niveau: Niveau::Bon, texte: "Portail captif : les noms du point d'accès sont résolus par Padaone.".into() }
        } else {
            Constat {
                niveau: Niveau::Attention,
                texte: "Portail captif : réglage DNS non installé, les téléphones diront « sans internet ». \
                        Voir « padaone point-acces portail »."
                    .into(),
            }
        });
        self.constats.push(if i.port_80 {
            Constat {
                niveau: Niveau::Bon,
                texte: "Port 80 autorisé : Padaone répond aux tests de connexion des téléphones."
                    .into(),
            }
        } else {
            Constat {
                niveau: Niveau::Attention,
                texte: "Port 80 réservé à l'administrateur : voir « padaone point-acces portail »."
                    .into(),
            }
        });
        self.portail = Some(i);
        self
    }

    /// Première carte capable de faire point d'accès (de préférence une carte non connectée,
    /// par exemple une clé USB, pour garder internet sur l'autre).
    pub fn carte_conseillee(&self) -> Option<&Carte> {
        let capables = || self.cartes.iter().filter(|c| c.point_acces);
        capables().find(|c| c.connexion.is_none()).or_else(|| capables().next())
    }

    pub fn bloquant(&self) -> bool {
        self.constats.iter().any(|c| c.niveau == Niveau::Bloquant)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Sorties relevées sur le ThinkPad de Julien (nmcli 1.46).
    const STATUT: &str = "wlp0s20f3:wifi:connected:Auto SFR_1518_5GHZ\n\
                          lo:loopback:connected (externally):lo\n\
                          p2p-dev-wlp0s20f3:wifi-p2p:disconnected:\n\
                          enp0s31f6:ethernet:unavailable:\n";
    const CARTE: &str = "GENERAL.DRIVER:iwlwifi\nIP4.ADDRESS[1]:192.168.1.23/24\n\
                         WIFI-PROPERTIES.WEP:yes\nWIFI-PROPERTIES.AP:yes\n\
                         WIFI-PROPERTIES.2GHZ:yes\nWIFI-PROPERTIES.5GHZ:yes\nWIFI-PROPERTIES.6GHZ:no\n";
    const DROITS: &str = "org.freedesktop.NetworkManager.network-control:yes\n\
                          org.freedesktop.NetworkManager.settings.modify.system:yes\n\
                          org.freedesktop.NetworkManager.wifi.share.open:yes\n\
                          org.freedesktop.NetworkManager.wifi.share.protected:auth\n";

    fn diagnostic() -> Diagnostic {
        let interfaces = lire_interfaces(STATUT);
        let wifi = interfaces.iter().find(|i| i.genre == "wifi").unwrap();
        Diagnostic::new(vec![lire_carte(wifi, CARTE)], lire_droits(DROITS))
    }

    #[test]
    fn deux_points_echappes() {
        assert_eq!(decouper(r"a:b\:c:"), vec!["a", "b:c", ""]);
    }

    #[test]
    fn interfaces_et_connexion_active() {
        let i = lire_interfaces(STATUT);
        assert_eq!(i.len(), 4);
        assert_eq!(i[0].connexion.as_deref(), Some("Auto SFR_1518_5GHZ"));
        assert_eq!(i[2].connexion, None);
    }

    #[test]
    fn capacites_de_la_carte() {
        let d = diagnostic();
        let c = &d.cartes[0];
        assert!(c.point_acces && c.ghz2 && c.ghz5);
        assert_eq!(c.pilote.as_deref(), Some("iwlwifi"));
        assert_eq!(c.adresse.as_deref(), Some("192.168.1.23/24"));
        assert!(c.capacite.contains("8 à 14"));
    }

    #[test]
    fn constats_lisibles() {
        let d = diagnostic();
        assert!(!d.bloquant());
        let textes: Vec<&str> = d.constats.iter().map(|c| c.texte.as_str()).collect();
        assert!(textes.iter().any(|t| t.contains("coupe « Auto SFR_1518_5GHZ »")));
        assert!(textes.iter().any(|t| t.contains("mot de passe administrateur")));
        assert_eq!(d.droits.partage_protege, Droit::Authentification);
    }

    #[test]
    fn carte_libre_preferee() {
        let mut d = diagnostic();
        let mut usb = d.cartes[0].clone();
        usb.nom = "wlx00c0ca".into();
        usb.connexion = None;
        d.cartes.push(usb);
        assert_eq!(d.carte_conseillee().map(|c| c.nom.as_str()), Some("wlx00c0ca"));
    }

    #[test]
    fn sans_carte_capable_c_est_bloquant() {
        let d = Diagnostic::new(vec![], lire_droits(DROITS));
        assert!(d.bloquant());
    }
}
