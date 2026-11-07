//! Valeurs aléatoires tirées du hasard du système : code de séance, jetons, clé secrète.

use anyhow::{anyhow, Result};

/// Code de séance à 6 chiffres, par exemple « 482731 ».
pub fn code_seance() -> Result<String> {
    let n = getrandom::u32().map_err(|e| anyhow!("hasard indisponible : {e}"))?;
    Ok(format!("{:06}", n % 1_000_000))
}

/// Chaîne hexadécimale de 32 caractères (128 bits), pour les jetons et la clé secrète.
pub fn hexa() -> Result<String> {
    let mut octets = [0u8; 16];
    getrandom::fill(&mut octets).map_err(|e| anyhow!("hasard indisponible : {e}"))?;
    Ok(octets.iter().map(|o| format!("{o:02x}")).collect())
}

/// Clé secrète de l'écran et de la console, gardée d'un lancement à l'autre dans `fichier`
/// (lisible par l'utilisateur seul) : les pages ouvertes se reconnectent après une relance.
/// `nouvelle` : en tirer une autre (par exemple si un lien a été projeté par erreur).
pub fn cle_persistante(fichier: &std::path::Path, nouvelle: bool) -> Result<String> {
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;

    if !nouvelle {
        if let Ok(texte) = std::fs::read_to_string(fichier) {
            let cle = texte.trim();
            if est_jeton(cle) {
                return Ok(cle.to_string());
            }
        }
    }
    let cle = hexa()?;
    if let Some(dossier) = fichier.parent() {
        std::fs::create_dir_all(dossier)?;
    }
    let _ = std::fs::remove_file(fichier);
    let mut f =
        std::fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(fichier)?;
    writeln!(f, "{cle}")?;
    Ok(cle)
}

/// Emplacement par défaut : `$XDG_CONFIG_HOME/padaone/cle` ou `~/.config/padaone/cle`.
pub fn fichier_cle() -> Option<std::path::PathBuf> {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .filter(|v| !v.is_empty())
        .map(std::path::PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME").map(|h| std::path::PathBuf::from(h).join(".config"))
        })?;
    Some(base.join("padaone").join("cle"))
}

/// Vrai si la chaîne a la forme d'un jeton produit par [`hexa`].
pub fn est_jeton(texte: &str) -> bool {
    texte.len() == 32 && texte.bytes().all(|c| c.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn code_a_six_chiffres() {
        for _ in 0..50 {
            let c = code_seance().unwrap();
            assert_eq!(c.len(), 6);
            assert!(c.bytes().all(|b| b.is_ascii_digit()));
        }
    }

    #[test]
    fn cle_gardee_puis_renouvelee() {
        use std::os::unix::fs::PermissionsExt;
        let dossier = std::env::temp_dir().join(format!("padaone-cle-{}", hexa().unwrap()));
        let fichier = dossier.join("cle");
        let a = cle_persistante(&fichier, false).unwrap();
        assert_eq!(cle_persistante(&fichier, false).unwrap(), a, "même clé au lancement suivant");
        let mode = std::fs::metadata(&fichier).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600, "lisible par l'utilisateur seul");
        let b = cle_persistante(&fichier, true).unwrap();
        assert_ne!(a, b);
        assert_eq!(cle_persistante(&fichier, false).unwrap(), b);
        std::fs::remove_dir_all(dossier).unwrap();
    }

    #[test]
    fn jetons_distincts_et_reconnus() {
        let a = hexa().unwrap();
        assert!(est_jeton(&a));
        assert_ne!(a, hexa().unwrap());
        assert!(!est_jeton("abc"));
        assert!(!est_jeton(&"z".repeat(32)));
    }
}
