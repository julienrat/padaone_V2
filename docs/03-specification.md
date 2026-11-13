# Padaone — spécification

*Version de travail du 6 novembre 2026. Document de référence pour le développement.
La version illustrée (schémas, tableaux complets) est le document « Padaone — nouvelles interactions et plan
de développement » sur claude.ai. En cas d'écart, ce fichier fait foi pour le code et doit être tenu à jour.*

## 1. Cadre

- PC de l'animateur sous Linux (NetworkManager), qui fait serveur, projette et peut créer un point d'accès Wi-Fi.
- 25 à 60 participants, chacun avec son smartphone et un navigateur, sans appli ni compte.
- Aucun internet requis pendant une séance. Aucune donnée ne sort de la salle.
- Trois interfaces : téléphone (`/p`), projection (`/ecran`), console de l'animateur (`/console`).

## 2. Interactions

Priorité : **V1** = première version utilisable en formation, **V2** = interactions signatures et capteurs,
**V3** = délibération avancée.

### Socle

| Interaction | Téléphone | Écran | Règles | Version |
| --- | --- | --- | --- | --- |
| Sondage | Choisir une option (2 à 6) | Barres horizontales, effectifs | Résultats masqués par défaut jusqu'à « Révéler » | V1 |
| Nuage de mots | 1 à 3 mots de 25 caractères au plus | Nuage à placement stable | Voir § 3 | V1 |
| Question ouverte modérée | Texte libre court | Messages validés par l'animateur | Rien n'est projeté sans validation | V1 |
| Questions du public | Poser une question, voter pour celles des autres | Liste triée par votes | Modération avant affichage | V1 |
| Échelle | Curseur de 1 à 5 ou de 1 à 10 | Histogramme et médiane | | V1 |

### Interactions inventées (extrait, détail dans le document illustré)

| Interaction | Principe | Version |
| --- | --- | --- |
| Météo à deux axes | Point dans un carré énergie × humeur ; nuage de points | V1 |
| Pari de certitude | Réponse + certitude (je devine / je pense / sûr·e) ; grille juste-faux × sûr-pas sûr, case « fausses certitudes » en avant | V1 |
| Signal discret | Barre permanente : trop vite, perdu·e, un exemple, une pause ; jauges sur la console seulement, effacées après 3 min, alerte à 20 % | V1 |
| ROTI et mot de la fin | 0 à 5 doigts puis un mot | V1 |
| Cartons | Le téléphone devient un carton de couleur plein écran, levé et compté | V1 |
| Miroir | Répondre et estimer la part du groupe qui répond pareil | V2 |
| Brassage | Couleur + animal en plein écran pour former des groupes | V2 |
| Fourchettes | Estimer avec un minimum et un maximum ; la vraie valeur tombe | V2 |
| Tournoi d'idées | Duels entre idées du groupe, classement en direct | V2 |
| Verdict | Jugement majoritaire : une mention par proposition (à rejeter … excellent), classement par mention médiane | V2 |
| File de parole équitable | Demander la parole ; ceux qui n'ont pas parlé passent d'abord | V2 |
| Radar avant/après | Autopositionnement sur 5-6 compétences au début et à la fin ; export pour Qualiopi | V2 |
| Tri par affinités, Relance, Tamis, Consentement, Tirelire, Courbe d'énergie, Sismographe | Délibération et rétrospective | V3 |

Options transverses : **avant/après** (rejouer une activité et comparer) et **par groupe** (résultats par groupe du Brassage).

## 3. Nuage de mots

- Clé de regroupement : minuscules, accents retirés, ponctuation retirée, articles de tête retirés,
  pluriel simple (« s » ou « x » final sur les mots de plus de 3 lettres, sauf « ss »).
  Implémentation de départ : `padaone_core::mots::cle`.
- Texte affiché pour une clé : la saisie la plus fréquente (égalité : la première arrivée).
- Console : masquer un mot, fusionner deux clés (la seconde rejoint la première).
- Écran : mots horizontaux, taille proportionnelle à la racine carrée du nombre de mentions,
  placement en spirale dans l'ordre d'arrivée des clés ; un mot posé garde sa place et grossit.
