//! Mini-serveur DNS du portail captif : construction des réponses (fonctions pures).
//!
//! Le dnsmasq que NetworkManager lance pour le point d'accès transmet toutes les questions
//! à Padaone (`server=127.0.0.1#5380`, voir [`crate::portail`]). Padaone répond :
//! - type A : l'adresse du PC, pour tous les noms (portail ouvert ou silencieux),
//!   ou seulement pour `padaone.lan` (portail désactivé : les autres noms n'existent pas) ;
//! - autres types (AAAA…) : réponse vide, pour que les téléphones se rabattent sur l'IPv4.
//!
//! Aucune dépendance : on ne lit que l'en-tête et la première question.

/// Port local où Padaone répond aux questions transmises par dnsmasq.
pub const PORT_DNS: u16 = 5380;
/// Durée de vie des réponses, courte : l'adresse du PC change quand on change de salle.
const TTL: u32 = 10;
/// Nom simple du PC sur le point d'accès.
pub const NOM_SIMPLE: &str = "padaone.lan";

const TYPE_A: u16 = 1;
const CLASSE_IN: u16 = 1;

/// Une question DNS lue dans une requête.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Question {
    pub nom: String,
    pub genre: u16,
    pub classe: u16,
    /// Fin de la section question dans la requête (octet suivant).
    fin: usize,
}

/// Lit la première question d'une requête. `None` si la requête est illisible ou n'en est pas une.
pub fn lire_question(requete: &[u8]) -> Option<Question> {
    if requete.len() < 12 {
        return None;
    }
    let drapeaux = u16::from_be_bytes([requete[2], requete[3]]);
    let est_reponse = drapeaux & 0x8000 != 0;
    let opcode = (drapeaux >> 11) & 0xF;
    let questions = u16::from_be_bytes([requete[4], requete[5]]);
    if est_reponse || opcode != 0 || questions == 0 {
        return None;
    }
    let mut i = 12;
    let mut etiquettes = Vec::new();
    loop {
        let n = usize::from(*requete.get(i)?);
        if n == 0 {
            i += 1;
            break;
        }
        // Pas de compression dans une question de requête.
        if n & 0xC0 != 0 || etiquettes.len() > 127 {
            return None;
        }
        let etiquette = requete.get(i + 1..i + 1 + n)?;
        etiquettes.push(String::from_utf8_lossy(etiquette).to_ascii_lowercase());
        i += 1 + n;
    }
    let genre = u16::from_be_bytes([*requete.get(i)?, *requete.get(i + 1)?]);
    let classe = u16::from_be_bytes([*requete.get(i + 2)?, *requete.get(i + 3)?]);
    Some(Question { nom: etiquettes.join("."), genre, classe, fin: i + 4 })
}

/// Construit la réponse à une requête.
/// `tous_les_noms` : vrai en portail ouvert ou silencieux, faux en portail désactivé.
pub fn repondre(requete: &[u8], ip: [u8; 4], tous_les_noms: bool) -> Option<Vec<u8>> {
    let q = lire_question(requete)?;
    let connu = tous_les_noms || q.nom == NOM_SIMPLE;
    let repondre_a = connu && q.genre == TYPE_A && q.classe == CLASSE_IN;
    // QR=1, AA=1, RD recopié, RA=1 ; code 3 (nom inexistant) pour un nom inconnu.
    let rd = requete[2] & 0x01;
    let code = if connu { 0 } else { 3 };
    let mut r = Vec::with_capacity(q.fin + 16);
    r.extend_from_slice(&requete[..2]);
    r.push(0x84 | rd);
    r.push(0x80 | code);
    r.extend_from_slice(&1u16.to_be_bytes()); // une question
    r.extend_from_slice(&u16::from(repondre_a).to_be_bytes()); // réponses
    r.extend_from_slice(&[0, 0, 0, 0]); // autorité, additionnel
    r.extend_from_slice(&requete[12..q.fin]);
    if repondre_a {
        r.extend_from_slice(&[0xC0, 0x0C]); // pointeur vers le nom de la question
        r.extend_from_slice(&TYPE_A.to_be_bytes());
        r.extend_from_slice(&CLASSE_IN.to_be_bytes());
        r.extend_from_slice(&TTL.to_be_bytes());
        r.extend_from_slice(&4u16.to_be_bytes());
        r.extend_from_slice(&ip);
    }
    Some(r)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Requête type A pour `nom`, identifiant 0x1234, récursion demandée.
    fn requete(nom: &str, genre: u16) -> Vec<u8> {
        let mut r = vec![0x12, 0x34, 0x01, 0x00, 0, 1, 0, 0, 0, 0, 0, 0];
        for e in nom.split('.') {
            r.push(e.len() as u8);
            r.extend_from_slice(e.as_bytes());
        }
        r.push(0);
        r.extend_from_slice(&genre.to_be_bytes());
        r.extend_from_slice(&1u16.to_be_bytes());
        r
    }

    const IP: [u8; 4] = [192, 168, 1, 23];

    #[test]
    fn question_lue() {
        let q = lire_question(&requete("ConnectivityCheck.gstatic.com", 1)).unwrap();
        assert_eq!(q.nom, "connectivitycheck.gstatic.com");
        assert_eq!((q.genre, q.classe), (1, 1));
    }

    #[test]
    fn tous_les_noms_menent_au_pc() {
        let r = repondre(&requete("connectivitycheck.gstatic.com", 1), IP, true).unwrap();
        assert_eq!(&r[..2], &[0x12, 0x34], "même identifiant");
        assert_eq!(r[2] & 0x80, 0x80, "c'est une réponse");
        assert_eq!(r[3] & 0x0F, 0, "pas d'erreur");
        assert_eq!(u16::from_be_bytes([r[6], r[7]]), 1, "une réponse");
        assert_eq!(&r[r.len() - 4..], &IP);
    }

    #[test]
    fn ipv6_sans_reponse() {
        let r = repondre(&requete("captive.apple.com", 28), IP, true).unwrap();
        assert_eq!(u16::from_be_bytes([r[6], r[7]]), 0);
        assert_eq!(r[3] & 0x0F, 0);
    }

    #[test]
    fn portail_desactive_seul_padaone_lan_existe() {
        let r = repondre(&requete("captive.apple.com", 1), IP, false).unwrap();
        assert_eq!(r[3] & 0x0F, 3, "nom inexistant");
        let r = repondre(&requete("padaone.lan", 1), IP, false).unwrap();
        assert_eq!(&r[r.len() - 4..], &IP);
    }

    #[test]
    fn requetes_illisibles_ignorees() {
        assert_eq!(lire_question(&[0; 5]), None);
        let mut reponse = requete("a.b", 1);
        reponse[2] |= 0x80;
        assert_eq!(lire_question(&reponse), None);
        let tronquee = &requete("padaone.lan", 1)[..15];
        assert_eq!(lire_question(tronquee), None);
    }
}
