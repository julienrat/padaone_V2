# Padaone — consignes pour Claude Code

## Le projet

Padaone transforme les smartphones des participants en outils de participation pendant une formation
d'adultes ou une réunion publique. Le PC Linux de l'animateur fait serveur local, projette les résultats
au vidéoprojecteur et peut créer son propre point d'accès Wi-Fi si le réseau de la salle lâche.
Public visé : 25 à 60 participants. Porteur : Julien, formateur indépendant.

Langue du projet : **français** pour l'interface, la documentation, les commentaires, les messages de commit
et les noms du domaine. Réponds à Julien en français.

## Principes non négociables

- **Local d'abord** : aucune donnée ne sort de la salle, aucun appel à internet pendant une séance,
  aucun CDN, aucune télémétrie. Polices, scripts et styles sont embarqués dans le binaire.
- **Zéro installation côté participant** : une page web ouverte par QR code, sans compte.
- **Anonymat** : rien ne s'affiche à l'écran sous 3 réponses (`padaone_core::SEUIL_ANONYMAT`) ;
  aucun auteur visible, même pour l'animateur.
- **L'animateur garde la main** : rien n'est projeté sans son geste (révéler, figer, masquer).
- **Accessibilité** : cibles tactiles d'au moins 44 px, contraste d'au moins 4,5:1, jamais la couleur seule
  pour porter le sens, vrais `<button>` et `<label>`, repli en boutons pour chaque capteur.
- **Linux d'abord** : le réseau passe par NetworkManager. Windows et macOS : serveur et activités
  complets (code commun, `crates/padaone/src/chemins.rs` pour les dossiers) ; point d'accès et portail
  captif Linux seulement. Vérifier : `cargo clippy --workspace --all-targets --target x86_64-pc-windows-msvc`
  et `--target aarch64-apple-darwin`.

## Commandes

- Compiler : `cargo build`
- Tester : `cargo test --workspace`
- Analyser : `cargo clippy --workspace --all-targets -- -D warnings`
- Formater : `cargo fmt --all` (un hook le fait aussi après chaque modification de fichier `.rs`)
- Lancer : `cargo run -p padaone` (options : `--port`, `--adresse`, `--titre`, `--reprendre <journal>`,
  `--journaux <dossier>`, `--sans-journal`, `--reseau-simule`, `--sans-portail`, `--nouvelle-cle` ;
  à venir : `--demo`). La clé secrète de l'écran et de la console est gardée dans `~/.config/padaone/cle`.
- Point d'accès : `cargo run -p padaone -- point-acces diag|droits|portail` (lecture seule, permis) ;
  `point-acces demarrer|arreter` **modifient le réseau** : seulement avec l'accord de Julien. Pour essayer sans
  rien toucher : ajouter `--reseau-simule` (au serveur comme aux sous-commandes).
  Les liens à ouvrir (participants, écran et console avec la clé) s'affichent au lancement.
- AppImage : `scripts/creer-appimage.sh` (binaire statique musl ; résultat dans `target/appimage/`).
- Publier une version : mettre à jour `version` (Cargo.toml) et `paquets/notes-de-version.md`, commiter,
  puis `git tag vX.Y.Z && git push origin vX.Y.Z` : GitHub Actions teste et publie Linux, Windows, macOS.
- Une tâche n'est terminée que si fmt, clippy et les tests passent. Le skill `/verifier` enchaîne les trois.

## Architecture

Workspace Cargo :

- `crates/padaone-core` : domaine pur (séance, participants, activités, agrégations, événements).
  Aucune entrée/sortie, aucune dépendance réseau. Toute la logique métier est testée ici.
- `crates/padaone-net` : point d'accès Wi-Fi via NetworkManager (`nmcli` en phase 0, D-Bus avec zbus
  ensuite), diagnostic des cartes, texte du QR Wi-Fi. Les commandes sont construites par des fonctions pures.
- `crates/padaone` : le binaire. Serveur axum + tokio, WebSocket, pages web embarquées (rust-embed),
  QR codes (qrcode), journal des événements, sous-commandes CLI (clap).
- `crates/padaone/web` : trois interfaces sans étape de build (voir `crates/padaone/web/LISEZMOI.md`).

