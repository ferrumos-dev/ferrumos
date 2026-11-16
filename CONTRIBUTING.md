# Contribuer à FerrumOS

## Préparer le changement

1. Repérer le composant concerné dans [l'architecture de `src/`](./docs/architecture.md) ou le prototype dans le [README](./README.md). Lier la tâche existante si elle couvre le changement.
2. Créer une branche de travail et limiter la PR à un comportement, un correctif ou une décision documentée.
3. Pour le Rust, appliquer la [norme de qualité](./docs/greenlights/objecitf_obligatoire_2/md/normes_rust.md). Pour un contrat modifié, mettre à jour [les interfaces noyau](./docs/greenlights/objecitf_obligatoire_2/md/contrats_noyau.md) et le test correspondant.
4. Exécuter les contrôles applicables et joindre les commandes, la cible et le résultat observé. Identifier une vérification encore à effectuer dans la PR.
5. Demander la revue d’un autre membre. Corriger les points ouverts et vérifier l’état des contrôles pour la révision à fusionner. Voir [les contrôles de CI](./docs/greenlights/objecitf_obligatoire_2/md/ci_cd.md).

## Modifier les sources du projet

Le code commun se trouve dans `src/` et se compile depuis la racine du dépôt.
Placer chaque changement dans le module responsable décrit par
[l'architecture](./docs/architecture.md). Le module `kernel` interdit `unsafe` ;
les mécanismes matériels appartiennent à `arch` et aux pilotes. Documenter les
interfaces publiques et les invariants des opérations bas niveau.

Exécuter les [contrôles de qualité et de compilation](./README.md#quality-checks)
avec la chaîne Rust épinglée. Ajouter des tests de comportement avec les premières
implémentations. Les projets dans `POC/` gardent leur propre chaîne nightly et leurs
commandes : les contrôles de la racine ne les compilent pas.

## Placer la documentation

| Contenu | Emplacement |
| --- | --- |
| Découverte du projet et accès aux PoC | [README racine](./README.md) |
| Procédure spécifique à un PoC | Son README dans [POC](./POC/) |
| Compilation du projet et contrôles de qualité | [README racine](./README.md#getting-started) |
| Structure des sources et responsabilités des modules | [Architecture du projet](./docs/architecture.md) |
| Architecture, critères de validation, qualité, tests et exploitation Greenlight | [docs/greenlights](./docs/greenlights/README.md), dans le dossier de l’objectif concerné. Les nouvelles fiches de l’objectif obligatoire 2 sont rangées dans `md/` et `pdf/` |
| Énoncés, dossiers EIP et supports | [Index de la documentation](./docs/README.md) |

Pour un nouveau fichier Markdown, utiliser un nom court en `snake_case`. Garder les liens relatifs, encoder les espaces sous la forme `%20` et mettre à jour l’index concerné. Pour une preuve, indiquer la révision, la cible et la date. Garder les images de build et les journaux dans les artefacts associés à la vérification. Conserver un lien vers l’artefact dans la PR ou le document qui s’appuie dessus.
