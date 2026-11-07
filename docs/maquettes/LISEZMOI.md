# Maquettes de Padaone

Une maquette par écran, issues du canevas de design « Padaone — maquettes » (direction visuelle V2).
Elles servent de **référence visuelle et de structure** pour coder les interfaces de `crates/padaone/web/`.

Format : « Design Component » (`.dc.html`). Ce ne sont pas des pages autonomes :

- le balisage est du HTML avec des styles en ligne : couleurs, tailles, espacements, ordre des éléments ;
- `{{nom}}` est un emplacement rempli par la fonction `renderVals()` du script en bas du fichier ;
- `<sc-for list="…">` répète un bloc, `<sc-if value="…">` l'affiche sous condition ;
- le script `support.js` du canevas n'est pas fourni : ouvrir le fichier dans un navigateur n'affichera pas
  le rendu final. Pour voir le rendu, utiliser le canevas sur claude.ai.

| Fichier | Écran |
| --- | --- |
| `Ecran-Accueil.dc.html` | Projection : rejoindre la séance (QR Wi-Fi et QR séance) |
| `Ecran-Nuage.dc.html` | Projection : nuage de mots animé |
| `Ecran-Applaudimetre.dc.html` | Projection : jauge de l'applaudimètre |
| `Ecran-Pouces.dc.html` | Projection : pouce en l'air |
| `Ecran-Pari.dc.html` | Projection : Pari de certitude |
| `Ecran-Verdict.dc.html` | Projection : Verdict (jugement majoritaire) |
| `Tel-Rejoindre.dc.html` | Téléphone : rejoindre, couleur, activer les capteurs |
| `Tel-Nuage.dc.html` | Téléphone : saisie du nuage de mots |
| `Tel-Applaudimetre.dc.html` | Téléphone : applaudimètre (micro ou tapotement) |
| `Tel-Pouce.dc.html` | Téléphone : pouce en l'air |
| `Tel-Meteo.dc.html` | Téléphone : niveau à bulle (météo) |
| `Tel-Pari.dc.html` | Téléphone : Pari de certitude |
| `Tel-Verdict.dc.html` | Téléphone : Verdict |
| `Tel-Reconnexion.dc.html` | Téléphone : bascule sur le point d'accès |
| `Console.dc.html` | Console de l'animateur (nuage en cours, modération, signaux, capteurs) |
| `Point-acces.dc.html` | Réglages du point d'accès Wi-Fi et du HTTPS |

Les données affichées (questions, chiffres, noms de réseau) sont des exemples.
