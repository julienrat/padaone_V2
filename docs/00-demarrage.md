# Démarrer le développement de Padaone avec VS Code et Claude Code

## 1. Préparer le poste (une fois)

Le plus simple : lancer le script, qui demande confirmation à chaque étape.

```bash
cd ~/Developpement/padaone
bash scripts/installer-environnement.sh
```

Il vérifie ou installe :

- **la place disque** : prévoir 8 Go libres (Rust, dossier de compilation `target/`, VS Code) ;
- **les paquets système** : compilateur C, pkg-config, git, curl, jq, python3, NetworkManager, iw ;
- **Rust** via rustup, avec rustfmt et clippy (la version est fixée par `rust-toolchain.toml`) ;
- **les extensions VS Code** recommandées (liste dans `.vscode/extensions.json`) ;
- **Claude Code** en ligne de commande : `curl -fsSL https://claude.ai/install.sh | bash` ;
- **la configuration** : copie de `modeles/vscode` vers `.vscode` et de `modeles/claude` vers `.claude`
  (voir `modeles/LISEZMOI.md`) ;
- **git** : un premier commit, pour pouvoir revenir en arrière après chaque modification.

Sans le script, la copie de la configuration se fait ainsi, depuis la racine du projet :

```bash
cp -r modeles/vscode .vscode
cp -r modeles/claude .claude
```

À la main, si besoin :

| Outil | Installation |
| --- | --- |
| VS Code 1.94 ou plus | https://code.visualstudio.com (paquet .deb ou .rpm) |
| Extension Claude Code | Extensions (Ctrl+Maj+X), chercher « Claude Code », éditeur Anthropic (`anthropic.claude-code`) |
| Claude Code en terminal | `curl -fsSL https://claude.ai/install.sh \| bash`, puis `claude --version` ; diagnostic : `claude doctor` |
| Rust | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |

Claude Code demande un abonnement Pro, Max, Team, Enterprise ou un compte Console.
L'extension VS Code embarque sa propre copie de Claude Code ; la commande `claude` dans le terminal
intégré nécessite en plus l'installation en ligne de commande.

## 2. Ouvrir le projet

```bash
code ~/Developpement/padaone
```

1. Accepter « Faire confiance aux auteurs » : l'extension Claude Code ne fonctionne pas en mode restreint.
2. Accepter l'installation des extensions recommandées.
3. Vérifier : menu Terminal › Exécuter la tâche › « Padaone : tester ». Les tests doivent passer.

Tâches prêtes (Terminal › Exécuter la tâche) : compiler, tester, vérifier (fmt + clippy + tests), lancer,
diagnostic Wi-Fi en lecture seule, nettoyer (`cargo clean`, libère plusieurs Go).
Débogage : onglet Exécuter et déboguer › « Déboguer padaone ».

## 3. Travailler avec Claude Code

Ouvrir Claude Code avec l'icône en haut à droite de l'éditeur, ou taper `claude` dans le terminal intégré.
Claude Code lit automatiquement `CLAUDE.md` à la racine : contexte, principes, commandes, architecture.

Commandes du projet (skills dans `.claude/skills/`) :

| Commande | Ce qu'elle fait |
| --- | --- |
| `/avancer` | Lit la feuille de route, propose la prochaine tâche et un plan, attend ton accord |
| `/verifier` | Formatage, clippy et tests, avec un résumé des problèmes |
| `/nouvelle-interaction nuage-de-mots` | Implémente une interaction de bout en bout (domaine, serveur, trois interfaces, tests) |

Réglages partagés (`.claude/settings.json`) :

- Claude Code peut lancer sans demander : `cargo build/test/clippy/fmt/run`, `git status/diff/log`
  et le diagnostic Wi-Fi en lecture seule.
- Il demande avant : `cargo add`, `git commit`, `git push`, toute autre commande `nmcli` ou `iw`, `sudo`.
- Il ne peut pas : `rm -rf`, `git push --force`.
- Après chaque modification d'un fichier `.rs`, un hook le formate (`.claude/hooks/formater-rust.sh`).
- Tes réglages personnels vont dans `.claude/settings.local.json` et tes notes dans `CLAUDE.local.md`
  (tous deux ignorés par git).

Premières demandes conseillées :

1. « Lis CLAUDE.md et la feuille de route, puis /avancer. » Il doit proposer d'activer les dépendances
   et de monter le serveur axum minimal.
2. « Ajoute la sous-commande `padaone point-acces diag` en lecture seule et montre-moi ce qu'elle voit
   sur mon PC. »
3. « /nouvelle-interaction sondage »

Bonnes habitudes :

- Demander d'abord un plan pour toute tâche de plus d'un fichier, le relire, puis valider.
- Faire un commit après chaque étape qui marche : on peut toujours revenir en arrière.
- Les commandes qui modifient le réseau (`nmcli connection add/up/down`) se lancent toi-même :
  le point d'accès coupe la connexion Wi-Fi du PC.

## 4. Place disque

Le dossier `target/` grossit vite (1 à 3 Go une fois axum et tokio activés). Pour le vider :
tâche « Padaone : nettoyer » ou `cargo clean`.
