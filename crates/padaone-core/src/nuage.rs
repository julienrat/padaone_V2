//! Nuage de mots : 1 à 3 mots de 25 caractères au plus par participant.
//!
//! Règles (spécification § 3) :
//! - les saisies sont regroupées par clé ([`crate::mots::cle`]) ;
//! - texte affiché pour une clé : la saisie la plus fréquente (égalité : la première arrivée) ;
//! - les clés sont rendues dans l'ordre de leur première arrivée (placement stable à l'écran) ;
//! - la console peut masquer un mot et fusionner deux clés (la seconde rejoint la première).
//!
//! La modération ([`ModerationMots`]) sert aussi au mot de la fin du ROTI.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::activite::{
    question_valide, texte_valide, Activite, ErreurConfiguration, ErreurReponse, Rang,
};
use crate::mots;

/// Nombre de mots par participant, au plus.
pub const MOTS_MAX: usize = 3;
/// Longueur d'un mot ou d'une expression, en caractères.
pub const LONGUEUR_MOT_MAX: usize = 25;
/// Nombre de mots projetés au plus (les plus cités).
pub const MOTS_ECRAN_MAX: usize = 40;

fn trois() -> usize {
    MOTS_MAX
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Nuage {
    pub question: String,
    /// Nombre de mots demandés à chaque participant (1 à 3).
    #[serde(default = "trois")]
    pub max_mots: usize,
    /// Masquages et fusions décidés par l'animateur (rejoués depuis le journal).
    #[serde(skip)]
    pub moderation: ModerationMots,
}

impl Nuage {
    pub fn validee(&self) -> Result<Self, ErreurConfiguration> {
        if !(1..=MOTS_MAX).contains(&self.max_mots) {
            return Err(ErreurConfiguration::Reglage);
        }
        Ok(Nuage {
            question: question_valide(&self.question)?,
            max_mots: self.max_mots,
            moderation: ModerationMots::default(),
        })
    }
}

/// Masquages et fusions de mots, par clé.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ModerationMots {
    masques: BTreeSet<String>,
    /// Clé fusionnée → clé qui l'absorbe.
    fusions: BTreeMap<String, String>,
}

impl ModerationMots {
    pub fn masquer(&mut self, cle: &str, on: bool) {
        if on {
            self.masques.insert(cle.to_string());
        } else {
            self.masques.remove(cle);
        }
    }

    /// Fusionne la clé `de` dans la clé `vers`. Refuse une fusion qui bouclerait.
    pub fn fusionner(&mut self, de: &str, vers: &str) -> Result<(), ErreurReponse> {
        let cible = self.cle_finale(vers);
        if de == vers || cible == de || de.is_empty() || vers.is_empty() {
            return Err(ErreurReponse::ChoixInconnu);
        }
        self.fusions.insert(de.to_string(), cible);
        Ok(())
    }

    /// Annule la fusion d'une clé : elle redevient un mot à part.
    pub fn separer(&mut self, cle: &str) {
        self.fusions.remove(cle);
    }

    /// Clé après application des fusions (suit la chaîne, sans boucle possible).
    pub fn cle_finale(&self, cle: &str) -> String {
        let mut c = cle.to_string();
        for _ in 0..=self.fusions.len() {
            match self.fusions.get(&c) {
                Some(suivante) => c = suivante.clone(),
                None => break,
            }
        }
        c
    }

    pub fn est_masque(&self, cle: &str) -> bool {
        self.masques.contains(cle)
    }

    pub fn vider(&mut self) {
        self.masques.clear();
        self.fusions.clear();
    }
}

/// Un mot du nuage, après regroupement.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MotNuage {
    pub cle: String,
    pub texte: String,
    /// Nombre de participants qui l'ont proposé.
    pub n: usize,
    pub masque: bool,
    /// Clés fusionnées dans celle-ci (pour pouvoir les séparer depuis la console).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub fusionnes: Vec<String>,
}

