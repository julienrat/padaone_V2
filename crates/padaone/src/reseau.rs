//! Réseau vu du serveur : adresse annoncée aux téléphones, QR codes, et point d'accès
//! piloté depuis la console.
//!
//! Les commandes `nmcli` peuvent durer plusieurs secondes : elles tournent dans une tâche
//! à part (`spawn_blocking`) ; la séance continue pendant ce temps. Rien ne modifie le
//! réseau sans un geste de l'animateur (bouton de la console).

use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use padaone_net::diagnostic::Diagnostic;
use padaone_net::{
    point_acces, suggestion, texte_qr_wifi, Demande, Executeur, ModePortail, PointAccesDemarre,
    Securite,
};
use qrcode::render::svg;
use qrcode::QrCode;
use serde::{Deserialize, Serialize};

use crate::etat::Partage;

/// Intervalle de relevé du nombre d'appareils connectés au point d'accès.
const RELEVE_APPAREILS: Duration = Duration::from_secs(5);

/// Commandes réseau de la console (champ `c`), à côté des commandes de séance.
#[derive(Debug, PartialEq, Eq, Deserialize)]
#[serde(tag = "c", rename_all = "snake_case")]
pub enum CommandeReseau {
    /// Relit les cartes et les droits (lecture seule).
    Diagnostic,
    PointAccesDemarrer(Demande),
    PointAccesArreter,
    /// Nouveau nom de réseau et nouveau mot de passe suggérés.
    PointAccesSuggerer,
    PointAccesOptions {
        #[serde(default)]
        arret_fin_seance: Option<bool>,
        #[serde(default)]
        portail: Option<ModePortail>,
    },
}

/// Adresses utiles à l'écran et à la console (jamais envoyées aux participants).
#[derive(Debug, Clone, Serialize)]
pub struct InfoReseau {
    /// Adresse à taper sur le téléphone, par exemple « 192.168.1.37:8080/p ».
    pub adresse: String,
    /// Lien complet encodé dans le QR code de la séance.
    pub lien: String,
    /// QR code de la séance, en SVG.
    pub qr: String,
    /// Wi-Fi Padaone à rejoindre, quand le point d'accès est actif.
    pub wifi: Option<InfoWifi>,
}

