# AppImage de Padaone

Fichiers copiés dans l'AppImage par `scripts/creer-appimage.sh` :

- `AppRun` : lanceur. Par double-clic, il ouvre un terminal et la console dans le navigateur ;
  depuis un terminal, il se comporte comme `padaone` (sous-commandes et options comprises).
- `padaone.desktop` : raccourci de bureau.
- `padaone.svg` : icône (les trois bulles du logo).

Le binaire est compilé en statique (`x86_64-unknown-linux-musl`) pour tourner sur tout Linux 64 bits,
quelle que soit la version de la glibc. Les pages web sont embarquées dans le binaire.
