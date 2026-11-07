//! Le trait commun à toutes les interactions et la collecte des réponses.
//!
//! Une interaction (sondage, nuage de mots, échelle…) décrit seulement :
//! - sa configuration (la structure qui implémente [`Activite`]) ;
//! - la forme d'une réponse, sa validation et son nettoyage ;
//! - le calcul du résultat agrégé.
//!
//! Le stockage des réponses (une par participant, remplaçable) est commun :
//! c'est [`Collecte`]. Ajouter une interaction ne touche ni au réseau ni au stockage.

use std::collections::BTreeMap;

use thiserror::Error;

/// Identifiant interne d'un participant. Il ne révèle rien de la personne :
/// c'est un simple numéro d'arrivée, jamais affiché ni envoyé à la console.
pub type IdParticipant = u32;

/// Numéro d'arrivée d'une réponse dans une activité (croissant, jamais réutilisé).
/// Sert à l'ordre d'affichage et à désigner un message à modérer sans dévoiler son auteur.
pub type Rang = u64;

/// Raison pour laquelle une réponse est refusée.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ErreurReponse {
    #[error("ce choix n'existe pas")]
    ChoixInconnu,
    #[error("cette valeur est hors de l'échelle")]
    HorsEchelle,
    #[error("la réponse est vide")]
    Vide,
    #[error("la réponse est trop longue")]
    TropLongue,
    #[error("trop de mots")]
    TropDeMots,
    #[error("vous avez atteint le nombre maximal de questions")]
    TropDeQuestions,
    #[error("cette question n'existe pas ou n'est pas encore validée")]
    QuestionInconnue,
    #[error("cette réponse ne correspond pas à l'activité en cours")]
    TypeIncompatible,
}

/// Raison pour laquelle une configuration d'activité est refusée.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ErreurConfiguration {
    #[error("la question est vide")]
    QuestionVide,
    #[error("la question dépasse {LONGUEUR_QUESTION_MAX} caractères")]
    QuestionTropLongue,
    #[error("il faut de {0} à {1} options")]
    NombreOptions(usize, usize),
    #[error("une option est vide")]
    OptionVide,
    #[error("une option dépasse {LONGUEUR_OPTION_MAX} caractères")]
    OptionTropLongue,
    #[error("deux options sont identiques")]
    OptionsEnDouble,
    #[error("la bonne réponse doit être l'une des options")]
    BonneReponse,
    #[error("réglage hors des limites permises")]
    Reglage,
}

/// Longueur maximale (en caractères) d'une question.
pub const LONGUEUR_QUESTION_MAX: usize = 200;
/// Longueur maximale (en caractères) d'une option, pour rester lisible à 10 m.
pub const LONGUEUR_OPTION_MAX: usize = 60;

/// Comportement propre à une interaction.
pub trait Activite {
    /// Réponse d'un participant.
    type Reponse: Clone;
    /// Résultat agrégé, calculé à partir de toutes les réponses.
    type Resultat;

    /// Vérifie une réponse et la renvoie nettoyée (espaces, casse…), prête à être stockée.
    fn valider(&self, reponse: Self::Reponse) -> Result<Self::Reponse, ErreurReponse>;

    /// Calcule le résultat agrégé à partir des réponses, dans l'ordre d'arrivée.
    /// Ne dépend jamais de l'identité des participants.
    fn resultat<'a>(
        &self,
        reponses: impl Iterator<Item = (Rang, &'a Self::Reponse)>,
    ) -> Self::Resultat
    where
        Self::Reponse: 'a;
}

/// Une activité et les réponses reçues, une par participant au plus.
#[derive(Debug, Clone, PartialEq)]
pub struct Collecte<A: Activite> {
    pub activite: A,
    reponses: BTreeMap<IdParticipant, (Rang, A::Reponse)>,
    prochain_rang: Rang,
}

impl<A: Activite> Collecte<A> {
    pub fn new(activite: A) -> Self {
        Self { activite, reponses: BTreeMap::new(), prochain_rang: 1 }
    }

    /// Enregistre la réponse d'un participant, ou remplace la précédente
    /// (elle prend alors un nouveau rang d'arrivée).
    pub fn repondre(
        &mut self,
        participant: IdParticipant,
        reponse: A::Reponse,
    ) -> Result<(), ErreurReponse> {
        let reponse = self.activite.valider(reponse)?;
        self.reponses.insert(participant, (self.prochain_rang, reponse));
        self.prochain_rang += 1;
        Ok(())
    }

    pub fn reponse_de(&self, participant: IdParticipant) -> Option<&A::Reponse> {
        self.reponses.get(&participant).map(|(_, r)| r)
    }

