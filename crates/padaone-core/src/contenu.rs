//! Configurations, réponses et résultats de toutes les interactions, et le contenu
//! d'une étape du déroulé (la collecte propre à chaque interaction).
//!
//! Ajouter une interaction : une variante dans [`Configuration`], [`Reponse`] (si sa forme
//! est nouvelle), [`Resultat`] et `Contenu`, puis compléter les `match` de ce fichier.

use serde::{Deserialize, Serialize};

use crate::activite::{Collecte, ErreurConfiguration, ErreurReponse, IdParticipant, Rang};
use crate::echelle::{Echelle, ResultatEchelle};
use crate::meteo::{Meteo, Point, ResultatMeteo};
use crate::nuage::{self, ModerationMots, Nuage, ResultatNuage};
use crate::ordre::{Ordre, ResultatOrdre};
use crate::ouverte::{Ouverte, ResultatOuverte};
use crate::pari::{Pari, ReponsePari, ResultatPari};
use crate::questions::{ConfigQuestions, DonneesQuestions, Questions, ResultatQuestions, Vote};
use crate::roti::{ReponseRoti, ResultatRoti, Roti};
use crate::sondage::{ResultatSondage, Sondage};

/// Configuration d'une activité, telle que la console la crée.
/// En JSON : `{"type": "sondage", "question": "…", "options": ["…", "…"]}`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Configuration {
    Sondage(Sondage),
    /// Même configuration qu'un sondage ; le téléphone devient un carton de couleur.
    Cartons(Sondage),
    Nuage(Nuage),
    Echelle(Echelle),
    Meteo(Meteo),
    Pari(Pari),
    Roti(Roti),
    Ouverte(Ouverte),
    Questions(ConfigQuestions),
    /// Remise en ordre par glisser-déposer.
    Ordre(Ordre),
}

impl Configuration {
    /// Nettoie et valide la configuration.
    pub fn validee(&self) -> Result<Self, ErreurConfiguration> {
        Ok(match self {
            Configuration::Sondage(s) => {
                Configuration::Sondage(Sondage::new(&s.question, &s.options)?)
            }
            Configuration::Cartons(s) => {
                Configuration::Cartons(Sondage::new(&s.question, &s.options)?)
            }
            Configuration::Nuage(n) => Configuration::Nuage(n.validee()?),
            Configuration::Echelle(e) => Configuration::Echelle(e.validee()?),
            Configuration::Meteo(m) => Configuration::Meteo(m.validee()?),
            Configuration::Pari(p) => Configuration::Pari(p.validee()?),
            Configuration::Roti(r) => Configuration::Roti(r.validee()?),
            Configuration::Ouverte(o) => Configuration::Ouverte(o.validee()?),
            Configuration::Questions(q) => Configuration::Questions(q.validee()?),
            Configuration::Ordre(o) => Configuration::Ordre(o.validee()?),
        })
    }

    /// Durée de la minuterie de l'activité, en secondes, si elle en a une.
    pub fn duree_minuterie(&self) -> Option<u32> {
        match self {
            Configuration::Ordre(o) => o.duree,
            _ => None,
        }
    }

    /// Version montrée aux téléphones et à l'écran : sans la bonne réponse du pari
    /// tant que le résultat n'est pas révélé.
    pub fn publique(&self, revelee: bool) -> Self {
        match self {
            Configuration::Pari(p) if !revelee => Configuration::Pari(p.sans_bonne_reponse()),
            Configuration::Ordre(o) => Configuration::Ordre(o.publique(revelee)),
            autre => autre.clone(),
        }
    }
}

/// Réponse d'un participant. En JSON, un seul champ qui dit sa forme :
/// `{"choix": 1}`, `{"mots": ["…"]}`, `{"valeur": 4}`, `{"point": {"x": 20, "y": 80}}`,
/// `{"pari": {"choix": 0, "certitude": 2}}`, `{"roti": {"valeur": 4, "mot": "…"}}`,
/// `{"texte": "…"}`, `{"question": "…"}`, `{"vote": {"question": 3, "on": true}}`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Reponse {
    Choix(usize),
    Mots(Vec<String>),
    Valeur(u8),
    Point(Point),
    Pari(ReponsePari),
    Roti(ReponseRoti),
    Texte(String),
    Question(String),
    Vote(Vote),
    /// Remise en ordre : positions affichées, dans l'ordre proposé.
    Ordre(Vec<usize>),
}

/// Résultat agrégé d'une activité (sérialisé sans étiquette : la vue porte déjà le type).
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
pub enum Resultat {
    Sondage(ResultatSondage),
    Nuage(ResultatNuage),
    Echelle(ResultatEchelle),
    Meteo(ResultatMeteo),
    Pari(ResultatPari),
    Roti(ResultatRoti),
    Ouverte(ResultatOuverte),
    Questions(ResultatQuestions),
    Ordre(ResultatOrdre),
}