/// Regroupe les saisies de chaque participant (une liste par participant, dans l'ordre
/// d'arrivée). Un participant compte au plus une fois par mot, même après fusion.
pub fn regrouper<'a>(
    reponses: impl Iterator<Item = &'a [String]>,
    moderation: &ModerationMots,
) -> Vec<MotNuage> {
    struct Groupe {
        n: usize,
        /// Texte saisi → (nombre, ordre de première apparition).
        textes: Vec<(String, usize)>,
        cles: BTreeSet<String>,
    }
    let mut ordre: Vec<String> = Vec::new();
    let mut groupes: BTreeMap<String, Groupe> = BTreeMap::new();
    for saisies in reponses {
        let mut deja = BTreeSet::new();
        for saisie in saisies {
            let Some(cle) = mots::cle(saisie) else { continue };
            let finale = moderation.cle_finale(&cle);
            let groupe = groupes.entry(finale.clone()).or_insert_with(|| {
                ordre.push(finale.clone());
                Groupe { n: 0, textes: Vec::new(), cles: BTreeSet::new() }
            });
            if cle != finale {
                groupe.cles.insert(cle);
            }
            match groupe.textes.iter_mut().find(|(t, _)| t == saisie) {
                Some((_, compte)) => *compte += 1,
                None => groupe.textes.push((saisie.clone(), 1)),
            }
            if deja.insert(finale) {
                groupe.n += 1;
            }
        }
    }
    ordre
        .into_iter()
        .filter_map(|cle| {
            let g = groupes.remove(&cle)?;
            // Le plus fréquent ; à égalité, le premier arrivé (max_by_key garde le dernier maximum,
            // d'où le parcours à rebours).
            let texte = g.textes.iter().rev().max_by_key(|(_, c)| *c).map(|(t, _)| t.clone())?;
            Some(MotNuage {
                masque: moderation.est_masque(&cle),
                texte,
                n: g.n,
                fusionnes: g.cles.into_iter().collect(),
                cle,
            })
        })
        .collect()
}

/// Mots projetables : non masqués, et les plus cités seulement s'il y en a trop
/// (l'ordre d'arrivée est conservé pour garder le placement stable).
pub fn pour_ecran(mots: Vec<MotNuage>) -> Vec<MotNuage> {
    let mut visibles: Vec<MotNuage> = mots.into_iter().filter(|m| !m.masque).collect();
    if visibles.len() > MOTS_ECRAN_MAX {
        let mut effectifs: Vec<usize> = visibles.iter().map(|m| m.n).collect();
        effectifs.sort_unstable_by(|a, b| b.cmp(a));
        let seuil = effectifs[MOTS_ECRAN_MAX - 1];
        // Tous les mots au-dessus du seuil, puis ceux au seuil dans l'ordre d'arrivée.
        let au_dessus = visibles.iter().filter(|m| m.n > seuil).count();
        let mut au_seuil = MOTS_ECRAN_MAX - au_dessus;
        visibles.retain(|m| {
            if m.n > seuil {
                true
            } else if m.n == seuil && au_seuil > 0 {
                au_seuil -= 1;
                true
            } else {
                false
            }
        });
    }
    visibles.iter_mut().for_each(|m| m.fusionnes.clear());
    visibles
}