    pub fn nombre_de_reponses(&self) -> usize {
        self.reponses.len()
    }

    /// Efface les réponses. Les rangs continuent de croître.
    pub fn vider(&mut self) {
        self.reponses.clear();
    }

    pub fn resultat(&self) -> A::Resultat {
        let mut ordre: Vec<(Rang, &A::Reponse)> =
            self.reponses.values().map(|(rang, r)| (*rang, r)).collect();
        ordre.sort_by_key(|(rang, _)| *rang);
        self.activite.resultat(ordre.into_iter())
    }
}

/// Retire les espaces de début et de fin et réduit les espaces multiples.
pub fn nettoyer(texte: &str) -> String {
    texte.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Nettoie et valide une question (ou une consigne).
pub fn question_valide(question: &str) -> Result<String, ErreurConfiguration> {
    let q = nettoyer(question);
    if q.is_empty() {
        Err(ErreurConfiguration::QuestionVide)
    } else if q.chars().count() > LONGUEUR_QUESTION_MAX {
        Err(ErreurConfiguration::QuestionTropLongue)
    } else {
        Ok(q)
    }
}

/// Nettoie et valide une liste d'options : entre `min` et `max`, non vides, sans doublon.
pub fn options_valides(
    options: &[String],
    min: usize,
    max: usize,
) -> Result<Vec<String>, ErreurConfiguration> {
    let options: Vec<String> = options.iter().map(|o| nettoyer(o)).collect();
    if !(min..=max).contains(&options.len()) {
        return Err(ErreurConfiguration::NombreOptions(min, max));
    }
    for (i, option) in options.iter().enumerate() {
        if option.is_empty() {
            return Err(ErreurConfiguration::OptionVide);
        }
        if option.chars().count() > LONGUEUR_OPTION_MAX {
            return Err(ErreurConfiguration::OptionTropLongue);
        }
        let cle = option.to_lowercase();
        if options[..i].iter().any(|o| o.to_lowercase() == cle) {
            return Err(ErreurConfiguration::OptionsEnDouble);
        }
    }
    Ok(options)
}

/// Nettoie un texte libre de participant et vérifie sa longueur.
pub fn texte_valide(texte: &str, max: usize) -> Result<String, ErreurReponse> {
    let t = nettoyer(texte);
    if t.is_empty() {
        Err(ErreurReponse::Vide)
    } else if t.chars().count() > max {
        Err(ErreurReponse::TropLongue)
    } else {
        Ok(t)
    }
}

/// Indice valide dans une liste de `n` éléments.
pub fn choix_valide(choix: usize, n: usize) -> Result<usize, ErreurReponse> {
    if choix < n {
        Ok(choix)
    } else {
        Err(ErreurReponse::ChoixInconnu)
    }
}

/// Médiane d'une liste de valeurs (moyenne des deux valeurs centrales si le nombre est pair).
pub fn mediane(valeurs: &mut [f64]) -> Option<f64> {
    if valeurs.is_empty() {
        return None;
    }
    valeurs.sort_by(|a, b| a.total_cmp(b));
    let n = valeurs.len();
    Some(if n % 2 == 1 { valeurs[n / 2] } else { (valeurs[n / 2 - 1] + valeurs[n / 2]) / 2.0 })
}

/// Moyenne arrondie au dixième.
pub fn moyenne(valeurs: &[f64]) -> Option<f64> {
    if valeurs.is_empty() {
        None
    } else {
        let m = valeurs.iter().sum::<f64>() / valeurs.len() as f64;
        Some((m * 10.0).round() / 10.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mediane_paire_et_impaire() {
        assert_eq!(mediane(&mut []), None);
        assert_eq!(mediane(&mut [3.0, 1.0, 2.0]), Some(2.0));
        assert_eq!(mediane(&mut [4.0, 1.0, 2.0, 5.0]), Some(3.0));
    }

    #[test]
    fn moyenne_au_dixieme() {
        assert_eq!(moyenne(&[1.0, 2.0, 2.0]), Some(1.7));
    }

    #[test]
    fn options_controlees() {
        let o = |v: &[&str]| v.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        assert_eq!(options_valides(&o(&[" A ", "B"]), 2, 6), Ok(o(&["A", "B"])));
        assert_eq!(
            options_valides(&o(&["A"]), 2, 6),
            Err(ErreurConfiguration::NombreOptions(2, 6))
        );
        assert_eq!(
            options_valides(&o(&["A", "a"]), 2, 6),
            Err(ErreurConfiguration::OptionsEnDouble)
        );
    }
}
