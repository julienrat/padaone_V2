---
name: verifier
description: Vérifie que le workspace est propre (formatage, clippy sans avertissement, tests verts) et résume les problèmes. À utiliser avant de déclarer une tâche terminée ou avant un commit.
allowed-tools: Bash(cargo fmt *) Bash(cargo clippy *) Bash(cargo test *) Bash(cargo build *)
---

Vérifie l'état du workspace Padaone, dans cet ordre, depuis la racine du projet :

1. `cargo fmt --all -- --check`. Si des fichiers ne sont pas formatés, lance `cargo fmt --all` puis signale-le.
2. `cargo clippy --workspace --all-targets -- -D warnings`.
3. `cargo test --workspace`.

Ensuite, réponds en français avec :

- une ligne par étape : réussie ou échouée ;
- pour chaque échec, le fichier, la ligne et la cause en une phrase, puis la correction proposée ;
- si tout est vert, une seule phrase le disant.

Ne corrige pas toi-même les erreurs de clippy ou les tests en échec sans le demander,
sauf si la correction est mécanique (formatage, import inutilisé).
