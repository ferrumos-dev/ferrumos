# 01 · Caractérisation du problème difficile : la réutilisation du tas noyau

> **Objectif obligatoire 1** du guide Tech 4, livrable « Characterization note on the hard problem, dated October, with the success criteria ».
>
> | | |
> |---|---|
> | Problème identifié et mesuré | **17 octobre 2026** (Rayan Benhabiles, `docs/greenlights/objecitf_obligatoire_1/tas_noyau/experiences/compte_rendu_experiences_1_2.md`) |
> | Mesures complétées | 19 octobre 2026 (expériences 3 à 5) |
> | Note formalisée | **12 novembre 2026** (Elias Lebrun), avant toute mesure des solutions candidates |
> | Validation mentor | **à obtenir** au prochain suivi (le guide l'attend avant fin octobre : retard à assumer devant le mentor) |

---

## 1. Énoncé du problème technique

Le noyau FerrumOS doit pouvoir **allouer et libérer de la mémoire dynamique en continu** (tampon du shell, structures de l'ordonnanceur, files des pilotes, `Box`, `Vec`, `String`), **sans que la quantité totale allouée au cours de sa vie soit plafonnée par la taille du tas**.

Aujourd'hui, il ne le peut pas : la mémoire libérée n'est **jamais réutilisée** tant qu'au moins une allocation reste vivante. Comme un noyau garde toujours quelque chose en vie, en pratique elle n'est jamais réutilisée.

## 2. Ce qui le rend difficile

| Contrainte | Conséquence sur la solution |
|---|---|
| **Ressources** : tas fixe de 100 Kio, aucun allocateur sous-jacent (`no_std`, pas de `malloc` hôte) | L'allocateur doit stocker ses propres métadonnées **dans** la mémoire qu'il gère, sans en gaspiller |
| **Fiabilité** : une corruption du tas corrompt tout le noyau (domaine de panne unique du noyau monolithique, cf. note de décision d'architecture) | Correction prouvée par des tests sous QEMU avant toute optimisation |
| **Temps** : l'allocation peut être appelée depuis du code sensible à la latence (chemins d'interruption à terme) | Coût par opération borné et mesuré, pas seulement moyen |
| **Compatibilité** : l'interface imposée est `GlobalAlloc` de Rust (`alloc(Layout)`, `dealloc(ptr, Layout)`), avec des alignements arbitraires | Gérer l'alignement sans perdre d'octets, et sans en-tête obligatoire |
| **Concurrence** : un verrou tournant protège le tas ; les interruptions peuvent survenir pendant une allocation | Pas de blocage, conserver la protection actuelle |
| **Mesure** : les cycles mesurés sous QEMU sans accélération varient de 15 à 20 % entre deux exécutions identiques | Juger d'abord sur les grandeurs déterministes (itérations, erreurs, octets), le temps sur l'hôte |

## 3. Ce que donne l'approche naïve, mesuré

L'approche naïve est en place : un **allocateur à pointeur (bump)**. Le pointeur avance à chaque allocation et ne revient au début que si **toutes** les allocations ont été libérées.

Mesures du 17 et du 19 octobre 2026 (3 exécutions identiques, QEMU 11.1.0, `nightly-2026-01-15`), **reproduites à l'identique le 12 novembre 2026** (QEMU 10.2.1, voir `results/qemu/`) :

| Expérience | Résultat bump | Ce que ça montre |
|---|---|---|
| **exp2** : 4 blocs de 64 o gardés vivants, puis allouer/libérer 1 Kio en boucle, objectif 10 000 | **99 itérations** (1 % de l'objectif), puis refus | 0 octet libéré n'est réutilisé ; 99 × 1 Kio ≈ la taille du tas |
| exp2 : mémoire « utilisée » en fin de boucle | **101 632 o** pour **256 o** réellement vivants | Le compteur `used` mesure l'avancée du pointeur, pas la mémoire vivante |
| **exp1** : 10 000 opérations aléatoires, ≤ 32 blocs vivants (≤ 16 Kio) | 0 erreur, mais **13 épuisements** du tas | Le test ne survit qu'en libérant tout artificiellement 13 fois ; un vrai noyau ne peut pas |
| exp3 : remplir de blocs de 1 Kio, tout libérer, 3 tours | 100 / 100 / 100 blocs | Seule qualité du bump : aucun octet de métadonnée |

**Pourquoi ce n'est pas suffisant** : avec 100 Kio de tas, un noyau qui alloue 1 Kio par seconde s'arrête en moins de deux minutes, même s'il ne garde jamais plus de 2 Kio vivants. Et toute allocation standard (`Box`, `Vec`) faite après l'épuisement **panique le noyau**.

## 4. Critères de réussite chiffrés

Fixés le 12 novembre 2026, **avant** toute mesure des solutions candidates. Ils s'appliquent à l'allocateur intégré au noyau, mesuré par les expériences existantes sous QEMU (critères C1 à C5) et par le banc hôte (C6, C7).

| # | Critère | Seuil de réussite | Avant (bump) | Source de la mesure |
|---|---|---|---|---|
| **C1** | La mémoire libérée est réutilisable | **exp2 ≥ 10 000 itérations** | 99 | `experiment_2` sous QEMU |
| **C2** | Plus de remise à zéro artificielle | **exp1 : 0 épuisement** ; scénario `long_run` (100 000 opérations sans libération globale) : **0 refus** | 13 ; à mesurer | `experiment_1` sous QEMU ; banc hôte |
| **C3** | Correction | **0 erreur** (chevauchement, alignement, contenu) sur toutes les expériences, exp1 rejouée sur **6 graines** | 0 sur 1 graine | QEMU + banc hôte |
| **C4** | Capacité utile conservée | exp3 **≥ 95 blocs de 1 Kio** par tour (perte de métadonnées ≤ 5 %) ; après un changement de profil (64 o → 1 Kio), **≥ 95 blocs de 1 Kio** | 100 ; à mesurer | `experiment_3` + banc hôte |
| **C5** | Retour à vide | `used_after = 0` après libération complète (exp1, exp3) | 0 | QEMU |
| **C6** | Coût par opération borné | sur l'hôte (release), **p99 ≤ 1 000 ns** par allocation ou libération, sur la charge exp1 et sur `long_run` | à mesurer | banc hôte |
| **C7** | Mémoire observable | les statistiques exposent la **mémoire réellement vivante** et la **fragmentation** | non (limites 4 et 5 du compte rendu) | `allocator::stats()` |

**Le problème est considéré comme résolu si C1 à C5 sont atteints sous QEMU et C6, C7 sur le banc.** Un critère manqué sera publié avec l'écart, pas reformulé.

## 5. Hors périmètre (assumé)

- Agrandir le tas à chaud (limite 6 du compte rendu) : le seuil reste 100 Kio.
- Les cadres de tables de pages jamais rendus (limite 7) : problème de la couche pages, pas du tas.
- La concurrence multi-tâches réelle : impossible tant que l'ordonnanceur n'a pas de changement de contexte (expérience 5).

## Sources

- `docs/greenlights/objecitf_obligatoire_1/tas_noyau/experiences/compte_rendu_experiences_1_2.md` et `..._3_4_ET_5.md` (Rayan Benhabiles, 17 et 19 octobre 2026)
- `docs/greenlights/objecitf_obligatoire_1/tas_noyau/experiences/logs/` (sorties brutes), `POC/ferrum_Poc_adaptative_sheduller/test_environment.md`
- Reproduction du 12 novembre 2026 : `docs/greenlights/objecitf_obligatoire_1/tas_noyau/results/qemu/bump_2026-11-12.log`
