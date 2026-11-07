---
name: avancer
description: Lit la feuille de route, identifie la prochaine tâche non cochée de la phase en cours et propose un plan pour la réaliser. À utiliser en début de séance de travail ou quand Julien demande « on fait quoi maintenant ? ».
---

1. Lis `docs/04-feuille-de-route.md` et repère la phase en cours (la première qui a des cases non cochées).
2. Prends la première tâche non cochée de cette phase. Si elle dépend d'une autre tâche non faite, prends celle-là.
3. Lis ce qu'il faut pour la comprendre : `CLAUDE.md`, la section utile de `docs/03-specification.md`,
   les maquettes concernées et le code existant.
4. Présente à Julien, en français et en moins de 15 lignes :
   - la tâche choisie et pourquoi elle vient maintenant ;
   - les fichiers qui seront créés ou modifiés ;
   - comment on saura qu'elle est finie (tests, commande à lancer, vérification à la main) ;
   - ce qui nécessite son intervention (matériel, téléphone, commande réseau à lancer lui-même).
5. Attends son accord avant de coder. Une fois la tâche terminée et `/verifier` au vert, coche la case
   dans la feuille de route.

Si une étape de validation (losange) termine la phase, rappelle les critères à vérifier en salle ou sur le matériel.
