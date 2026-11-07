# Padaone

Les smartphones des participants deviennent des outils de participation pendant une formation ou une
réunion publique : nuage de mots, sondages, applaudimètre, pouce en l'air, jugement majoritaire…
Le PC Linux de l'animateur fait serveur local, projette les résultats et peut créer son propre Wi-Fi.
Aucune appli à installer, aucun compte, aucune donnée qui sort de la salle.

**État : phase 0.** Le serveur local tourne : un sondage fonctionne de bout en bout (téléphones,
projection, console), avec journal de séance et reprise après plantage. Suite : `docs/04-feuille-de-route.md`.

## Démarrer

```bash
bash scripts/installer-environnement.sh   # prépare le poste (Rust, VS Code, Claude Code, configuration)
code .                                     # ouvre le projet dans VS Code
cargo test --workspace                     # vérifie que tout passe
cargo run -p padaone                       # lance le serveur et affiche les liens à ouvrir
```

Guide complet du poste de développement : [docs/00-demarrage.md](docs/00-demarrage.md).

**Vous êtes formateur ou formatrice ?** Lisez le [guide du formateur](docs/guide/guide-formateur.md) :
toutes les fonctionnalités, captures d'écran à l'appui.

## AppImage

```bash
scripts/creer-appimage.sh                   # fabrique target/appimage/Padaone-<version>-x86_64.AppImage
```

Un seul fichier d'environ 2,5 Mo, sans installation, qui tourne sur tout Linux 64 bits (binaire statique).
Double-clic : un terminal s'ouvre avec les liens de la séance, et la console s'ouvre dans le navigateur.
Depuis un terminal, il s'utilise comme `padaone` : `./Padaone-0.1.0-x86_64.AppImage point-acces diag`.

## Organisation

```text
crates/
  padaone-core/   domaine pur : séance, activités, agrégations (testé, sans entrée/sortie)
  padaone-net/    point d'accès Wi-Fi via NetworkManager, QR Wi-Fi
  padaone/        binaire : serveur, WebSocket, pages web embarquées
    web/          interfaces téléphone (p/), projection (ecran/), console (console/), thème (commun/)
docs/
  guide/                   guide du formateur, avec captures d'écran
  00-demarrage.md          installer le poste, travailler avec Claude Code
  01-inventaire-existant.md
  02-plan-et-maquettes.md  résumé des décisions
  03-specification.md      ce que le logiciel doit faire, protocole
  04-feuille-de-route.md   phases et cases à cocher
  maquettes/               une maquette par écran
paquets/appimage/          lanceur, raccourci et icône de l'AppImage
scripts/                   installation du poste, fabrication de l'AppImage
CLAUDE.md                  consignes pour Claude Code
modeles/claude/            à copier en .claude/ : réglages, hook de formatage, skills (/avancer, /verifier, /nouvelle-interaction)
modeles/vscode/            à copier en .vscode/ : extensions, tâches, débogage
```

## Licence

[GNU AGPL v3 ou ultérieure](LICENSE) (AGPL-3.0-or-later). Vous pouvez utiliser, étudier, modifier et
redistribuer Padaone ; toute version modifiée, y compris proposée en service réseau, doit être
partagée sous la même licence.
