# 09 · Intégration et mesures avant / après

> **Objectif obligatoire 1**, livrables « Proof of integration: commit or merge, demonstration, tests » et « Before and after measurements on the success criteria ».
> Intégration réalisée le 12 novembre 2026 (Elias Lebrun). Commitée dans `18fd351` sur la branche `obj1/heap-allocator`, **pas encore fusionnée** : la preuve « commit / merge » sera le lien de la PR une fois relue (voir §6).

## 1. Ce qui a été intégré

| Fichier | Changement |
|---|---|
| `libs/ferrum_alloc/` | **nouveau** : bibliothèque `no_std` sans dépendance : bump, liste chaînée (A), blocs fixes (B/B'), statistiques communes, 9 tests unitaires |
| `POC/ferrum_Poc_adaptative_sheduller/Cargo.toml` | dépendance `ferrum_alloc` ; features `heap-linked-list` (**défaut**), `heap-fixed-block`, `heap-bump` ; image `experiment_6` |
| `POC/.../src/allocator.rs` | l'allocateur global devient `Locked<KernelHeap>` ; l'API publique (`stats`, `try_alloc`, `free`, `HEAP_START`, `HEAP_SIZE`) est inchangée, `HeapStats` gagne `live_bytes`, `largest_free_block`, `rejected_frees`, `fragmentation_permille()` |
| `POC/.../src/experiments.rs` | exp1 publie `exhaustions=` dans son RECORD ; **nouvelle expérience 6** (changement de profil) |
| `POC/.../src/testing.rs` | `Record::report_with` pour des champs supplémentaires |
| `POC/.../src/lib.rs` | chaque image affiche l'allocateur compilé (`heap allocator: linked_list`) |
| `POC/.../tests/experiment_6.rs` | image dédiée à l'expérience 6 |
| `POC/.../tests/baseline.json` | **seuils relevés aux critères de réussite** (voir §3) |
| `scripts/kernel_tests/test_results.py` | les exécutions de référence deviennent celles de la liste chaînée ; nouveau test : les journaux bump d'octobre sont désormais **rejetés** par la baseline (12/12 tests ✅) |
| `.github/workflows/kernel-tests.yml` | déclenché aussi par `libs/ferrum_alloc/**` et `research/heap-allocator/**` ; nouveau job qui lance les tests unitaires et le banc |
| `POC/.../test_environment.md` | allocateur et version de QEMU mis à jour |

## 2. Ce que l'intégration remplace et rend possible

- **Remplace** l'allocateur à pointeur, qui ne rendait la mémoire que lorsque tout était libéré.
- **Rend possible** un noyau qui tourne longtemps : la quantité totale allouée n'est plus plafonnée par la taille du tas. Avec le bump, 100 Kio alloués au total (même libérés) suffisaient à arrêter le noyau.
- **Rend observable** la mémoire réellement vivante (`live_bytes`) et la fragmentation (limites 4 et 5 du compte rendu d'octobre, critère C7). Le shell et le moniteur de Rayan peuvent les afficher.
- **Garde le « avant » rejouable** : `--no-default-features --features heap-bump` reconstruit exactement le noyau d'octobre.

## 3. Mesures avant / après sur les critères d'octobre

Avant = bump (octobre, reproduit le 12/11). Après = liste chaînée intégrée, 3 exécutions de la suite QEMU complète, valeurs identiques aux 3 exécutions.

| # | Critère | Seuil | **Avant** | **Après** | Atteint ? |
|---|---|---|---|---|---|
| C1 | exp2, itérations tenues | ≥ 10 000 | **99** | **10 000** | ✅ (×101) |
| C2 | exp1, épuisements | 0 | 13 | **0** | ✅ |
| C2 | `long_run`, refus sur 100 000 opérations (banc hôte) | 0 | 2 058 | **0** | ✅ |
| C3 | erreurs, toutes expériences, exp1 sur 6 graines | 0 | 0 | **0** | ✅ |
| C4 | exp3, capacité en blocs de 1 Kio | ≥ 95 | 100 | **100** | ✅ (aucune perte) |
| C4 | exp6, blocs de 1 Kio après des blocs de 64 o | ≥ 95 | 100 | **100** | ✅ |
| C5 | `used` après libération complète (exp1, exp3, exp6) | 0 | 0 | **0** | ✅ |
| C6 | p99 par opération sur l'hôte, exp1 / `long_run` | ≤ 1 000 ns | 17–23 / 22–23 ns | **51–54 / 51–52 ns** | ✅ (20× sous le seuil) |
| C7 | mémoire vivante et fragmentation exposées | oui | non | **oui** | ✅ |

Mesure « mémoire utilisée » d'exp2 : **101 632 o avant, 256 o après**, soit exactement les 4 blocs de 64 o réellement vivants.

**Tous les critères sont atteints.** Le prix est visible en C6 : une opération coûte environ 2,5 fois plus qu'avec le bump (qui ne faisait qu'avancer un pointeur), tout en restant 20 fois sous le seuil.

Sous QEMU, à titre indicatif (cycles, ±20 %) : exp1 passe de 666 M à 780 M cycles en médiane (+17 %, dans le bruit). La boucle d'exp2 coûte environ 11 300 cycles par itération après (écriture d'1 Kio comprise), contre 11 700 pour les 99 itérations du bump.

