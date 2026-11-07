//! Connexions WebSocket (`/ws`) : une tâche par client.
//!
//! 1. Le client envoie `bonjour` avec son rôle (et son jeton, son code ou la clé).
//! 2. Le serveur répond `bienvenue` puis envoie la vue du rôle (`etat`).
//! 3. Ensuite : les messages du client sont traités, et la vue est renvoyée
//!    à chaque diffusion (au plus 10 fois par seconde).

use std::sync::Arc;
use std::time::Duration;

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::response::Response;
use padaone_core::IdParticipant;

use crate::alea;
use crate::etat::Partage;
use crate::protocole::{
    etat, etat_avec, CommandeConsole, InfoSeance, MessageClient, MessageServeur, Role,
};
use crate::reseau;

/// Taille maximale d'un message reçu : largement assez pour une commande.
const TAILLE_MAX: usize = 16 * 1024;
/// Délai laissé au client pour dire `bonjour`.
const DELAI_BONJOUR: Duration = Duration::from_secs(10);

pub async fn route(State(partage): State<Arc<Partage>>, upgrade: WebSocketUpgrade) -> Response {
    upgrade.max_message_size(TAILLE_MAX).on_upgrade(move |socket| connexion(socket, partage))
}

/// Identité du client une fois le `bonjour` accepté.
#[derive(Clone, Copy)]
enum Client {
    Participant(IdParticipant),
    Ecran,
    Console,
}

async fn connexion(mut socket: WebSocket, partage: Arc<Partage>) {
    let client = match accueillir(&mut socket, &partage).await {
        Ok(c) => c,
        Err(message) => {
            let m = MessageServeur::Erreur { message, fatale: true };
            let _ = socket.send(Message::Text(m.json().into())).await;
            return;
        }
    };
    let participant = match client {
        Client::Participant(id) => Some(id),
        _ => None,
    };
    let numero = partage.ouvrir_connexion(participant);
    let mut diffusion = partage.abonner();
    // Version des adresses déjà envoyée (écran et console reçoivent `reseau` à chaque changement).
    let mut version_reseau = partage.reseau().version;

    if envoyer(&mut socket, vue(&partage, client)).await {
        loop {
            tokio::select! {
                recu = socket.recv() => match recu {
                    Some(Ok(Message::Text(texte))) => {
                        if let Some(reponse) = traiter(&partage, client, &texte) {
                            if !envoyer(&mut socket, reponse.json()).await {
                                break;
                            }
                        }
                    }
                    Some(Ok(Message::Close(_))) | None | Some(Err(_)) => break,
                    Some(Ok(_)) => {}
                },
                change = diffusion.changed() => {
                    if change.is_err() {
                        break;
                    }
                    if !matches!(client, Client::Participant(_)) {
                        let info = {
                            let r = partage.reseau();
                            (r.version != version_reseau).then(|| (r.version, r.info.clone()))
                        };
                        if let Some((v, info)) = info {
                            version_reseau = v;
                            if !envoyer(&mut socket, MessageServeur::Reseau(info).json()).await {
                                break;
                            }
                        }
                    }
                    if !envoyer(&mut socket, vue(&partage, client)).await {
                        break;
                    }
                }
            }
        }
    }
    partage.fermer_connexion(numero);
}