/// Nettoie une liste de mots : textes valides, sans doublon de clé, de 1 à `max`.
pub fn mots_valides(mots: Vec<String>, max: usize) -> Result<Vec<String>, ErreurReponse> {
    let mut propres: Vec<String> = Vec::new();
    let mut cles = BTreeSet::new();
    for m in mots {
        if m.trim().is_empty() {
            continue;
        }
        let t = texte_valide(&m, LONGUEUR_MOT_MAX)?;
        let Some(cle) = mots::cle(&t) else { continue };
        if cles.insert(cle) {
            propres.push(t);
        }
    }
    if propres.is_empty() {
        Err(ErreurReponse::Vide)
    } else if propres.len() > max {
        Err(ErreurReponse::TropDeMots)
    } else {
        Ok(propres)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ResultatNuage {
    pub mots: Vec<MotNuage>,
    pub participants: usize,
}

impl Activite for Nuage {
    type Reponse = Vec<String>;
    type Resultat = ResultatNuage;

    fn valider(&self, mots: Vec<String>) -> Result<Vec<String>, ErreurReponse> {
        mots_valides(mots, self.max_mots)
    }

    fn resultat<'a>(
        &self,
        reponses: impl Iterator<Item = (Rang, &'a Vec<String>)>,
    ) -> ResultatNuage {
        let reponses: Vec<&Vec<String>> = reponses.map(|(_, r)| r).collect();
        ResultatNuage {
            participants: reponses.len(),
            mots: regrouper(reponses.iter().map(|r| r.as_slice()), &self.moderation),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::activite::Collecte;

    fn v(mots: &[&str]) -> Vec<String> {
        mots.iter().map(|s| s.to_string()).collect()
    }

    fn nuage() -> Collecte<Nuage> {
        Collecte::new(
            Nuage {
                question: "Une bonne réunion ?".into(),
                max_mots: 3,
                moderation: Default::default(),
            }
            .validee()
            .unwrap(),
        )
    }

    #[test]
    fn regroupe_et_garde_l_ordre_d_arrivee() {
        let mut c = nuage();
        c.repondre(1, v(&["Respect", "écoute"])).unwrap();
        c.repondre(2, v(&["Écoutes"])).unwrap();
        c.repondre(3, v(&["écoute", "café"])).unwrap();
        let r = c.resultat();
        let resume: Vec<(&str, usize)> = r.mots.iter().map(|m| (m.texte.as_str(), m.n)).collect();
        assert_eq!(resume, vec![("Respect", 1), ("écoute", 3), ("café", 1)]);
        assert_eq!(r.participants, 3);
    }

    #[test]
    fn texte_le_plus_frequent_puis_le_premier() {
        let mut c = nuage();
        c.repondre(1, v(&["Écoute"])).unwrap();
        c.repondre(2, v(&["écoute"])).unwrap();
        assert_eq!(c.resultat().mots[0].texte, "Écoute", "égalité : le premier arrivé");
        c.repondre(3, v(&["écoute"])).unwrap();
        assert_eq!(c.resultat().mots[0].texte, "écoute", "le plus fréquent");
    }

    #[test]
    fn saisie_validee() {
        let c = nuage();
        assert_eq!(c.activite.valider(v(&["", " "])), Err(ErreurReponse::Vide));
        assert_eq!(c.activite.valider(v(&["w", "x", "y", "z"])), Err(ErreurReponse::TropDeMots));
        assert_eq!(c.activite.valider(v(&[&"x".repeat(26)])), Err(ErreurReponse::TropLongue));
        // Doublons de clé retirés : « écoute » et « Écoutes » ne comptent qu'une fois.
        assert_eq!(c.activite.valider(v(&["écoute", "Écoutes"])), Ok(v(&["écoute"])));
    }

    #[test]
    fn masquer_et_fusionner() {
        let mut c = nuage();
        c.repondre(1, v(&["écouter", "nul"])).unwrap();
        c.repondre(2, v(&["écoute"])).unwrap();
        c.activite.moderation.masquer("nul", true);
        c.activite.moderation.fusionner("ecouter", "ecoute").unwrap();
        let r = c.resultat();
        let ecoute = r.mots.iter().find(|m| m.cle == "ecoute").unwrap();
        assert_eq!(ecoute.n, 2);
        assert_eq!(ecoute.fusionnes, v(&["ecouter"]));
        let ecran = pour_ecran(r.mots);
        assert!(ecran.iter().all(|m| m.cle != "nul"), "mot masqué absent de l'écran");

        c.activite.moderation.separer("ecouter");
        assert_eq!(c.resultat().mots.len(), 3);
    }

    #[test]
    fn un_participant_compte_une_fois_apres_fusion() {
        let mut c = nuage();
        c.repondre(1, v(&["écouter", "écoute"])).unwrap();
        c.activite.moderation.fusionner("ecouter", "ecoute").unwrap();
        assert_eq!(c.resultat().mots[0].n, 1);
    }

    #[test]
    fn fusion_en_boucle_refusee() {
        let mut m = ModerationMots::default();
        m.fusionner("a", "b").unwrap();
        assert!(m.fusionner("b", "a").is_err());
        assert!(m.fusionner("a", "a").is_err());
        m.fusionner("c", "a").unwrap();
        assert_eq!(m.cle_finale("c"), "b", "une fusion vers un mot déjà fusionné va au bout");
    }

    #[test]
    fn ecran_limite_aux_mots_les_plus_cites() {
        let mut mots: Vec<MotNuage> = (0..50)
            .map(|i| MotNuage {
                cle: format!("m{i}"),
                texte: format!("m{i}"),
                n: if i % 2 == 0 { 2 } else { 1 },
                masque: false,
                fusionnes: vec![],
            })
            .collect();
        mots[49].masque = true;
        let ecran = pour_ecran(mots);
        assert_eq!(ecran.len(), MOTS_ECRAN_MAX);
        assert_eq!(ecran.iter().filter(|m| m.n == 2).count(), 25);
        let indices: Vec<usize> = ecran.iter().map(|m| m.cle[1..].parse().unwrap()).collect();
        assert!(indices.windows(2).all(|w| w[0] < w[1]), "ordre d'arrivée conservé");
    }
}
