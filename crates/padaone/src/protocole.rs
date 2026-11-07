//! Messages WebSocket, en JSON avec un champ `t`.
//! Le format fait foi dans `docs/03-specification.md`, section « Protocole ».

use padaone_core::{Commande, IdActivite, Reponse, TypeSignal};

use crate::reseau::{CommandeReseau, InfoReseau};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Participant,
    Ecran,
    Console,
}

/// Message reçu d'un client.
#[derive(Debug, PartialEq, Deserialize)]
#[serde(tag = "t", rename_all = "snake_case")]
pub enum MessageClient {
    Bonjour {
        role: Role,
        #[serde(default)]
        jeton: Option<String>,
        #[serde(default)]
        code: Option<String>,
        #[serde(default)]
        cle: Option<String>,
    },
    Reponse {
        activite: IdActivite,
        #[serde(flatten)]
        reponse: Reponse,
    },
    /// Signal discret : active le signal, ou l'éteint s'il l'était déjà.
    Signal {
        #[serde(rename = "type")]
        genre: TypeSignal,
    },
    Ping,
    Commande(CommandeConsole),
    /// Messages prévus mais pas encore traités (`niveau`…) : ignorés.
    #[serde(other)]
    Autre,
}

#[derive(Debug, Serialize)]
pub struct InfoSeance {
    pub titre: String,
    pub code: String,
}

/// Commande de la console : pilotage de la séance (journalisé) ou du réseau (non journalisé).
#[derive(Debug, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum CommandeConsole {
    Seance(Commande),
    Reseau(CommandeReseau),
}

/// Message envoyé à un client. La vue `etat` est construite à part (voir [`etat`]).
#[derive(Debug, Serialize)]
#[serde(tag = "t", rename_all = "snake_case")]
pub enum MessageServeur {
    Bienvenue {
        #[serde(skip_serializing_if = "Option::is_none")]
        jeton: Option<String>,
        seance: InfoSeance,
        #[serde(skip_serializing_if = "Option::is_none")]
        reseau: Option<InfoReseau>,
    },
    /// `fatale` : le client ne doit pas se reconnecter (clé ou code faux).
    Erreur {
        message: String,
        fatale: bool,
    },
    Pong,
    /// Adresses et Wi-Fi mis à jour (écran et console), par exemple après le démarrage du point d'accès.
    Reseau(InfoReseau),
}

impl MessageServeur {
    pub fn erreur(message: impl Into<String>) -> Self {
        MessageServeur::Erreur { message: message.into(), fatale: false }
    }

    pub fn json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }
}

/// Message `etat` : la vue du rôle, à plat, avec `"t": "etat"`.
pub fn etat<V: Serialize>(vue: &V) -> String {
    etat_avec(vue, None)
}

/// Message `etat` avec un champ de plus (la console reçoit ainsi `point_acces`).
pub fn etat_avec<V: Serialize>(vue: &V, extra: Option<(&str, serde_json::Value)>) -> String {
    match serde_json::to_value(vue) {
        Ok(serde_json::Value::Object(mut champs)) => {
            champs.insert("t".into(), "etat".into());
            if let Some((nom, valeur)) = extra {
                champs.insert(nom.into(), valeur);
            }
            serde_json::Value::Object(champs).to_string()
        }
        _ => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lire(json: &str) -> MessageClient {
        serde_json::from_str(json).unwrap()
    }

    #[test]
    fn bonjour_participant() {
        assert_eq!(
            lire(r#"{"t":"bonjour","role":"participant","code":"482731"}"#),
            MessageClient::Bonjour {
                role: Role::Participant,
                jeton: None,
                code: Some("482731".into()),
                cle: None
            }
        );
    }

    #[test]
    fn reponse_a_un_sondage() {
        assert_eq!(
            lire(r#"{"t":"reponse","activite":3,"choix":1}"#),
            MessageClient::Reponse { activite: 3, reponse: Reponse::Choix(1) }
        );
    }

    #[test]
    fn commande_de_la_console() {
        assert_eq!(
            lire(r#"{"t":"commande","c":"lancer","id":2}"#),
            MessageClient::Commande(CommandeConsole::Seance(Commande::Lancer { id: 2 }))
        );
    }

    #[test]
    fn messages_futurs_ignores() {
        assert_eq!(lire(r#"{"t":"niveau","activite":1,"valeur":40}"#), MessageClient::Autre);
    }

    #[test]
    fn commandes_reseau() {
        assert_eq!(
            lire(r#"{"t":"commande","c":"point_acces_arreter"}"#),
            MessageClient::Commande(CommandeConsole::Reseau(CommandeReseau::PointAccesArreter))
        );
        let m = lire(
            r#"{"t":"commande","c":"point_acces_demarrer","nom_reseau":"Padaone-Tilleul",
                "securite":{"type":"wpa2","mot_de_passe":"tilleul-42"},"bande":"ghz2_4","adresse":"garder"}"#,
        );
        assert!(matches!(
            m,
            MessageClient::Commande(CommandeConsole::Reseau(CommandeReseau::PointAccesDemarrer(_)))
        ));
        // Une commande de séance reste une commande de séance.
        assert!(matches!(
            lire(r#"{"t":"commande","c":"accueil"}"#),
            MessageClient::Commande(CommandeConsole::Seance(Commande::Accueil))
        ));
    }

    #[test]
    fn signal_discret() {
        assert_eq!(
            lire(r#"{"t":"signal","type":"perdu"}"#),
            MessageClient::Signal { genre: TypeSignal::Perdu }
        );
    }

    #[test]
    fn reponses_des_nouvelles_activites() {
        let m = lire(r#"{"t":"reponse","activite":2,"point":{"x":10,"y":90}}"#);
        assert!(matches!(m, MessageClient::Reponse { activite: 2, reponse: Reponse::Point(_) }));
        let m = lire(r#"{"t":"reponse","activite":2,"mots":["écoute","respect"]}"#);
        assert!(matches!(m, MessageClient::Reponse { reponse: Reponse::Mots(_), .. }));
        let m = lire(r#"{"t":"reponse","activite":2,"vote":{"question":4,"on":true}}"#);
        assert!(matches!(m, MessageClient::Reponse { reponse: Reponse::Vote(_), .. }));
    }

    #[test]
    fn etat_porte_le_type() {
        #[derive(Serialize)]
        struct V {
            titre: &'static str,
        }
        let v: serde_json::Value = serde_json::from_str(&etat(&V { titre: "F" })).unwrap();
        assert_eq!(v["t"], "etat");
        assert_eq!(v["titre"], "F");
    }
}
