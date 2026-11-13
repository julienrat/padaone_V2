//! État partagé du serveur : la séance, le journal et les connexions ouvertes.
//!
//! L'état est protégé par un verrou court (jamais tenu pendant un `await`).
//! Chaque modification lève un drapeau ; une tâche de diffusion le relève au plus
//! 10 fois par seconde et réveille alors toutes les connexions, qui recalculent leur vue.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use padaone_core::{Commande, Evenement, IdParticipant, Seance};
use tokio::sync::watch;

use crate::journal::Journal;
use padaone_net::Executeur;

use crate::reseau::EtatReseau;

/// Intervalle minimal entre deux diffusions : 10 mises à jour par seconde au plus.
pub const INTERVALLE_DIFFUSION: Duration = Duration::from_millis(100);
/// Toutes les combien de diffusions on purge les signaux expirés (une fois par seconde).
const PURGES_PAR_TOUR: u32 = 10;

pub struct Etat {
    pub seance: Seance,
    journal: Option<Journal>,
    /// Connexions ouvertes : participant associé (`None` pour l'écran et la console).
    connexions: HashMap<u64, Option<IdParticipant>>,
}

impl Etat {
    /// Journalise un événement déjà appliqué à la séance.
    pub fn journaliser(&mut self, evenement: &Evenement) {
        if let Some(j) = &mut self.journal {
            if let Err(e) = j.ecrire(evenement) {
                tracing::error!("journal : écriture impossible ({e:#})");
            }
        }
    }

    /// Nombre de participants dont au moins un téléphone est connecté.
    pub fn connectes(&self) -> usize {
        let mut ids: Vec<IdParticipant> = self.connexions.values().flatten().copied().collect();
        ids.sort_unstable();
        ids.dedup();
        ids.len()
    }
}

pub struct Partage {
    etat: Mutex<Etat>,
    /// Clé secrète exigée par l'écran et la console.
    pub cle: String,
    /// Adresses annoncées et point d'accès (verrou séparé : jamais tenu pendant une commande réseau).
    reseau: Mutex<EtatReseau>,
    /// Exécution des commandes réseau (`nmcli`, ou simulation).
    pub executeur: Arc<dyn Executeur>,
    change: AtomicBool,
    prochaine_connexion: AtomicU64,
    diffusion: watch::Sender<u64>,
    debut: Instant,
}

impl Partage {
    pub fn new(
        seance: Seance,
        journal: Option<Journal>,
        cle: String,
        reseau: EtatReseau,
        executeur: Arc<dyn Executeur>,
    ) -> Arc<Self> {
        let (diffusion, _) = watch::channel(0);
        let partage = Arc::new(Partage {
            etat: Mutex::new(Etat { seance, journal, connexions: HashMap::new() }),
            cle,
            reseau: Mutex::new(reseau),
            executeur,
            change: AtomicBool::new(false),
            prochaine_connexion: AtomicU64::new(1),
            diffusion,
            debut: Instant::now(),
        });
        tokio::spawn(diffuser(Arc::clone(&partage)));
        tokio::spawn(crate::reseau::surveiller(Arc::clone(&partage)));
        partage
    }

    /// État du réseau, en lecture.
    pub fn reseau(&self) -> MutexGuard<'_, EtatReseau> {
        self.reseau.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Modifie l'état du réseau et programme une diffusion.
    pub fn modifier_reseau<R>(&self, f: impl FnOnce(&mut EtatReseau) -> R) -> R {
        let r = f(&mut self.reseau());
        self.change.store(true, Ordering::Release);
        r
    }

    /// Accès en lecture seule (le drapeau de changement n'est pas levé).
    pub fn lire(&self) -> MutexGuard<'_, Etat> {
        // Un verrou empoisonné signifie qu'une tâche a paniqué en le tenant :
        // l'état reste cohérent car chaque modification est validée avant d'être appliquée.
        self.etat.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Modifie l'état et programme une diffusion.
    pub fn modifier<R>(&self, f: impl FnOnce(&mut Etat) -> R) -> R {
        let r = f(&mut self.lire());
        self.change.store(true, Ordering::Release);
        r
    }

    /// Enregistre une connexion et renvoie son numéro.
    pub fn ouvrir_connexion(&self, participant: Option<IdParticipant>) -> u64 {
        let n = self.prochaine_connexion.fetch_add(1, Ordering::Relaxed);
        self.modifier(|e| e.connexions.insert(n, participant));
        n
    }

    pub fn fermer_connexion(&self, n: u64) {
        self.modifier(|e| e.connexions.remove(&n));
    }

    /// Instant présent en millisecondes depuis le lancement (pour les signaux éphémères).
    pub fn maintenant(&self) -> u64 {
        // Millisecondes depuis 1970 : l'heure de fin d'une minuterie, journalisée, reste valable
        // après une reprise. `debut` sert de repli si l'horloge du système est avant 1970.
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or_else(|_| self.debut.elapsed().as_millis(), |d| d.as_millis())
            .try_into()
            .unwrap_or(u64::MAX)
    }

    /// Récepteur réveillé à chaque diffusion.
    pub fn abonner(&self) -> watch::Receiver<u64> {
        self.diffusion.subscribe()
    }
}

/// Tâche de diffusion : au plus une notification par intervalle, seulement si l'état a changé.
/// Une fois par seconde, elle efface aussi les signaux discrets expirés.
async fn diffuser(partage: Arc<Partage>) {
    let mut horloge = tokio::time::interval(INTERVALLE_DIFFUSION);
    horloge.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    let mut tour = 0u32;
    loop {
        horloge.tick().await;
        tour = (tour + 1) % PURGES_PAR_TOUR;
        if tour == 0 {
            let maintenant = partage.maintenant();
            let mut etat = partage.lire();
            if etat.seance.a_des_signaux() && etat.seance.purger_signaux(maintenant) {
                partage.change.store(true, Ordering::Release);
            }
            // Minuteries échues : on fige l'activité (événement journalisé).
            for id in etat.seance.minuteries_echues(maintenant) {
                if let Ok(ev) = etat.seance.executer(Commande::Figer { id: Some(id), on: true }) {
                    tracing::info!("minuterie échue : activité {id} figée");
                    etat.journaliser(&ev);
                    partage.change.store(true, Ordering::Release);
                }
            }
        }
        if partage.change.swap(false, Ordering::AcqRel) {
            partage.diffusion.send_modify(|n| *n = n.wrapping_add(1));
        }
    }
}
