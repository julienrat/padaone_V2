Padaone fonctionne désormais sous **Linux, Windows et macOS**. Les téléphones des participants deviennent des outils de participation, les résultats s'affichent au vidéoprojecteur. Tout tourne sur l'ordinateur de l'animateur, sans compte, sans internet, anonyme.

## Télécharger

| Système | Fichier | Lancer |
| --- | --- | --- |
| **Linux** (64 bits) | `Padaone-0.2.0-x86_64.AppImage` | Rendre exécutable (clic droit › Propriétés › Permissions), puis double-clic |
| **Windows** 10 et 11 | `Padaone-0.2.0-windows-x86_64.zip` | Extraire, double-clic sur `Padaone.exe` |
| **macOS** 11 et plus (Apple et Intel) | `Padaone-0.2.0-macos-universel.zip` | Décompresser, clic droit sur `Padaone` › Ouvrir |

Au premier lancement :
- **Windows** affiche « Windows a protégé votre ordinateur » : *Informations complémentaires* › *Exécuter quand même*. Autorisez ensuite Padaone sur les **réseaux privés** dans la fenêtre du pare-feu, sinon les téléphones ne joindront pas le PC.
- **macOS** bloque les programmes non signés par Apple : clic droit › *Ouvrir* › *Ouvrir*, et acceptez les connexions entrantes.

Chaque archive contient un fichier `LISEZMOI.txt`. Mode d'emploi complet : **[guide du formateur](https://github.com/julienrat/padaone_V2/blob/main/docs/guide/guide-formateur.md)**.

## Nouveautés de la 0.2.0

- Versions **Windows** (exécutable autonome) et **macOS** (binaire universel Apple Silicon et Intel).
- Lancé par double-clic, Padaone ouvre la console dans le navigateur.
- Fabrication et publication automatiques des trois versions (GitHub Actions), après les tests sur les trois systèmes.

## Rappel des fonctionnalités

Sondage, cartons de couleur, nuage de mots, échelle, météo à deux axes, pari de certitude, question ouverte modérée, questions du public, ROTI ; signal discret ; rien n'est projeté sans l'animateur ni sous 3 réponses ; journal et reprise après incident. Sous **Linux** seulement : point d'accès Wi-Fi créé par le PC et portail captif.

## Limites connues

- Version d'essai, pas encore validée en salle avec un groupe réel.
- Exécutables non signés (avertissements Windows et macOS au premier lancement).
- Sous Windows et macOS, pas de point d'accès piloté par Padaone : Wi-Fi de la salle, routeur de voyage ou partage de connexion du système.

Les empreintes SHA-256 des fichiers sont dans `SHA256SUMS.txt`. Licence : GNU AGPL v3 ou ultérieure.
