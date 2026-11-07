# Inventaire de l'existant — outils de participation par smartphone

*Phase 1 du projet « logiciel de participation lors d'une formation » — état des lieux au 6 novembre 2026.*
*Les prix et limites des offres gratuites sont indicatifs : les sources se contredisent souvent, à revérifier avant toute comparaison chiffrée.*

---

## 1. Panorama en un coup d'œil

L'existant se répartit en six familles :

| Famille | Exemples | Ce qu'on peut en retenir pour le projet |
|---|---|---|
| **A. Outils commerciaux généralistes** (« audience response ») | Mentimeter, Slido, Wooclap, AhaSlides, Vevox, Poll Everywhere | Le standard d'usage : rejoindre par code/QR, sans compte, résultats en direct. Modèle freemium très bridé. |
| **B. Quiz gamifiés** | Kahoot!, Wayground (ex-Quizizz), Crowdpurr | Mécaniques de jeu (minuteur, classement, équipes). Peu adaptés à l'expression d'opinions. |
| **C. Outils d'atelier / facilitation** | Klaxoon, Stormz, Beekast | Méthodes d'animation outillées (brainstorm, vote 100 points, multicritère, World Café). |
| **D. Libres / auto-hébergeables** | Claper, Digistorm, Particify (ARSnova), frag.jetzt, Fuiz, ClassQuiz, Razzia, Pulse | Base de code et d'idées réutilisable. **Fuiz est écrit en Rust.** |
| **E. Délibération et participation citoyenne** | Pol.is, All Our Ideas, Mieux Voter, Decidim, OpenSlides | Les mécaniques de décision collective les plus riches, quasi absentes des outils de formation. |
| **F. Hors smartphone / hybride** | Plickers, Quizzbox (boîtiers), Poll Everywhere (SMS), Digibuzzer | Inclusion des personnes sans smartphone ou sans réseau. |

---

## 2. Outils commerciaux généralistes (famille A)

