# FerrumOS · Dossier Greenlight

Ce dossier réunit les références de contribution sur l’ordonnancement, les interfaces du noyau, la qualité Rust, la CI, les tests et l’exploitation. Pour chaque changement, utiliser la fiche du composant et rattacher les preuves à la révision vérifiée.

| Objectif | Documents |
| --- | --- |
| Obligatoire 1 · difficulté technique et expérimentation | [Ordonnanceur à priorités](./objecitf_obligatoire_1/ordonnancement_a_priorites.md) |
| Obligatoire 2 · architecture et fiabilité | [Note de décision, schémas T02/T04, normes Rust, CI/CD, tests, exploitation et registre](./objecitf_obligatoire_2/README.md) |

## Preuves à réunir

| Exigence | Référence de travail | Preuve de validation |
| --- | --- | --- |
| Ordonnancement | Round Robin du banc archivé et simulateur à priorités du dépôt | S01 à S06 sur la politique retenue, puis S07 sur l’image intégrée |
| Architecture et contrats | PDF T02/T04 et carte des interfaces | Schémas relus, à jour avec les chemins et la révision du prototype choisi |
| Qualité du code | Normes Rust et revue de PR | Résultats des contrôles de qualité, écarts mesurés et exceptions motivées |
| CI/CD | Guide de configuration du pipeline | Historique des jobs, échec de contrôle détecté, images et journaux rattachés à la révision |
| Tests | 8/8 sur le banc QEMU indépendant, panique volontaire détectée | Tests du chemin critique intégré, couverture mesurée et régressions réelles tracées |
| Exploitation | Commandes des PoC et procédure historique du banc | Reproduction datée par un tiers, durée, difficultés et corrections documentaires |

## Références

Sources internes · *Technical Track - Student Guide – Tech 4*, p. 7–11 et 20–21, [organisation technique](../ORGANIZATION.md), archive de preuves FerrumOS d’octobre 2026.
