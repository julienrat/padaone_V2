# Padaone — plan de développement et maquettes (phase 2 du projet)

*6 novembre 2026*

## Liens
- Document complet (interactions inventées, architecture, point d'accès, feuille de route, risques) : https://claude.ai/artifact/JKbX9S4nhVtNwUQufYCahW
- Maquettes (10 écrans) : https://claude.ai/artifact/6wWqkHEP2PRkkJW1Pxw8bG

## Décisions prises
- Poste animateur : **Linux d'abord** (NetworkManager), Windows plus tard.
- Taille de groupe visée : **25 à 60 personnes**.
- Participants : page web via QR code, sans appli ni compte.
- Un seul programme Rust (Tauri 2 + axum/tokio + SQLite), interfaces web partagées (Svelte).
- Nom de travail : Padaone (nom du dossier de code).

## Point clé réseau
- Les cartes Wi-Fi Intel tiennent souvent 8 à 14 appareils en point d'accès (retours d'utilisateurs) : pour 25-60 personnes, **clé USB Wi-Fi dédiée** ou **routeur de voyage**.
- Bascule sans perte : le PC garde son adresse IP sur le réseau Padaone, les pages ouvertes se reconnectent seules.
- Portail captif + QR Wi-Fi standard + adresse simple `padaone.lan` (via `/etc/NetworkManager/dnsmasq-shared.d/`).

## 17 interactions inventées
Ouvrir : Météo à deux axes, Miroir, Brassage · Comprendre : Pari de certitude, Fourchettes, Signal discret · Produire : Tournoi d'idées, Tri par affinités, Relance · Décider : Verdict (jugement majoritaire), File de parole équitable, Tamis, Consentement, Tirelire · Clore : Radar avant/après, Courbe d'énergie, ROTI et mot de la fin.

## Nuages de mots (ajout du 6 nov.)
- 1 à 3 mots par personne, regroupement automatique (accents, pluriels, mots vides), fusion et masquage dans la console.
- Placement en spirale stable : un mot posé ne bouge plus, il grossit. Variantes : nuage comparé, du nuage au vote, nuage rejoué.

## Capteurs du téléphone (ajout du 6 nov.)
- Applaudimètre (micro du PC en V1, micro des téléphones ensuite), Pouce en l'air (accéléromètre : debout = pour, tête en bas = contre, à plat = je passe), Niveau à bulle (météo en inclinant), Sismographe, Secouer, Cartons de couleur, Mur de photos, Chuchotomètre, vibreur.
- **Contrainte clé : micro et mouvements des téléphones exigent HTTPS dans le navigateur.** Solution retenue : certificat par installation à la manière de Plex (`*.<id>.<domaine>`), obtenu quand le PC a internet, utilisable hors ligne sur le point d'accès. Sans HTTPS : mode boutons + micro du PC.
- Un seul bouton « Activer les capteurs » à l'arrivée ; refuser ne bloque rien ; le son n'est jamais enregistré.

## Direction visuelle V2
- Plus jeune et chaleureuse : fond lilas, violet électrique, corail, jaune soleil, menthe, rose ; contours épais et ombres décalées ; titres Bricolage Grotesque, texte Lexend.

## Feuille de route (semaines à temps plein, estimation)
0. Prototypes de risque (2) → feu vert réseau
1. Socle de séance (5) → pilote 1 (10-15 pers.)
2. Réseau autonome (3) → pilote 2 (30 pers., bascule en direct)
3. Interactions signatures (6) → pilote 3 (50 pers.)
4. Capteurs et HTTPS (4) → pilote capteurs (30 téléphones, sans internet)
5. Déroulés et rapports (4) → version 1.0 publique (semaine 24)
6. Délibération (5)

## Questions ouvertes
- Licence (AGPL-3.0 envisagée).
- Choix et test d'une clé USB Wi-Fi compatible point d'accès.
- Nom de domaine et petit service de délivrance des certificats (ou option « mon propre domaine » seulement).
