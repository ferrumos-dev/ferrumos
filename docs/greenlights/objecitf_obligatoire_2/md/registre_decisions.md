# Registre des décisions techniques

Consigner une décision au moment de sa validation. Pour chaque entrée, garder le contexte, les options réellement étudiées, les conséquences et les preuves. Une proposition conserve ce statut jusqu’à l’arbitrage de l’équipe.

| Sujet | Statut | Proposition et alternative à comparer | Preuve à joindre après arbitrage |
| --- | --- | --- | --- |
| Architecture du noyau | Proposée | Monolithique modulaire pour intégrer les PoC. Micronoyau pour déplacer des services derrière des frontières de protection. | [Note de décision](../pdf/01_FerrumOS_Note_de_decision.pdf), choix daté, coût accepté et schéma mis à jour |
| Politique de sélection | Proposée | Priorités fixes et FIFO à priorité égale. Comparer le quantum par priorité et le calcul adaptatif. | [Spécification et scénarios S01 à S07](../../objecitf_obligatoire_1/ordonnancement_a_priorites.md), résultats et politique retenue |
| Validation en CI | Proposée | Contrôles séparés pour qualité, compilation et QEMU. Comparer avec la validation limitée à la compilation des PoC. | [Plan CI](./ci_cd.md), lien des exécutions nominale et en échec, conditions de fusion |

## Modèle d’entrée

```markdown
### Titre de la décision

- Date et statut
- Contexte et problème
- Options examinées et comparaison selon les mêmes critères
- Option choisie et raison
- Conséquences acceptées et condition de réexamen
- Liens vers le schéma, la PR et les tests concernés
```

Pour une décision révisée, conserver l’entrée initiale. Ajouter la décision qui la remplace et le fait ou la mesure qui justifie le changement.

## Rédiger et utiliser une décision dans une PR

Écrire l’entrée dans ce registre au moment où la PR introduit le choix. Reprendre son identifiant dans la description de la PR et dans la stratégie de tests. Utiliser une date `AAAA-MM-JJ`, le statut `Proposée`, `Acceptée` ou `Remplacée` et au moins deux options évaluées sur les mêmes critères. Indiquer le fichier touché, la preuve à obtenir et les conditions qui justifieraient une révision.

### Exemple à adapter pour la politique de priorité

Dans `docs/greenlights/objecitf_obligatoire_2/md/registre_decisions.md`, ajouter une entrée comme celle-ci lorsque l’équipe ouvre la discussion. Compléter la date, le statut et les liens dès qu’une décision ou un résultat est disponible.

```markdown
### ADR-001 · Règle de choix de la prochaine tâche

- Date à renseigner lors de la revue
- Statut proposé
- Contexte. Le PoC adaptatif simule la progression des tâches.
- Option A. Priorité fixe, ordre FIFO à priorité égale.
- Option B. Priorité par niveau avec un quantum pour départager les tâches de même niveau.
- Critères. Ordre observé, attente maximale et coût de la sélection.
- Proposition. A, pour tester un contrat déterministe à partir de séquences d’événements.
- Contrepartie. Une charge haute continue peut retarder les tâches basses.
- Fichier. POC/ferrum_Poc_adaptative_sheduller/src/scheduler.rs, pour lire le simulateur.
- Preuves. S01 à S06 sur le sélecteur portable, puis S07 sur l’image intégrée.
- Liens. Issue, PR, révision, journal et schéma à renseigner après validation.
- Réexamen. Attente mesurée incompatible avec le besoin du shell ou des tâches de fond.
```

### Transformer le choix en vérifications de code

Associer chaque règle à une observation distincte. La règle « priorité haute d’abord » correspond à S02 et à l’identifiant retourné. La règle FIFO correspond à S03 et S04, avec l’ordre avant et après cession ou réveil. La preuve d’une exécution CPU correspond à S07 et à une trace de bascule. Relier les scénarios à la PR et préciser le périmètre de chaque preuve.

Pour relever la révision et vérifier les fichiers avant revue, lancer depuis la racine du dépôt.

```sh
git rev-parse HEAD
git diff --check
git diff -- 'docs/greenlights/objecitf_obligatoire_2/md/registre_decisions.md'
```

**Critère de revue.** Le relecteur peut relier la règle retenue à un fichier, une modification et un scénario d’acceptation. Lorsqu’une mesure ou un test motive un changement, conserver les deux entrées et relier la preuve à la décision qui remplace l’ancienne.
