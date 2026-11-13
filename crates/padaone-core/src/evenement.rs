//! Événements de la séance.
//!
//! Chaque changement d'état est un événement. Le binaire les écrit dans le journal
//! (JSON Lines) et la reprise après plantage les rejoue dans l'ordre avec
//! [`crate::seance::Seance::rejouer`]. Les signaux discrets et les niveaux de
//! l'applaudimètre ne sont pas des événements : ils sont éphémères.

use serde::{Deserialize, Serialize};

use crate::activite::{IdParticipant, Rang};
use crate::seance::{Configuration, IdActivite, Reponse};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "e", rename_all = "snake_case")]
pub enum Evenement {
    /// Toujours le premier événement d'un journal.
    SeanceOuverte {
        titre: String,
        code: String,
    },
    TitreChange {
        titre: String,
    },
    /// Le jeton est anonyme : il permet seulement de retrouver un téléphone qui se reconnecte.
    ParticipantInscrit {
        id: IdParticipant,
        jeton: String,
    },
    ActiviteAjoutee {
        id: IdActivite,
        configuration: Configuration,
    },
    ActiviteSupprimee {
        id: IdActivite,
    },
    ActiviteLancee {
        id: IdActivite,
    },
    RetourAccueil,
    Revelee {
        id: IdActivite,
        on: bool,
    },
    Figee {
        id: IdActivite,
        on: bool,
    },
    Reinitialisee {
        id: IdActivite,
    },
    ReponseEnregistree {
        activite: IdActivite,
        participant: IdParticipant,
        reponse: Reponse,
    },
    MotMasque {
        id: IdActivite,
        cle: String,
        on: bool,
    },
    MotsFusionnes {
        id: IdActivite,
        de: String,
        vers: String,
    },
    MotSepare {
        id: IdActivite,
        cle: String,
    },
    /// Message de la question ouverte ou question du public validé (ou retiré) pour l'écran.
    ElementValide {
        id: IdActivite,
        rang: Rang,
        on: bool,
    },
    /// Minuterie démarrée (heure de fin en millisecondes depuis 1970, réponses rouvertes)
    /// ou arrêtée (`fin` absente). À l'échéance, le serveur fige l'activité (événement `figee`).
    MinuterieReglee {
        id: IdActivite,
        fin: Option<u64>,
    },
}