/// Attend le `bonjour`, vérifie le code ou la clé, et envoie `bienvenue`.
async fn accueillir(socket: &mut WebSocket, partage: &Partage) -> Result<Client, String> {
    let premier = tokio::time::timeout(DELAI_BONJOUR, socket.recv())
        .await
        .map_err(|_| "Délai dépassé.".to_string())?;
    let Some(Ok(Message::Text(texte))) = premier else {
        return Err("Connexion interrompue.".into());
    };
    let Ok(MessageClient::Bonjour { role, jeton, code, cle }) = serde_json::from_str(&texte) else {
        return Err("Le premier message doit être « bonjour ».".into());
    };

    let (client, jeton) = match role {
        Role::Participant => {
            let (code_seance, connu) = {
                let e = partage.lire();
                let connu = jeton.as_deref().and_then(|j| e.seance.participant(j));
                (e.seance.code().to_string(), connu)
            };
            if code.as_deref().map(|c| c.replace(' ', "")) != Some(code_seance) {
                return Err("Ce code ne correspond pas à la séance en cours.".into());
            }
            match (connu, jeton) {
                (Some(id), Some(j)) => (Client::Participant(id), Some(j)),
                _ => {
                    let j = alea::hexa().map_err(|e| e.to_string())?;
                    debug_assert!(alea::est_jeton(&j));
                    let id = partage.modifier(|e| {
                        let (id, evenement) = e.seance.inscrire(&j);
                        if let Some(ev) = evenement {
                            e.journaliser(&ev);
                        }
                        id
                    });
                    tracing::info!("participant n° {id} inscrit");
                    (Client::Participant(id), Some(j))
                }
            }
        }
        Role::Ecran | Role::Console => {
            if cle.as_deref() != Some(partage.cle.as_str()) {
                return Err("Clé absente ou incorrecte : utilisez le lien affiché au lancement \
                            de Padaone."
                    .into());
            }
            let c = if role == Role::Ecran { Client::Ecran } else { Client::Console };
            (c, None)
        }
    };

    let seance = {
        let e = partage.lire();
        InfoSeance { titre: e.seance.titre().into(), code: e.seance.code().into() }
    };
    let reseau = match client {
        Client::Participant(_) => None,
        _ => Some(partage.reseau().info.clone()),
    };
    let bienvenue = MessageServeur::Bienvenue { jeton, seance, reseau };
    if envoyer(socket, bienvenue.json()).await {
        Ok(client)
    } else {
        Err("Connexion interrompue.".into())
    }
}

/// Traite un message du client ; renvoie éventuellement une réponse directe.
fn traiter(partage: &Arc<Partage>, client: Client, texte: &str) -> Option<MessageServeur> {
    let message: MessageClient = match serde_json::from_str(texte) {
        Ok(m) => m,
        Err(_) => return Some(MessageServeur::erreur("Message illisible.")),
    };
    match (message, client) {
        (MessageClient::Ping, _) => Some(MessageServeur::Pong),
        (MessageClient::Reponse { activite, reponse }, Client::Participant(id)) => partage
            .modifier(|e| match e.seance.repondre(id, activite, reponse) {
                Ok(ev) => {
                    e.journaliser(&ev);
                    None
                }
                Err(err) => Some(MessageServeur::erreur(majuscule(&err.to_string()))),
            }),
        (MessageClient::Signal { genre }, Client::Participant(id)) => {
            let maintenant = partage.maintenant();
            partage.modifier(|e| match e.seance.signaler(id, genre, maintenant) {
                Ok(_) => None,
                Err(err) => Some(MessageServeur::erreur(majuscule(&err.to_string()))),
            })
        }
        (MessageClient::Commande(CommandeConsole::Reseau(commande)), Client::Console) => {
            reseau::lancer(partage, commande).map(MessageServeur::erreur)
        }
        (MessageClient::Commande(CommandeConsole::Seance(commande)), Client::Console) => partage
            .modifier(|e| match e.seance.executer(commande) {
                Ok(ev) => {
                    tracing::info!("console : {ev:?}");
                    e.journaliser(&ev);
                    None
                }
                Err(err) => Some(MessageServeur::erreur(majuscule(&err.to_string()))),
            }),
        (
            MessageClient::Reponse { .. }
            | MessageClient::Commande(_)
            | MessageClient::Signal { .. },
            _,
        ) => Some(MessageServeur::erreur("Action non autorisée pour ce rôle.")),
        (MessageClient::Bonjour { .. } | MessageClient::Autre, _) => None,
    }
}

/// Vue propre au client, au format du message `etat`.
fn vue(partage: &Partage, client: Client) -> String {
    let maintenant = partage.maintenant();
    let e = partage.lire();
    match client {
        Client::Participant(id) => etat(&e.seance.vue_participant(id, maintenant)),
        Client::Ecran => etat(&e.seance.vue_ecran(e.connectes())),
        Client::Console => {
            let point_acces = serde_json::to_value(&partage.reseau().point_acces).ok();
            etat_avec(
                &e.seance.vue_console(e.connectes(), maintenant),
                point_acces.map(|v| ("point_acces", v)),
            )
        }
    }
}

/// Envoie un texte ; renvoie faux si la connexion est perdue.
async fn envoyer(socket: &mut WebSocket, texte: String) -> bool {
    socket.send(Message::Text(texte.into())).await.is_ok()
}

/// Met une majuscule au début d'un message d'erreur du domaine.
fn majuscule(texte: &str) -> String {
    let mut c = texte.chars();
    match c.next() {
        Some(premiere) => premiere.to_uppercase().chain(c).collect::<String>() + ".",
        None => String::new(),
    }
}