## 4. Preuve que ça marche : tests

| Niveau | Quoi | Résultat du 12/11 |
|---|---|---|
| Tests unitaires (hôte) | 9 tests dans `libs/ferrum_alloc` : réutilisation, alignement sans perte, épuisement, double libération, capacité, changement de profil, repli | 9/9 ✅ (`cargo test`) |
| Tests noyau (QEMU) | 9 tests de capacité + 6 images d'expériences | 7/7 images ✅, 3 exécutions |
| Garde-fou de non-régression | `baseline.json` relevé aux critères | Rejouer le bump donne **❌ 3 règles violées** (`exp1.exhaustions`, `exp2.iterations`, `exp2.used_after`) : la CI bloquerait un retour en arrière. Voir `results/qemu/bump_run1_report.md`. |
| Banc de décision en CI | job `heap-allocator` de `kernel-tests.yml` | à vérifier au premier passage en CI |

## 5. Démonstration

À faire en suivi ou à la soutenance (2 minutes) :

```sh
cd POC/ferrum_Poc_adaptative_sheduller
# Avant : exp2 s'arrête à 99
cargo test --no-default-features --features heap-bump --test experiment_2
# Après : exp2 atteint 10 000
cargo test --test experiment_2
```

Sortie attendue, avant puis après :

```
heap allocator: bump
	churn of 1024 B survived 99 iterations out of 10000 (102400 B heap, 4 long lived blocks)
	freed memory is reusable: no
heap allocator: linked_list
	churn of 1024 B survived 10000 iterations out of 10000 (102400 B heap, 4 long lived blocks)
	freed memory is reusable: yes
```

(Extraits de `results/qemu/bump_run1_2026-11-12.log` et `results/qemu/linked-list_run1_2026-11-12.log`.)

Une capture d'écran de la démonstration reste à ajouter dans `results/` le jour où elle est faite.

## 6. Reste à faire pour que la preuve soit complète

- [ ] Relecture de la PR par un autre membre (Rayan ou Mathias), puis merge sur `main` → **ajouter ici le lien de la PR et le hash du commit de merge**.
- [ ] Premier passage vert de `kernel-tests.yml` en CI → **ajouter le lien du run**.
- [ ] Mettre à jour la note d'architecture (T04, contrat `MemoryReporter`) : le relevé mémoire peut maintenant venir de `allocator::stats()` (`live_bytes`, `free`).

## 7. Limites reconnues

- **Pas de masquage des interruptions** pendant que le verrou du tas est tenu : si un gestionnaire d'interruption allouait, le noyau se bloquerait. Aucun ne le fait aujourd'hui ; à corriger (`without_interrupts`) avant d'allouer dans un gestionnaire.
- **Un seul cœur, pas de vrai multi-tâches** : la concurrence réelle n'est toujours pas testée (expérience 5, limite A du compte rendu d'octobre).
- **Coût linéaire** en nombre de trous : la solution ne tiendrait pas une charge de milliers de petits blocs vivants très fragmentés ; le scénario le plus dur mesuré compte 48 blocs vivants.
- **Tas fixe de 100 Kio** : la réutilisation est résolue, la taille ne l'est pas. Une demande unique de plus de 100 Kio échoue toujours.
- **Échec d'allocation toujours fatal sur le chemin standard** (`Box`, `Vec`) : l'allocateur renvoie un pointeur nul, Rust appelle le gestionnaire d'erreur d'allocation, le noyau panique. Il arrive simplement beaucoup plus tard.
- **Les mesures noyau ont été faites avec QEMU 10.2.1**, pas 11.1.0 comme en octobre : les grandeurs déterministes sont identiques, les cycles ne sont pas comparables entre les deux.
