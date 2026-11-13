//! Remise en ordre : les participants remettent des éléments dans le bon ordre (étapes d'un
//! processus, cartes, séquence logique) par glisser-déposer, puis valident leur proposition.
//!
//! Règles :
//! - de 3 à 8 éléments, chacun avec un texte (40 caractères au plus) et une couleur ; tous ont
//!   la même forme (rectangle arrondi) ;
//! - l'ordre de la configuration est le bon ordre ; les téléphones reçoivent les éléments
//!   **mélangés** (mélange déterministe, jamais l'ordre exact) et répondent avec les positions
//!   affichées : rien dans leur vue ne trahit la solution avant la révélation ;
//! - minuterie facultative (10 s à 60 min) : voir `Seance::executer_a` ;
//! - score d'une proposition : nombre d'éléments à leur place exacte.

use serde::{Deserialize, Serialize};

use crate::activite::{
    nettoyer, question_valide, Activite, ErreurConfiguration, ErreurReponse, Rang,
};

pub const ELEMENTS_MIN: usize = 3;
pub const ELEMENTS_MAX: usize = 8;
pub const LONGUEUR_ELEMENT_MAX: usize = 40;
/// Durée de la minuterie, en secondes.
pub const DUREE_MIN: u32 = 10;
pub const DUREE_MAX: u32 = 3600;

