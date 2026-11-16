## Tests noyau QEMU — `c88bde4`

> Révision : arbre de travail non commité au-dessus de `c88bde4`, commité ensuite dans `18fd351` (le titre et `results.json` indiquent `c88bde4`). Cible : images de test x86_64 du PoC d’ordonnancement sous QEMU 10.2.1, allocateur `heap-fixed-block`. Date : 2026-11-12. Environnement : [`test_environment.md`](../../../../../../POC/ferrum_Poc_adaptative_sheduller/test_environment.md).

| Image de test | Statut | Tests |
|---|---|---|
| `lib` | ✅ passed | 9/9 |
| `experiment_1` | ✅ passed | — |
| `experiment_2` | ✅ passed | — |
| `experiment_3` | ✅ passed | — |
| `experiment_4` | ✅ passed | — |
| `experiment_5` | ✅ passed | — |
| `experiment_6` | ✅ passed | — |

| Mesure | Règle (baseline) | Précédent | Actuel | Écart | Statut |
|---|---|---|---|---|---|
| `exp1_alloc_free_sequence.iterations` | >= 10000 |  | 10000 |  | ✅ |
| `exp1_alloc_free_sequence.errors` | <= 0 |  | 0 |  | ✅ |
| `exp1_alloc_free_sequence.used_after` | <= 0 |  | 0 |  | ✅ |
| `exp1_alloc_free_sequence.exhaustions` | <= 0 |  | 0 |  | ✅ |
| `exp2_long_lived_vs_churn.iterations` | >= 10000 |  | 10000 |  | ✅ |
| `exp2_long_lived_vs_churn.errors` | <= 0 |  | 0 |  | ✅ |
| `exp2_long_lived_vs_churn.used_after` | <= 256 |  | 256 |  | ✅ |
| `exp3_exhaustion_and_recovery.iterations` | >= 3 |  | 3 |  | ✅ |
| `exp3_exhaustion_and_recovery.errors` | <= 0 |  | 0 |  | ✅ |
| `exp3_exhaustion_and_recovery.used_after` | <= 0 |  | 0 |  | ✅ |
| `exp4_page_cycle.iterations` | >= 500 |  | 500 |  | ✅ |
| `exp4_page_cycle.errors` | <= 0 |  | 0 |  | ✅ |
| `exp4_page_cycle.used_after` | <= 0 |  | 0 |  | ✅ |
| `exp5_under_interrupts.iterations` | >= 10200 |  | 10200 |  | ✅ |
| `exp5_under_interrupts.errors` | <= 0 |  | 0 |  | ✅ |
| `exp5_under_interrupts.used_after` | <= 0 |  | 0 |  | ✅ |
| `box_alloc_64B.errors` | <= 0 |  | 0 |  | ✅ |
| `exp6_phase_change.iterations` | >= 95 |  | 100 |  | ✅ |
| `exp6_phase_change.errors` | <= 0 |  | 0 |  | ✅ |
| `exp6_phase_change.used_after` | <= 0 |  | 0 |  | ✅ |

<details><summary>Cycles (indicatif, ±20 % sous QEMU, jamais bloquant)</summary>

| Mesure | Précédent | Actuel | Écart |
|---|---|---|---|
| `box_alloc_64B.cycles` |  | 178878 |  |
| `exp1_alloc_free_sequence.cycles` |  | 850434358 |  |
| `exp2_long_lived_vs_churn.cycles` |  | 156925294 |  |
| `exp3_exhaustion_and_recovery.cycles` |  | 228487674 |  |
| `exp4_page_cycle.cycles` |  | 152627424 |  |
| `exp5_under_interrupts.cycles` |  | 994786930 |  |
| `exp6_phase_change.cycles` |  | 6118218 |  |

</details>

Aucun run précédent disponible pour la comparaison.

**✅ Aucune régression par rapport à la baseline.**
