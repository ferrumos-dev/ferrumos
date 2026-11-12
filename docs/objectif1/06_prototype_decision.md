# 06 · Prototype de décision : liste chaînée (A) contre blocs de taille fixe (B)

> **Objectif obligatoire 1**, livrable « Decision prototype: code of both candidates, protocol, measurements side by side ».
> Mesures du **12 novembre 2026** (Elias Lebrun). Critères fixés avant la mesure dans `01_caracterisation_probleme.md`.

## 1. Les deux candidats

Retenus à l'issue de l'état de l'art (`05_etat_de_l_art.md`) parce qu'ils sont les deux familles les plus courantes dans les noyaux Rust et les plus simples à intégrer sans dépendance :

| | **A · liste chaînée** | **B · blocs de taille fixe** |
|---|---|---|
| Principe | Liste des zones libres triée par adresse, premier ajustement, fusion des voisins à la libération | Classes 16, 32, … 2048 o ; un bloc libéré va en tête de la liste de sa classe ; au-delà de 2 Kio, ou si la classe est vide, repli sur A |
| Origine | Crate `linked_list_allocator` (phil-opp), noyau Redox ; réécrite pour FerrumOS | Blog « Writing an OS in Rust », chapitre *Allocator Designs* |
| Coût attendu | O(nombre de trous) | O(1) pour les petites tailles |
| Code | [`libs/ferrum_alloc/src/linked_list.rs`](../../libs/ferrum_alloc/src/linked_list.rs) | [`libs/ferrum_alloc/src/fixed_block.rs`](../../libs/ferrum_alloc/src/fixed_block.rs) |

Variante **B'** : B plus une **récupération** (*reclaim*) : quand le repli échoue, toutes les listes de classes sont rendues à A, qui les fusionne, puis la demande est retentée. Elle a été ajoutée **après** une première mesure qui montrait que B, tel que décrit dans le blog, ne tenait pas le critère C4 (voir §4). B et B' sont mesurés tous les deux.

