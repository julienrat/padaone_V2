# Padaone — feuille de route

Durées estimées en semaines de travail à temps plein. Claude Code coche les cases au fil de l'eau
(skill `/avancer`). Une étape de validation (◆) se vérifie en vrai, sur le matériel ou en salle.

## Phase 0 — Prototypes de risque (2 semaines)

- [x] Squelette du workspace, environnement VS Code et consignes Claude Code
- [x] Activer les dépendances de base (tokio, axum, serde, serde_json, tracing) et vérifier les versions
- [x] Serveur axum minimal : pages statiques embarquées (`/p`, `/ecran`, `/console`) et WebSocket `/ws`
  (directement avec le protocole v1 et un sondage de bout en bout, testé avec un navigateur et des clients simulés)
- [x] Sous-commande `padaone point-acces diag` (lecture seule) : cartes Wi-Fi, mode point d'accès, bandes, droits
- [ ] Sous-commandes `point-acces demarrer` et `arreter` via `nmcli` (tests manuels par Julien)
  (code fait et essayé en `--reseau-simule` ; reste le premier essai réel par Julien)
- [ ] Mesurer combien de téléphones tient la carte du ThinkPad, puis une clé USB
- [ ] Portail captif : vérifier l'ouverture automatique de la page sur Android et iPhone
  (code fait : DNS local, port 80, trois modes ; reste l'installation unique par Julien et l'essai sur téléphones)
- [ ] Page de test des capteurs (micro, orientation) en HTTP puis en HTTPS sur 6 téléphones
- [ ] Robots : 200 participants simulés sur le WebSocket
- [ ] Relecture des maquettes par 2 ou 3 formateurs
- ◆ **Feu vert réseau** : 30 connexions tenues et page ouverte seule sur Android et iOS

## Phase 1 — Socle de séance (5 semaines)

- [x] Séance : code à 6 chiffres, participants anonymes (jeton), reconnexion automatique
- [x] Clé secrète pour la console et l'écran, affichée au lancement
- [x] Journal des événements (JSON Lines) et reprise après plantage (`--reprendre`)
- [ ] Écran d'accueil : QR de la séance, QR Wi-Fi, code, nombre de connectés
  (fait : QR de la séance, adresse, code, connectés ; reste : QR Wi-Fi quand le point d'accès existera)
- [x] Console : déroulé, ajouter / lancer / révéler / figer / suivant
- [x] Sondage (téléphone, écran, console ; révéler, figer, réinitialiser)
- [x] Nuage de mots (regroupement, masquage, fusion, placement stable)
- [x] Échelle
- [x] Question ouverte modérée et questions du public
- [x] Météo à deux axes
- [x] Pari de certitude
- [x] Signal discret
- [x] ROTI et mot de la fin
- [x] Cartons de couleur
- [ ] Applaudimètre avec le micro du PC (crate cpal)
- [ ] Export CSV des résultats
- [ ] Polices embarquées (Bricolage Grotesque, Lexend)
- ◆ **Pilote 1** : une vraie formation de 10 à 15 personnes, sans incident bloquant

## Phase 2 — Réseau autonome (3 semaines)

- [ ] Écran « Point d'accès » complet dans la console (réglages du § 5 de la spécification)
  (fait : carte, nom, mot de passe généré, sécurité ouvert/WPA2, bande, canal, adresse gardée ou par défaut,
  arrêt en fin de séance, diagnostic, aperçu ; reste : WPA3, portail captif, démarrage proposé automatiquement)
- [x] Diagnostic des cartes avec capacité estimée
- [x] Portail captif et adresse simple `padaone.lan` (à valider sur Android et iPhone)
- [ ] Bascule sans perte (le PC garde son adresse, les pages se reconnectent)
- [x] AppImage (binaire statique musl, `scripts/creer-appimage.sh`)
- [ ] Paquet .deb (avec le réglage dnsmasq installé une fois)
- ◆ **Pilote 2** : 30 personnes sur le réseau Padaone, bascule déclenchée en pleine séance

## Phase 3 — Interactions signatures (6 semaines)

- [ ] Brassage (groupes)
- [ ] Miroir
- [ ] Fourchettes
- [ ] Tournoi d'idées
- [ ] Verdict (jugement majoritaire)
- [ ] File de parole équitable
- [ ] Radar avant/après
- [ ] Options avant/après et par groupe pour toutes les activités
- ◆ **Pilote 3** : 50 personnes, en réunion publique ou en grand groupe

## Phase 4 — Capteurs et HTTPS (4 semaines)

- [ ] Certificat par installation et renouvellement (`padaone-tls`)
- [ ] Bouton « Activer les capteurs » et repli automatique en mode boutons
- [ ] Pouce en l'air (orientation)
- [ ] Applaudimètre des téléphones (micro)
- [ ] Niveau à bulle
- [ ] Secouer
- [ ] Mur de photos
- ◆ **Pilote capteurs** : 30 téléphones Android et iPhone en HTTPS, sans internet dans la salle

## Phase 5 — Déroulés et rapports (4 semaines)

- [ ] Déroulé préparé à l'avance et bibliothèque de modèles
- [ ] Télécommande de l'animateur sur téléphone
- [ ] Rapport PDF de séance (positionnement et évaluation des acquis pour Qualiopi)
- [ ] Application Tauri (fenêtres console et projection)
- ◆ **Version 1.0 publique** : documentation, site, licence choisie

## Phase 6 — Délibération (5 semaines)

- [ ] Tamis, Consentement, Tirelire, Tri par affinités, Relance, Courbe d'énergie, Sismographe
- [ ] Ensuite : version Windows, paquet Flatpak, traductions
