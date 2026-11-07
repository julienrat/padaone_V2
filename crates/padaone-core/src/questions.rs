//! Questions du public : chacun pose jusqu'à 3 questions et vote pour celles des autres.
//! Une question n'est visible (téléphones et écran) qu'après validation par l'animateur.
//! L'écran les trie par votes.
//!
//! L'auteur d'une question n'est connu que du serveur, pour limiter le nombre de
//! questions et montrer à chacun l'état des siennes. Il n'apparaît dans aucune vue.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::activite::{
    question_valide, texte_valide, ErreurConfiguration, ErreurReponse, IdParticipant, Rang,
    LONGUEUR_QUESTION_MAX,
};

/// Questions par participant, au plus.
pub const QUESTIONS_PAR_PARTICIPANT: usize = 3;
/// Questions projetées au plus (les plus votées).
pub const QUESTIONS_ECRAN_MAX: usize = 6;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfigQuestions {
    /// Consigne, par exemple « Vos questions pour l'intervenante ».
    pub question: String,
}

impl ConfigQuestions {
    pub fn validee(&self) -> Result<Self, ErreurConfiguration> {
        Ok(ConfigQuestions { question: question_valide(&self.question)? })
    }
}

/// Vote d'un participant pour une question (ou retrait du vote).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Vote {
    pub question: Rang,
    pub on: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct QuestionPosee {
    rang: Rang,
    auteur: IdParticipant,
    texte: String,
    valide: bool,
    votes: BTreeSet<IdParticipant>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Questions {
    pub config: ConfigQuestions,
    liste: Vec<QuestionPosee>,
    prochain_rang: Rang,
}

/// Une question telle que la console (toutes) ou l'écran (validées) la voit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct QuestionVue {
    pub rang: Rang,
    pub texte: String,
    pub votes: usize,
    pub valide: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ResultatQuestions {
    /// Triées par votes décroissants, puis par ordre d'arrivée.
    pub questions: Vec<QuestionVue>,
    pub total: usize,
}

impl ResultatQuestions {
    pub fn pour_ecran(mut self) -> Self {
        self.questions.retain(|q| q.valide);
        self.questions.truncate(QUESTIONS_ECRAN_MAX);
        self
    }
}

/// Ce qu'un participant voit : les questions validées (pour voter) et l'état des siennes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DonneesQuestions {
    pub validees: Vec<QuestionAVoter>,
    pub miennes: Vec<MaQuestion>,
    pub restantes: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct QuestionAVoter {
    pub rang: Rang,
    pub texte: String,
    pub votes: usize,
    pub mon_vote: bool,
    pub de_moi: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MaQuestion {
    pub texte: String,
    pub valide: bool,
}

impl Questions {
    pub fn new(config: ConfigQuestions) -> Self {
        Questions { config, liste: Vec::new(), prochain_rang: 1 }
    }

    pub fn poser(&mut self, auteur: IdParticipant, texte: &str) -> Result<(), ErreurReponse> {
        let texte = texte_valide(texte, LONGUEUR_QUESTION_MAX)?;
        if self.liste.iter().filter(|q| q.auteur == auteur).count() >= QUESTIONS_PAR_PARTICIPANT {
            return Err(ErreurReponse::TropDeQuestions);
        }
        self.liste.push(QuestionPosee {
            rang: self.prochain_rang,
            auteur,
            texte,
            valide: false,
            votes: BTreeSet::new(),
        });
        self.prochain_rang += 1;
        Ok(())
    }

    /// Vote (ou retire son vote) pour une question validée.
    pub fn voter(&mut self, participant: IdParticipant, vote: Vote) -> Result<(), ErreurReponse> {
        let q = self
            .liste
            .iter_mut()
            .find(|q| q.rang == vote.question && q.valide)
            .ok_or(ErreurReponse::QuestionInconnue)?;
        if vote.on {
            q.votes.insert(participant);
        } else {
            q.votes.remove(&participant);
        }
        Ok(())
    }

    /// Validation par l'animateur (ou retrait de la validation).
    pub fn moderer(&mut self, rang: Rang, valide: bool) -> Result<(), ErreurReponse> {
        let q = self
            .liste
            .iter_mut()
            .find(|q| q.rang == rang)
            .ok_or(ErreurReponse::QuestionInconnue)?;
        q.valide = valide;
        Ok(())
    }

    /// Nombre de participants qui ont posé au moins une question.
    pub fn nombre_d_auteurs(&self) -> usize {
        self.liste.iter().map(|q| q.auteur).collect::<BTreeSet<_>>().len()
    }

    pub fn vider(&mut self) {
        self.liste.clear();
    }

    fn triees(&self) -> Vec<&QuestionPosee> {
        let mut l: Vec<&QuestionPosee> = self.liste.iter().collect();
        l.sort_by(|a, b| b.votes.len().cmp(&a.votes.len()).then(a.rang.cmp(&b.rang)));
        l
    }

    pub fn resultat(&self) -> ResultatQuestions {
        ResultatQuestions {
            questions: self
                .triees()
                .into_iter()
                .map(|q| QuestionVue {
                    rang: q.rang,
                    texte: q.texte.clone(),
                    votes: q.votes.len(),
                    valide: q.valide,
                })
                .collect(),
            total: self.liste.len(),
        }
    }

    pub fn donnees_participant(&self, p: IdParticipant) -> DonneesQuestions {
        let miennes: Vec<MaQuestion> = self
            .liste
            .iter()
            .filter(|q| q.auteur == p)
            .map(|q| MaQuestion { texte: q.texte.clone(), valide: q.valide })
            .collect();
        DonneesQuestions {
            validees: self
                .triees()
                .into_iter()
                .filter(|q| q.valide)
                .map(|q| QuestionAVoter {
                    rang: q.rang,
                    texte: q.texte.clone(),
                    votes: q.votes.len(),
                    mon_vote: q.votes.contains(&p),
                    de_moi: q.auteur == p,
                })
                .collect(),
            restantes: QUESTIONS_PAR_PARTICIPANT.saturating_sub(miennes.len()),
            miennes,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn questions() -> Questions {
        Questions::new(ConfigQuestions { question: "Vos questions".into() })
    }

    #[test]
    fn trois_questions_par_participant() {
        let mut q = questions();
        for i in 0..3 {
            q.poser(1, &format!("Question {i} ?")).unwrap();
        }
        assert_eq!(q.poser(1, "Encore ?"), Err(ErreurReponse::TropDeQuestions));
        assert!(q.poser(2, "Moi aussi ?").is_ok());
        assert_eq!(q.nombre_d_auteurs(), 2);
    }

    #[test]
    fn vote_seulement_sur_question_validee() {
        let mut q = questions();
        q.poser(1, "Pourquoi ?").unwrap();
        let vote = Vote { question: 1, on: true };
        assert_eq!(q.voter(2, vote), Err(ErreurReponse::QuestionInconnue));
        q.moderer(1, true).unwrap();
        q.voter(2, vote).unwrap();
        q.voter(2, vote).unwrap(); // un seul vote par personne
        assert_eq!(q.resultat().questions[0].votes, 1);
        q.voter(2, Vote { question: 1, on: false }).unwrap();
        assert_eq!(q.resultat().questions[0].votes, 0);
    }

    #[test]
    fn tri_par_votes_et_ecran_filtre() {
        let mut q = questions();
        q.poser(1, "A ?").unwrap();
        q.poser(2, "B ?").unwrap();
        q.poser(3, "C ?").unwrap();
        for r in 1..=3 {
            q.moderer(r, r != 3).unwrap();
        }
        q.voter(3, Vote { question: 2, on: true }).unwrap();
        let ecran = q.resultat().pour_ecran();
        let textes: Vec<&str> = ecran.questions.iter().map(|x| x.texte.as_str()).collect();
        assert_eq!(textes, vec!["B ?", "A ?"]);
    }

    #[test]
    fn le_participant_voit_ses_questions_sans_les_autres_en_attente() {
        let mut q = questions();
        q.poser(1, "La mienne ?").unwrap();
        q.poser(2, "Pas encore validée ?").unwrap();
        let d = q.donnees_participant(1);
        assert_eq!(d.miennes, vec![MaQuestion { texte: "La mienne ?".into(), valide: false }]);
        assert!(d.validees.is_empty());
        assert_eq!(d.restantes, 2);
    }
}
