# 08 · Note de décision technique : allocateur du tas noyau

> **Objectif obligatoire 1**, livrable « Technical decision note ».
>
> | | |
> |---|---|
> | Date | 12 novembre 2026 |
> | Rédaction | Elias Lebrun |
> | Statut | **proposée**, à valider en réunion d'équipe et en revue de PR (relecteurs proposés : Rayan pour la mémoire, Mathias pour le noyau) |
> | Mesures | [`06_prototype_decision.md`](06_prototype_decision.md) |
>
> Mots techniques : le [lexique de la note](#lexique-de-la-note) ci-dessous, et le [lexique complet](lexique.md).

## En bref

FerrumOS gardera la solution A, une liste des zones de mémoire libres qui recolle les zones voisines quand on rend de la mémoire. Elle et sa concurrente B′ atteignent tous les objectifs chiffrés. A est retenue pour trois raisons : son cas le plus lent reste très court, elle repère une erreur de programmation courante au lieu de la laisser abîmer la mémoire, et son code est plus petit. Le prix accepté : chaque demande coûte un peu plus de temps qu’avant, et le code est à maintenir par l’équipe.

## Lexique de la note

| Terme | En clair |
| --- | --- |
| Tas | La réserve de mémoire du noyau, 100 Kio, où il prend de la place puis la rend. |
| Allocateur | Le code qui distribue cette mémoire et récupère ce qui est rendu. |
| Bump | L’allocateur d’origine : il n’avance que dans un sens et ne réutilise la mémoire que lorsque tout est rendu. |
| Liste chaînée triée par adresse, fusion (A) | La liste des zones libres, dans l’ordre de la mémoire. Quand une zone rendue touche une zone libre, les deux sont recollées. |
| Blocs de taille fixe, récupération (B, B′) | Des compartiments de tailles standard. B′ ajoute une récupération : quand la mémoire manque, les compartiments vides sont rendus et recollés. |
| TLSF, buddy, slab | Trois autres familles d’allocateurs étudiées dans l’état de l’art ([note 05](05_etat_de_l_art.md)). |
| Crate | Une bibliothèque Rust, ici une bibliothèque publique qu’on aurait pu utiliser à la place de notre code. |
| En-tête par bloc | Quelques octets placés devant chaque morceau de mémoire pour l’organiser. Ils prennent de la place. |
| Granularité de 16 o | La plus petite quantité distribuée : une demande de 8 octets en reçoit 16. |
| p99 | La durée sous laquelle se trouvent 99 % des demandes : elle décrit les demandes lentes, pas la moyenne. |
| Pire cas | La demande la plus lente observée. Pour un noyau, elle compte plus que la moyenne. |
| ns, µs | Milliardième et millionième de seconde. 243 µs, c’est environ 5 000 fois plus que 50 ns. |
| O(n), O(n²) | Le coût grandit comme le nombre d’éléments (n), ou comme son carré (n²) : deux fois plus d’éléments, quatre fois plus de temps. |
| Verrou tenu | Pendant une demande, le tas est verrouillé : tout autre traitement qui voudrait de la mémoire attend. Un pire cas long bloque donc tout le noyau. |
| Interruption | Un signal du matériel qui interrompt le noyau pour un court traitement. S’il attend le verrou, le système se fige. |
| Double libération | Rendre deux fois le même morceau de mémoire, une erreur de programmation. A la refuse ; B′ ne la voit pas. |
| Fragmentation, ‰ | La mémoire libre est éparpillée en petits morceaux. Mesurée en pour mille : 0 ‰ signifie un seul morceau libre. |
| `unsafe` | Code que le compilateur Rust ne peut pas vérifier seul ; chaque bloc est justifié par un commentaire. |
| Feature Cargo | Une option de compilation, qui permet de choisir l’allocateur du noyau. |
| Changement de contexte, contention | Passer réellement d’une tâche à une autre ; la contention, c’est quand plusieurs tâches attendent le même verrou. |
| C1 à C7, exp1 à exp6, `long_run` | Les critères de la [note 01](01_caracterisation_probleme.md) et les épreuves du [prototype de décision](06_prototype_decision.md). |

## Décision

**Le tas du noyau utilise l'allocateur à liste chaînée triée par adresse, avec fusion des voisins (candidat A)**, implémenté dans `libs/ferrum_alloc` et sélectionné par défaut dans le noyau (feature Cargo `heap-linked-list`).

## Sur quelles mesures elle repose

| Critère | Seuil | A, mesuré | B', mesuré |
|---|---|---|---|
| C1 réutilisation (exp2) | ≥ 10 000 | **10 000** | 10 000 |
| C2 épuisements exp1 / refus `long_run` | 0 / 0 | **0 / 0** | 0 / 0 |
| C3 erreurs (6 graines, toutes expériences) | 0 | **0** | 0 |
| C4 capacité exp3 / après changement de profil | ≥ 95 / ≥ 95 | **100 / 100** | 100 / 100 |
| C5 `used` après libération complète | 0 | **0** | 0 |
| C6 p99 par opération (hôte) | ≤ 1 000 ns | **51–54 ns** | 28–30 ns |
| Pire cas mesuré : 1re allocation après changement de profil | (non fixé à l'avance) | **22–77 ns** ; 150 k cycles QEMU | **243–343 µs** ; 4,6 M cycles QEMU |
| Double libération | (non fixé à l'avance) | détectée et refusée | non détectée |
| Fragmentation max `long_run` | (C7 : exposée) | 193 ‰ | 420 ‰ |

Les deux candidats atteignent tous les seuils fixés à l'avance. La décision se joue donc sur ce que les seuils ne départageaient pas, **mesuré lui aussi** :

1. Le gain de B' sur le p99 (environ 23 ns) ne change rien au regard du critère : A est déjà 20 fois sous le seuil.
2. Le pire cas de B' est de **quatre ordres de grandeur** au-dessus de son cas courant, et se produit **verrou tenu**. Dans un noyau où une interruption peut attendre ce verrou, c'est le défaut le plus grave des deux.
3. A refuse une double libération au lieu de corrompre le tas silencieusement.
4. A, c'est environ 210 lignes de code hors tests ; B', c'est A plus environ 150 lignes et un second état à garder cohérent.

## Approches écartées, et pourquoi

| Approche | Raison de l'écart |
|---|---|
| **Bump** (existant) | Échoue C1 (99/10 000) et C2 (13 épuisements, 2 058 refus sur `long_run`). Conservé derrière `heap-bump` pour rejouer le « avant ». |
| **B · blocs fixes, version du blog** | Échoue C4 : 0 bloc de 1 Kio après un changement de profil, au lieu de 100. |
| **B' · blocs fixes + récupération** | Passe tous les critères, mais pire cas de 243–343 µs (O(n²)) verrou tenu, pas de détection de double libération, environ 70 % de code en plus. Pourrait revenir si un futur profil de charge rend le p99 critique *et* si la récupération est rendue linéaire (tri fusion de la liste avant fusion). |
| **TLSF** (exploration, `07_prototype_exploration.md`) | Aucun gain mesurable sur A au p99 (48–49 ns contre 51–54 ns), et un en-tête par bloc qui coûte 4 % de capacité sur des blocs de 1 Kio et 33 % sur des blocs de 64 o. |
| **Buddy, slab** (état de l'art seulement) | Non prototypés. Buddy : arrondi aux puissances de deux (une demande de 520 o en consomme 1 024). Slab : suppose des caches d'objets typés que le noyau n'a pas encore. Voir `05_etat_de_l_art.md`. |
| **Utiliser une crate** (`linked_list_allocator`, `talc`) | Ajoute une dépendance à un noyau qui épingle déjà ses versions (`test_environment.md`). Notre implémentation est petite, testée, et porte les statistiques dont nous avons besoin (C7). À réévaluer si elle devient un poids : `talc` est la piste suivie en veille. |

## Conséquences acceptées

| Coût | Ce qu'il représente | Ce qu'on fait |
|---|---|---|
| **Complexité O(nombre de trous)** à chaque allocation et libération | Sous une forte fragmentation, le coût croît avec le nombre de trous ; sur `long_run`, il reste à 51–54 ns au p99 | Le banc `long_run` est dans la CI : une dérive sera visible |
| **Granularité de 16 o** | Une demande de 8 o en consomme 16 | Accepté : pas de demande inférieure à 16 o dans le noyau aujourd'hui |
| **Code maison à maintenir** (≈ 210 lignes hors tests, 10 éléments `unsafe` : 6 blocs, 3 fonctions, 1 `impl Send`, chacun justifié par un commentaire `SAFETY` que Clippy exige) | Pas de communauté pour corriger nos bogues | 9 tests unitaires, 7 images QEMU, revue des blocs `unsafe` demandée à Benjamin (pôle 1, tâche 1.7) |
| **Fragmentation externe possible** (193 ‰ au pire mesuré) | Une demande peut échouer alors que la somme des trous suffirait | La métrique est désormais exposée (`largest_free_block`, `fragmentation_permille`) |
| **Verrou sans masquage des interruptions** (inchangé) | Un gestionnaire d'interruption qui allouerait pendant une allocation bloquerait le noyau | Limite documentée dans `09_integration.md` ; aucun gestionnaire n'alloue aujourd'hui |

## Révision

Cette décision est à revoir si l'une de ces mesures change : le p99 de `long_run` dépasse 500 ns, un nouveau scénario fait monter la fragmentation au-dessus de 500 ‰, ou le noyau gagne un vrai changement de contexte (expérience 5 multi-tâches) qui révèle de la contention.
