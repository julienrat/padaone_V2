//! Sondage : chaque participant choisit une option parmi 2 à 6.
//! L'écran affiche des barres horizontales avec les effectifs, une fois révélé.
//!
//! Les **cartons** utilisent la même mécanique : seul l'affichage change
//! (le téléphone devient un carton de couleur plein écran).

use serde::{Deserialize, Serialize};

use crate::activite::{
    choix_valide, options_valides, question_valide, Activite, ErreurConfiguration, ErreurReponse,
    Rang,
};

/// Nombre d'options autorisé : 6 éléments au plus par graphique projeté.
pub const OPTIONS_MIN: usize = 2;
pub const OPTIONS_MAX: usize = 6;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sondage {
    pub question: String,
    pub options: Vec<String>,
}

impl Sondage {
    /// Crée un sondage après nettoyage (espaces en trop) et validation.
    pub fn new(question: &str, options: &[String]) -> Result<Self, ErreurConfiguration> {
        Ok(Self {
            question: question_valide(question)?,
            options: options_valides(options, OPTIONS_MIN, OPTIONS_MAX)?,
        })
    }
}

/// Résultat d'un sondage : effectif de chaque option, dans l'ordre des options.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ResultatSondage {
    pub effectifs: Vec<usize>,
    pub total: usize,
}

/// Compte les choix de chaque option (choix hors limites ignorés).
pub fn compter(n: usize, choix: impl Iterator<Item = usize>) -> ResultatSondage {
    let mut effectifs = vec![0; n];
    let mut total = 0;
    for c in choix {
        if let Some(e) = effectifs.get_mut(c) {
            *e += 1;
            total += 1;
        }
    }
    ResultatSondage { effectifs, total }
}

impl Activite for Sondage {
    /// Indice de l'option choisie.
    type Reponse = usize;
    type Resultat = ResultatSondage;

    fn valider(&self, choix: usize) -> Result<usize, ErreurReponse> {
        choix_valide(choix, self.options.len())
    }

    fn resultat<'a>(&self, reponses: impl Iterator<Item = (Rang, &'a usize)>) -> ResultatSondage {
        compter(self.options.len(), reponses.map(|(_, &c)| c))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::activite::Collecte;

    fn options(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    fn sondage() -> Sondage {
        Sondage::new("Votre format préféré ?", &options(&["Atelier", "Exposé", "Jeu"])).unwrap()
    }

    #[test]
    fn configuration_nettoyee() {
        let s = Sondage::new("  Une   question ? ", &options(&[" Oui ", "Non"])).unwrap();
        assert_eq!(s.question, "Une question ?");
        assert_eq!(s.options, options(&["Oui", "Non"]));
    }

    #[test]
    fn de_deux_a_six_options() {
        let erreur = Err(ErreurConfiguration::NombreOptions(2, 6));
        assert_eq!(Sondage::new("Q", &options(&["A"])), erreur);
        let sept = options(&["A", "B", "C", "D", "E", "F", "G"]);
        assert_eq!(Sondage::new("Q", &sept), erreur);
        assert!(Sondage::new("Q", &options(&["A", "B", "C", "D", "E", "F"])).is_ok());
    }

    #[test]
    fn configurations_invalides_refusees() {
        assert_eq!(
            Sondage::new("  ", &options(&["A", "B"])),
            Err(ErreurConfiguration::QuestionVide)
        );
        assert_eq!(Sondage::new("Q", &options(&["A", " "])), Err(ErreurConfiguration::OptionVide));
        assert_eq!(
            Sondage::new("Q", &options(&["Oui", "oui"])),
            Err(ErreurConfiguration::OptionsEnDouble)
        );
    }

    #[test]
    fn une_reponse_par_participant_et_remplacable() {
        let mut c = Collecte::new(sondage());
        c.repondre(1, 0).unwrap();
        c.repondre(2, 2).unwrap();
        c.repondre(1, 2).unwrap(); // le participant 1 change d'avis
        assert_eq!(c.nombre_de_reponses(), 2);
        assert_eq!(c.reponse_de(1), Some(&2));
        assert_eq!(c.resultat(), ResultatSondage { effectifs: vec![0, 0, 2], total: 2 });
    }

    #[test]
    fn choix_inconnu_refuse() {
        let mut c = Collecte::new(sondage());
        assert_eq!(c.repondre(1, 3), Err(ErreurReponse::ChoixInconnu));
        assert_eq!(c.nombre_de_reponses(), 0);
    }
}
