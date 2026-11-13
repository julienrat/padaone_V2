Nouvelle activité : **la remise en ordre**. Les participants remettent des étapes, des cartes ou une séquence dans le bon ordre par glisser-déposer, avec une minuterie si vous le souhaitez. Padaone fonctionne sous **Linux, Windows et macOS** ; tout tourne sur l'ordinateur de l'animateur, sans compte, sans internet, anonyme.

## Télécharger

| Système | Fichier | Lancer |
| --- | --- | --- |
| **Linux** (64 bits) | `Padaone-0.3.0-x86_64.AppImage` | Rendre exécutable (clic droit › Propriétés › Permissions), puis double-clic |
| **Windows** 10 et 11 | `Padaone-0.3.0-windows-x86_64.zip` | Extraire, double-clic sur `Padaone.exe` |
| **macOS** 11 et plus (Apple et Intel) | `Padaone-0.3.0-macos-universel.zip` | Décompresser, clic droit sur `Padaone` › Ouvrir |

Au premier lancement :
- **Windows** affiche « Windows a protégé votre ordinateur » : *Informations complémentaires* › *Exécuter quand même*. Autorisez ensuite Padaone sur les **réseaux privés** dans la fenêtre du pare-feu, sinon les téléphones ne joindront pas le PC.
- **macOS** bloque les programmes non signés par Apple : clic droit › *Ouvrir* › *Ouvrir*, et acceptez les connexions entrantes.

Mode d'emploi complet : **[guide du formateur](https://github.com/julienrat/padaone_V2/blob/main/docs/guide/guide-formateur.md)**.

## Nouveautés de la 0.3.0

- **Remise en ordre** : de 3 à 8 éléments (texte de 40 caractères, couleur au choix parmi dix, tous en rectangle arrondi). Les téléphones les reçoivent mélangés ; le participant les fait glisser (au doigt ou à la souris, ou avec les flèches) puis **valide** sa proposition.
- **Minuterie** (10 s à 60 min) : affichée sur les téléphones, à l'écran et dans la console ; à zéro, les réponses sont figées. La console peut la relancer ou l'arrêter.
- À la révélation : le bon ordre, le nombre de propositions parfaites et, pour chaque élément, la part de la salle qui l'a bien placé ; chaque participant voit son propre score.

## Rappel des fonctionnalités

Sondage, cartons de couleur, nuage de mots, échelle, météo à deux axes, pari de certitude, question ouverte modérée, questions du public, remise en ordre, ROTI ; signal discret ; rien n'est projeté sans l'animateur ni sous 3 réponses ; journal et reprise après incident. Sous **Linux** seulement : point d'accès Wi-Fi créé par le PC et portail captif.

## Limites connues

- Version d'essai, pas encore validée en salle avec un groupe réel.
- Exécutables non signés (avertissements Windows et macOS au premier lancement).
- Sous Windows et macOS, pas de point d'accès piloté par Padaone : Wi-Fi de la salle, routeur de voyage ou partage de connexion du système.

Les empreintes SHA-256 des fichiers sont dans `SHA256SUMS.txt`. Licence : GNU AGPL v3 ou ultérieure.