- Suggestions de mots déjà proposés : désactivées par défaut.
- Variantes V2 : nuage comparé (avant/après, groupe A/B), du nuage au vote, nuage rejoué.

## 3 bis. Remise en ordre et minuterie

- Les éléments sont configurés dans le bon ordre ; tous ont la même forme (rectangle arrondi). Les vues du téléphone et de l'écran les donnent
  **mélangés** (mélange déterministe, rejoué à l'identique depuis le journal, jamais le bon ordre ni son
  inverse) ; le participant répond avec les positions affichées. La `solution` (positions affichées dans
  le bon ordre) n'apparaît dans ces vues qu'une fois le résultat révélé. La console reçoit la configuration
  dans le bon ordre.
- Score d'une proposition : nombre d'éléments à leur place exacte. Résultat : `total`, `parfaits`,
  `scores` (effectif par score de 0 à n), `bien_places` (par élément, dans le bon ordre), `moyenne`.
- Minuterie (activités qui ont une `duree`) : démarre au premier lancement (événement `minuterie_reglee`
  avec l'heure de fin en millisecondes depuis 1970) ; à l'échéance, le serveur fige l'activité (événement
  `figee`, vérifié chaque seconde) ; `reinitialiser` l'efface. Les vues portent `reste_ms` (temps restant) ;
  chaque client en déduit l'échéance selon sa propre horloge.
- Glisser-déposer par une poignée (événements pointeur : doigt et souris) ; repli : boutons ↑ ↓ et
  touches fléchées sur la poignée.

## 4. Capteurs du téléphone

| Interaction | Capteur | Règle |
| --- | --- | --- |
| Applaudimètre | Micro du PC (V1) ou des téléphones (V2) | Niveau 0-100 envoyé 5 fois par seconde pendant la mesure ; niveau de salle = moyenne des niveaux reçus depuis moins de 1,5 s ; pic retenu par option ; le son n'est jamais enregistré ni transmis |
| Pouce en l'air | Orientation (`deviceorientation`) | `beta > 55°` : pour ; `beta < -55°` : contre ; `abs(beta) < 25°` et `abs(gamma) < 25°` : je passe ; changement retenu après 300 ms stables |
| Niveau à bulle | Orientation | La bille suit l'inclinaison ; poser à plat valide |
| Secouer | Accélération | Pic d'accélération au-delà d'un seuil à calibrer |
| Mur de photos | `<input type="file" accept="image/*" capture>` | Photos réduites côté téléphone avant envoi |
| Vibreur | `navigator.vibrate` | Absent sur iPhone : jamais indispensable |

- Micro et mouvements des téléphones : **contexte sécurisé obligatoire (HTTPS)**. En HTTP, ou si l'autorisation
  est refusée, l'interaction passe en mode boutons (tapoter pour l'applaudimètre, trois boutons pour le pouce).
- iPhone : `DeviceMotionEvent.requestPermission()` doit être appelé depuis un geste (bouton « Activer les capteurs »).
- HTTPS en salle (phase 4) : un certificat par installation, sur le modèle de Plex (`*.<id>.<domaine>`),
  obtenu quand le PC a internet, utilisable hors ligne sur le point d'accès. Certificat auto-signé exclu.

## 5. Point d'accès Wi-Fi

Commandes `nmcli` (profil `padaone-ap`), construites par `padaone_net::arguments_creation` :

```text
nmcli connection add type wifi con-name padaone-ap [ifname <carte>] autoconnect no ssid <nom>
      802-11-wireless.mode ap [802-11-wireless.band bg|a] [802-11-wireless.channel <n>]
      ipv4.method shared [ipv4.addresses <ip>/24]
      [wifi-sec.key-mgmt wpa-psk wifi-sec.proto rsn wifi-sec.pairwise ccmp wifi-sec.group ccmp wifi-sec.psk <mdp>]
nmcli connection up padaone-ap
nmcli connection down padaone-ap
```

- Diagnostic : `nmcli -t -f DEVICE,TYPE,STATE,CONNECTION device status` puis, pour chaque carte Wi-Fi,
  `nmcli -f WIFI-PROPERTIES device show <carte>` (champs AP, 2GHZ, 5GHZ à vérifier sur le poste).
