//! Pari de certitude : une réponse parmi 2 à 6 options, plus un degré de certitude
//! (0 « je devine », 1 « je pense », 2 « sûr·e »).
//!
//! L'écran croise juste/faux et sûr/pas sûr (« je devine » et « je pense » comptent
//! comme « pas sûr ») : acquis solide, à consolider, fausses certitudes, doute utile.
//! La bonne réponse n'est envoyée aux téléphones et à l'écran qu'une fois révélée.

use serde::{Deserialize, Serialize};

use crate::activite::{
    choix_valide, options_valides, question_valide, Activite, ErreurConfiguration, ErreurReponse,
    Rang,
};
use crate::sondage::{OPTIONS_MAX, OPTIONS_MIN};

/// Degré de certitude le plus élevé (« sûr·e »).
pub const CERTITUDE_SURE: u8 = 2;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pari {
    pub question: String,
    pub options: Vec<String>,
    /// Indice de la bonne réponse. Absent des vues tant que le résultat n'est pas révélé.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bonne: Option<usize>,
}

impl Pari {
    pub fn validee(&self) -> Result<Self, ErreurConfiguration> {
        let options = options_valides(&self.options, OPTIONS_MIN, OPTIONS_MAX)?;
        match self.bonne {
            Some(b) if b < options.len() => {
                Ok(Pari { question: question_valide(&self.question)?, options, bonne: Some(b) })
            }
            _ => Err(ErreurConfiguration::BonneReponse),
        }
    }

    /// Copie sans la bonne réponse, pour les vues d'avant la révélation.
    pub fn sans_bonne_reponse(&self) -> Self {
        Pari { bonne: None, ..self.clone() }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReponsePari {
    pub choix: usize,
    pub certitude: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ResultatPari {
    pub effectifs: Vec<usize>,
    pub total: usize,
    pub bonne: usize,
    /// Juste et sûr·e.
    pub acquis_solide: usize,
    /// Juste mais pas sûr·e.
    pub a_consolider: usize,
    /// Faux et sûr·e : à reprendre en priorité.
    pub fausses_certitudes: usize,
    /// Faux et pas sûr·e.
    pub doute_utile: usize,
    /// Fausses certitudes réparties par option choisie.
    pub fausses_certitudes_par_option: Vec<usize>,
}

impl Activite for Pari {
    type Reponse = ReponsePari;
    type Resultat = ResultatPari;

    fn valider(&self, r: ReponsePari) -> Result<ReponsePari, ErreurReponse> {
        choix_valide(r.choix, self.options.len())?;
        if r.certitude > CERTITUDE_SURE {
            return Err(ErreurReponse::HorsEchelle);
        }
        Ok(r)
    }

    fn resultat<'a>(
        &self,
        reponses: impl Iterator<Item = (Rang, &'a ReponsePari)>,
    ) -> ResultatPari {
        let n = self.options.len();
        let bonne = self.bonne.unwrap_or(0);
        let mut r = ResultatPari {
            effectifs: vec![0; n],
            total: 0,
            bonne,
            acquis_solide: 0,
            a_consolider: 0,
            fausses_certitudes: 0,
            doute_utile: 0,
            fausses_certitudes_par_option: vec![0; n],
        };
        for (_, p) in reponses {
            let Some(e) = r.effectifs.get_mut(p.choix) else { continue };
            *e += 1;
            r.total += 1;
            let sur = p.certitude >= CERTITUDE_SURE;
            match (p.choix == bonne, sur) {
                (true, true) => r.acquis_solide += 1,
                (true, false) => r.a_consolider += 1,
                (false, true) => {
                    r.fausses_certitudes += 1;
                    r.fausses_certitudes_par_option[p.choix] += 1;
                }
                (false, false) => r.doute_utile += 1,
            }
        }
        r
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::activite::Collecte;

    fn pari() -> Pari {
        Pari {
            question: "Capitale ?".into(),
            options: vec!["Lyon".into(), "Paris".into(), "Nice".into()],
            bonne: Some(1),
        }
    }

    #[test]
    fn la_bonne_reponse_est_obligatoire_et_valide() {
        assert!(pari().validee().is_ok());
        let sans = Pari { bonne: None, ..pari() };
        assert_eq!(sans.validee(), Err(ErreurConfiguration::BonneReponse));
        let hors = Pari { bonne: Some(3), ..pari() };
        assert_eq!(hors.validee(), Err(ErreurConfiguration::BonneReponse));
    }

    #[test]
    fn certitude_de_zero_a_deux() {
        assert_eq!(
            pari().valider(ReponsePari { choix: 0, certitude: 3 }),
            Err(ErreurReponse::HorsEchelle)
        );
        assert_eq!(
            pari().valider(ReponsePari { choix: 3, certitude: 0 }),
            Err(ErreurReponse::ChoixInconnu)
        );
    }

    #[test]
    fn grille_juste_faux_par_certitude() {
        let mut c = Collecte::new(pari());
        let r = |choix, certitude| ReponsePari { choix, certitude };
        c.repondre(1, r(1, 2)).unwrap(); // acquis solide
        c.repondre(2, r(1, 1)).unwrap(); // à consolider (« je pense »)
        c.repondre(3, r(2, 2)).unwrap(); // fausse certitude sur Nice
        c.repondre(4, r(2, 2)).unwrap(); // idem
        c.repondre(5, r(0, 0)).unwrap(); // doute utile
        let res = c.resultat();
        assert_eq!(res.effectifs, vec![1, 2, 2]);
        assert_eq!((res.acquis_solide, res.a_consolider), (1, 1));
        assert_eq!((res.fausses_certitudes, res.doute_utile), (2, 1));
        assert_eq!(res.fausses_certitudes_par_option, vec![0, 0, 2]);
    }

    #[test]
    fn bonne_reponse_cachee() {
        let json = serde_json::to_value(pari().sans_bonne_reponse()).unwrap();
        assert!(json.get("bonne").is_none());
    }
}
