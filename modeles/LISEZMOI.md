# Modèles de configuration à installer

Ces deux dossiers doivent être copiés à la racine du projet sous des noms cachés.
Ils n'ont pas pu y être déposés directement : par sécurité, les outils distants n'écrivent pas dans
`.vscode/` ni dans `.claude/`, deux dossiers dont le contenu peut lancer des commandes sur ton PC.
Relis-les, puis copie-les.

| Modèle | Destination | Contenu |
| --- | --- | --- |
| `vscode/` | `.vscode/` | Extensions recommandées, réglages (format à l'enregistrement, clippy), tâches, débogage |
| `claude/` | `.claude/` | Permissions de Claude Code, hook de formatage Rust, skills `/avancer`, `/verifier`, `/nouvelle-interaction` |

Copie (depuis la racine du projet) :

```bash
cp -r modeles/vscode .vscode
cp -r modeles/claude .claude
```

Le script `scripts/installer-environnement.sh` propose aussi cette copie, avec confirmation.
Une fois copiés, ces modèles peuvent être supprimés ou gardés comme référence.

Ce que fait `claude/settings.json` :

- autorise sans demander : `cargo build/check/test/clippy/fmt/run/doc/tree`, `git status/diff/log/show`,
  et le diagnostic Wi-Fi en lecture seule ;
- demande avant : `cargo add/remove`, `git commit/push`, toute autre commande `nmcli` ou `iw`, `sudo` ;
- interdit : `rm -rf`, `git push --force`, la lecture de `.env` ;
- après chaque écriture d'un fichier `.rs` par Claude Code, lance `claude/hooks/formater-rust.sh`
  (rustfmt sur ce seul fichier, ne bloque jamais).
