## Tests noyau QEMU — `40e1cc7`

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
| `box_alloc_64B.cycles` | 183089 | 172432 | -10657 (-6%) |
| `exp1_alloc_free_sequence.cycles` | 830573986 | 672028656 | -158545330 (-19%) |
| `exp2_long_lived_vs_churn.cycles` | 140611380 | 119295150 | -21316230 (-15%) |
| `exp3_exhaustion_and_recovery.cycles` | 258594879 | 212905004 | -45689875 (-18%) |
| `exp4_page_cycle.cycles` | 186805420 | 155070058 | -31735362 (-17%) |
| `exp5_under_interrupts.cycles` | 1076733644 | 885733758 | -190999886 (-18%) |
| `exp6_phase_change.cycles` | 129458 | 112164 | -17294 (-13%) |

</details>

Comparé au run précédent `187d5ac` (2026-11-16T20:48:08+00:00).

**✅ Aucune régression par rapport à la baseline.**