Briques prévues plus tard : `padaone-tls` (HTTPS par installation, rustls + instant-acme),
`padaone-audio` (micro du PC pour l'applaudimètre, cpal), application Tauri 2.

Les dépendances prévues sont déclarées dans `[workspace.dependencies]` du `Cargo.toml` racine.
Pour en activer une dans un crate : `nom = { workspace = true }`. Vérifie les versions récentes
avant de les activer. Toute nouvelle dépendance se justifie dans le message de commit.

## Modèle et protocole

- Une interaction = un module de `padaone-core` qui implémente le trait `Activite` (configuration,
  réponse, résultat). Ajouter une interaction ne touche ni au réseau ni au stockage.
  Procédure complète : skill `/nouvelle-interaction`.
- Messages WebSocket en JSON avec un champ `t` (type). Le format fait foi dans
  `docs/03-specification.md`, section « Protocole » : toute évolution du protocole met à jour ce document.
- Chaque changement d'état est un événement journalisé (JSON Lines). La reprise après plantage relit le journal.
- Rôles : participant (jeton anonyme gardé dans le navigateur), écran et console (clé secrète affichée au lancement).
- Les écrans reçoivent au plus 10 mises à jour par seconde, quelle que soit l'affluence.

## Réseau et capteurs

- Point d'accès : profil NetworkManager `padaone-ap` (mode `ap`, `ipv4.method shared`).
- **Ne lance jamais une commande qui modifie le réseau ou le système** (`nmcli connection add/up/down`,
  `iw`, `sudo`, écriture dans `/etc`) sans l'accord explicite de Julien dans la conversation.
  Les commandes en lecture seule (`nmcli -t -f ... device status`) sont permises.
- Cartes Wi-Fi Intel : souvent 8 à 14 appareils au plus en point d'accès. Clé USB recommandée.
- Capteurs des téléphones (micro, accéléromètre, orientation) : disponibles **uniquement en HTTPS**.
  En HTTP, l'interface passe d'elle-même en mode boutons. Ne suppose jamais qu'un capteur est disponible.
- Le son n'est jamais enregistré : le téléphone n'envoie qu'un niveau de 0 à 100.

## Style de code

- Rust 2021, `unsafe` interdit, pas de `unwrap()` ni `expect()` hors tests et hors `main`
  (lints du workspace). Erreurs typées avec thiserror dans les crates, anyhow dans le binaire.
- Noms du domaine en français, sans accents dans les identifiants : `Seance`, `Activite`, `NuageDeMots`,
  `Participant`. Les termes techniques usuels (handler, router, state) restent acceptés.
- Commentaires et documentation en français. Tests unitaires à côté du code (`#[cfg(test)]`),
  au moins un test par règle métier (seuil d'anonymat, regroupement des mots, médiane du jugement majoritaire…).
- Commits en français, à l'impératif, préfixés par la phase : `[phase 1] nuage : regroupe les pluriels`.

## Interface

- Direction visuelle « jeune » : jetons dans `crates/padaone/web/commun/theme.css` (fond lilas, encre
  `#1B1530`, violet, corail, soleil, menthe, rose, ciel ; contours 3 px ; ombres décalées ; grands rayons ;
  titres Bricolage Grotesque, texte Lexend).
- Maquettes de référence, une par écran : `docs/maquettes/` (format Design Component, lire le balisage
  et les styles ; voir `docs/maquettes/LISEZMOI.md`).
- Projection en 1280 × 720, lisible à 10 m : 6 éléments au plus par graphique, gros caractères.
- Téléphone en 390 × 844 : une question à la fois, barre « Signal discret » toujours accessible en bas.

## Où trouver quoi

- Feuille de route et cases à cocher : `docs/04-feuille-de-route.md` (à tenir à jour à chaque étape)
- Spécification fonctionnelle et technique : `docs/03-specification.md`
- Inventaire de l'existant : `docs/01-inventaire-existant.md`
- Résumé des décisions : `docs/02-plan-et-maquettes.md`
- Installation du poste : `docs/00-demarrage.md`
- `.claude/` et `.vscode/` sont des copies de `modeles/claude` et `modeles/vscode` : si tu modifies l'une,
  reporte la modification dans le modèle.

## Phase en cours

Phase 0, prototypes de risque. Fait : serveur axum, pages embarquées, WebSocket avec le protocole v1,
journal et reprise, et toutes les interactions V1 sans matériel (sondage, cartons, nuage, échelle, météo,
pari, ROTI, question ouverte, questions du public, remise en ordre avec minuterie, signal discret), point d'accès (diagnostic, démarrage,
arrêt, page de la console, QR Wi-Fi à l'écran). Reste : premier essai réel du point d'accès par Julien,
applaudimètre (micro du PC).
Prochaine étape suggérée : sous-commande `padaone point-acces diag` (lecture seule), puis les robots
(200 participants simulés sur le WebSocket).
Le skill `/avancer` lit la feuille de route et propose la prochaine tâche.

## À ne pas faire

- Pas de dépendance ni de code qui appelle internet pendant une séance.
- Pas de modification de la configuration réseau ou système sans accord (voir « Réseau et capteurs »).
- Ne pas committer `target/`, les journaux de séance (`*.jsonl`), `CLAUDE.local.md`, `.claude/settings.local.json`.
- Ne pas supprimer ni réécrire les documents de `docs/` sans le demander : on les complète.
