# 05 · État de l'art : allocateurs de tas pour un noyau `no_std`

> **Objectif obligatoire 1**, livrable « State-of-the-art note with the comparison of approaches and sources cited ».
> Rédigé le 12 novembre 2026 (Elias Lebrun). Données des crates relevées sur l'API crates.io le **12/11/2026**.

## 1. Le problème posé aux approches

Rappel (`01_caracterisation_probleme.md`) : tas fixe de 100 Kio, interface `GlobalAlloc`, aucun allocateur sous-jacent, alignements arbitraires jusqu'à 64 o (et plus pour les pages), majorité de petites demandes. Il faut **réutiliser la mémoire libérée** sans perdre de capacité ni rendre le coût imprévisible.

## 2. Les approches existantes

| Approche | Principe | Où on la trouve |
|---|---|---|
| **Bump** (pointeur) | Le pointeur avance ; rien n'est rendu avant libération totale | FerrumOS aujourd'hui ; phil-opp, *Allocator Designs* |
| **Liste chaînée de trous** (*free list*), premier ajustement | Les zones libres forment une liste stockée dans la mémoire libre elle-même ; fusion des voisins | crate `linked_list_allocator` (phil-opp, rust-osdev) ; noyau Redox (dépendance `linked_list_allocator`, épinglée en 0.6.1 dans une MR) |
| **Blocs de taille fixe / listes ségrégées** | Une liste par classe de taille, O(1) ; repli pour les grandes tailles | phil-opp, *Allocator Designs* ; MR Redox « Use slab allocator for kernel heap » (blocs fixes jusqu'à 4 096 o, repli sur la liste chaînée au-delà) |
| **Slab** | Caches d'objets de même type, pré-initialisés, par cœur | Bonwick 1994 (Solaris) ; Linux SLUB ; Asterinas (`osdk-heap-allocator`, caches par CPU) ; crate `slabmalloc` |
| **Buddy** (compagnons) | Blocs de taille 2^k, découpe et fusion par paires | Knowlton 1965 ; Linux (allocateur de pages) ; crate `buddy_system_allocator` (rCore) |
| **TLSF** | Deux niveaux de listes ségrégées + bitmaps, O(1) garanti, en-têtes de frontière | Masmano et al. 2004 (temps réel) ; crate `rlsf` |
| **Talc** | Listes ségrégées par plages, en-têtes de frontière, conçu pour les noyaux et le wasm | crate `talc` ; noyau Hermit (allocateur global du noyau) |

## 3. Comparaison sur nos critères

Critères tirés de la caractérisation. « Maturité » et « maintenance » d'après crates.io au 12/11/2026 ; les complexités d'après les sources ; **la ligne « mesuré » ne concerne que ce que nous avons exécuté nous-mêmes** (`06_prototype_decision.md`, `07_prototype_exploration.md`).

| Critère | Bump | Liste chaînée | Blocs fixes | Slab | Buddy | TLSF | Talc |
|---|---|---|---|---|---|---|---|
| Réutilise la mémoire libérée (C1) | **non** | oui | oui, par classe | oui, par type | oui | oui | oui |
| Coût par opération | O(1) | O(trous) | O(1) | O(1) | O(log taille) | O(1) garanti | O(1) en pratique |
| Perte mémoire typique | aucune (rien n'est rendu) | arrondi 16 o, pas d'en-tête | arrondi à la classe (≤ ×2) | arrondi à l'objet, pages par cache | arrondi à 2^k (≤ ×2) | en-tête par bloc | en-tête par bloc |
| Fragmentation externe | n/a | oui, limitée par la fusion | mémoire bloquée dans les classes | faible par cache | faible (fusion par paires) | faible | faible |
| Complexité d'intégration | triviale | faible (~200 lignes ou 1 crate) | faible + repli | élevée (typage des objets, caches) | moyenne | crate ou ~500 lignes | crate |
| Dépendances ajoutées | 0 | 0 (maison) ou 1 crate | 0 (maison) | 1 crate | 1 crate | 1 crate | 1 crate |
| Licence (crate) | — | Apache-2.0/MIT | — | MIT (`slabmalloc`) | MIT | MIT/Apache-2.0 | MIT |
| Maturité (crate) | — | `linked_list_allocator` 0.10.6 (14/04/2026), 7,2 M téléchargements, depuis 2016 | — | `slabmalloc` 0.11.0, **dernière version 24/08/2022** | `buddy_system_allocator` 0.13.0 (30/03/2026), 3,9 M | `rlsf` 0.2.3 (27/07/2026), 2,4 M | `talc` 5.1.1 (09/09/2026), 2,6 M |
| **Mesuré par nous** | exp2 = 99 | exp2 = 10 000, p99 51–54 ns | B : échoue au changement de profil ; B' : p99 28–30 ns, pire cas 243–343 µs | non | non | exp3 = 96, p99 47–49 ns | non |

## 4. Hypothèse de travail (avant les prototypes)

Écrite le 12/11/2026 avant les mesures : **la liste chaînée avec fusion est la meilleure approche à ce stade**, parce qu'elle est la plus simple qui réutilise toute la mémoire sans en-tête ni arrondi important ; les blocs fixes sont le concurrent sérieux, plus rapides mais exposés à la mémoire « bloquée » dans les classes.

Pourquoi slab et buddy ne sont pas prototypés :

- **Slab** suppose des types d'objets connus et nombreux (tâches, inodes, tampons réseau). Le noyau n'en a pas encore ; ce serait optimiser un profil de charge qui n'existe pas. `slabmalloc` n'a plus de version depuis 2022.
- **Buddy** gaspille jusqu'à la moitié d'un bloc sur des tailles non puissances de deux. Avec 100 Kio, c'est trop cher. Il reste pertinent pour les **cadres physiques**, et c'est ce que fait Asterinas (`osdk-frame-allocator`).

Les deux candidats prototypés sont donc **A, la liste chaînée**, et **B, les blocs fixes** ; TLSF est exploré à part (`07_prototype_exploration.md`). La suite a confirmé l'hypothèse, pour des raisons en partie imprévues (pire cas de B', voir `08_note_decision.md`).

## 5. Sources (consultées le 12/11/2026 sauf mention contraire)

**Publications**

- P. R. Wilson, M. S. Johnstone, M. Neely, D. Boles, *Dynamic Storage Allocation: A Survey and Critical Review*, IWMM 1995 (taxonomie des allocateurs, fragmentation).
- J. Bonwick, *The Slab Allocator: An Object-Caching Kernel Memory Allocator*, USENIX Summer 1994.
- M. Masmano, I. Ripoll, A. Crespo, J. Real, *TLSF: a New Dynamic Memory Allocator for Real-Time Systems*, ECRTS 2004.
- K. C. Knowlton, *A Fast Storage Allocator*, Communications of the ACM 8(10), 1965 (buddy).

**Documentation et code**

- P. Oppermann, *Writing an OS in Rust : Allocator Designs* : https://os.phil-opp.com/allocator-designs/ (bump, liste chaînée, blocs fixes ; base de B).
- Redox OS, MR « Use slab allocator for kernel heap » : https://gitlab.redox-os.org/redox-os/kernel/-/merge_requests/76 ; commit « Use older linked_list_allocator version » (MR 92).
- Asterinas OSTD, module tas : https://docs.rs/crate/ostd/0.18.0/source/src/mm/heap/mod.rs ; `osdk-heap-allocator` : https://docs.rs/crate/osdk-heap-allocator/0.17.2/source/README.md.
- Hermit, allocateur du noyau basé sur Talc : https://docs.rs/hermit-kernel/0.11.0/src/hermit/mm/mod.rs.html.
- crates.io API, fiches `linked_list_allocator`, `talc`, `rlsf`, `buddy_system_allocator`, `slabmalloc` (versions, dates, téléchargements, licences relevés le 12/11/2026).
- Code source `rlsf` 0.2.3 (lu pour l'adaptateur du prototype d'exploration : `Tlsf::insert_free_block_ptr`, `allocate`, `deallocate`).

**Sources internes**

- `docs/greenlights/objecitf_obligatoire_1/tas_noyau/experiences/compte_rendu_experiences_1_2.md` et `..._3_4_ET_5.md` (Rayan, octobre 2026) : protocole et limites qui ont servi de grille.
- `docs/greenlights/Objectif obligatoire 2/` (Mathias, novembre 2026) : comparaison Redox / Asterinas / Theseus / Tock / Hubris côté architecture.

> Limite de cette note : les publications sont citées d'après leur contenu connu de l'équipe, pas relues intégralement pour ce document. Seules les lignes « mesuré » viennent de nos exécutions.
