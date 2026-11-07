//! Question ouverte modérée : chaque participant envoie un texte court
//! (il peut le remplacer). Rien n'est projeté sans la validation de l'animateur.
//!
//! Un message est désigné par son rang d'arrivée, jamais par son auteur. Si le
//! participant modifie son texte, le nouveau message doit être validé à nouveau.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::activite::{
    question_valide, texte_valide, Activite, ErreurConfiguration, ErreurReponse, Rang,
};

/// Longueur maximale d'un message, en caractères.
pub const LONGUEUR_MESSAGE_MAX: usize = 280;
/// Nombre de messages validés projetés au plus (les plus récents).
pub const MESSAGES_ECRAN_MAX: usize = 6;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ouverte {
    pub question: String,
    /// Rangs des messages validés par l'animateur.
    #[serde(skip)]
    pub valides: BTreeSet<Rang>,
}

impl Ouverte {
    pub fn validee(&self) -> Result<Self, ErreurConfiguration> {
        Ok(Ouverte { question: question_valide(&self.question)?, valides: BTreeSet::new() })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MessageOuvert {
    pub rang: Rang,
    pub texte: String,
    pub valide: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ResultatOuverte {
    pub messages: Vec<MessageOuvert>,
    pub total: usize,
}

impl ResultatOuverte {
    /// Pour l'écran : seulement les messages validés, les plus récents.
    pub fn pour_ecran(mut self) -> Self {
        self.messages.retain(|m| m.valide);
        let surplus = self.messages.len().saturating_sub(MESSAGES_ECRAN_MAX);
        self.messages.drain(..surplus);
        self
    }
}

impl Activite for Ouverte {
    type Reponse = String;
    type Resultat = ResultatOuverte;

    fn valider(&self, texte: String) -> Result<String, ErreurReponse> {
        texte_valide(&texte, LONGUEUR_MESSAGE_MAX)
    }

    fn resultat<'a>(&self, reponses: impl Iterator<Item = (Rang, &'a String)>) -> ResultatOuverte {
        let messages: Vec<MessageOuvert> = reponses
            .map(|(rang, texte)| MessageOuvert {
                rang,
                texte: texte.clone(),
                valide: self.valides.contains(&rang),
            })
            .collect();
        ResultatOuverte { total: messages.len(), messages }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::activite::Collecte;

    #[test]
    fn rien_a_l_ecran_sans_validation() {
        let mut c =
            Collecte::new(Ouverte { question: "Une idée ?".into(), valides: BTreeSet::new() });
        c.repondre(1, "Plus de pauses".into()).unwrap();
        c.repondre(2, "Des exemples".into()).unwrap();
        assert!(c.resultat().pour_ecran().messages.is_empty());

        let rang = c.resultat().messages[1].rang;
        c.activite.valides.insert(rang);
        let ecran = c.resultat().pour_ecran();
        assert_eq!(ecran.messages.len(), 1);
        assert_eq!(ecran.messages[0].texte, "Des exemples");
    }

    #[test]
    fn un_message_modifie_doit_etre_revalide() {
        let mut c = Collecte::new(Ouverte { question: "Q".into(), valides: BTreeSet::new() });
        c.repondre(1, "Premier jet".into()).unwrap();
        let rang = c.resultat().messages[0].rang;
        c.activite.valides.insert(rang);
        c.repondre(1, "Autre chose".into()).unwrap();
        assert!(c.resultat().pour_ecran().messages.is_empty());
    }

    #[test]
    fn message_vide_ou_trop_long_refuse() {
        let o = Ouverte { question: "Q".into(), valides: BTreeSet::new() };
        assert_eq!(o.valider("   ".into()), Err(ErreurReponse::Vide));
        assert_eq!(o.valider("x".repeat(281)), Err(ErreurReponse::TropLongue));
    }
}
