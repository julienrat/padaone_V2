#!/usr/bin/env bash
# Hook PostToolUse : formate le fichier Rust que Claude Code vient d'écrire ou de modifier.
# Reçoit sur l'entrée standard le JSON de l'outil (tool_input.file_path).
# Ne bloque jamais : en cas de souci, on laisse passer (code de sortie 0).

entree="$(cat)"

fichier="$(printf '%s' "$entree" | python3 -c \
  'import json, sys; print(json.load(sys.stdin).get("tool_input", {}).get("file_path", ""))' \
  2>/dev/null)"

if [ -z "$fichier" ] && command -v jq >/dev/null 2>&1; then
  fichier="$(printf '%s' "$entree" | jq -r '.tool_input.file_path // empty' 2>/dev/null)"
fi

case "$fichier" in
  *.rs)
    if [ -f "$fichier" ]; then
      rustfmt --edition 2021 "$fichier" >/dev/null 2>&1 || true
    fi
    ;;
esac

exit 0
