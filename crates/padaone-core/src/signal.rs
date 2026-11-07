//! Signal discret : une barre toujours présente sur le téléphone (trop vite, perdu·e,
//! un exemple, une pause). Les compteurs ne s'affichent que sur la console, jamais à l'écran.
//!
//! Un signal s'efface après 3 minutes ou quand le participant le retouche.
//! Alerte quand un signal concerne au moins 20 % des participants connectés.
//! Les signaux sont éphémères : ils ne sont pas journalisés.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::activite::IdParticipant;

/// Durée de vie d'un signal, en millisecondes.
pub const DUREE_SIGNAL_MS: u64 = 3 * 60 * 1000;
/// Part des connectés à partir de laquelle la console alerte, en pourcentage.
pub const SEUIL_ALERTE_POURCENT: usize = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TypeSignal {
    Vite,
    Perdu,
    Exemple,
    Pause,
}

impl TypeSignal {
    pub const TOUS: [TypeSignal; 4] =
        [TypeSignal::Vite, TypeSignal::Perdu, TypeSignal::Exemple, TypeSignal::Pause];
}

#[derive(Debug, Clone, Default)]
pub struct Signaux {
    /// (type, participant) → instant d'émission en millisecondes.
    actifs: BTreeMap<(TypeSignal, IdParticipant), u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CompteSignal {
    #[serde(rename = "type")]
    pub genre: TypeSignal,
    pub n: usize,
    pub alerte: bool,
}

impl Signaux {
    /// Active le signal, ou l'éteint s'il était déjà actif. Renvoie vrai s'il est actif.
    pub fn basculer(&mut self, p: IdParticipant, genre: TypeSignal, maintenant: u64) -> bool {
        self.purger(maintenant);
        if self.actifs.remove(&(genre, p)).is_some() {
            false
        } else {
            self.actifs.insert((genre, p), maintenant);
            true
        }
    }

    /// Retire les signaux expirés. Renvoie vrai si quelque chose a changé.
    pub fn purger(&mut self, maintenant: u64) -> bool {
        let avant = self.actifs.len();
        self.actifs.retain(|_, &mut t| maintenant.saturating_sub(t) < DUREE_SIGNAL_MS);
        avant != self.actifs.len()
    }

    pub fn est_vide(&self) -> bool {
        self.actifs.is_empty()
    }

    /// Signaux actifs d'un participant (pour allumer ses boutons).
    pub fn de(&self, p: IdParticipant, maintenant: u64) -> Vec<TypeSignal> {
        self.actifs
            .iter()
            .filter(|((_, q), &t)| *q == p && maintenant.saturating_sub(t) < DUREE_SIGNAL_MS)
            .map(|((g, _), _)| *g)
            .collect()
    }

    /// Compteurs pour la console. `connectes` : participants connectés en ce moment.
    pub fn compter(&self, maintenant: u64, connectes: usize) -> Vec<CompteSignal> {
        TypeSignal::TOUS
            .iter()
            .map(|&genre| {
                let n = self
                    .actifs
                    .iter()
                    .filter(|((g, _), &t)| {
                        *g == genre && maintenant.saturating_sub(t) < DUREE_SIGNAL_MS
                    })
                    .count();
                let alerte = n > 0 && n * 100 >= SEUIL_ALERTE_POURCENT * connectes.max(1);
                CompteSignal { genre, n, alerte }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn n(s: &Signaux, genre: TypeSignal, t: u64, connectes: usize) -> (usize, bool) {
        let c = s.compter(t, connectes).into_iter().find(|c| c.genre == genre).unwrap();
        (c.n, c.alerte)
    }

    #[test]
    fn basculer_allume_puis_eteint() {
        let mut s = Signaux::default();
        assert!(s.basculer(1, TypeSignal::Perdu, 0));
        assert_eq!(s.de(1, 0), vec![TypeSignal::Perdu]);
        assert!(!s.basculer(1, TypeSignal::Perdu, 10));
        assert!(s.de(1, 10).is_empty());
    }

    #[test]
    fn efface_apres_trois_minutes() {
        let mut s = Signaux::default();
        s.basculer(1, TypeSignal::Vite, 0);
        assert_eq!(n(&s, TypeSignal::Vite, DUREE_SIGNAL_MS - 1, 10).0, 1);
        assert_eq!(n(&s, TypeSignal::Vite, DUREE_SIGNAL_MS, 10).0, 0);
        assert!(s.purger(DUREE_SIGNAL_MS));
        assert!(s.est_vide());
    }

    #[test]
    fn alerte_a_vingt_pour_cent() {
        let mut s = Signaux::default();
        s.basculer(1, TypeSignal::Perdu, 0);
        assert_eq!(n(&s, TypeSignal::Perdu, 0, 10), (1, false));
        s.basculer(2, TypeSignal::Perdu, 0);
        assert_eq!(n(&s, TypeSignal::Perdu, 0, 10), (2, true));
        assert_eq!(n(&s, TypeSignal::Pause, 0, 10), (0, false));
    }
}
