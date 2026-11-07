#!/usr/bin/env bash
# Prépare un poste Linux pour développer Padaone avec VS Code et Claude Code.
# Chaque étape qui installe quelque chose demande confirmation.
# Usage : bash scripts/installer-environnement.sh

set -u

racine="$(cd "$(dirname "$0")/.." && pwd)"

titre() { printf '\n\033[1;35m== %s ==\033[0m\n' "$1"; }
info() { printf '  %s\n' "$1"; }
ok() { printf '  \033[32mOK\033[0m %s\n' "$1"; }
attention() { printf '  \033[33m!\033[0m %s\n' "$1"; }
demander() {
  local reponse
  read -r -p "  $1 [o/N] " reponse
  [[ "$reponse" =~ ^[oOyY]$ ]]
}

titre "Place disque"
libre_go=$(df -Pk "$HOME" | awk 'NR==2 {print int($4/1024/1024)}')
info "Espace libre dans $HOME : ${libre_go} Go"
if [ "$libre_go" -lt 8 ]; then
  attention "Moins de 8 Go libres. Rust, la compilation (dossier target/) et VS Code en demandent 5 à 8."
  attention "Libère de la place avant de continuer (par exemple : ~/.cache, anciens téléchargements)."
  demander "Continuer quand même ?" || exit 1
fi

titre "Paquets système"
if command -v apt-get >/dev/null 2>&1; then
  paquets="build-essential pkg-config git curl jq python3 network-manager iw"
  commande="sudo apt-get update && sudo apt-get install -y $paquets"
elif command -v dnf >/dev/null 2>&1; then
  paquets="gcc gcc-c++ make pkgconf-pkg-config git curl jq python3 NetworkManager iw"
  commande="sudo dnf install -y $paquets"
elif command -v pacman >/dev/null 2>&1; then
  paquets="base-devel pkgconf git curl jq python networkmanager iw"
  commande="sudo pacman -S --needed $paquets"
else
  commande=""
  attention "Gestionnaire de paquets non reconnu : installe à la main un compilateur C, pkg-config, git, curl, jq, python3, NetworkManager et iw."
fi
if [ -n "$commande" ]; then
  info "Commande prévue : $commande"
  if demander "Installer ces paquets (mot de passe administrateur demandé) ?"; then
    bash -c "$commande" && ok "Paquets installés"
  fi
fi

titre "Rust"
if command -v cargo >/dev/null 2>&1; then
  ok "$(cargo --version)"
else
  info "Rust n'est pas installé. Installation officielle via rustup (https://rustup.rs)."
  if demander "Installer Rust avec rustup ?"; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal
    # shellcheck disable=SC1091
    . "$HOME/.cargo/env"
  fi
fi
if command -v rustup >/dev/null 2>&1; then
  rustup component add rustfmt clippy >/dev/null 2>&1 && ok "rustfmt et clippy présents"
fi

titre "NetworkManager"
if command -v nmcli >/dev/null 2>&1; then
  ok "nmcli présent"
  nmcli -t -f DEVICE,TYPE,STATE device status | awk -F: '$2=="wifi" {print "  carte Wi-Fi : " $1 " (" $3 ")"}'
else
  attention "nmcli introuvable : le point d'accès Wi-Fi de Padaone ne pourra pas fonctionner."
fi

titre "VS Code"
if command -v code >/dev/null 2>&1; then
  ok "$(code --version | head -1)"
  extensions="rust-lang.rust-analyzer vadimcn.vscode-lldb tamasfe.even-better-toml fill-labs.dependi usernamehw.errorlens anthropic.claude-code"
  if demander "Installer les extensions recommandées ($extensions) ?"; then
    for e in $extensions; do
      code --install-extension "$e" >/dev/null 2>&1 && ok "$e" || attention "échec : $e"
    done
  fi
else
  attention "VS Code introuvable. Installe-le depuis https://code.visualstudio.com (paquet .deb ou .rpm), version 1.94 ou plus."
fi

titre "Claude Code"
if command -v claude >/dev/null 2>&1; then
  ok "$(claude --version 2>/dev/null)"
else
  info "La commande claude est utile dans le terminal de VS Code, en plus de l'extension."
  info "Installation officielle : curl -fsSL https://claude.ai/install.sh | bash"
  if demander "Installer Claude Code maintenant ?"; then
    curl -fsSL https://claude.ai/install.sh | bash
  fi
fi
info "Un abonnement Pro, Max, Team, Enterprise ou un compte Console est nécessaire pour Claude Code."

titre "Configuration VS Code et Claude Code"
cd "$racine" || exit 1
for paire in "vscode:.vscode" "claude:.claude"; do
  modele="modeles/${paire%%:*}"
  cible="${paire##*:}"
  if [ -d "$cible" ]; then
    ok "$cible existe déjà (non modifié)"
  elif [ -d "$modele" ]; then
    info "Fichiers de $modele :"
    find "$modele" -type f | sed 's/^/    /'
    if demander "Copier $modele vers $cible (relis-les d'abord si besoin) ?"; then
      cp -r "$modele" "$cible" && ok "$cible installé"
    fi
  fi
done

titre "Dépôt git"
cd "$racine" || exit 1
if [ -d .git ]; then
  ok "Dépôt git déjà initialisé"
else
  info "git permet de revenir en arrière après une modification de Claude Code."
  if demander "Initialiser le dépôt git et faire un premier commit ?"; then
    git init -q && git add -A && git commit -qm "Initialisation de l'environnement Padaone" && ok "Premier commit fait"
  fi
fi

titre "Vérification"
if command -v cargo >/dev/null 2>&1; then
  if cargo test --workspace --quiet; then
    ok "Le squelette compile et les tests passent"
  else
    attention "La compilation ou les tests ont échoué : ouvre le projet dans VS Code et demande à Claude Code de regarder."
  fi
fi

titre "Et maintenant"
info "1. code $racine"
info "2. Accepte les extensions recommandées et fais confiance au dossier."
info "3. Ouvre Claude Code (icône en haut à droite de l'éditeur) et tape : /avancer"
