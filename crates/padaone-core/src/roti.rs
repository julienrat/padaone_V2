//! ROTI (retour sur le temps investi) et mot de la fin : de 0 à 5 doigts,
//! puis un mot facultatif. L'écran montre la répartition, la moyenne et un petit nuage.
//! Les mots se modèrent comme ceux du nuage (masquer, fusionner).

use serde::{Deserialize, Serialize};

use crate::activite::{
    moyenne, question_valide, Activite, ErreurConfiguration, ErreurReponse, Rang,
};
use crate::nuage::{mots_valides, regrouper, ModerationMots, MotNuage};

/// Nombre de doigts au plus.
pub const ROTI_MAX: u8 = 5;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Roti {
    pub question: String,
    #[serde(skip)]
    pub moderation: ModerationMots,
}

impl Roti {
    pub fn validee(&self) -> Result<Self, ErreurConfiguration> {
        Ok(Roti {
            question: question_valide(&self.question)?,
            moderation: ModerationMots::default(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReponseRoti {
    pub valeur: u8,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mot: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ResultatRoti {
    /// Effectif de 0 à 5 doigts.
    pub effectifs: Vec<usize>,
    pub total: usize,
    pub moyenne: Option<f64>,
    pub mots: Vec<MotNuage>,
}

impl Activite for Roti {
    type Reponse = ReponseRoti;
    type Resultat = ResultatRoti;

    fn valider(&self, r: ReponseRoti) -> Result<ReponseRoti, ErreurReponse> {
        if r.valeur > ROTI_MAX {
            return Err(ErreurReponse::HorsEchelle);
        }
        let mot = match r.mot {
            Some(m) if !m.trim().is_empty() => mots_valides(vec![m], 1)?.into_iter().next(),
            _ => None,
        };
        Ok(ReponseRoti { valeur: r.valeur, mot })
    }

    fn resultat<'a>(
        &self,
        reponses: impl Iterator<Item = (Rang, &'a ReponseRoti)>,
    ) -> ResultatRoti {
        let reponses: Vec<&ReponseRoti> = reponses.map(|(_, r)| r).collect();
        let mut effectifs = vec![0; usize::from(ROTI_MAX) + 1];
        for r in &reponses {
            if let Some(e) = effectifs.get_mut(usize::from(r.valeur)) {
                *e += 1;
            }
        }
        let valeurs: Vec<f64> = reponses.iter().map(|r| f64::from(r.valeur)).collect();
        let mots: Vec<Vec<String>> =
            reponses.iter().map(|r| r.mot.iter().cloned().collect()).collect();
        ResultatRoti {
            total: reponses.len(),
            moyenne: moyenne(&valeurs),
            mots: regrouper(mots.iter().map(|m| m.as_slice()), &self.moderation),
            effectifs,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::activite::Collecte;

    fn r(valeur: u8, mot: Option<&str>) -> ReponseRoti {
        ReponseRoti { valeur, mot: mot.map(String::from) }
    }

    #[test]
    fn de_zero_a_cinq_doigts() {
        let roti = Roti { question: "Utile ?".into(), moderation: Default::default() };
        assert_eq!(roti.valider(r(6, None)), Err(ErreurReponse::HorsEchelle));
        assert_eq!(roti.valider(r(0, Some("  "))), Ok(r(0, None)), "mot vide : pas de mot");
    }

    #[test]
    fn repartition_moyenne_et_mots() {
        let mut c =
            Collecte::new(Roti { question: "Utile ?".into(), moderation: Default::default() });
        c.repondre(1, r(5, Some("Merci"))).unwrap();
        c.repondre(2, r(4, Some("merci !"))).unwrap();
        c.repondre(3, r(2, None)).unwrap();
        let res = c.resultat();
        assert_eq!(res.effectifs, vec![0, 0, 1, 0, 1, 1]);
        assert_eq!(res.moyenne, Some(3.7));
        assert_eq!(res.mots.len(), 1);
        assert_eq!(res.mots[0].n, 2);
    }
}