/// Couleur d'arrière-plan (10 au choix). Le texte posé dessus est choisi par les interfaces
/// pour garder un contraste d'au moins 4,5:1 ; le sens n'est jamais porté par la couleur seule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Couleur {
    #[default]
    Violet,
    Corail,
    Soleil,
    Menthe,
    Rose,
    Ciel,
    Lilas,
    Peche,
    Blanc,
    Encre,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Element {
    pub texte: String,
    #[serde(default)]
    pub couleur: Couleur,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ordre {
    /// Consigne, par exemple « Remettez les étapes d'une réunion dans l'ordre ».
    pub question: String,
    /// Les éléments dans le **bon ordre** (configuration de la console) ou mélangés
    /// (vues des téléphones et de l'écran, voir [`Ordre::publique`]).
    pub elements: Vec<Element>,
    /// Durée de la minuterie en secondes ; sans minuterie si absente.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duree: Option<u32>,
    /// Après la révélation seulement : positions affichées, dans le bon ordre.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub solution: Option<Vec<usize>>,
    /// Position affichée → indice dans le bon ordre. Calculé, jamais envoyé.
    #[serde(skip)]
    pub melange: Vec<usize>,
}

impl Ordre {
    pub fn validee(&self) -> Result<Self, ErreurConfiguration> {
        let question = question_valide(&self.question)?;
        if !(ELEMENTS_MIN..=ELEMENTS_MAX).contains(&self.elements.len()) {
            return Err(ErreurConfiguration::NombreOptions(ELEMENTS_MIN, ELEMENTS_MAX));
        }
        let mut elements = Vec::with_capacity(self.elements.len());
        for e in &self.elements {
            let texte = nettoyer(&e.texte);
            if texte.is_empty() {
                return Err(ErreurConfiguration::OptionVide);
            }
            if texte.chars().count() > LONGUEUR_ELEMENT_MAX {
                return Err(ErreurConfiguration::OptionTropLongue);
            }
            if elements.iter().any(|x: &Element| x.texte.to_lowercase() == texte.to_lowercase()) {
                return Err(ErreurConfiguration::OptionsEnDouble);
            }
            elements.push(Element { texte, couleur: e.couleur });
        }
        if let Some(d) = self.duree {
            if !(DUREE_MIN..=DUREE_MAX).contains(&d) {
                return Err(ErreurConfiguration::Reglage);
            }
        }
        let melange = melanger(&question, &elements);
        Ok(Ordre { question, elements, duree: self.duree, solution: None, melange })
    }

    /// Version envoyée aux téléphones et à l'écran : éléments mélangés ; la solution
    /// (positions affichées dans le bon ordre) n'y figure qu'une fois le résultat révélé.
    pub fn publique(&self, revelee: bool) -> Self {
        let elements = self.melange.iter().filter_map(|&i| self.elements.get(i).cloned()).collect();
        let solution = revelee.then(|| {
            (0..self.elements.len())
                .filter_map(|i| self.melange.iter().position(|&m| m == i))
                .collect()
        });
        Ordre {
            question: self.question.clone(),
            elements,
            duree: self.duree,
            solution,
            melange: Vec::new(),
        }
    }

    /// Indices dans le bon ordre d'une proposition exprimée en positions affichées.
    fn vers_indices(&self, positions: &[usize]) -> Vec<usize> {
        positions.iter().filter_map(|&p| self.melange.get(p).copied()).collect()
    }
}

/// Mélange déterministe (il se rejoue à l'identique depuis le journal), qui ne laisse jamais
/// le bon ordre ni l'ordre exactement inverse.
fn melanger(question: &str, elements: &[Element]) -> Vec<usize> {
    // Graine : empreinte FNV-1a des textes.
    let mut graine: u64 = 0xcbf2_9ce4_8422_2325;
    for octet in question.bytes().chain(elements.iter().flat_map(|e| e.texte.bytes())) {
        graine = (graine ^ u64::from(octet)).wrapping_mul(0x0100_0000_01b3);
    }
    let n = elements.len();
    let mut ordre: Vec<usize> = (0..n).collect();
    // Fisher-Yates avec un générateur xorshift.
    for i in (1..n).rev() {
        graine ^= graine << 13;
        graine ^= graine >> 7;
        graine ^= graine << 17;
        let j = (graine % (i as u64 + 1)) as usize;
        ordre.swap(i, j);
    }
    let identite: Vec<usize> = (0..n).collect();
    let inverse: Vec<usize> = (0..n).rev().collect();
    if ordre == identite || ordre == inverse {
        ordre.rotate_left(1);
        if ordre == inverse {
            ordre.swap(0, 1);
        }
    }
    ordre
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ResultatOrdre {
    /// Nombre de propositions validées.
    pub total: usize,
    /// Propositions entièrement justes.
    pub parfaits: usize,
    /// `scores[k]` : nombre de propositions avec exactement `k` éléments bien placés.
    pub scores: Vec<usize>,
    /// Pour chaque élément, dans le bon ordre : nombre de propositions qui le placent juste.
    pub bien_places: Vec<usize>,
    /// Score moyen (éléments bien placés par proposition), au dixième.
    pub moyenne: Option<f64>,
}

impl Activite for Ordre {
    /// Positions affichées, dans l'ordre proposé par le participant.
    type Reponse = Vec<usize>;
    type Resultat = ResultatOrdre;

    fn valider(&self, positions: Vec<usize>) -> Result<Vec<usize>, ErreurReponse> {
        let n = self.elements.len();
        let mut vu = vec![false; n];
        if positions.len() != n {
            return Err(ErreurReponse::ChoixInconnu);
        }
        for &p in &positions {
            match vu.get_mut(p) {
                Some(v) if !*v => *v = true,
                _ => return Err(ErreurReponse::ChoixInconnu),
            }
        }
        Ok(positions)
    }

    fn resultat<'a>(
        &self,
        reponses: impl Iterator<Item = (Rang, &'a Vec<usize>)>,
    ) -> ResultatOrdre {
        let n = self.elements.len();
        let mut r = ResultatOrdre {
            total: 0,
            parfaits: 0,
            scores: vec![0; n + 1],
            bien_places: vec![0; n],
            moyenne: None,
        };
        let mut somme = 0usize;
        for (_, positions) in reponses {
            let indices = self.vers_indices(positions);
            let score = indices.iter().enumerate().filter(|(rang, &i)| *rang == i).count();
            for (rang, &i) in indices.iter().enumerate() {
                if rang == i {
                    r.bien_places[i] += 1;
                }
            }
            r.scores[score] += 1;
            r.parfaits += usize::from(score == n);
            r.total += 1;
            somme += score;
        }
        if r.total > 0 {
            r.moyenne = Some((somme as f64 / r.total as f64 * 10.0).round() / 10.0);
        }
        r
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::activite::Collecte;

    fn element(t: &str) -> Element {
        Element { texte: t.into(), couleur: Couleur::Ciel }
    }

    fn ordre() -> Ordre {
        Ordre {
            question: "Les étapes d'une réunion".into(),
            elements: ["Accueil", "Ordre du jour", "Débat", "Décision", "Conclusion"]
                .iter()
                .map(|t| element(t))
                .collect(),
            duree: Some(90),
            solution: None,
            melange: vec![],
        }
        .validee()
        .unwrap()
    }

    /// Positions affichées qui reconstituent le bon ordre.
    fn bonne_reponse(o: &Ordre) -> Vec<usize> {
        o.publique(true).solution.unwrap()
    }

    #[test]
    fn de_trois_a_huit_elements() {
        let mut o = ordre();
        o.elements.truncate(2);
        assert_eq!(o.validee(), Err(ErreurConfiguration::NombreOptions(3, 8)));
        let mut o = ordre();
        o.elements = (0..9).map(|i| element(&format!("Étape {i}"))).collect();
        assert_eq!(o.validee(), Err(ErreurConfiguration::NombreOptions(3, 8)));
    }

    #[test]
    fn textes_controles() {
        let mut o = ordre();
        o.elements[0].texte = "x".repeat(41);
        assert_eq!(o.validee(), Err(ErreurConfiguration::OptionTropLongue));
        o.elements[0].texte = "  ".into();
        assert_eq!(o.validee(), Err(ErreurConfiguration::OptionVide));
        o.elements[0].texte = "débat".into();
        assert_eq!(o.validee(), Err(ErreurConfiguration::OptionsEnDouble));
    }

    #[test]
    fn duree_de_la_minuterie() {
        let mut o = ordre();
        o.duree = Some(5);
        assert_eq!(o.validee(), Err(ErreurConfiguration::Reglage));
        o.duree = Some(3601);
        assert_eq!(o.validee(), Err(ErreurConfiguration::Reglage));
        o.duree = None;
        assert!(o.validee().is_ok());
    }

    #[test]
    fn melange_stable_et_jamais_trivial() {
        let o = ordre();
        assert_eq!(o.melange, ordre().melange, "rejoué à l'identique");
        assert_ne!(o.melange, vec![0, 1, 2, 3, 4]);
        assert_ne!(o.melange, vec![4, 3, 2, 1, 0]);
        for n in ELEMENTS_MIN..=ELEMENTS_MAX {
            let els: Vec<Element> = (0..n).map(|i| element(&format!("E{i}"))).collect();
            let m = melanger("Q", &els);
            let mut tri = m.clone();
            tri.sort_unstable();
            assert_eq!(tri, (0..n).collect::<Vec<_>>(), "permutation complète");
            assert_ne!(m, (0..n).collect::<Vec<_>>());
        }
    }

    #[test]
    fn la_vue_publique_ne_trahit_pas_la_solution() {
        let o = ordre();
        let p = o.publique(false);
        assert_eq!(p.solution, None);
        assert_ne!(p.elements, o.elements, "éléments mélangés");
        let json = serde_json::to_string(&p).unwrap();
        assert!(!json.contains("melange"));
        let revelee = o.publique(true);
        let reconstitue: Vec<&str> =
            revelee.solution.unwrap().iter().map(|&p| revelee.elements[p].texte.as_str()).collect();
        assert_eq!(
            reconstitue,
            vec!["Accueil", "Ordre du jour", "Débat", "Décision", "Conclusion"]
        );
    }

    #[test]
    fn la_proposition_doit_etre_une_permutation() {
        let o = ordre();
        assert_eq!(o.valider(vec![0, 1, 2]), Err(ErreurReponse::ChoixInconnu));
        assert_eq!(o.valider(vec![0, 0, 1, 2, 3]), Err(ErreurReponse::ChoixInconnu));
        assert_eq!(o.valider(vec![0, 1, 2, 3, 5]), Err(ErreurReponse::ChoixInconnu));
        assert!(o.valider(vec![4, 3, 2, 1, 0]).is_ok());
    }

    #[test]
    fn scores_et_elements_bien_places() {
        let o = ordre();
        let juste = bonne_reponse(&o);
        // Deux premiers éléments échangés : 3 bien placés sur 5.
        let mut presque = juste.clone();
        presque.swap(0, 1);
        let mut c = Collecte::new(o);
        c.repondre(1, juste.clone()).unwrap();
        c.repondre(2, juste).unwrap();
        c.repondre(3, presque).unwrap();
        let r = c.resultat();
        assert_eq!((r.total, r.parfaits), (3, 2));
        assert_eq!(r.scores, vec![0, 0, 0, 1, 0, 2]);
        assert_eq!(r.bien_places, vec![2, 2, 3, 3, 3]);
        assert_eq!(r.moyenne, Some(4.3));
    }
}
