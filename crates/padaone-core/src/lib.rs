//! Domaine de Padaone.
//!
//! Ce crate contient toute la logique métier, sans réseau ni disque :
//! la séance, les participants, les activités (sondage, nuage de mots,
//! pouce en l'air, applaudimètre…) et le calcul des résultats.
//! Tout ce qui est ici doit être couvert par des tests unitaires.
//!
//! Organisation (voir `docs/03-specification.md`) :
//! - [`seance`] : la séance, son code, ses participants, son déroulé, les commandes ;
//! - [`activite`] : le trait commun à toutes les interactions et la collecte des réponses ;
//! - [`contenu`] : configurations, réponses et résultats de toutes les interactions ;
//! - un module par interaction : [`sondage`] (et cartons), [`nuage`], [`echelle`], [`meteo`],
//!   [`pari`], [`roti`], [`ouverte`], [`questions`] (à venir : `pouce`, `applaudimetre`…) ;
//! - [`signal`] : le signal discret (trop vite, perdu·e, exemple, pause), non journalisé ;
//! - [`evenement`] : les événements journalisés, rejoués à la reprise ;
//! - [`vue`] : ce que voit chaque rôle (participant, écran, console) ;
//! - [`mots`] : regroupement des mots du nuage ;

pub mod activite;
pub mod contenu;
pub mod echelle;
pub mod evenement;
pub mod meteo;
pub mod mots;
pub mod nuage;
pub mod ordre;
pub mod ouverte;
pub mod pari;
pub mod questions;
pub mod roti;
pub mod seance;
pub mod signal;
pub mod sondage;
pub mod vue;

pub use activite::{Activite, IdParticipant};
pub use evenement::Evenement;
pub use seance::{Commande, Configuration, ErreurSeance, IdActivite, Reponse, Resultat, Seance};
pub use signal::TypeSignal;

/// Nombre minimal de réponses avant d'afficher un résultat à l'écran.
/// Protège l'anonymat dans les petits groupes.
pub const SEUIL_ANONYMAT: usize = 3;

/// Indique si un résultat peut être projeté.
pub fn affichable(nombre_de_reponses: usize) -> bool {
    nombre_de_reponses >= SEUIL_ANONYMAT
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rien_ne_s_affiche_sous_le_seuil() {
        assert!(!affichable(0));
        assert!(!affichable(2));
        assert!(affichable(3));
    }
}
