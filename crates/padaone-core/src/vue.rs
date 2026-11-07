//! Vues envoyées aux clients, une par rôle.
//!
//! - Participant : l'activité en cours et sa propre réponse, rien sur les autres.
//! - Écran : le résultat seulement s'il est révélé **et** s'il atteint le seuil d'anonymat.
//! - Console : tout le déroulé et tous les résultats, mais jamais qui a répondu quoi,
//!   et les compteurs du signal discret.
//!
//! La bonne réponse d'un pari n'apparaît dans les vues du téléphone et de l'écran
//! qu'une fois le résultat révélé.

use serde::Serialize;

use crate::activite::IdParticipant;
use crate::affichable;
use crate::questions::DonneesQuestions;
use crate::seance::{Configuration, IdActivite, Reponse, Resultat, Seance};
use crate::signal::{CompteSignal, TypeSignal};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct VueParticipant {
    pub titre: String,
    pub activite: Option<ActiviteParticipant>,
    /// Signaux discrets actifs de ce participant.
    pub mes_signaux: Vec<TypeSignal>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ActiviteParticipant {
    pub id: IdActivite,
    #[serde(flatten)]
    pub configuration: Configuration,
    /// Faux quand les réponses sont figées.
    pub ouverte: bool,
    /// Vrai quand l'animateur a révélé le résultat (le pari montre alors la bonne réponse).
    pub revelee: bool,
    pub ma_reponse: Option<Reponse>,
    /// Questions du public : questions validées à voter et état des siennes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub donnees: Option<DonneesQuestions>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct VueEcran {
    pub titre: String,
    pub code: String,
    pub connectes: usize,
    pub activite: Option<ActiviteEcran>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ActiviteEcran {
    pub id: IdActivite,
    #[serde(flatten)]
    pub configuration: Configuration,
    pub revelee: bool,
    pub figee: bool,
    pub reponses: usize,
    /// Absent tant que l'activité n'est pas révélée ou compte moins de 3 réponses.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resultat: Option<Resultat>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct VueConsole {
    pub titre: String,
    pub code: String,
    pub connectes: usize,
    pub inscrits: usize,
    pub courante: Option<IdActivite>,
    pub deroule: Vec<EtapeConsole>,
    pub signaux: Vec<CompteSignal>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct EtapeConsole {
    pub id: IdActivite,
    #[serde(flatten)]
    pub configuration: Configuration,
    pub revelee: bool,
    pub figee: bool,
    pub reponses: usize,
    /// Vrai si le résultat atteint le seuil d'anonymat et peut donc être projeté.
    pub affichable: bool,
    pub resultat: Resultat,
}

impl Seance {
    /// `maintenant` : instant en millisecondes (pour l'expiration des signaux).
    pub fn vue_participant(&self, participant: IdParticipant, maintenant: u64) -> VueParticipant {
        let activite = self.courante.and_then(|id| self.etape(id)).map(|e| ActiviteParticipant {
            id: e.id,
            configuration: e.contenu.configuration().publique(e.revelee),
            ouverte: !e.figee,
            revelee: e.revelee,
            ma_reponse: e.contenu.reponse_de(participant),
            donnees: e.contenu.donnees_participant(participant),
        });
        VueParticipant {
            titre: self.titre.clone(),
            activite,
            mes_signaux: self.signaux.de(participant, maintenant),
        }
    }

    /// `connectes` : nombre de participants dont le téléphone est connecté en ce moment.
    pub fn vue_ecran(&self, connectes: usize) -> VueEcran {
        let activite = self.courante.and_then(|id| self.etape(id)).map(|e| {
            let reponses = e.contenu.nombre_de_reponses();
            ActiviteEcran {
                id: e.id,
                configuration: e.contenu.configuration().publique(e.revelee),
                revelee: e.revelee,
                figee: e.figee,
                reponses,
                resultat: (e.revelee && affichable(reponses))
                    .then(|| e.contenu.resultat().pour_ecran()),
            }
        });
        VueEcran { titre: self.titre.clone(), code: self.code.clone(), connectes, activite }
    }

    pub fn vue_console(&self, connectes: usize, maintenant: u64) -> VueConsole {
        let deroule = self
            .deroule
            .iter()
            .map(|e| {
                let reponses = e.contenu.nombre_de_reponses();
                EtapeConsole {
                    id: e.id,
                    configuration: e.contenu.configuration(),
                    revelee: e.revelee,
                    figee: e.figee,
                    reponses,
                    affichable: affichable(reponses),
                    resultat: e.contenu.resultat(),
                }
            })
            .collect();
        VueConsole {
            titre: self.titre.clone(),
            code: self.code.clone(),
            connectes,
            inscrits: self.nombre_d_inscrits(),
            courante: self.courante,
            deroule,
            signaux: self.signaux.compter(maintenant, connectes),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::seance::tests::seance_avec_sondage;
    use crate::seance::{Commande, Reponse, Resultat};
    use crate::sondage::ResultatSondage;

    #[test]
    fn ecran_masque_tant_que_non_revele() {
        let (mut s, _) = seance_avec_sondage();
        for p in 1..=3 {
            s.repondre(p, 1, Reponse::Choix(0)).unwrap();
        }
        let v = s.vue_ecran(3);
        assert_eq!(v.activite.as_ref().map(|a| a.reponses), Some(3));
        assert_eq!(v.activite.and_then(|a| a.resultat), None);
    }

    #[test]
    fn ecran_masque_sous_le_seuil_meme_revele() {
        let (mut s, _) = seance_avec_sondage();
        s.executer(Commande::Reveler { id: None, on: true }).unwrap();
        s.repondre(1, 1, Reponse::Choix(0)).unwrap();
        s.repondre(2, 1, Reponse::Choix(1)).unwrap();
        assert_eq!(s.vue_ecran(3).activite.and_then(|a| a.resultat), None);

        s.repondre(3, 1, Reponse::Choix(1)).unwrap();
        assert_eq!(
            s.vue_ecran(3).activite.and_then(|a| a.resultat),
            Some(Resultat::Sondage(ResultatSondage { effectifs: vec![1, 2, 0], total: 3 }))
        );
    }

    #[test]
    fn participant_ne_voit_que_sa_reponse() {
        let (mut s, _) = seance_avec_sondage();
        s.repondre(1, 1, Reponse::Choix(2)).unwrap();
        let v1 = s.vue_participant(1, 0).activite.unwrap();
        assert_eq!(v1.ma_reponse, Some(Reponse::Choix(2)));
        assert!(v1.ouverte);
        assert_eq!(s.vue_participant(2, 0).activite.unwrap().ma_reponse, None);
        let json = serde_json::to_string(&s.vue_participant(2, 0)).unwrap();
        assert!(!json.contains("effectifs"), "aucun résultat dans la vue participant");
    }

    #[test]
    fn console_voit_tout_avec_le_drapeau_affichable() {
        let (mut s, _) = seance_avec_sondage();
        s.repondre(1, 1, Reponse::Choix(0)).unwrap();
        let v = s.vue_console(1, 0);
        assert_eq!(v.deroule.len(), 1);
        assert!(!v.deroule[0].affichable);
        assert_eq!(
            v.deroule[0].resultat,
            Resultat::Sondage(ResultatSondage { effectifs: vec![1, 0, 0], total: 1 })
        );
    }

    #[test]
    fn format_json_de_la_vue_ecran() {
        let (s, _) = seance_avec_sondage();
        let json = serde_json::to_value(s.vue_ecran(2)).unwrap();
        assert_eq!(json["activite"]["type"], "sondage");
        assert_eq!(json["activite"]["options"][1], "Thé");
        assert!(json["activite"].get("resultat").is_none());
    }

    #[test]
    fn bonne_reponse_du_pari_cachee_jusqu_a_la_revelation() {
        use crate::pari::Pari;
        use crate::seance::{Configuration, Seance};
        let (mut s, _) = Seance::ouvrir("F", "1").unwrap();
        let pari =
            Pari { question: "Q".into(), options: vec!["A".into(), "B".into()], bonne: Some(1) };
        s.executer(Commande::Ajouter(Configuration::Pari(pari))).unwrap();
        s.executer(Commande::Lancer { id: 1 }).unwrap();
        s.inscrire("a");
        let json = serde_json::to_value(s.vue_participant(1, 0)).unwrap();
        assert!(json["activite"].get("bonne").is_none());
        let json = serde_json::to_value(s.vue_ecran(1)).unwrap();
        assert!(json["activite"].get("bonne").is_none());
        s.executer(Commande::Reveler { id: None, on: true }).unwrap();
        let json = serde_json::to_value(s.vue_participant(1, 0)).unwrap();
        assert_eq!(json["activite"]["bonne"], 1);
    }

    #[test]
    fn signaux_sur_la_console_seulement() {
        let (mut s, _) = seance_avec_sondage();
        s.signaler(1, crate::signal::TypeSignal::Perdu, 0).unwrap();
        let ecran = serde_json::to_string(&s.vue_ecran(3)).unwrap();
        assert!(!ecran.contains("perdu"));
        let console = s.vue_console(3, 0);
        assert_eq!(console.signaux.iter().map(|c| c.n).sum::<usize>(), 1);
        assert_eq!(s.vue_participant(1, 0).mes_signaux, vec![crate::signal::TypeSignal::Perdu]);
        assert!(s.vue_participant(2, 0).mes_signaux.is_empty());
    }
}