| Outil | Origine / positionnement | Points forts | Offre gratuite (indicatif) | Intégrations |
|---|---|---|---|---|
| **Mentimeter** | Suède — présentations, ateliers, formation | Diapos interactives, mécaniques originales : *100 points*, *grille 2×2*, *Pin on image*, *Compare* (avant/après), réactions | Très limitée (2 questions par présentation ou 50 participants/mois selon les sources) | Import de présentations |
| **Slido** (Cisco) | Réunions, conférences, webinaires | Q&A modéré avec votes (*upvote*), 5 types de sondages, quiz, « Ideas » | 100 participants, 3 sondages par session | PowerPoint, Google Slides, Webex, Zoom, Teams |
| **Wooclap** | Belgique (Bruxelles) — enseignement sup., formation pro | 20+ types de questions, dont **Test de concordance de script** (formation santé), légender une image, texte à trous, feuille d'émargement, mode équipe | 5 questions/mois selon une source | PowerPoint, Google Slides, Teams, Zoom, LMS (LTI 1.3) |
| **AhaSlides** | Budget, PME, éducation | 30+ types de diapos, *Idea Board*, *Draw Answer*, roue aléatoire, filtre de grossièretés, synthèse IA | 7 à 50 participants selon les sources | Google Slides, PowerPoint, Zoom, Teams |
| **Vevox** | Événements, formation, enseignement sup. | Contrôle fin de l'anonymat, présence vérifiée, marque blanche, RGPD | 100 participants/session | PowerPoint, Teams, LMS |
| **Poll Everywhere** | États-Unis, depuis 2007 | **Vote par SMS** (sans appli ni internet mobile), accessibilité (lecteur d'écran, contraste) | 25 à 40 participants | PowerPoint, Keynote, Google Slides, LMS |
| **Pigeonhole Live** | Grandes conférences | Q&A multi-modérateurs, multi-sessions | 30 participants/session | — |
| **DirectPoll** | Sondages éclair | Un lien, aucun compte | Basique gratuit | — |

**À noter (2025-2026)** : Klaxoon racheté par Wrike (janvier 2025), Wooclap a annoncé le rachat de Vevox (septembre 2026), Quizizz rebaptisé Wayground. Le marché se concentre.

---

## 3. Quiz gamifiés (famille B)

| Outil | Points forts | Limites pour la formation adulte |
|---|---|---|
| **Kahoot!** | Classement, musique, rythme, mode équipe, « Missions » | Compétition et vitesse privilégiées sur la réflexion ; peut mettre en difficulté des adultes peu à l'aise |
| **Wayground** (ex-Quizizz) | 25+ aménagements d'accessibilité (police dyslexie, lecture à voix haute), mode asynchrone | Orienté révision, plafond de 150 participants sur la plupart des offres |
| **Crowdpurr** | Quiz événementiel : paris, élimination, multimédia | Divertissement plus que pédagogie |

---

## 4. Outils d'atelier et de facilitation (famille C)

| Outil | Origine | Mécaniques | Remarques |
|---|---|---|---|
| **Klaxoon** | France (Rennes), racheté par Wrike | Tableaux visuels type post-it, vote, priorisation, 14 types de questions, « mode aventure » | Orienté équipes projet, design thinking |
| **Stormz** | — | Génération d'idées, **budget de 100 $**, évaluation multicritère, analyse de risques, World Café, Open Space ; idées anonymes et mélangées | Jusqu'à 200 participants ; offre Pro XL gratuite pour l'éducation et les associations |
| **Beekast** | France (Paris) | Mur de messages, sondages, quiz, brainstorm puis vote, nuage de mots, mur de selfies, parcours en autonomie | Entièrement en français, cible formation et RH |

---

## 5. Outils libres et auto-hébergeables (famille D)

| Outil | Licence | Technologie | Mécaniques | Intérêt pour le projet |
|---|---|---|---|---|
| **Fuiz** | AGPL-3.0+ | **Rust** (actix-web), moteur de jeu compilable en WASM, déploiement Cloudflare Workers | Quiz (QCM, réponse texte), **bingo**, tableau de liège d'images, équipes, classement ; jusqu'à 1000 joueurs | **Seul projet Rust repéré.** Revendique la coopération plutôt que la compétition. Référence d'architecture directe. |
| **Claper** | AGPL-3.0 | Elixir, Phoenix LiveView, PostgreSQL | Q&A (anonyme ou nominatif, tri par popularité, épinglage), sondages, quiz, formulaires, réactions emoji, sous-titres en direct | Interactions greffées sur un PDF/PPTX importé ; vue présentateur et vue gestion séparées ; ~800 étoiles, actif |
| **Digistorm** (La Digitale) | AGPL-3.0 | Node.js, Vue, Redis | Sondages, questionnaires, brainstorm par catégories, nuage de mots, « séquences » | Français, sans compte participant (code à 7 chiffres, lien ou QR), sans collecte de données revendiquée. Suite associée : Digiwords, Digipad, **Digiscreen** (fond de vidéoprojection interactif), **Digibuzzer** |
| **Particify / ARSnova** | GPL-3.0 (serveur), MIT (client) | Java, TypeScript | Quiz, sondages, Q&A, retours, modération | Issu de l'université de Giessen (THM), longue histoire en enseignement sup. |
| **frag.jetzt** | — | TypeScript | Q&A d'amphi | Spécialisé questions du public |
| **ClassQuiz** | MPL-2.0 | Python (FastAPI), SvelteKit | Quiz type Kahoot | Auto-hébergement lourd (PostgreSQL, Redis, Meilisearch) |
| **Razzia** (ex-Rahoot) | MIT | Node.js, TypeScript | Quiz type Kahoot | **Protocole WebSocket documenté pour clients tiers, y compris un buzzer ESP32** |
| **Pulse** | MIT | Node.js, Socket.io, SQLite, React | Nuage de mots, sondage, classement, note 5 étoiles, Q&A avec votes, brainstorm | **Télécommande présentateur appairée par QR** (pause, minuteur, afficher/masquer résultats), pluie d'emojis, export CSV ; petit projet |
| **Joyconf** | — | Elixir, Phoenix | Réactions emoji uniquement | Emojis qui flottent par-dessus les diapos (extension Chrome) ; limite d'un clic toutes les 3 s |
| **Quiqui** (TH Nürnberg) | — | Node.js, socket.io | Quiz | Questions en YAML versionnées dans Git, sans base de données |

---

## 6. Délibération et participation citoyenne (famille E)

| Outil | Type | Mécanique centrale | Licence | Idée réutilisable en salle |
|---|---|---|---|---|
| **Pol.is** | En ligne, temps réel | Les participants écrivent de courtes affirmations, puis votent **d'accord / pas d'accord / passer** sur celles des autres ; regroupement automatique des opinions | AGPL-3.0 | Carte 2D des groupes d'opinion + liste des points de **consensus** et de **division**. *Attention : peu fiable sous 20-30 participants.* |
| **All Our Ideas** | En ligne | **Comparaison par paires** : « voici deux idées, choisissez-en une » ; chacun peut ajouter une idée | Libre | Duels A/B projetés, classement qui bouge en direct ; très rapide en salle |
| **Mieux Voter** | En ligne + appli Android hors ligne | **Jugement majoritaire** : chaque proposition reçoit une mention (Excellent → À rejeter), la médiane désigne le gagnant | GPL-3.0 / MIT selon les briques | Histogramme des mentions par proposition avec la médiane marquée ; appli « urne » sur un seul téléphone qui circule |
| **OpenSlides** | Web, assemblées | Ordre du jour, motions, élections, **service de projection dédié** | MIT | Gestion d'assemblée générale et vote formel |
| **Decidim** | Plateforme en ligne (Barcelone) | Processus participatifs, budgets, propositions ; vote rapide pendant les réunions en ligne | AGPL-3.0 | Articulation entre temps en salle et suites en ligne |
| **Loomio, Consul, Cap Collectif, Kialo…** | Plateformes en ligne, asynchrones | Décision par consentement (Loomio), cartographie d'arguments (Kialo), propositions citoyennes | Divers | Mécaniques de décision à transposer, pas des outils de salle |

---

## 7. Hors smartphone et hybrides (famille F)

| Solution | Principe | Intérêt |
|---|---|---|
| **Plickers** | Cartes papier codées que le participant oriente (A/B/C/D) ; l'animateur les scanne avec son téléphone | Zéro équipement côté participant |
| **Quizzbox** (Clermont-Ferrand) | Boîtiers de vote + smartphones dans la même session, vote nominatif, émargement | Formations réglementaires (CACES, habilitations), assemblées |
| **Poll Everywhere** | Vote par SMS | Fonctionne sans données mobiles |
| **Digibuzzer / buzzer ESP32 (Razzia)** | Buzzer connecté | Piste matérielle bon marché |

---

## 8. Catalogue des mécaniques d'interaction recensées

Union dédoublonnée des types proposés par Mentimeter, Wooclap, Slido, AhaSlides et les outils libres.

**Choisir / voter**
- Choix unique ou multiple (avec ou sans bonne réponse), vrai/faux
- Sondage d'opinion
- Échelle de notation (1-5, étoiles, Likert)
- Répartition de points (100 points / budget)
- Grille 2×2 / matrice (ex. impact × effort)
- Jugement majoritaire *(seulement Mieux Voter)*
- Comparaison par paires *(seulement All Our Ideas)*
- D'accord / pas d'accord / passer + regroupement *(seulement Pol.is)*

**Ordonner / prioriser**
- Classement par préférence
- Remettre dans l'ordre (bonne réponse)

**Associer / catégoriser**
- Associer des paires
- Glisser-déposer dans des catégories
- Texte à trous
- Test de concordance de script *(Wooclap)*

**Sur une image**
- Placer un point sur une image (*Pin on image*, carte de chaleur)
- Trouver une zone, légender une image
- Dessiner sa réponse *(AhaSlides)*

**Texte libre / idées**
- Question ouverte, réponse courte
- Nuage de mots
- Brainstorm / mur d'idées, avec vote ensuite
- Mur de messages

**Questions du public (Q&A)**
- Questions avec votes pour faire remonter les plus demandées
- Modération avant publication, « marquer comme répondu », épingler
- Regroupement ou synthèse par IA

**Nombres et formulaires**
- Trouver un nombre (estimation)
- Formulaire rapide, questionnaire

**Aides à l'animation**
- Minuteur, roue aléatoire (tirage au sort)
- Comparaison avant/après
- Diapos de contenu, médias

**Jeu**
- Quiz avec classement, équipes, séries de bonnes réponses, bingo

**Réactions**
- Emojis en direct, pluie d'emojis par-dessus la projection

---

## 9. Patterns d'interface observés

**Côté participant (smartphone)**
- Entrée par **code court** (6-7 chiffres), **lien** ou **QR code**, **sans compte ni appli** : c'est devenu la norme
- Anonymat par défaut, avec option nominative (émargement, quiz)
- Une seule question à l'écran, rythmée par l'animateur

**Côté projection**
- Résultats en direct (barres, nuage), ou **masqués jusqu'à la révélation**
- Classement animé pour les quiz
- Comparaison de deux votes (avant/après)
- Superposition par-dessus des diapos existantes (Claper, Joyconf, extensions PowerPoint)

**Côté animateur**
- Vue présentateur distincte de la vue projetée
- **Télécommande sur smartphone** (Pulse, Quizzbox) : avancer, figer, révéler, lancer le minuteur
- Modération des contributions avant affichage
- Export des résultats (CSV, Excel, PDF) et émargement

---

## 10. Premiers constats : limites de l'existant et pistes

1. **Modèle économique** : les offres gratuites sont très bridées (5 questions/mois, 30 à 50 participants), et les licences coûtent 12 à 50 €/mois par animateur. Point sensible pour un formateur indépendant.
2. **Dépendance au cloud et à internet** : aucun outil recensé ne documente un vrai mode **réseau local / hors ligne**. Or le Wi-Fi des salles de formation est souvent faible. Un serveur Rust local (portable de l'animateur + point d'accès) serait un vrai différenciateur.
3. **Données personnelles** : la plupart sont hébergés hors de la salle ; seul La Digitale revendique l'absence de collecte. Un outil local règle la question RGPD par construction.
4. **Mécaniques délibératives absentes des outils de formation** : jugement majoritaire, comparaison par paires, consentement, consensus/division à la Pol.is n'existent que dans des plateformes citoyennes en ligne. C'est un espace libre entre formation adulte et participation citoyenne.
5. **Inadaptation aux petits groupes** : la plupart des outils visent des amphis ou des conférences. En formation adulte (8 à 25 personnes), l'anonymat est fragile, les statistiques peu parlantes, et les visualisations doivent rester lisibles avec 12 réponses.
6. **Gamification et public adulte** : le modèle Kahoot (vitesse, classement) peut braquer des adultes ; Fuiz montre une voie plus coopérative.
7. **Inclusion** : peu de solutions pour les personnes sans smartphone, hormis Plickers, les boîtiers et le SMS. L'accessibilité (contraste, taille, lecteur d'écran) reste rarement mise en avant.

