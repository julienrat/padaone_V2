//! Journal de séance au format JSON Lines : un événement par ligne, avec l'heure.
//!
//! Le fichier vit hors du dépôt (`~/.local/share/padaone/seances/` par défaut).
//! Chaque ligne est écrite et vidée sur le disque immédiatement : un plantage
//! ne perd au plus que l'événement en cours. `padaone --reprendre <journal>`
//! relit le fichier et continue d'y écrire.

use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use padaone_core::Evenement;
use serde::{Deserialize, Serialize};

/// Une ligne du journal : l'heure (millisecondes depuis 1970) et l'événement.
#[derive(Serialize, Deserialize)]
struct Ligne {
    h: u64,
    #[serde(flatten)]
    evenement: Evenement,
}

pub struct Journal {
    chemin: PathBuf,
    fichier: File,
}

impl Journal {
    /// Dossier par défaut : `seances` dans le dossier des données (voir [`crate::chemins`]).
    pub fn dossier_par_defaut() -> Option<PathBuf> {
        crate::chemins::donnees().map(|d| d.join("seances"))
    }

    /// Crée un nouveau journal dans `dossier`, nommé d'après l'heure et le code.
    pub fn creer(dossier: &Path, code: &str) -> Result<Self> {
        fs::create_dir_all(dossier)
            .with_context(|| format!("impossible de créer le dossier {}", dossier.display()))?;
        let chemin = dossier.join(format!("seance-{}-{code}.jsonl", maintenant() / 1000));
        let fichier = OpenOptions::new()
            .create_new(true)
            .append(true)
            .open(&chemin)
            .with_context(|| format!("impossible de créer {}", chemin.display()))?;
        Ok(Self { chemin, fichier })
    }

    /// Rouvre un journal existant pour y ajouter les événements suivants.
    pub fn rouvrir(chemin: &Path) -> Result<Self> {
        let fichier = OpenOptions::new()
            .append(true)
            .open(chemin)
            .with_context(|| format!("impossible d'ouvrir {}", chemin.display()))?;
        Ok(Self { chemin: chemin.to_path_buf(), fichier })
    }

    /// Lit tous les événements d'un journal, dans l'ordre.
    pub fn lire(chemin: &Path) -> Result<Vec<Evenement>> {
        let fichier = File::open(chemin)
            .with_context(|| format!("impossible de lire {}", chemin.display()))?;
        let mut evenements = Vec::new();
        for (i, ligne) in BufReader::new(fichier).lines().enumerate() {
            let ligne = ligne?;
            if ligne.trim().is_empty() {
                continue;
            }
            let l: Ligne = serde_json::from_str(&ligne)
                .with_context(|| format!("ligne {} illisible dans {}", i + 1, chemin.display()))?;
            evenements.push(l.evenement);
        }
        Ok(evenements)
    }

    pub fn chemin(&self) -> &Path {
        &self.chemin
    }

    /// Ajoute un événement en fin de journal et le pousse sur le disque.
    pub fn ecrire(&mut self, evenement: &Evenement) -> Result<()> {
        let mut texte =
            serde_json::to_string(&Ligne { h: maintenant(), evenement: evenement.clone() })?;
        texte.push('\n');
        self.fichier.write_all(texte.as_bytes())?;
        self.fichier.sync_data()?;
        Ok(())
    }
}

fn maintenant() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_millis() as u64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use padaone_core::{Commande, Seance};

    #[test]
    fn ecrire_puis_relire() {
        let dossier = std::env::temp_dir().join(format!("padaone-test-{}", maintenant()));
        let (mut s, e0) = Seance::ouvrir("Test", "123456").unwrap();
        let mut j = Journal::creer(&dossier, "123456").unwrap();
        j.ecrire(&e0).unwrap();
        j.ecrire(&s.executer(Commande::Accueil).unwrap()).unwrap();
        let (_, e) = s.inscrire("jeton");
        j.ecrire(&e.unwrap()).unwrap();

        let relus = Journal::lire(j.chemin()).unwrap();
        assert_eq!(relus.len(), 3);
        assert_eq!(relus[0], e0);
        assert_eq!(relus[1], Evenement::RetourAccueil);
        assert!(Seance::rejouer(&relus).unwrap().participant("jeton").is_some());
        fs::remove_dir_all(dossier).unwrap();
    }
}