#[derive(Debug, Clone, Serialize)]
pub struct InfoWifi {
    pub nom: String,
    /// Absent pour un réseau ouvert.
    pub mot_de_passe: Option<String>,
    /// QR code « Wi-Fi » lu par l'appareil photo des téléphones, en SVG.
    pub qr: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct MessageReseau {
    pub texte: String,
    pub erreur: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct Suggestion {
    pub nom_reseau: String,
    pub mot_de_passe: String,
}

/// Ce que la console voit du point d'accès (champ `point_acces` de sa vue).
#[derive(Debug, Clone, Serialize)]
pub struct EtatPointAcces {
    /// Vrai avec `--reseau-simule` : rien n'est réellement modifié.
    pub simule: bool,
    /// Opération en cours (« Démarrage du point d'accès… »).
    pub en_cours: Option<String>,
    /// Résultat de la dernière opération.
    pub message: Option<MessageReseau>,
    pub diagnostic: Option<Diagnostic>,
    /// Point d'accès démarré par cette séance (avec son mot de passe, pour l'animateur).
    pub actif: Option<PointAccesDemarre>,
    /// Appareils ayant reçu une adresse du point d'accès.
    pub appareils: Option<usize>,
    /// Arrêter le point d'accès et revenir au Wi-Fi précédent quand Padaone s'arrête.
    pub arret_fin_seance: bool,
    /// Adresse du PC annoncée aux téléphones avant tout point d'accès.
    pub adresse_initiale: String,
    pub suggestion: Suggestion,
    /// Réaction aux tests de connexion des téléphones (portail captif).
    pub portail: ModePortail,
    /// Padaone répond sur le port 80 (tests de connexion des téléphones).
    pub portail_port_80: bool,
    /// Padaone répond aux questions DNS transmises par dnsmasq.
    pub portail_dns: bool,
}

pub struct EtatReseau {
    pub info: InfoReseau,
    /// Augmente à chaque changement d'`info` : les écrans et consoles reçoivent alors `reseau`.
    pub version: u64,
    pub point_acces: EtatPointAcces,
    port: u16,
    code: String,
}

impl EtatReseau {
    pub fn new(ip: &str, port: u16, code: &str, simule: bool) -> Result<Self> {
        let mut e = EtatReseau {
            info: InfoReseau {
                adresse: String::new(),
                lien: String::new(),
                qr: String::new(),
                wifi: None,
            },
            version: 0,
            point_acces: EtatPointAcces {
                simule,
                en_cours: None,
                message: None,
                diagnostic: None,
                actif: None,
                appareils: None,
                arret_fin_seance: true,
                adresse_initiale: ip.to_string(),
                suggestion: suggerer(),
                portail: ModePortail::Ouvrir,
                portail_port_80: false,
                portail_dns: false,
            },
            port,
            code: code.to_string(),
        };
        e.annoncer(ip, None)?;
        Ok(e)
    }

    /// Adresse du PC à donner aux téléphones du point d'accès (réponses DNS du portail).
    pub fn ip_portail(&self) -> Option<[u8; 4]> {
        let ip = self
            .point_acces
            .actif
            .as_ref()
            .map_or(&self.point_acces.adresse_initiale, |a| &a.adresse);
        ip.parse::<std::net::Ipv4Addr>().ok().map(|i| i.octets())
    }

    /// Change l'adresse annoncée aux téléphones (et le Wi-Fi éventuel).
    fn annoncer(&mut self, ip: &str, wifi: Option<InfoWifi>) -> Result<()> {
        let hote = format!("{ip}:{}", self.port);
        let lien = format!("http://{hote}/p?code={}", self.code);
        self.info = InfoReseau { adresse: format!("{hote}/p"), qr: qr_svg(&lien)?, lien, wifi };
        self.version += 1;
        Ok(())
    }
}

fn suggerer() -> Suggestion {
    let h = getrandom::u32().unwrap_or(0);
    Suggestion {
        nom_reseau: suggestion::nom_reseau(h),
        mot_de_passe: suggestion::mot_de_passe(h / 7),
    }
}

/// QR code en SVG, encre de Padaone sur fond blanc, sans en-tête XML.
pub fn qr_svg(texte: &str) -> Result<String> {
    let code = QrCode::new(texte.as_bytes()).context("texte trop long pour un QR code")?;
    let svg = code
        .render::<svg::Color>()
        .min_dimensions(240, 240)
        .dark_color(svg::Color("#1b1530"))
        .light_color(svg::Color("#ffffff"))
        .build();
    Ok(svg.find("<svg").map_or(svg.clone(), |debut| svg[debut..].to_string()))
}

/// Traite une commande réseau. Renvoie un message d'erreur immédiat si elle est refusée.
pub fn lancer(partage: &Arc<Partage>, commande: CommandeReseau) -> Option<String> {
    let libelle = match &commande {
        CommandeReseau::PointAccesSuggerer => {
            partage.modifier_reseau(|r| r.point_acces.suggestion = suggerer());
            return None;
        }
        CommandeReseau::PointAccesOptions { arret_fin_seance, portail } => {
            partage.modifier_reseau(|r| {
                if let Some(a) = arret_fin_seance {
                    r.point_acces.arret_fin_seance = *a;
                }
                if let Some(p) = portail {
                    r.point_acces.portail = *p;
                }
            });
            return None;
        }
        CommandeReseau::Diagnostic => "Diagnostic des cartes Wi-Fi…",
        CommandeReseau::PointAccesDemarrer(_) => "Démarrage du point d'accès…",
        CommandeReseau::PointAccesArreter => "Arrêt du point d'accès…",
    };
    let deja = partage.modifier_reseau(|r| {
        if r.point_acces.en_cours.is_some() {
            return true;
        }
        r.point_acces.en_cours = Some(libelle.to_string());
        r.point_acces.message = None;
        false
    });
    if deja {
        return Some("Une opération réseau est déjà en cours.".into());
    }
    let precedente =
        partage.reseau().point_acces.actif.as_ref().and_then(|a| a.connexion_precedente.clone());
    let partage = Arc::clone(partage);
    tokio::spawn(async move {
        let ex = Arc::clone(&partage.executeur);
        let issue =
            tokio::task::spawn_blocking(move || executer(ex.as_ref(), commande, precedente)).await;
        let issue = issue.unwrap_or_else(|e| Issue::erreur(format!("tâche interrompue : {e}")));
        appliquer(&partage, issue);
    });
    None
}

/// Résultat d'une opération réseau, calculé hors du verrou.
struct Issue {
    diagnostic: Option<Diagnostic>,
    /// `Some(Some(pa))` : démarré ; `Some(None)` : arrêté ; `None` : inchangé.
    actif: Option<Option<PointAccesDemarre>>,
    message: Option<MessageReseau>,
}

impl Issue {
    fn erreur(texte: String) -> Self {
        Issue {
            diagnostic: None,
            actif: None,
            message: Some(MessageReseau { texte, erreur: true }),
        }
    }
}

fn executer(ex: &dyn Executeur, commande: CommandeReseau, precedente: Option<String>) -> Issue {
    let rediagnostiquer = || point_acces::diagnostiquer(ex).ok();
    match commande {
        CommandeReseau::Diagnostic => match point_acces::diagnostiquer(ex) {
            Ok(d) => Issue { diagnostic: Some(d), actif: None, message: None },
            Err(e) => Issue::erreur(majuscule(&e.to_string())),
        },
        CommandeReseau::PointAccesDemarrer(demande) => match point_acces::demarrer(ex, &demande) {
            Ok(pa) => {
                tracing::info!(
                    "point d'accès « {} » démarré sur {} ({})",
                    pa.nom_reseau,
                    pa.carte,
                    pa.adresse
                );
                Issue {
                    message: Some(MessageReseau {
                        texte: format!(
                            "Point d'accès « {} » actif sur {}. Adresse du PC : {}.",
                            pa.nom_reseau, pa.carte, pa.adresse
                        ),
                        erreur: false,
                    }),
                    actif: Some(Some(pa)),
                    diagnostic: rediagnostiquer(),
                }
            }
            Err(e) => {
                tracing::warn!("démarrage du point d'accès impossible : {e}");
                Issue { diagnostic: rediagnostiquer(), ..Issue::erreur(majuscule(&e.to_string())) }
            }
        },
        CommandeReseau::PointAccesArreter => {
            match point_acces::arreter(ex, precedente.as_deref()) {
                Ok(avertissement) => {
                    tracing::info!("point d'accès arrêté");
                    Issue {
                        message: Some(MessageReseau {
                            texte: avertissement.unwrap_or_else(|| "Point d'accès arrêté.".into()),
                            erreur: false,
                        }),
                        actif: Some(None),
                        diagnostic: rediagnostiquer(),
                    }
                }
                Err(e) => Issue {
                    diagnostic: rediagnostiquer(),
                    ..Issue::erreur(majuscule(&e.to_string()))
                },
            }
        }
        CommandeReseau::PointAccesSuggerer | CommandeReseau::PointAccesOptions { .. } => {
            Issue { diagnostic: None, actif: None, message: None }
        }
    }
}

fn appliquer(partage: &Partage, issue: Issue) {
    partage.modifier_reseau(|r| {
        r.point_acces.en_cours = None;
        if let Some(d) = issue.diagnostic {
            r.point_acces.diagnostic = Some(d);
        }
        r.point_acces.message = issue.message;
        match issue.actif {
            Some(Some(pa)) => {
                let wifi = info_wifi(&pa);
                let ip = pa.adresse.clone();
                r.point_acces.actif = Some(pa);
                if let Err(e) = r.annoncer(&ip, wifi) {
                    tracing::error!("QR code impossible : {e:#}");
                }
            }
            Some(None) => {
                r.point_acces.actif = None;
                r.point_acces.appareils = None;
                let ip = r.point_acces.adresse_initiale.clone();
                if let Err(e) = r.annoncer(&ip, None) {
                    tracing::error!("QR code impossible : {e:#}");
                }
            }
            None => {}
        }
    });
}

fn info_wifi(pa: &PointAccesDemarre) -> Option<InfoWifi> {
    let qr = qr_svg(&texte_qr_wifi(&pa.nom_reseau, &pa.securite)).ok()?;
    let mot_de_passe = match &pa.securite {
        Securite::Wpa2 { mot_de_passe } => Some(mot_de_passe.clone()),
        Securite::Ouvert => None,
    };
    Some(InfoWifi { nom: pa.nom_reseau.clone(), mot_de_passe, qr })
}

/// Tâche de fond : premier diagnostic au lancement (lecture seule), puis relevé régulier
/// du nombre d'appareils connectés au point d'accès.
pub async fn surveiller(partage: Arc<Partage>) {
    lancer(&partage, CommandeReseau::Diagnostic);
    let mut horloge = tokio::time::interval(RELEVE_APPAREILS);
    loop {
        horloge.tick().await;
        let carte = partage.reseau().point_acces.actif.as_ref().map(|a| a.carte.clone());
        let Some(carte) = carte else { continue };
        let ex = Arc::clone(&partage.executeur);
        let n = tokio::task::spawn_blocking(move || point_acces::appareils(ex.as_ref(), &carte))
            .await
            .ok()
            .flatten();
        if partage.reseau().point_acces.appareils != n {
            partage.modifier_reseau(|r| r.point_acces.appareils = n);
        }
    }
}

/// À l'arrêt de Padaone : éteint le point d'accès démarré par la séance, si l'animateur l'a demandé.
pub async fn arret_fin_de_seance(partage: Arc<Partage>) {
    let a_arreter = {
        let r = partage.reseau();
        r.point_acces
            .actif
            .as_ref()
            .filter(|_| r.point_acces.arret_fin_seance)
            .map(|a| a.connexion_precedente.clone())
    };
    let Some(precedente) = a_arreter else { return };
    println!("  Arrêt du point d'accès et retour au Wi-Fi précédent…");
    let ex = Arc::clone(&partage.executeur);
    match tokio::task::spawn_blocking(move || {
        point_acces::arreter(ex.as_ref(), precedente.as_deref())
    })
    .await
    {
        Ok(Ok(None)) => println!("  Point d'accès arrêté."),
        Ok(Ok(Some(avertissement))) => println!("  Point d'accès arrêté. {avertissement}"),
        Ok(Err(e)) => println!("  Arrêt du point d'accès impossible : {e}"),
        Err(e) => println!("  Arrêt du point d'accès interrompu : {e}"),
    }
}

fn majuscule(texte: &str) -> String {
    let mut c = texte.chars();
    match c.next() {
        Some(p) => p.to_uppercase().chain(c).collect::<String>() + ".",
        None => String::new(),
    }
}
