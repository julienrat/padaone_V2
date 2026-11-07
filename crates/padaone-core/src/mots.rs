//! Regroupement des mots du nuage de mots.
//!
//! Deux saisies qui ne diffèrent que par la casse, les accents, un pluriel
//! simple ou un article de tête reçoivent la même clé :
//! « Écoute », « écoutes » et « l'écoute » comptent pour un seul mot.

/// Mots vides retirés en tête d'expression.
const ARTICLES: &[&str] = &["le", "la", "les", "un", "une", "des", "du", "de", "l", "d"];

/// Renvoie la clé de regroupement d'une saisie, ou `None` si elle est vide
/// une fois nettoyée.
pub fn cle(saisie: &str) -> Option<String> {
    let propre: String = saisie
        .to_lowercase()
        .chars()
        .map(sans_accent)
        .map(|c| if c.is_alphanumeric() { c } else { ' ' })
        .collect();

    let mut mots: Vec<&str> = propre.split_whitespace().collect();
    while mots.len() > 1 && ARTICLES.contains(&mots[0]) {
        mots.remove(0);
    }
    if mots.is_empty() || (mots.len() == 1 && ARTICLES.contains(&mots[0])) {
        return None;
    }
    let mots: Vec<String> = mots.into_iter().map(singulier).collect();
    Some(mots.join(" "))
}

/// Pluriel simple : retire un « s » ou un « x » final sur les mots de plus de 3 lettres.
fn singulier(mot: &str) -> String {
    let n = mot.chars().count();
    if n > 3 && (mot.ends_with('s') || mot.ends_with('x')) && !mot.ends_with("ss") {
        mot[..mot.len() - 1].to_string()
    } else {
        mot.to_string()
    }
}

fn sans_accent(c: char) -> char {
    match c {
        'à' | 'â' | 'ä' | 'á' | 'ã' | 'å' => 'a',
        'ç' => 'c',
        'é' | 'è' | 'ê' | 'ë' => 'e',
        'î' | 'ï' | 'í' | 'ì' => 'i',
        'ô' | 'ö' | 'ó' | 'ò' | 'õ' => 'o',
        'û' | 'ü' | 'ú' | 'ù' => 'u',
        'ÿ' | 'ý' => 'y',
        'ñ' => 'n',
        'œ' => 'o',
        'æ' => 'a',
        _ => c,
    }
}

#[cfg(test)]
mod tests {
    use super::cle;

    #[test]
    fn casse_accents_et_pluriels_regroupes() {
        assert_eq!(cle("Écoute"), cle("ecoutes"));
        assert_eq!(cle("réunion"), cle("Réunions"));
        assert_eq!(cle("l'écoute"), cle("écoute"));
    }

    #[test]
    fn expressions_conservees() {
        assert_eq!(cle("  Ordre du   jour "), Some("ordre du jour".to_string()));
    }

    #[test]
    fn saisies_vides_ignorees() {
        assert_eq!(cle(""), None);
        assert_eq!(cle(" ... "), None);
        assert_eq!(cle("le"), None);
    }

    #[test]
    fn mots_courts_et_doubles_s_intacts() {
        assert_eq!(cle("bus"), Some("bus".to_string()));
        assert_eq!(cle("stress"), Some("stress".to_string()));
    }
}