impl Resultat {
    /// Version projetable : sans les mots masqués ni les messages non validés.
    pub fn pour_ecran(self) -> Self {
        match self {
            Resultat::Nuage(mut r) => {
                r.mots = nuage::pour_ecran(r.mots);
                Resultat::Nuage(r)
            }
            Resultat::Roti(mut r) => {
                r.mots = nuage::pour_ecran(r.mots);
                Resultat::Roti(r)
            }
            Resultat::Ouverte(r) => Resultat::Ouverte(r.pour_ecran()),
            Resultat::Questions(r) => Resultat::Questions(r.pour_ecran()),
            autre => autre,
        }
    }
}

/// Contenu d'une étape : la collecte propre à chaque interaction.
#[derive(Debug, Clone)]
pub(crate) enum Contenu {
    Sondage(Collecte<Sondage>),
    Cartons(Collecte<Sondage>),
    Nuage(Collecte<Nuage>),
    Echelle(Collecte<Echelle>),
    Meteo(Collecte<Meteo>),
    Pari(Collecte<Pari>),
    Roti(Collecte<Roti>),
    Ouverte(Collecte<Ouverte>),
    Questions(Questions),
    Ordre(Collecte<Ordre>),
}

/// Applique la même expression à toutes les collectes ; `Questions` a son propre traitement.
macro_rules! selon_collecte {
    ($contenu:expr, $c:ident => $e:expr, $q:ident => $eq:expr) => {
        match $contenu {
            Contenu::Sondage($c) | Contenu::Cartons($c) => $e,
            Contenu::Nuage($c) => $e,
            Contenu::Echelle($c) => $e,
            Contenu::Meteo($c) => $e,
            Contenu::Pari($c) => $e,
            Contenu::Roti($c) => $e,
            Contenu::Ouverte($c) => $e,
            Contenu::Ordre($c) => $e,
            Contenu::Questions($q) => $eq,
        }
    };
}

impl Contenu {
    /// `configuration` doit avoir été validée.
    pub(crate) fn new(configuration: &Configuration) -> Self {
        match configuration {
            Configuration::Sondage(s) => Contenu::Sondage(Collecte::new(s.clone())),
            Configuration::Cartons(s) => Contenu::Cartons(Collecte::new(s.clone())),
            Configuration::Nuage(n) => Contenu::Nuage(Collecte::new(n.clone())),
            Configuration::Echelle(e) => Contenu::Echelle(Collecte::new(e.clone())),
            Configuration::Meteo(m) => Contenu::Meteo(Collecte::new(m.clone())),
            Configuration::Pari(p) => Contenu::Pari(Collecte::new(p.clone())),
            Configuration::Roti(r) => Contenu::Roti(Collecte::new(r.clone())),
            Configuration::Ouverte(o) => Contenu::Ouverte(Collecte::new(o.clone())),
            Configuration::Ordre(o) => Contenu::Ordre(Collecte::new(o.clone())),
            Configuration::Questions(q) => Contenu::Questions(Questions::new(q.clone())),
        }
    }

    pub(crate) fn configuration(&self) -> Configuration {
        match self {
            Contenu::Sondage(c) => Configuration::Sondage(c.activite.clone()),
            Contenu::Cartons(c) => Configuration::Cartons(c.activite.clone()),
            Contenu::Nuage(c) => Configuration::Nuage(c.activite.clone()),
            Contenu::Echelle(c) => Configuration::Echelle(c.activite.clone()),
            Contenu::Meteo(c) => Configuration::Meteo(c.activite.clone()),
            Contenu::Pari(c) => Configuration::Pari(c.activite.clone()),
            Contenu::Roti(c) => Configuration::Roti(c.activite.clone()),
            Contenu::Ouverte(c) => Configuration::Ouverte(c.activite.clone()),
            Contenu::Ordre(c) => Configuration::Ordre(c.activite.clone()),
            Contenu::Questions(q) => Configuration::Questions(q.config.clone()),
        }
    }

    /// Nombre de participants qui ont répondu (auteurs de questions pour les questions du public).
    pub(crate) fn nombre_de_reponses(&self) -> usize {
        selon_collecte!(self, c => c.nombre_de_reponses(), q => q.nombre_d_auteurs())
    }

