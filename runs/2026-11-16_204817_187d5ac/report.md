## Tests noyau QEMU — `187d5ac`

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
| `exp1_alloc_free_sequence.iterations` | >= 10000 | 10000 | 10000 | = | ✅ |
| `exp1_alloc_free_sequence.errors` | <= 0 | 0 | 0 | = | ✅ |
| `exp1_alloc_free_sequence.used_after` | <= 0 | 0 | 0 | = | ✅ |
| `exp1_alloc_free_sequence.exhaustions` | <= 0 | 0 | 0 | = | ✅ |
| `exp2_long_lived_vs_churn.iterations` | >= 10000 | 10000 | 10000 | = | ✅ |
| `exp2_long_lived_vs_churn.errors` | <= 0 | 0 | 0 | = | ✅ |
| `exp2_long_lived_vs_churn.used_after` | <= 256 | 256 | 256 | = | ✅ |
| `exp3_exhaustion_and_recovery.iterations` | >= 3 | 3 | 3 | = | ✅ |
| `exp3_exhaustion_and_recovery.errors` | <= 0 | 0 | 0 | = | ✅ |
| `exp3_exhaustion_and_recovery.used_after` | <= 0 | 0 | 0 | = | ✅ |
| `exp4_page_cycle.iterations` | >= 500 | 500 | 500 | = | ✅ |
| `exp4_page_cycle.errors` | <= 0 | 0 | 0 | = | ✅ |
| `exp4_page_cycle.used_after` | <= 0 | 0 | 0 | = | ✅ |
| `exp5_under_interrupts.iterations` | >= 10200 | 10200 | 10200 | = | ✅ |
| `exp5_under_interrupts.errors` | <= 0 | 0 | 0 | = | ✅ |
| `exp5_under_interrupts.used_after` | <= 0 | 0 | 0 | = | ✅ |
| `box_alloc_64B.errors` | <= 0 | 0 | 0 | = | ✅ |
| `exp6_phase_change.iterations` | >= 95 | 100 | 100 | = | ✅ |
| `exp6_phase_change.errors` | <= 0 | 0 | 0 | = | ✅ |
| `exp6_phase_change.used_after` | <= 0 | 0 | 0 | = | ✅ |

<details><summary>Cycles (indicatif, ±20 % sous QEMU, jamais bloquant)</summary>

| Mesure | Précédent | Actuel | Écart |
|---|---|---|---|
| `box_alloc_64B.cycles` | 162370 | 183089 | +20719 (+13%) |
| `exp1_alloc_free_sequence.cycles` | 610633750 | 830573986 | +219940236 (+36%) |
| `exp2_long_lived_vs_churn.cycles` | 100438034 | 140611380 | +40173346 (+40%) |
| `exp3_exhaustion_and_recovery.cycles` | 195055330 | 258594879 | +63539549 (+33%) |
| `exp4_page_cycle.cycles` | 134067856 | 186805420 | +52737564 (+39%) |
| `exp5_under_interrupts.cycles` | 834343430 | 1076733644 | +242390214 (+29%) |
| `exp6_phase_change.cycles` | 109914 | 129458 | +19544 (+18%) |

</details>

Comparé au run précédent `901ced4` (2026-11-16T20:20:48+00:00).

**✅ Aucune régression par rapport à la baseline.**
