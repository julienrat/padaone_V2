#!/usr/bin/env bash
# Fabrique l'AppImage de Padaone : target/appimage/Padaone-<version>-x86_64.AppImage
#
# Étapes :
# 1. binaire statique (musl) en mode release : tourne sur tout Linux 64 bits ;
# 2. dossier AppDir avec le lanceur, le raccourci et l'icône de paquets/appimage/ ;
# 3. appimagetool (téléchargé une fois dans target/outils/ s'il n'est pas installé).
#
# Internet n'est nécessaire qu'à la fabrication (cible musl, appimagetool), jamais à l'usage.
set -euo pipefail

RACINE="$(cd "$(dirname "$0")/.." && pwd)"
cd "$RACINE"
CIBLE=x86_64-unknown-linux-musl
VERSION="$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)"
SORTIE="target/appimage"
APPDIR="$SORTIE/AppDir"
OUTILS="target/outils"
APPIMAGETOOL_URL="https://github.com/AppImage/appimagetool/releases/download/continuous/appimagetool-x86_64.AppImage"

echo "Padaone $VERSION — fabrication de l'AppImage"

# 1. Binaire statique
if ! rustup target list --installed | grep -qx "$CIBLE"; then
  echo "Ajout de la cible Rust $CIBLE (une seule fois)…"
  rustup target add "$CIBLE"
fi
cargo build --release --locked -p padaone --target "$CIBLE"
BINAIRE="target/$CIBLE/release/padaone"
if ldd "$BINAIRE" > /dev/null 2>&1; then
  echo "Attention : le binaire n'est pas statique (ldd y trouve des bibliothèques)." >&2
fi

# 2. AppDir
rm -rf "$APPDIR"
mkdir -p "$APPDIR/usr/bin" "$APPDIR/usr/share/applications" "$APPDIR/usr/share/icons/hicolor/scalable/apps"
install -m 755 "$BINAIRE" "$APPDIR/usr/bin/padaone"
install -m 755 paquets/appimage/AppRun "$APPDIR/AppRun"
install -m 644 paquets/appimage/padaone.desktop "$APPDIR/padaone.desktop"
install -m 644 paquets/appimage/padaone.desktop "$APPDIR/usr/share/applications/padaone.desktop"
install -m 644 paquets/appimage/padaone.svg "$APPDIR/padaone.svg"
install -m 644 paquets/appimage/padaone.svg "$APPDIR/usr/share/icons/hicolor/scalable/apps/padaone.svg"
ln -sf padaone.svg "$APPDIR/.DirIcon"

# 3. appimagetool
OUTIL="${APPIMAGETOOL:-$(command -v appimagetool || true)}"
if [ -z "$OUTIL" ]; then
  OUTIL="$OUTILS/appimagetool-x86_64.AppImage"
  if [ ! -x "$OUTIL" ]; then
    echo "Téléchargement d'appimagetool (une seule fois) dans $OUTILS…"
    mkdir -p "$OUTILS"
    curl -fL --progress-bar -o "$OUTIL" "$APPIMAGETOOL_URL"
    chmod +x "$OUTIL"
  fi
fi
FICHIER="$SORTIE/Padaone-$VERSION-x86_64.AppImage"
# APPIMAGE_EXTRACT_AND_RUN : appimagetool tourne même sans FUSE.
ARCH=x86_64 APPIMAGE_EXTRACT_AND_RUN=1 "$OUTIL" --no-appstream "$APPDIR" "$FICHIER"

echo
echo "AppImage prête : $FICHIER ($(du -h "$FICHIER" | cut -f1))"
echo "Essai : $FICHIER --version   ·   double-clic pour lancer une séance"
