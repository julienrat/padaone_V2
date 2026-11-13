//! La séance : son titre, son code, ses participants anonymes et son déroulé.
//!
//! Tout changement durable passe par [`Seance::appliquer`], qui valide puis applique un
//! [`Evenement`]. Les commandes de la console et les réponses des participants
//! sont d'abord traduites en événements : le binaire n'a plus qu'à les journaliser.
//! Les signaux discrets, éphémères, sont à part ([`Seance::signaler`]).

use std::collections::BTreeMap;

use serde::Deserialize;
use thiserror::Error;

use crate::activite::{ErreurConfiguration, ErreurReponse, IdParticipant, Rang};
use crate::contenu::Contenu;
pub use crate::contenu::{Configuration, Reponse, Resultat};
use crate::evenement::Evenement;
use crate::signal::{Signaux, TypeSignal};

/// Identifiant d'une activité du déroulé (entier, comme dans le protocole).
pub type IdActivite = u32;

/// Longueur maximale du titre de la séance.
pub const LONGUEUR_TITRE_MAX: usize = 80;

/// Commandes de la console. En JSON, le champ `c` donne la commande.
/// Quand `id` est absent, la commande vise l'activité en cours.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(tag = "c", rename_all = "snake_case")]
pub enum Commande {
    Ajouter(Configuration),
    Supprimer {
        id: IdActivite,
    },
    Lancer {
        id: IdActivite,
    },
    Accueil,
    Reveler {
        #[serde(default)]
        id: Option<IdActivite>,
        on: bool,
    },
    /// Minuterie : `on` la (re)démarre pour sa durée et rouvre les réponses, sinon l'arrête.
    /// À exécuter avec l'heure ([`Seance::executer_a`]).
    Minuterie {
        #[serde(default)]
        id: Option<IdActivite>,
        on: bool,
    },
    Figer {
        #[serde(default)]
        id: Option<IdActivite>,
        on: bool,
    },
    Reinitialiser {
        #[serde(default)]
        id: Option<IdActivite>,
    },
    Titre {
        titre: String,
    },
    /// Nuage de mots et ROTI : masquer (ou réafficher) un mot à l'écran.
    MasquerMot {
        #[serde(default)]
        id: Option<IdActivite>,
        cle: String,
        on: bool,
    },
    /// Nuage de mots et ROTI : la clé `de` rejoint la clé `vers`.
    Fusionner {
        #[serde(default)]
        id: Option<IdActivite>,
        de: String,
        vers: String,
    },
    /// Annule la fusion d'une clé.
    Separer {
        #[serde(default)]
        id: Option<IdActivite>,
        cle: String,
    },
    /// Question ouverte et questions du public : valider (ou retirer) un élément pour l'écran.
    Valider {
        #[serde(default)]
        id: Option<IdActivite>,
        rang: Rang,
        on: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ErreurSeance {
    #[error("le journal doit commencer par l'ouverture de la séance")]
    JournalSansOuverture,
    #[error("la séance est déjà ouverte")]
    DejaOuverte,
    #[error("le titre est vide ou trop long")]
    TitreInvalide,
    #[error("cette activité n'existe pas")]
    ActiviteInconnue,
    #[error("aucune activité n'est en cours")]
    AucuneActiviteEnCours,
    #[error("cette activité n'est pas celle en cours")]
    PasEnCours,
    #[error("les réponses sont figées")]
    Figee,
    #[error("participant inconnu")]
    ParticipantInconnu,
    #[error("identifiant déjà utilisé")]
    IdentifiantPris,
    #[error("cette activité ne se modère pas ainsi")]
    ModerationImpossible,
    #[error("cette activité n'a pas de minuterie")]
    SansMinuterie,
    #[error("cette commande a besoin de l'heure")]
    HeureNecessaire,
    #[error("{0}")]
    Configuration(#[from] ErreurConfiguration),
    #[error("{0}")]
    Reponse(#[from] ErreurReponse),
}

/// Une activité du déroulé.
#[derive(Debug, Clone)]
pub(crate) struct Etape {
    pub(crate) id: IdActivite,
    pub(crate) contenu: Contenu,
    /// Résultat projeté (sous réserve du seuil d'anonymat).
    pub(crate) revelee: bool,
    /// Plus aucune réponse acceptée.
    pub(crate) figee: bool,
    /// Heure de fin de la minuterie (millisecondes depuis 1970), si elle tourne ou a tourné.
    pub(crate) fin_minuterie: Option<u64>,
}

impl Etape {
    /// Temps restant de la minuterie à l'heure `maintenant` (0 une fois échue).
    pub(crate) fn reste_ms(&self, maintenant: u64) -> Option<u64> {
        self.fin_minuterie.map(|f| f.saturating_sub(maintenant))
    }
}

#[derive(Debug, Clone)]
pub struct Seance {
    pub(crate) titre: String,
    pub(crate) code: String,
    participants: BTreeMap<String, IdParticipant>,
    pub(crate) deroule: Vec<Etape>,
    pub(crate) courante: Option<IdActivite>,
    /// Signaux discrets, éphémères (non journalisés).
    pub(crate) signaux: Signaux,
}

impl Seance {
    /// Ouvre une séance. Renvoie aussi l'événement d'ouverture, à journaliser en premier.
    pub fn ouvrir(titre: &str, code: &str) -> Result<(Self, Evenement), ErreurSeance> {
        let titre = titre_valide(titre)?;
        let e = Evenement::SeanceOuverte { titre: titre.clone(), code: code.to_string() };
        let s = Seance {
            titre,
            code: code.to_string(),
            participants: BTreeMap::new(),
            deroule: Vec::new(),
            courante: None,
            signaux: Signaux::default(),
        };
        Ok((s, e))
    }

    /// Reconstruit une séance à partir des événements d'un journal.
    pub fn rejouer<'a>(
        evenements: impl IntoIterator<Item = &'a Evenement>,
    ) -> Result<Self, ErreurSeance> {
        let mut it = evenements.into_iter();
        let mut s = match it.next() {
            Some(Evenement::SeanceOuverte { titre, code }) => Seance::ouvrir(titre, code)?.0,
            _ => return Err(ErreurSeance::JournalSansOuverture),
        };
        for e in it {
            s.appliquer(e)?;
        }
        Ok(s)
    }

    pub fn titre(&self) -> &str {
        &self.titre
    }

    pub fn code(&self) -> &str {
        &self.code
    }

    pub fn activite_courante(&self) -> Option<IdActivite> {
        self.courante
    }

    pub fn nombre_d_inscrits(&self) -> usize {
        self.participants.len()
    }

    /// Retrouve un participant par son jeton.
    pub fn participant(&self, jeton: &str) -> Option<IdParticipant> {
        self.participants.get(jeton).copied()
    }

    /// Inscrit un nouveau participant. Si le jeton est déjà connu, renvoie son identifiant
    /// sans événement.
    pub fn inscrire(&mut self, jeton: &str) -> (IdParticipant, Option<Evenement>) {
        if let Some(id) = self.participant(jeton) {
            return (id, None);
        }
        let id = self.participants.values().max().map_or(1, |m| m + 1);
        let e = Evenement::ParticipantInscrit { id, jeton: jeton.to_string() };
        self.participants.insert(jeton.to_string(), id);
        (id, Some(e))
    }

    /// Traduit une commande de la console en événement, l'applique et le renvoie.
    pub fn executer(&mut self, commande: Commande) -> Result<Evenement, ErreurSeance> {
        let e = match commande {
            Commande::Ajouter(configuration) => Evenement::ActiviteAjoutee {
                id: self.deroule.iter().map(|e| e.id).max().map_or(1, |m| m + 1),
                configuration: configuration.validee()?,
            },
            Commande::Supprimer { id } => Evenement::ActiviteSupprimee { id },
            Commande::Lancer { id } => Evenement::ActiviteLancee { id },
            Commande::Accueil => Evenement::RetourAccueil,
            Commande::Reveler { id, on } => Evenement::Revelee { id: self.cible(id)?, on },
            Commande::Figer { id, on } => Evenement::Figee { id: self.cible(id)?, on },
            Commande::Reinitialiser { id } => Evenement::Reinitialisee { id: self.cible(id)? },
            Commande::Titre { titre } => Evenement::TitreChange { titre },
            Commande::Minuterie { .. } => return Err(ErreurSeance::HeureNecessaire),
            Commande::MasquerMot { id, cle, on } => {
                Evenement::MotMasque { id: self.cible(id)?, cle, on }
            }
            Commande::Fusionner { id, de, vers } => {
                Evenement::MotsFusionnes { id: self.cible(id)?, de, vers }
            }
            Commande::Separer { id, cle } => Evenement::MotSepare { id: self.cible(id)?, cle },
            Commande::Valider { id, rang, on } => {
                Evenement::ElementValide { id: self.cible(id)?, rang, on }
            }
        };
        self.appliquer(&e)?;
        Ok(e)
    }

    /// Enregistre la réponse d'un participant à l'activité en cours.
    pub fn repondre(
        &mut self,
        participant: IdParticipant,
        activite: IdActivite,
        reponse: Reponse,
    ) -> Result<Evenement, ErreurSeance> {
        let e = Evenement::ReponseEnregistree { activite, participant, reponse };
        self.appliquer(&e)?;
        Ok(e)
    }

    /// Valide puis applique un événement. En cas d'erreur, la séance est inchangée.
    pub fn appliquer(&mut self, e: &Evenement) -> Result<(), ErreurSeance> {
        match e {
            Evenement::SeanceOuverte { .. } => return Err(ErreurSeance::DejaOuverte),
            Evenement::TitreChange { titre } => self.titre = titre_valide(titre)?,
            Evenement::ParticipantInscrit { id, jeton } => {
                if self.participants.values().any(|p| p == id)
                    || self.participants.contains_key(jeton)
                {
                    return Err(ErreurSeance::IdentifiantPris);
                }
                self.participants.insert(jeton.clone(), *id);
            }
            Evenement::ActiviteAjoutee { id, configuration } => {
                if self.deroule.iter().any(|e| e.id == *id) {
                    return Err(ErreurSeance::IdentifiantPris);
                }
                let configuration = configuration.validee()?;
                self.deroule.push(Etape {
                    id: *id,
                    contenu: Contenu::new(&configuration),
                    revelee: false,
                    figee: false,
                    fin_minuterie: None,
                });
            }
            Evenement::ActiviteSupprimee { id } => {
                let i = self.indice(*id)?;
                self.deroule.remove(i);
                if self.courante == Some(*id) {
                    self.courante = None;
                }
            }
            Evenement::ActiviteLancee { id } => {
                self.indice(*id)?;
                self.courante = Some(*id);
            }
            Evenement::RetourAccueil => self.courante = None,
            Evenement::Revelee { id, on } => self.etape_mut(*id)?.revelee = *on,
            Evenement::Figee { id, on } => self.etape_mut(*id)?.figee = *on,
            Evenement::Reinitialisee { id } => {
                let etape = self.etape_mut(*id)?;
                etape.contenu.vider();
                etape.revelee = false;
                etape.figee = false;
                etape.fin_minuterie = None;
            }
            Evenement::ReponseEnregistree { activite, participant, reponse } => {
                if !self.participants.values().any(|p| p == participant) {
                    return Err(ErreurSeance::ParticipantInconnu);
                }
                match self.courante {
                    None => return Err(ErreurSeance::AucuneActiviteEnCours),
                    Some(c) if c != *activite => return Err(ErreurSeance::PasEnCours),
                    Some(_) => {}
                }
                let etape = self.etape_mut(*activite)?;
                if etape.figee {
                    return Err(ErreurSeance::Figee);
                }
                etape.contenu.repondre(*participant, reponse)?;
            }
            Evenement::MotMasque { id, cle, on } => {
                self.moderation_mots(*id)?.masquer(cle, *on);
            }
            Evenement::MotsFusionnes { id, de, vers } => {
                self.moderation_mots(*id)?.fusionner(de, vers)?;
            }
            Evenement::MotSepare { id, cle } => {
                self.moderation_mots(*id)?.separer(cle);
            }
            Evenement::ElementValide { id, rang, on } => {
                match self.etape_mut(*id)?.contenu.valider_element(*rang, *on) {
                    Some(r) => r?,
                    None => return Err(ErreurSeance::ModerationImpossible),
                }
            }
            Evenement::MinuterieReglee { id, fin } => {
                let etape = self.etape_mut(*id)?;
                if fin.is_some() && etape.contenu.configuration().duree_minuterie().is_none() {
                    return Err(ErreurSeance::SansMinuterie);
                }
                etape.fin_minuterie = *fin;
                if fin.is_some() {
                    etape.figee = false;
                }
            }
        }
        Ok(())
    }

    /// Exécute une commande à l'heure `maintenant` (millisecondes depuis 1970) et renvoie les
    /// événements à journaliser. En plus de [`Seance::executer`] :
    /// - lancer une activité à minuterie la démarre, si elle n'a pas encore tourné ;
    /// - la commande `minuterie` la redémarre ou l'arrête.
    pub fn executer_a(
        &mut self,
        commande: Commande,
        maintenant: u64,
    ) -> Result<Vec<Evenement>, ErreurSeance> {
        let fin_dans = |s: &Self, id: IdActivite| -> Option<u64> {
            let duree = s.etape(id)?.contenu.configuration().duree_minuterie()?;
            Some(maintenant + u64::from(duree) * 1000)
        };
        match commande {
            Commande::Minuterie { id, on } => {
                let id = self.cible(id)?;
                self.etape(id).ok_or(ErreurSeance::ActiviteInconnue)?;
                let fin = if on {
                    Some(fin_dans(self, id).ok_or(ErreurSeance::SansMinuterie)?)
                } else {
                    None
                };
                let e = Evenement::MinuterieReglee { id, fin };
                self.appliquer(&e)?;
                Ok(vec![e])
            }
            Commande::Lancer { id } => {
                let mut evenements = vec![self.executer(Commande::Lancer { id })?];
                let jamais_partie =
                    self.etape(id).is_some_and(|e| e.fin_minuterie.is_none() && !e.figee);
                if let Some(fin) = fin_dans(self, id).filter(|_| jamais_partie) {
                    let e = Evenement::MinuterieReglee { id, fin: Some(fin) };
                    self.appliquer(&e)?;
                    evenements.push(e);
                }
                Ok(evenements)
            }
            autre => Ok(vec![self.executer(autre)?]),
        }
    }

    /// Activités dont la minuterie est échue et qui acceptent encore des réponses :
    /// le serveur les fige (commande `figer`).
    pub fn minuteries_echues(&self, maintenant: u64) -> Vec<IdActivite> {
        self.deroule
            .iter()
            .filter(|e| !e.figee && e.fin_minuterie.is_some_and(|f| f <= maintenant))
            .map(|e| e.id)
            .collect()
    }

    /// Active ou éteint un signal discret. `maintenant` : instant en millisecondes.
    /// Renvoie vrai si le signal est désormais actif.
    pub fn signaler(
        &mut self,
        participant: IdParticipant,
        genre: TypeSignal,
        maintenant: u64,
    ) -> Result<bool, ErreurSeance> {
        if !self.participants.values().any(|p| *p == participant) {
            return Err(ErreurSeance::ParticipantInconnu);
        }
        Ok(self.signaux.basculer(participant, genre, maintenant))
    }

    /// Efface les signaux expirés ; vrai si quelque chose a changé.
    pub fn purger_signaux(&mut self, maintenant: u64) -> bool {
        self.signaux.purger(maintenant)
    }

    pub fn a_des_signaux(&self) -> bool {
        !self.signaux.est_vide()
    }

    fn moderation_mots(
        &mut self,
        id: IdActivite,
    ) -> Result<&mut crate::nuage::ModerationMots, ErreurSeance> {
        self.etape_mut(id)?.contenu.moderation_mots().ok_or(ErreurSeance::ModerationImpossible)
    }

    fn cible(&self, id: Option<IdActivite>) -> Result<IdActivite, ErreurSeance> {
        id.or(self.courante).ok_or(ErreurSeance::AucuneActiviteEnCours)
    }

    fn indice(&self, id: IdActivite) -> Result<usize, ErreurSeance> {
        self.deroule.iter().position(|e| e.id == id).ok_or(ErreurSeance::ActiviteInconnue)
    }

    fn etape_mut(&mut self, id: IdActivite) -> Result<&mut Etape, ErreurSeance> {
        let i = self.indice(id)?;
        Ok(&mut self.deroule[i])
    }

    pub(crate) fn etape(&self, id: IdActivite) -> Option<&Etape> {
        self.deroule.iter().find(|e| e.id == id)
    }
}

fn titre_valide(titre: &str) -> Result<String, ErreurSeance> {
    let t = titre.split_whitespace().collect::<Vec<_>>().join(" ");
    if t.is_empty() || t.chars().count() > LONGUEUR_TITRE_MAX {
        Err(ErreurSeance::TitreInvalide)
    } else {
        Ok(t)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::nuage::Nuage;
    use crate::ouverte::Ouverte;
    use crate::sondage::Sondage;

    pub(crate) fn sondage(question: &str, options: &[&str]) -> Commande {
        Commande::Ajouter(Configuration::Sondage(Sondage {
            question: question.into(),
            options: options.iter().map(|s| s.to_string()).collect(),
        }))
    }

    /// Séance avec un sondage lancé et trois participants inscrits.
    pub(crate) fn seance_avec_sondage() -> (Seance, Vec<Evenement>) {
        let (mut s, e0) = Seance::ouvrir("Formation", "482731").unwrap();
        let mut journal = vec![e0];
        journal.push(s.executer(sondage("Café ou thé ?", &["Café", "Thé", "Rien"])).unwrap());
        journal.push(s.executer(Commande::Lancer { id: 1 }).unwrap());
        for j in ["a", "b", "c"] {
            journal.extend(s.inscrire(j).1);
        }
        (s, journal)
    }

    #[test]
    fn inscription_et_reconnexion_par_jeton() {
        let (mut s, _) = Seance::ouvrir("F", "1").unwrap();
        let (a, e) = s.inscrire("jeton-a");
        assert!(e.is_some());
        let (b, _) = s.inscrire("jeton-b");
        assert_ne!(a, b);
        let (a2, e2) = s.inscrire("jeton-a");
        assert_eq!(a, a2);
        assert!(e2.is_none(), "une reconnexion ne crée pas d'événement");
        assert_eq!(s.nombre_d_inscrits(), 2);
    }

    #[test]
    fn reponse_seulement_a_l_activite_en_cours() {
        let (mut s, _) = seance_avec_sondage();
        s.executer(sondage("Autre ?", &["Oui", "Non"])).unwrap();
        assert_eq!(s.repondre(1, 2, Reponse::Choix(0)), Err(ErreurSeance::PasEnCours));
        assert!(s.repondre(1, 1, Reponse::Choix(0)).is_ok());
        s.executer(Commande::Accueil).unwrap();
        assert_eq!(s.repondre(1, 1, Reponse::Choix(0)), Err(ErreurSeance::AucuneActiviteEnCours));
    }

    #[test]
    fn participant_inconnu_refuse() {
        let (mut s, _) = seance_avec_sondage();
        assert_eq!(s.repondre(99, 1, Reponse::Choix(0)), Err(ErreurSeance::ParticipantInconnu));
    }

    #[test]
    fn figer_bloque_les_reponses() {
        let (mut s, _) = seance_avec_sondage();
        s.executer(Commande::Figer { id: None, on: true }).unwrap();
        assert_eq!(s.repondre(1, 1, Reponse::Choix(0)), Err(ErreurSeance::Figee));
        s.executer(Commande::Figer { id: None, on: false }).unwrap();
        assert!(s.repondre(1, 1, Reponse::Choix(0)).is_ok());
    }

    #[test]
    fn reinitialiser_vide_et_remasque() {
        let (mut s, _) = seance_avec_sondage();
        s.repondre(1, 1, Reponse::Choix(0)).unwrap();
        s.executer(Commande::Reveler { id: None, on: true }).unwrap();
        s.executer(Commande::Reinitialiser { id: Some(1) }).unwrap();
        let etape = s.etape(1).unwrap();
        assert_eq!(etape.contenu.nombre_de_reponses(), 0);
        assert!(!etape.revelee);
    }

    #[test]
    fn supprimer_l_activite_en_cours_revient_a_l_accueil() {
        let (mut s, _) = seance_avec_sondage();
        s.executer(Commande::Supprimer { id: 1 }).unwrap();
        assert_eq!(s.activite_courante(), None);
        assert_eq!(s.executer(Commande::Lancer { id: 1 }), Err(ErreurSeance::ActiviteInconnue));
    }

    #[test]
    fn configuration_invalide_refusee_sans_effet() {
        let (mut s, _) = Seance::ouvrir("F", "1").unwrap();
        assert!(s.executer(sondage("Q", &["Seule"])).is_err());
        assert!(s.deroule.is_empty());
    }

    #[test]
    fn titre_valide_et_nettoye() {
        let (mut s, _) = Seance::ouvrir("F", "1").unwrap();
        s.executer(Commande::Titre { titre: "  Jour   2 ".into() }).unwrap();
        assert_eq!(s.titre(), "Jour 2");
        assert_eq!(
            s.executer(Commande::Titre { titre: " ".into() }),
            Err(ErreurSeance::TitreInvalide)
        );
    }

    #[test]
    fn la_reprise_rejoue_le_journal_a_l_identique() {
        let (mut s, mut journal) = seance_avec_sondage();
        journal.push(s.repondre(1, 1, Reponse::Choix(1)).unwrap());
        journal.push(s.repondre(2, 1, Reponse::Choix(1)).unwrap());
        journal.push(s.executer(Commande::Reveler { id: None, on: true }).unwrap());

        // Aller-retour en JSON, comme dans le fichier du journal.
        let lignes: Vec<String> =
            journal.iter().map(|e| serde_json::to_string(e).unwrap()).collect();
        let relu: Vec<Evenement> =
            lignes.iter().map(|l| serde_json::from_str(l).unwrap()).collect();

        let r = Seance::rejouer(&relu).unwrap();
        assert_eq!(r.participant("b"), s.participant("b"));
        assert_eq!(r.activite_courante(), Some(1));
        let etape = r.etape(1).unwrap();
        assert!(etape.revelee);
        assert_eq!(etape.contenu.resultat(), s.etape(1).unwrap().contenu.resultat());
    }

    #[test]
    fn un_journal_commence_par_l_ouverture() {
        let e = Evenement::RetourAccueil;
        assert_eq!(Seance::rejouer([&e]).err(), Some(ErreurSeance::JournalSansOuverture));
    }

    #[test]
    fn format_json_des_commandes() {
        let c: Commande = serde_json::from_str(
            r#"{"c":"ajouter","type":"sondage","question":"Q ?","options":["A","B"]}"#,
        )
        .unwrap();
        assert_eq!(c, sondage("Q ?", &["A", "B"]));
        let c: Commande = serde_json::from_str(r#"{"c":"reveler","on":true}"#).unwrap();
        assert_eq!(c, Commande::Reveler { id: None, on: true });
        let r: Reponse = serde_json::from_str(r#"{"choix":2}"#).unwrap();
        assert_eq!(r, Reponse::Choix(2));
    }

    fn nuage_lance() -> Seance {
        let (mut s, _) = Seance::ouvrir("F", "1").unwrap();
        let config =
            Nuage { question: "Un mot ?".into(), max_mots: 3, moderation: Default::default() };
        s.executer(Commande::Ajouter(Configuration::Nuage(config))).unwrap();
        s.executer(Commande::Lancer { id: 1 }).unwrap();
        for j in ["a", "b", "c"] {
            s.inscrire(j);
        }
        s
    }

    #[test]
    fn moderation_du_nuage_journalisee_et_rejouee() {
        let (_, e0) = Seance::ouvrir("F", "1").unwrap();
        let mut s = nuage_lance();
        let mut journal = vec![e0];
        // Reconstituer le journal équivalent à nuage_lance().
        let config =
            Nuage { question: "Un mot ?".into(), max_mots: 3, moderation: Default::default() };
        journal.push(Evenement::ActiviteAjoutee {
            id: 1,
            configuration: Configuration::Nuage(config),
        });
        journal.push(Evenement::ActiviteLancee { id: 1 });
        for (i, j) in ["a", "b", "c"].iter().enumerate() {
            journal.push(Evenement::ParticipantInscrit { id: i as u32 + 1, jeton: j.to_string() });
        }
        let mots = |m: &[&str]| Reponse::Mots(m.iter().map(|x| x.to_string()).collect());
        journal.push(s.repondre(1, 1, mots(&["écouter", "nul"])).unwrap());
        journal.push(s.repondre(2, 1, mots(&["écoute"])).unwrap());
        journal.push(
            s.executer(Commande::Fusionner {
                id: None,
                de: "ecouter".into(),
                vers: "ecoute".into(),
            })
            .unwrap(),
        );
        journal.push(
            s.executer(Commande::MasquerMot { id: None, cle: "nul".into(), on: true }).unwrap(),
        );

        let r = Seance::rejouer(&journal).unwrap();
        let Resultat::Nuage(n) = r.etape(1).unwrap().contenu.resultat() else { panic!() };
        assert_eq!(n.mots.len(), 2);
        assert_eq!(n.mots.iter().find(|m| m.cle == "ecoute").unwrap().n, 2);
        assert!(n.mots.iter().find(|m| m.cle == "nul").unwrap().masque);
    }

    #[test]
    fn moderation_refusee_sur_un_sondage() {
        let (mut s, _) = seance_avec_sondage();
        assert_eq!(
            s.executer(Commande::MasquerMot { id: None, cle: "x".into(), on: true }),
            Err(ErreurSeance::ModerationImpossible)
        );
        assert_eq!(
            s.executer(Commande::Valider { id: None, rang: 1, on: true }),
            Err(ErreurSeance::ModerationImpossible)
        );
    }

    #[test]
    fn mauvaise_forme_de_reponse_refusee() {
        let mut s = nuage_lance();
        assert_eq!(
            s.repondre(1, 1, Reponse::Choix(0)),
            Err(ErreurSeance::Reponse(ErreurReponse::TypeIncompatible))
        );
    }

    #[test]
    fn validation_d_un_message_ouvert() {
        let (mut s, _) = Seance::ouvrir("F", "1").unwrap();
        let config = Ouverte { question: "Une idée ?".into(), valides: Default::default() };
        s.executer(Commande::Ajouter(Configuration::Ouverte(config))).unwrap();
        s.executer(Commande::Lancer { id: 1 }).unwrap();
        s.inscrire("a");
        assert!(
            s.executer(Commande::Valider { id: None, rang: 1, on: true }).is_err(),
            "message inexistant"
        );
        s.repondre(1, 1, Reponse::Texte("Plus de pauses".into())).unwrap();
        s.executer(Commande::Valider { id: None, rang: 1, on: true }).unwrap();
        let Resultat::Ouverte(o) = s.etape(1).unwrap().contenu.resultat() else { panic!() };
        assert!(o.messages[0].valide);
    }

    #[test]
    fn signal_reserve_aux_participants() {
        let (mut s, _) = seance_avec_sondage();
        assert_eq!(s.signaler(99, TypeSignal::Perdu, 0), Err(ErreurSeance::ParticipantInconnu));
        assert_eq!(s.signaler(1, TypeSignal::Perdu, 0), Ok(true));
        assert!(s.a_des_signaux());
        assert!(s.purger_signaux(crate::signal::DUREE_SIGNAL_MS));
    }

    #[test]
    fn format_json_des_nouvelles_commandes() {
        let c: Commande = serde_json::from_str(r#"{"c":"fusionner","de":"a","vers":"b"}"#).unwrap();
        assert_eq!(c, Commande::Fusionner { id: None, de: "a".into(), vers: "b".into() });
        let c: Commande =
            serde_json::from_str(r#"{"c":"ajouter","type":"echelle","question":"Q","max":5}"#)
                .unwrap();
        assert!(matches!(c, Commande::Ajouter(Configuration::Echelle(_))));
        let r: Reponse = serde_json::from_str(r#"{"pari":{"choix":1,"certitude":2}}"#).unwrap();
        assert!(matches!(r, Reponse::Pari(_)));
    }

    fn ordre_lance(duree: Option<u32>) -> (Seance, Vec<Evenement>) {
        use crate::ordre::{Element, Ordre};
        let (mut s, e0) = Seance::ouvrir("F", "1").unwrap();
        let elements = ["Accueil", "Débat", "Décision"]
            .iter()
            .map(|t| Element { texte: t.to_string(), couleur: Default::default() })
            .collect();
        let config = Ordre {
            question: "Dans l'ordre".into(),
            elements,
            duree,
            solution: None,
            melange: vec![],
        };
        let mut journal =
            vec![e0, s.executer(Commande::Ajouter(Configuration::Ordre(config))).unwrap()];
        journal.extend(s.executer_a(Commande::Lancer { id: 1 }, 1_000).unwrap());
        s.inscrire("a");
        (s, journal)
    }

    #[test]
    fn la_minuterie_demarre_au_lancement() {
        let (s, journal) = ordre_lance(Some(60));
        assert_eq!(journal.last(), Some(&Evenement::MinuterieReglee { id: 1, fin: Some(61_000) }));
        assert_eq!(s.etape(1).unwrap().reste_ms(31_000), Some(30_000));
        let (sans, journal) = ordre_lance(None);
        assert!(!journal.iter().any(|e| matches!(e, Evenement::MinuterieReglee { .. })));
        assert_eq!(sans.etape(1).unwrap().reste_ms(5_000), None);
    }

    #[test]
    fn relancer_ne_redemarre_pas_la_minuterie() {
        let (mut s, _) = ordre_lance(Some(60));
        s.executer(Commande::Accueil).unwrap();
        let ev = s.executer_a(Commande::Lancer { id: 1 }, 50_000).unwrap();
        assert_eq!(ev.len(), 1, "seulement le lancement");
        assert_eq!(s.etape(1).unwrap().fin_minuterie, Some(61_000));
    }

    #[test]
    fn echeance_puis_redemarrage_et_arret() {
        let (mut s, _) = ordre_lance(Some(60));
        assert!(s.minuteries_echues(60_999).is_empty());
        assert_eq!(s.minuteries_echues(61_000), vec![1]);
        s.executer(Commande::Figer { id: Some(1), on: true }).unwrap();
        assert!(s.minuteries_echues(70_000).is_empty(), "déjà figée");

        // Redémarrer rouvre les réponses pour une nouvelle durée.
        s.executer_a(Commande::Minuterie { id: None, on: true }, 100_000).unwrap();
        assert!(!s.etape(1).unwrap().figee);
        assert_eq!(s.etape(1).unwrap().fin_minuterie, Some(160_000));
        s.executer_a(Commande::Minuterie { id: None, on: false }, 120_000).unwrap();
        assert!(s.minuteries_echues(500_000).is_empty(), "arrêtée");
    }

    #[test]
    fn minuterie_refusee_sans_duree_et_effacee_par_reinitialiser() {
        let (mut sans, _) = ordre_lance(None);
        assert_eq!(
            sans.executer_a(Commande::Minuterie { id: None, on: true }, 0),
            Err(ErreurSeance::SansMinuterie)
        );
        let (mut s, _) = ordre_lance(Some(60));
        s.executer(Commande::Reinitialiser { id: None }).unwrap();
        assert_eq!(s.etape(1).unwrap().fin_minuterie, None);
        assert_eq!(
            s.executer(Commande::Minuterie { id: None, on: true }),
            Err(ErreurSeance::HeureNecessaire)
        );
    }

    #[test]
    fn la_minuterie_se_rejoue_depuis_le_journal() {
        let (_, journal) = ordre_lance(Some(60));
        let lignes: Vec<String> =
            journal.iter().map(|e| serde_json::to_string(e).unwrap()).collect();
        let relu: Vec<Evenement> =
            lignes.iter().map(|l| serde_json::from_str(l).unwrap()).collect();
        let r = Seance::rejouer(&relu).unwrap();
        assert_eq!(r.etape(1).unwrap().fin_minuterie, Some(61_000));
        assert_eq!(r.minuteries_echues(61_000), vec![1]);
    }

    #[test]
    fn la_vue_participant_cache_l_ordre_jusqu_a_la_revelation() {
        let (mut s, _) = ordre_lance(Some(60));
        let json = serde_json::to_value(s.vue_participant(1, 1_000)).unwrap();
        let textes: Vec<&str> = json["activite"]["elements"]
            .as_array()
            .unwrap()
            .iter()
            .map(|e| e["texte"].as_str().unwrap())
            .collect();
        assert_ne!(textes, vec!["Accueil", "Débat", "Décision"]);
        assert!(json["activite"].get("solution").is_none());
        assert_eq!(json["activite"]["reste_ms"], 60_000);
        s.executer(Commande::Reveler { id: None, on: true }).unwrap();
        let json = serde_json::to_value(s.vue_participant(1, 1_000)).unwrap();
        assert!(json["activite"]["solution"].is_array());
    }
}