Le **bump** actuel est mesuré dans les mêmes conditions comme référence « avant » ([`bump.rs`](../../libs/ferrum_alloc/src/bump.rs), algorithme identique à celui d'octobre).

Les trois allocateurs vivent dans une seule bibliothèque `no_std` sans dépendance, **la même** qui est compilée dans le noyau (`09_integration.md`) : le code mesuré est le code intégré.

## 2. Protocole

### 2.1 Deux bancs, mêmes scénarios

| | Banc hôte | Banc noyau |
|---|---|---|
| Où | `research/heap-allocator/bench` | `POC/ferrum_Poc_adaptative_sheduller`, sous QEMU |
| Commande | `cargo run --release` | `bash scripts/kernel_tests/run.sh -- --no-default-features --features heap-<nom>` |
| Mesure | grandeurs déterministes + temps par opération (ns) | grandeurs déterministes + cycles (indicatifs, ±20 %) |
| Pourquoi | temps fiable, scénarios supplémentaires, 7 répétitions en 0,4 s | conditions réelles : vrai tas à `0x4444_4444_0000`, interruptions, verrou |

### 2.2 Conditions identiques pour tous les allocateurs

- Tas de **100 Kio**, début aligné sur 4 Kio (comme dans le noyau).
- **Tas neuf pour chaque scénario** (nouvelle région sur l'hôte, nouvelle image amorcée sous QEMU).
- Mêmes graines xorshift : `0x2026_1017` (exp1, comme en octobre), plus `1, 2, 3, 0xDEAD_BEEF, 0x1234_5678` pour la répétition de exp1 ; `0x2026_1112` pour `long_run`.
- Même machine : Intel Core i7-1355U, WSL2 (Linux 6.18), `rustc 1.99.1` stable pour l'hôte, `nightly-2026-01-15` + `bootimage 0.10.3` + QEMU 10.2.1 pour le noyau.
- 3 exécutions complètes du banc hôte (chaque temps y est déjà la médiane de 7 répétitions) et 3 exécutions de la suite QEMU par allocateur.

### 2.3 Scénarios

| Scénario | Description | Critère évalué |
|---|---|---|
| exp1 | 10 000 opérations, tailles 8–512 o, alignements 1–64 o, ≤ 32 blocs vivants ; libération globale si épuisement | C2 (épuisements), C3, C5 |
| exp1 × 6 graines | exp1 avec 6 graines différentes | C3 (limite B du compte rendu d'octobre) |
| exp2 | 4 blocs de 64 o vivants, boucle allouer/libérer 1 Kio, objectif 10 000 | **C1** |
| exp3 | remplir de blocs de 1 Kio jusqu'au refus, tout libérer, 3 tours | C4, C5 |
| changement de profil (exp6 dans le noyau) | remplir de blocs de 64 o, tout libérer, puis remplir de blocs de 1 Kio | C4 |
| `long_run` | 100 000 opérations, 7/8 de petites demandes (16–256 o) et 1/8 de grandes (512–2048 o), ≤ 48 blocs vivants, **sans libération globale** ; un refus est compté | C2, fragmentation |
| temps | chaque allocation et libération chronométrée seule (`Instant`), sans les vérifications ; p50, p99, max | C6 |

Les vérifications de correction sont celles du harnais de Rayan : alignement, absence de chevauchement avec les blocs vivants, motif relu avant libération.

## 3. Mesures côte à côte

### 3.1 Grandeurs déterministes, banc hôte (identiques aux 3 exécutions)

| Allocateur | exp1 erreurs | exp1 épuisements | 6 graines : erreurs / épuisements | **exp2 itérations** | exp2 `used` | exp3 capacité | profil : 64 o puis 1 Kio | `long_run` refus | fragmentation max |
|---|---|---|---|---|---|---|---|---|---|
| bump (avant) | 0 | **13** | 0 / 13 ×6 | **99** | 101 632 | 100/100/100 | 1600 puis 100 | **2 058** | 0 ‰ |
| **A** liste chaînée | 0 | **0** | 0 / 0 | **10 000** | 256 | 100/100/100 | 1600 puis **100** | **0** | 193 ‰ |
| B blocs fixes (blog) | 0 | 0 | 0 / 0 | 10 000 | 256 | 100/100/100 | 1600 puis **0** | 0 | 420 ‰ |
| **B'** blocs fixes + reclaim | 0 | **0** | 0 / 0 | **10 000** | 256 | 100/100/100 | 1600 puis **100** | **0** | 420 ‰ |

`used` d'exp2 = 256 o : exactement les 4 blocs de 64 o vivants (contre 101 632 o avec le bump, qui comptait l'avancée du pointeur).

### 3.2 Temps par opération, banc hôte (ns ; valeurs des 3 exécutions)

| Allocateur | exp1 p50 | exp1 p99 | `long_run` p99 | **1re allocation de 1 Kio après changement de profil** |
|---|---|---|---|---|
| bump (avant) | 15 / 15 / 15 | 20 / 23 / 17 | 23 / 22 / 22 | 22 / 32 / 21 |
| **A** liste chaînée | 32 / 33 / 32 | 52 / 54 / 51 | 52 / 51 / 51 | **77 / 22 / 28** |
| B blocs fixes (blog) | 20 / 20 / 19 | 29 / 30 / 28 | 34 / 31 / 29 | refusée |
| **B'** blocs fixes + reclaim | 20 / 20 / 20 | 28 / 28 / 29 | 30 / 29 / 29 | **243 266 / 342 751 / 242 589** |

Le chronomètre lui-même coûte 15 à 20 ns (c'est l'essentiel du chiffre du bump). Les maxima isolés (dizaines de µs, pour tous les allocateurs y compris le bump) sont des préemptions du système hôte ; ils sont dans les fichiers bruts mais ne sont pas interprétés.

### 3.3 Banc noyau sous QEMU (3 exécutions de la suite complète par allocateur)

| Allocateur | exp1 épuisements | **exp2 itérations** | exp2 `used` | exp3 | exp6 : blocs de 1 Kio | exp6 : cycles de la 1re allocation de 1 Kio (médiane) | exp2 cycles (médiane) | rapport baseline |
|---|---|---|---|---|---|---|---|---|
| bump | 13 | 99 | 101 632 | 100/100/100 | 100 | 129 664 | 1 162 170 (pour 99 itérations) | ❌ 3 règles violées |
| **A** liste chaînée | 0 | **10 000** | 256 | 100/100/100 | 100 | **149 682** | 113 270 456 | ✅ |
| **B'** blocs fixes + reclaim | 0 | **10 000** | 256 | 100/100/100 | 100 | **4 632 592** | 140 776 588 | ✅ |

Erreurs : **0** pour les trois allocateurs, aux 3 exécutions, sur les 7 images. Les grandeurs déterministes sont identiques d'une exécution à l'autre et identiques à celles du banc hôte.

Les cycles sous QEMU sans accélération varient de 15 à 20 % : seul l'écart d'exp6 (×31 entre A et B') dépasse largement ce bruit. Pour exp2, A est 20 % plus rapide que B' en médiane, ce qui est à la limite du bruit et **n'est pas retenu comme argument**.

## 4. Lecture des résultats

1. **Les deux candidats lèvent le verrou.** exp2 passe de 99 à 10 000 (C1), plus aucun épuisement artificiel (C2), aucune erreur sur 6 graines (C3), capacité intacte (C4), retour à vide (C5).
2. **B tel que décrit dans le blog échoue au changement de profil** : la mémoire libérée en blocs de 64 o reste dans la liste de la classe 64 et ne peut plus servir une demande de 1 Kio (0 bloc au lieu de 100). C'est cette mesure qui a motivé B'.
3. **B' est 1,7 fois plus rapide que A au p99** (≈ 29 ns contre ≈ 52 ns), mais les deux sont **20 fois sous le seuil** de C6 (1 000 ns) : le gain n'est pas nécessaire.
4. **B' a un pire cas pathologique** : la récupération rend les blocs un par un à une liste triée, soit O(n²). Mesuré : **240 à 340 µs** pour une seule allocation sur l'hôte, **4,6 millions de cycles** sous QEMU, verrou du tas tenu pendant tout ce temps.
5. **A détecte les doubles libérations** (compteur `rejected_frees`, test `double_free_is_rejected`) ; les listes de classes de B' ne le peuvent pas.
6. **A fragmente moins** sur `long_run` (193 ‰ contre 420 ‰). Pour B', la mémoire garée dans les classes compte comme libre mais ne sert qu'à sa classe.

La décision est rédigée dans `08_note_decision.md`.

## 5. Reproduire

```sh
# Banc hôte (Rust stable)
cd research/heap-allocator/bench
cargo run --release -- --output bench.json

# Tests unitaires de la bibliothèque
cd libs/ferrum_alloc && cargo test

# Banc noyau, un allocateur (environnement : TEST_ENVIRONMENT.md)
bash scripts/kernel_tests/run.sh -- --no-default-features --features heap-fixed-block
```

## 6. Fichiers bruts

| Contenu | Fichier |
|---|---|
| Banc hôte, 3 exécutions (tableau + JSON) | `results/host/bench_run{1,2,3}_2026-11-12.{md,json}` |
| Suite QEMU, 3 × 3 exécutions (journal série, résultats, rapport baseline) | `results/qemu/{bump,fixed-block,linked-list}_run{1,2,3}_*` |
| Reproduction du bump avec l'ancienne baseline (avant toute modification) | `results/qemu/bump_ancienne-baseline_*` |

Les exécutions du 12/11 ont été faites sur l'arbre de travail non commité, au-dessus du commit `c88bde4` (`results.json` indique ce commit).
