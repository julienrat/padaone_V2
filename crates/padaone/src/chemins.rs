//! Dossiers de l'utilisateur selon le système, sans dépendance.
//!
//! | Système | Configuration (clé secrète) | Données (journaux) |
//! | --- | --- | --- |
//! | Linux | `$XDG_CONFIG_HOME` ou `~/.config` | `$XDG_DATA_HOME` ou `~/.local/share` |
//! | macOS | `~/Library/Application Support` | `~/Library/Application Support` |
//! | Windows | `%APPDATA%` | `%APPDATA%` |
//!
//! Padaone y crée son sous-dossier `padaone`.

use std::path::PathBuf;

fn variable(nom: &str) -> Option<PathBuf> {
    std::env::var_os(nom).filter(|v| !v.is_empty()).map(PathBuf::from)
}

#[cfg(target_os = "macos")]
fn base(_xdg: &str, _repli: &str) -> Option<PathBuf> {
    variable("HOME").map(|h| h.join("Library").join("Application Support"))
}

#[cfg(windows)]
fn base(_xdg: &str, _repli: &str) -> Option<PathBuf> {
    variable("APPDATA")
        .or_else(|| variable("USERPROFILE").map(|p| p.join("AppData").join("Roaming")))
}

#[cfg(not(any(target_os = "macos", windows)))]
fn base(xdg: &str, repli: &str) -> Option<PathBuf> {
    variable(xdg).or_else(|| variable("HOME").map(|h| h.join(repli)))
}

/// Dossier de configuration de Padaone (clé secrète).
pub fn configuration() -> Option<PathBuf> {
    base("XDG_CONFIG_HOME", ".config").map(|b| b.join("padaone"))
}

/// Dossier des données de Padaone (journaux de séance).
pub fn donnees() -> Option<PathBuf> {
    base("XDG_DATA_HOME", ".local/share").map(|b| b.join("padaone"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dossiers_propres_a_padaone() {
        if let Some(c) = configuration() {
            assert!(c.ends_with("padaone"));
        }
        if let Some(d) = donnees() {
            assert!(d.ends_with("padaone"));
        }
    }
}