    pub(crate) fn reponse_de(&self, p: IdParticipant) -> Option<Reponse> {
        match self {
            Contenu::Sondage(c) | Contenu::Cartons(c) => {
                c.reponse_de(p).map(|&i| Reponse::Choix(i))
            }
            Contenu::Nuage(c) => c.reponse_de(p).map(|m| Reponse::Mots(m.clone())),
            Contenu::Echelle(c) => c.reponse_de(p).map(|&v| Reponse::Valeur(v)),
            Contenu::Meteo(c) => c.reponse_de(p).map(|&pt| Reponse::Point(pt)),
            Contenu::Pari(c) => c.reponse_de(p).map(|&r| Reponse::Pari(r)),
            Contenu::Roti(c) => c.reponse_de(p).map(|r| Reponse::Roti(r.clone())),
            Contenu::Ouverte(c) => c.reponse_de(p).map(|t| Reponse::Texte(t.clone())),
            Contenu::Ordre(c) => c.reponse_de(p).map(|o| Reponse::Ordre(o.clone())),
            Contenu::Questions(_) => None,
        }
    }

    /// Données propres au participant, au-delà de sa réponse (questions du public).
    pub(crate) fn donnees_participant(&self, p: IdParticipant) -> Option<DonneesQuestions> {
        match self {
            Contenu::Questions(q) => Some(q.donnees_participant(p)),
            _ => None,
        }
    }

    pub(crate) fn resultat(&self) -> Resultat {
        match self {
            Contenu::Sondage(c) | Contenu::Cartons(c) => Resultat::Sondage(c.resultat()),
            Contenu::Nuage(c) => Resultat::Nuage(c.resultat()),
            Contenu::Echelle(c) => Resultat::Echelle(c.resultat()),
            Contenu::Meteo(c) => Resultat::Meteo(c.resultat()),
            Contenu::Pari(c) => Resultat::Pari(c.resultat()),
            Contenu::Roti(c) => Resultat::Roti(c.resultat()),
            Contenu::Ouverte(c) => Resultat::Ouverte(c.resultat()),
            Contenu::Ordre(c) => Resultat::Ordre(c.resultat()),
            Contenu::Questions(q) => Resultat::Questions(q.resultat()),
        }
    }

    pub(crate) fn repondre(
        &mut self,
        p: IdParticipant,
        reponse: &Reponse,
    ) -> Result<(), ErreurReponse> {
        match (self, reponse) {
            (Contenu::Sondage(c) | Contenu::Cartons(c), Reponse::Choix(i)) => c.repondre(p, *i),
            (Contenu::Nuage(c), Reponse::Mots(m)) => c.repondre(p, m.clone()),
            (Contenu::Echelle(c), Reponse::Valeur(v)) => c.repondre(p, *v),
            (Contenu::Meteo(c), Reponse::Point(pt)) => c.repondre(p, *pt),
            (Contenu::Pari(c), Reponse::Pari(r)) => c.repondre(p, *r),
            (Contenu::Roti(c), Reponse::Roti(r)) => c.repondre(p, r.clone()),
            (Contenu::Ouverte(c), Reponse::Texte(t)) => c.repondre(p, t.clone()),
            (Contenu::Ordre(c), Reponse::Ordre(o)) => c.repondre(p, o.clone()),
            (Contenu::Questions(q), Reponse::Question(t)) => q.poser(p, t),
            (Contenu::Questions(q), Reponse::Vote(v)) => q.voter(p, *v),
            _ => Err(ErreurReponse::TypeIncompatible),
        }
    }

    /// Efface les réponses et la modération.
    pub(crate) fn vider(&mut self) {
        match self {
            Contenu::Nuage(c) => c.activite.moderation.vider(),
            Contenu::Roti(c) => c.activite.moderation.vider(),
            Contenu::Ouverte(c) => c.activite.valides.clear(),
            _ => {}
        }
        selon_collecte!(self, c => c.vider(), q => q.vider())
    }

    /// Modération des mots (nuage, mot de la fin du ROTI).
    pub(crate) fn moderation_mots(&mut self) -> Option<&mut ModerationMots> {
        match self {
            Contenu::Nuage(c) => Some(&mut c.activite.moderation),
            Contenu::Roti(c) => Some(&mut c.activite.moderation),
            _ => None,
        }
    }

    /// Valide (ou retire) un message de la question ouverte ou une question du public.
    /// Renvoie `None` si l'activité ne se modère pas ainsi.
    pub(crate) fn valider_element(
        &mut self,
        rang: Rang,
        on: bool,
    ) -> Option<Result<(), ErreurReponse>> {
        match self {
            Contenu::Ouverte(c) => {
                if on && !c.resultat().messages.iter().any(|m| m.rang == rang) {
                    return Some(Err(ErreurReponse::QuestionInconnue));
                }
                if on {
                    c.activite.valides.insert(rang);
                } else {
                    c.activite.valides.remove(&rang);
                }
                Some(Ok(()))
            }
            Contenu::Questions(q) => Some(q.moderer(rang, on)),
            _ => None,
        }
    }
}