**Mécaniques d'animation présentielle encore peu outillées** (à étudier pendant la phase de classement) : météo d'entrée, ROTI (retour sur le temps investi), *fist of five*, vote aux gommettes, baromètre / positionnement sur une ligne, 1-2-4-tous (Liberating Structures), décision par consentement.

---

## 11. Sources

**Comparatifs et outils commerciaux**
- [lafabriquedunet.fr — alternatives à Wooclap](https://www.lafabriquedunet.fr/logiciels/alternatives/alternative-wooclap)
- [XTriv — Best Slido Alternatives 2026](https://xtriv-marketing.pages.dev/blog/best-slido-alternatives/) *(blog d'un éditeur concurrent)*
- [Jotform — Mentimeter alternatives](https://www.jotform.com/ai/agents/mentimeter-alternatives/)
- [Mentimeter — fonctionnalités](https://www.mentimeter.com/features) · [Centre d'aide](https://help.mentimeter.com/)
- [Wooclap — fonctionnalités](https://www.wooclap.com/fr/fonctionnalites/) · [Klaxoon vs Wooclap](https://www.wooclap.com/fr/blog/klaxoon-vs-wooclap/)
- [Slido — features](https://www.slido.com/features)
- [AhaSlides — features](https://ahaslides.com/features/)
- [Stormz (Toolmage)](https://www.toolmage.com/fr/tool/stormz/)
- [Beekast (Outils TICE)](https://outilstice.com/2017/05/beekast-un-outil-pour-dynamiser-vos-cours-et-vos-formations/)
- [Quizzbox](https://quizzbox.com/?p=45)
- [Plickers (Outils TICE)](https://outilstice.com/2014/11/plickers-sondez-vos-apprenants-sans-ordinateur/)

**Libres et auto-hébergeables**
- [GitHub — topic audience-response-system](https://github.com/topics/audience-response-system?o=desc&s=stars)
- [Claper](https://github.com/ClaperCo/Claper)
- [Fuiz — game-backend](https://github.com/fuiz/game-backend)
- [Digistorm (Codeberg)](https://codeberg.org/ladigitale/digistorm) · [La Digitale](https://ladigitale.dev/en)
- [Particify](https://github.com/particify)
- [ClassQuiz](https://github.com/mawoka-myblock/ClassQuiz)
- [Razzia (ex-Rahoot)](https://github.com/Ralex91/Rahoot)
- [Pulse](https://github.com/thefraggle/pulse)
- [Joyconf (dev.to)](https://dev.to/grymoire/joyconf-a-live-emoji-reaction-app-for-presentations-5688)
- [SelfHostYourself — alternatives à Mentimeter](https://selfhostyourself.com/alternative-to/mentimeter)
- [SpeakUp (EPFL)](https://actu.epfl.ch/news/an-app-to-facilitate-classroom-discussions)

**Délibération et participation citoyenne**
- [Pol.is (Computational Democracy)](https://compdemocracy.org/Polis/)
- [All Our Ideas (Citizens Foundation)](https://citizens.is/all-our-ideas/)
- [Mieux Voter — jugement majoritaire](https://deploy-preview-6--mieuxvoter.netlify.app/en/le-jugement-majoritaire) · [GitHub MieuxVoter](https://github.com/MieuxVoter)
- [OpenSlides](https://github.com/OpenSlides/OpenSlides)
- [Decidim — composant Réunions](https://docs.decidim.org/en/develop/admin/components/meetings)
- [Banque des Territoires — outils numériques et démocratie participative](https://www.banquedesterritoires.fr/les-outils-numeriques-pour-augmenter-la-democratie-participative) *(2016)*
