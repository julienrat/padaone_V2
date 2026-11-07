---
name: nouvelle-interaction
description: Ajoute une interaction de participation (sondage, nuage de mots, pouce en l'air, applaudimètre, verdict…) de bout en bout, du domaine aux trois interfaces. À utiliser quand Julien demande d'implémenter une interaction de la spécification.
argument-hint: "[nom de l'interaction, ex. nuage-de-mots]"
disable-model-invocation: true
---

Implémente l'interaction « $ARGUMENTS » dans Padaone.

## 1. Comprendre

- Lis la ligne correspondante dans `docs/03-specification.md` (tableaux d'interactions, règles, protocole).
- Ouvre la ou les maquettes concernées dans `docs/maquettes/` (`Tel-*.dc.html`, `Ecran-*.dc.html`).
- Si un point de la règle métier n'est pas tranché (seuil, arrondi, ordre d'affichage), pose la question
  à Julien avant de coder plutôt que d'inventer.
- Présente un plan court : fichiers touchés, messages du protocole ajoutés, tests prévus. Attends son accord.

## 2. Domaine (`crates/padaone-core`)

- Un module dédié qui implémente le trait `Activite` : configuration, réponse d'un participant,
  validation, application, résultat agrégé.
- Respecte le seuil d'anonymat (`SEUIL_ANONYMAT`) dans le résultat destiné à l'écran.
- Les changements d'état passent par des événements journalisables.
- Tests unitaires : une réponse, une réponse modifiée, une réponse invalide, le résultat sous et au-dessus
  du seuil, et chaque règle propre à l'interaction.

## 3. Serveur (`crates/padaone`)

- Ajoute les messages au protocole (champ `t`) et mets à jour la section « Protocole » de
  `docs/03-specification.md` dans le même changement.
- Les vues envoyées sont différentes selon le rôle : participant (sa propre réponse), écran (résultat
  affichable seulement), console (tout, y compris la modération).

## 4. Interfaces (`crates/padaone/web`)

- Téléphone (`p/`), écran (`ecran/`), console (`console/`) avec les jetons de `commun/theme.css`.
- Si l'interaction utilise un capteur : détection de `window.isSecureContext`, demande d'autorisation sur
  un geste, et repli en boutons automatique.
- Accessibilité : vrais boutons, libellés, cibles d'au moins 44 px, sens jamais porté par la couleur seule.

## 5. Finir

- Lance le skill `/verifier`.
- Coche la case de l'interaction dans `docs/04-feuille-de-route.md`.
- Propose un message de commit : `[phase N] <interaction> : <ce qui a été fait>`.
