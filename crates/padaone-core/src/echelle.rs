//! Échelle : un curseur de 1 à 5 ou de 1 à 10. L'écran montre un histogramme et la médiane.
//!
//! Médiane : valeur centrale ; avec un nombre pair de réponses, moyenne des deux valeurs
//! centrales (elle peut donc tomber sur une demi-valeur, par exemple 3,5).

use serde::{Deserialize, Serialize};

use crate::activite::{
    mediane, moyenne, nettoyer, question_valide, Activite, ErreurConfiguration, ErreurReponse,
    Rang, LONGUEUR_OPTION_MAX,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Echelle {
    pub question: String,
    /// Valeur maximale : 5 ou 10. Le minimum est toujours 1.
    pub max: u8,
    /// Libellé du bas de l'échelle, par exemple « pas du tout ».
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub libelle_min: Option<String>,
    /// Libellé du haut de l'échelle, par exemple « tout à fait ».
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub libelle_max: Option<String>,
}

impl Echelle {
    pub fn validee(&self) -> Result<Self, ErreurConfiguration> {
        if self.max != 5 && self.max != 10 {
            return Err(ErreurConfiguration::Reglage);
        }
        let libelle = |l: &Option<String>| -> Result<Option<String>, ErreurConfiguration> {
            match l.as_deref().map(nettoyer).filter(|t| !t.is_empty()) {
                Some(t) if t.chars().count() > LONGUEUR_OPTION_MAX => {
                    Err(ErreurConfiguration::OptionTropLongue)
                }
                autre => Ok(autre),
            }
        };
        Ok(Echelle {
            question: question_valide(&self.question)?,
            max: self.max,
            libelle_min: libelle(&self.libelle_min)?,
            libelle_max: libelle(&self.libelle_max)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ResultatEchelle {
    /// Effectif de chaque valeur, de 1 à `max`.
    pub effectifs: Vec<usize>,
    pub total: usize,
    pub mediane: Option<f64>,
    pub moyenne: Option<f64>,
}

impl Activite for Echelle {
    type Reponse = u8;
    type Resultat = ResultatEchelle;

    fn valider(&self, valeur: u8) -> Result<u8, ErreurReponse> {
        if (1..=self.max).contains(&valeur) {
            Ok(valeur)
        } else {
            Err(ErreurReponse::HorsEchelle)
        }
    }

    fn resultat<'a>(&self, reponses: impl Iterator<Item = (Rang, &'a u8)>) -> ResultatEchelle {
        let mut effectifs = vec![0; usize::from(self.max)];
        let mut valeurs = Vec::new();
        for (_, &v) in reponses {
            if let Some(e) = effectifs.get_mut(usize::from(v).wrapping_sub(1)) {
                *e += 1;
                valeurs.push(f64::from(v));
            }
        }
        ResultatEchelle {
            total: valeurs.len(),
            moyenne: moyenne(&valeurs),
            mediane: mediane(&mut valeurs),
            effectifs,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::activite::Collecte;

    fn echelle(max: u8) -> Echelle {
        Echelle {
            question: "Utile ?".into(),
            max,
            libelle_min: None,
            libelle_max: Some(" Très ".into()),
        }
    }

    #[test]
    fn cinq_ou_dix_seulement() {
        assert!(echelle(5).validee().is_ok());
        assert!(echelle(10).validee().is_ok());
        assert_eq!(echelle(7).validee(), Err(ErreurConfiguration::Reglage));
        assert_eq!(echelle(5).validee().unwrap().libelle_max.as_deref(), Some("Très"));
    }

    #[test]
    fn valeurs_hors_echelle_refusees() {
        let e = echelle(5);
        assert_eq!(e.valider(0), Err(ErreurReponse::HorsEchelle));
        assert_eq!(e.valider(6), Err(ErreurReponse::HorsEchelle));
        assert_eq!(e.valider(5), Ok(5));
    }

    #[test]
    fn histogramme_et_mediane() {
        let mut c = Collecte::new(echelle(5));
        for (p, v) in [(1, 1), (2, 4), (3, 4), (4, 5)] {
            c.repondre(p, v).unwrap();
        }
        let r = c.resultat();
        assert_eq!(r.effectifs, vec![1, 0, 0, 2, 1]);
        assert_eq!(r.mediane, Some(4.0));
        assert_eq!(r.moyenne, Some(3.5));
        c.repondre(4, 2).unwrap();
        assert_eq!(c.resultat().mediane, Some(3.0), "nombre pair : moyenne des deux centrales");
    }
}
