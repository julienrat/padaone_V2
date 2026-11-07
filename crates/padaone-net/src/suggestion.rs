//! Nom de réseau et mot de passe faciles à lire au tableau et à taper sur un téléphone :
//! « Padaone-Tilleul » et « tilleul-42 ». Aucune lettre ambiguë, pas d'accent.

/// Noms d'arbres et de plantes, de 6 lettres au moins (le mot de passe fait donc 9 caractères au moins).
const MOTS: &[&str] = &[
    "tilleul",
    "erable",
    "platane",
    "bouleau",
    "noisetier",
    "charme",
    "olivier",
    "cerisier",
    "peuplier",
    "sureau",
    "aubepine",
    "lavande",
    "romarin",
    "bruyere",
    "glycine",
    "mimosa",
    "pommier",
    "figuier",
    "magnolia",
    "acacia",
    "cypres",
    "genet",
    "framboise",
    "myrtille",
    "camelia",
    "jasmin",
    "verveine",
    "sauge",
    "ortie",
    "trefle",
    "chataigne",
    "noyer",
];

/// Mot choisi à partir d'un nombre au hasard.
fn mot(hasard: u32) -> &'static str {
    MOTS[hasard as usize % MOTS.len()]
}

/// « tilleul-42 » : un mot et un nombre de 10 à 99.
pub fn mot_de_passe(hasard: u32) -> String {
    let nombre = (hasard / MOTS.len() as u32) % 90 + 10;
    format!("{}-{nombre}", mot(hasard))
}

/// « Padaone-Tilleul ».
pub fn nom_reseau(hasard: u32) -> String {
    let m = mot(hasard);
    let mut c = m.chars();
    let majuscule: String =
        c.next().map(|p| p.to_uppercase().chain(c).collect()).unwrap_or_default();
    format!("Padaone-{majuscule}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mots_de_passe_valides_pour_wpa2() {
        for h in [0, 1, 31, 32, 1000, u32::MAX] {
            let m = mot_de_passe(h);
            assert!((8..=63).contains(&m.len()), "{m}");
            assert!(m.is_ascii());
        }
        assert_eq!(mot_de_passe(0), "tilleul-10");
    }

    #[test]
    fn nom_avec_majuscule() {
        assert_eq!(nom_reseau(0), "Padaone-Tilleul");
        assert!(nom_reseau(u32::MAX).len() <= 32);
    }
}