- `ipv4.method shared` : NetworkManager lance dnsmasq (DHCP + DNS), adresse par défaut 10.42.0.1/24.
- Réglages de l'écran Point d'accès : carte, nom du réseau, sécurité (ouvert / WPA2 / WPA3), mot de passe
  lisible généré (« tilleul-42 »), bande, canal, adresse (garder l'actuelle pour une bascule sans coupure),
  adresse simple (`padaone.lan`), portail captif (ouvrir / silencieux / désactivé), arrêt en fin de séance.
- Adresse simple et portail captif : fichier `/etc/NetworkManager/dnsmasq-shared.d/padaone.conf`
  (installé une fois avec les droits administrateur) et serveur sur le port 80. Mise en œuvre :
  - le fichier contient `no-resolv` et `server=127.0.0.1#5380` : le dnsmasq du point d'accès transmet toutes
    les questions DNS à Padaone, qui écoute **uniquement en local** sur 127.0.0.1:5380 et répond par l'adresse
    actuelle du PC (TTL 10 s ; type A seulement, réponse vide pour AAAA). Le fichier ne dépend donc pas de
    l'adresse, qui change d'une salle à l'autre. En mode désactivé, seul `padaone.lan` existe ;
  - port 80 : `/etc/sysctl.d/60-padaone.conf` avec `net.ipv4.ip_unprivileged_port_start=80` ; Padaone réessaie
    d'écouter sur le port 80 toutes les 15 s, le réglage peut donc se faire sans le relancer ;
  - `padaone point-acces portail` affiche l'état de ces deux réglages et les commandes `sudo` à copier ;
  - option `--sans-portail` : ni DNS local, ni port 80.
- Routes de portail captif à servir : `/generate_204`, `/gen_204` (Android), `/hotspot-detect.html`,
  `/library/test/success.html` (Apple), `/connecttest.txt`, `/ncsi.txt` (Windows), `/success.txt`,
  `/canonical.html` (Firefox). Mode « ouvrir » : redirection (302) vers le lien de la séance avec `&portail=1` (la page affiche alors une aide :
  garder la fenêtre de connexion ouverte). Mode « silencieux » : réponse attendue (204 ou texte « Success »).
  Mode « désactivé » : 404. Dans les deux premiers modes, une requête vers un nom inconnu (pas une adresse IP,
  ni `localhost`, ni `padaone.lan`) est aussi redirigée vers la séance.
- Limite Android : le test HTTPS de Google ne peut pas réussir hors ligne. En mode silencieux, Android parle de
  « connexion limitée » ; le mode « ouvrir » transforme ce message en « Se connecter au réseau », qui ouvre la séance.
- Appareils connectés : `iw dev <carte> station dump` (lignes « Station »), à défaut les baux DHCP.
- QR Wi-Fi : `WIFI:T:WPA;S:<nom>;P:<mdp>;;` (échapper `\ ; , : "`), `padaone_net::texte_qr_wifi`.
- Implémentation (`padaone-net`) : `diagnostic` (analyse des sorties de nmcli, droits polkit, constats),
  `point_acces` (`diagnostiquer`, `demarrer`, `arreter`, `appareils`), `executeur` (`Systeme` lance nmcli avec
  `LC_ALL=C` ; `Simulation` imite un poste à deux cartes, option `--reseau-simule`), `suggestion` (nom et mot de
  passe lisibles). Le mot de passe n'apparaît jamais dans les messages d'erreur.
- Démarrage : suppression d'un ancien profil `padaone-ap`, création, `nmcli -w 30 connection up padaone-ap` ; en cas
  d'échec, la connexion précédente de la carte est rallumée. Arrêt : `connection down`, puis la connexion précédente
  est rallumée. Carte conseillée : une carte capable et libre (clé USB) plutôt que celle qui porte internet.
- « Garder l'adresse actuelle » : `ipv4.addresses` reprend l'adresse de la carte ; sans adresse, 10.42.0.1/24.
- À l'arrêt de Padaone (Ctrl+C), le point d'accès démarré par la séance est arrêté si l'option est cochée
  (cochée par défaut).
- Droits : NetworkManager vérifie les autorisations polkit `network-control`, `wifi.share.protected`,
  `wifi.share.open`, `settings.modify.system`. `padaone point-acces droits` les explique. Padaone ne demande ni ne
  stocke jamais de mot de passe administrateur et n'exécute jamais `sudo` ; `padaone point-acces portail` affiche les
  commandes à copier pour l'installation unique du portail captif et de `padaone.lan`.
- Capacité : cartes Intel souvent limitées à 8-14 appareils en point d'accès ; clé USB ou routeur de voyage
  au-delà. Avec une seule carte, le PC perd internet pendant la séance (sans conséquence).

## 6. Architecture

- `padaone-core` : domaine pur, testé. `padaone-net` : réseau. `padaone` : serveur et interfaces.
- Serveur : axum + tokio, un WebSocket `/ws` par client, pages embarquées (rust-embed), QR en SVG (qrcode).
- État en mémoire protégé par un verrou ; chaque modification produit un **événement** ajouté au journal
  JSON Lines de la séance (hors du dépôt, par exemple `~/.local/share/padaone/seances/`).
- Format d'une ligne du journal : `{"h": <millisecondes depuis 1970>, "e": "<événement>", …}`, par exemple
  `{"h":1793988095172,"e":"activite_lancee","id":1}`. Événements : `seance_ouverte` (toujours en premier),
  `titre_change`, `participant_inscrit` (`id`, `jeton`), `activite_ajoutee` (`id`, `configuration`),
  `activite_supprimee`, `activite_lancee`, `retour_accueil`, `revelee` (`id`, `on`), `figee` (`id`, `on`),
  `reinitialisee`, `reponse_enregistree` (`activite`, `participant`, `reponse`), `mot_masque` (`id`, `cle`, `on`),
  `mots_fusionnes` (`id`, `de`, `vers`), `mot_separe` (`id`, `cle`), `element_valide` (`id`, `rang`, `on`),
  `minuterie_reglee` (`id`, `fin` ou `null`). Les
  signaux discrets ne sont pas journalisés. Le journal ne contient aucun
  pseudo ni aucune adresse : le jeton sert seulement à retrouver un téléphone qui se reconnecte.
- Reprise : `padaone --reprendre <journal>` relit les événements, garde le code de la séance et continue
  d'écrire dans le même fichier. Les téléphones se reconnectent seuls avec leur jeton.
- Diffusion : une tâche envoie les vues aux clients au plus 10 fois par seconde quand l'état a changé,
  et une fois par seconde pour les éléments qui s'effacent avec le temps (signaux).
- Sécurité : la console et l'écran exigent une clé secrète passée dans l'adresse (`/console?cle=…`) ; les
  participants n'y ont jamais accès. La clé est tirée au premier lancement et gardée dans
  `~/.config/padaone/cle` (droits 600) : les pages ouvertes se reconnectent après une relance.
  `--nouvelle-cle` en tire une autre (lien projeté par erreur…).

## 7. Protocole WebSocket (v1, en cours)

Tous les messages sont des objets JSON avec un champ `t`. Les identifiants d'activité sont des entiers.
Un message dont le `t` est inconnu du serveur est ignoré (messages futurs). Implémentation :
`crates/padaone/src/protocole.rs` et `crates/padaone-core/src/vue.rs`.

### Client → serveur

| `t` | Rôle | Champs | Effet |
| --- | --- | --- | --- |
| `bonjour` | tous | `role` (`participant`, `ecran`, `console`), `code` et `jeton` (participant ; jeton absent à la première visite), `cle` (écran, console) | Ouvre la session du client ; premier message obligatoire, dans les 10 s |
| `reponse` | participant | `activite` + un champ selon le type, voir le tableau ci-dessous | Enregistre ou remplace la réponse ; refusée si l'activité n'est pas en cours, est figée ou si la forme ne correspond pas |
| `niveau` | participant | `activite`, `valeur` (0-100) | Niveau de l'applaudimètre, non journalisé |
| `signal` | participant | `type` (`vite`, `perdu`, `exemple`, `pause`) | Allume le signal, ou l'éteint s'il l'était ; effacé après 3 min ; non journalisé |
| `ping` | tous | | Maintien de la connexion (toutes les 15 s) ; le serveur répond `pong` |
| `commande` | console | `c` + paramètres, voir ci-dessous | Pilote la séance |

Configuration (`ajouter`) et réponse de chaque type d'activité :

| `type` | Champs de `ajouter` | Champ de `reponse` |
| --- | --- | --- |
| `sondage` | `question`, `options` (2 à 6) | `choix` : indice à partir de 0 |
| `cartons` | `question`, `options` (2 à 6, une couleur chacune) | `choix` |
| `nuage` | `question`, `max_mots` (1 à 3, 3 par défaut) | `mots` : liste de 1 à `max_mots` textes de 25 caractères au plus |
| `echelle` | `question`, `max` (5 ou 10), `libelle_min` et `libelle_max` facultatifs | `valeur` : de 1 à `max` |
| `meteo` | `question` | `point` : `{"x": humeur, "y": énergie}`, de 0 à 100 |
| `pari` | `question`, `options` (2 à 6), `bonne` (indice) | `pari` : `{"choix": i, "certitude": 0 à 2}` (je devine, je pense, sûr·e) |
| `roti` | `question` | `roti` : `{"valeur": 0 à 5, "mot": "…"}` (mot facultatif) |
| `ouverte` | `question` | `texte` : 280 caractères au plus |
| `questions` | `question` (consigne) | `question` : poser (200 caractères, 3 par personne) ; `vote` : `{"question": rang, "on": bool}` |
| `ordre` | `question`, `elements` (3 à 8, **dans le bon ordre** : `texte` ≤ 40 caractères, `couleur` parmi `violet`, `corail`, `soleil`, `menthe`, `rose`, `ciel`, `lilas`, `peche`, `blanc`, `encre`), `duree` (secondes, 10 à 3600, facultative) | `ordre` : positions **affichées**, dans l'ordre proposé (une permutation) |

Commandes de la console (`c`) : `ajouter` (`type` + champs ci-dessus), `supprimer` (`id`), `lancer` (`id`), `accueil`,
`reveler` (`id`, `on`), `figer` (`id`, `on`), `reinitialiser` (`id`), `masquer_mot` (`id`, `cle`, `on`),
`fusionner` (`id`, `de`, `vers` : la clé `de` rejoint `vers`), `separer` (`id`, `cle` : annule une fusion),
`valider` (`id`, `rang`, `on` : message de la question ouverte ou question du public, projeté ou non),
`mesurer` (`option`), `arreter_mesure`, `titre` (`titre`), `point_acces_demarrer` (réglages),
`point_acces_arreter`, `diagnostic`, `point_acces_suggerer` (nouveau nom et mot de passe suggérés),
`minuterie` (`id`, `on` : redémarre la minuterie pour sa durée et rouvre les réponses, ou l'arrête),
`point_acces_options` (`arret_fin_seance`, `portail` : `ouvrir`, `silencieux`, `desactive` ; chacun facultatif). `point_acces_demarrer` prend `carte` (facultatif), `nom_reseau`,
`securite` (`{"type":"wpa2","mot_de_passe":"…"}` ou `{"type":"ouvert"}`), `bande` (`auto`, `ghz2_4`, `ghz5`),
`canal` (facultatif), `adresse` (`garder`, `par_defaut` ou `{"fixe":"192.168.50.1/24"}`). Les commandes réseau ne
sont pas journalisées ; une seule tourne à la fois. Sauf pour `supprimer` et `lancer`, `id` est facultatif :
sans lui, la commande vise l'activité en cours. `masquer_mot`, `fusionner` et `separer` valent pour le nuage
et les mots du ROTI. Pas encore traitées : `mesurer`, `arreter_mesure`.

Exemples :

```json
{"t":"bonjour","role":"participant","code":"482731","jeton":"9f2c…"}
{"t":"commande","c":"ajouter","type":"sondage","question":"Café ou thé ?","options":["Café","Thé"]}
{"t":"reponse","activite":1,"choix":0}
{"t":"reponse","activite":2,"pari":{"choix":1,"certitude":2}}
{"t":"commande","c":"fusionner","de":"ecouter","vers":"ecoute"}
{"t":"signal","type":"perdu"}
```

### Serveur → client

| `t` | Champs |
| --- | --- |
| `bienvenue` | `jeton` (participant : à garder dans le navigateur), `seance` (`titre`, `code`), `reseau` (écran et console : `adresse`, `lien`, `qr` en SVG) |
| `etat` | Vue propre au rôle (ci-dessous) |
| `erreur` | `message` lisible en français, `fatale` (vrai si la clé ou le code est refusé : le client ne se reconnecte pas) |
| `pong` | |
| `reseau` | Écran et console : mêmes champs que `reseau` de `bienvenue` (`adresse`, `lien`, `qr`, `wifi` : `nom`, `mot_de_passe` ou `null`, `qr` du Wi-Fi), envoyé quand l'adresse ou le Wi-Fi change |

- Vue participant : `titre`, `mes_signaux` (signaux allumés), `activite` (`null` à l'accueil, sinon `id`, `type`,
  les champs de configuration, `ouverte`, `revelee`, `ma_reponse` sous la même forme que l'envoi, par exemple
  `{"choix": 1}`, ou `null`). Questions du public : `donnees` (`validees` à voter avec `votes`, `mon_vote`,
  `de_moi` ; `miennes` avec leur état ; `restantes`). Aucun résultat, aucune réponse des autres.
- La bonne réponse d'un pari (`bonne`) n'apparaît dans les vues participant et écran qu'une fois révélée.
- Vue écran : `titre`, `code`, `connectes` (téléphones connectés), `activite` (`null` à l'accueil, sinon `id`, `type`,
  `question`, `options`, `revelee`, `figee`, `reponses` et `resultat`, absent si non révélée ou si `reponses < 3`).
  L'adresse et le QR arrivent dans `bienvenue`. À venir : Wi-Fi éventuel.
- Vue console : `titre`, `code`, `connectes`, `inscrits`, `courante` (id ou `null`), `deroule` (chaque activité avec
  `revelee`, `figee`, `reponses`, `affichable` — seuil d'anonymat atteint — et `resultat` toujours présent, avec
  les mots masqués et les messages non validés), `signaux` (`type`, `n`, `alerte` à partir de 20 % des connectés).
  `point_acces` : `simule`, `en_cours`, `message` (`texte`, `erreur`), `diagnostic` (`cartes`, `droits`,
  `point_acces_actif`, `constats` avec `niveau` `bon` / `attention` / `bloquant`), `actif` (`carte`, `nom_reseau`,
  `securite`, `adresse`, `connexion_precedente`), `appareils`, `arret_fin_seance`, `adresse_initiale`, `suggestion`, `portail`, `portail_port_80`, `portail_dns`
  (le diagnostic porte aussi `portail` : `dns` et `port_80` installés ou non).
- `reponses` compte les participants qui ont répondu (pour les questions du public : ceux qui ont posé une question).
- Résultats (l'écran reçoit une version filtrée : sans mots masqués, seulement les éléments validés) :
  - sondage, cartons : `{"effectifs": [3, 5, 0], "total": 8}` (dans l'ordre des options) ;
  - nuage : `{"mots": [{"cle", "texte", "n", "masque", "fusionnes"}], "participants"}`, dans l'ordre d'arrivée
    des clés ; à l'écran, 40 mots au plus (les plus cités) ;
  - échelle : `effectifs` (de 1 à `max`), `total`, `mediane` (moyenne des deux centrales si nombre pair), `moyenne` ;
  - météo : `points` (triés, sans lien avec l'ordre d'arrivée), `total`, `moyenne` (`[x, y]`) ;
  - pari : `effectifs`, `total`, `bonne`, `acquis_solide`, `a_consolider`, `fausses_certitudes`, `doute_utile`,
    `fausses_certitudes_par_option` (« sûr·e » = certitude 2 ; « je devine » et « je pense » = pas sûr·e) ;
  - ROTI : `effectifs` (0 à 5 doigts), `total`, `moyenne`, `mots` (comme le nuage) ;
  - question ouverte : `messages` (`rang`, `texte`, `valide`), `total` ; à l'écran, les 6 derniers validés ;
  - questions du public : `questions` (`rang`, `texte`, `votes`, `valide`) triées par votes, `total` ;
    à l'écran, les 6 premières validées.

## 8. Feuille de route

Voir `docs/04-feuille-de-route.md` (phases, tâches, étapes de validation).
