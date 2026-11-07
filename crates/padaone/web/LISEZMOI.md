# Interfaces web de Padaone

Trois interfaces, servies par le binaire `padaone` et embarquées dedans (aucun CDN) :

| Dossier | Adresse | Pour qui | Maquettes de référence |
| --- | --- | --- | --- |
| `p/` | `/p` | Téléphones des participants (390 × 844) | `docs/maquettes/Tel-*.dc.html` |
| `ecran/` | `/ecran` | Vidéoprojecteur (1280 × 720, lisible à 10 m) | `docs/maquettes/Ecran-*.dc.html` |
| `console/` | `/console` | Animateur, sur le PC | `docs/maquettes/Console.dc.html`, `Point-acces.dc.html` |
| `commun/` | `/commun` | Jetons de design (`theme.css`), polices, client WebSocket partagé | |

Règles :

- Phase 1 : HTML, CSS et JavaScript en modules natifs, **sans étape de build**. Passage éventuel à Svelte plus tard.
- Toutes les couleurs, rayons et ombres viennent de `commun/theme.css`.
- Chaque interaction à capteur (micro, mouvement) a un repli en boutons, choisi automatiquement si `window.isSecureContext` est faux ou si l'autorisation est refusée.
- Le client WebSocket se reconnecte seul (attente croissante de 0,5 à 5 s) et renvoie son jeton anonyme pour retrouver le participant.
- Polices : déposer `BricolageGrotesque.woff2` et `Lexend.woff2` (licence SIL OFL, téléchargeables sur Google Fonts) dans `commun/polices/`.
