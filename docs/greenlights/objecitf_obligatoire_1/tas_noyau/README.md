# Objectif obligatoire 1 : veille et résolution du problème difficile

> Guide EIP Tech 4 2026-2027, objectif 1 : *Track what's out there and use it to crack your project's hard problem*.
>
> **Problème difficile retenu : le tas du noyau ne réutilise pas la mémoire libérée** (expérience 2 : 99 itérations sur 10 000 avec l'allocateur bump).
>
> État au **12 novembre 2026**. Dossier commité sur la branche `obj1/heap-allocator`, **pas encore relu ni fusionné** par l'équipe.
>
> Mots techniques : voir le [lexique](lexique.md).

## Résultat en une ligne

Allocateur à liste chaînée avec fusion, intégré au noyau : **exp2 passe de 99 à 10 000 itérations**, 0 épuisement, 0 erreur, aucune perte de capacité, p99 de 51–54 ns. Les critères C1 à C7 sont tous atteints (`09_integration.md`).

## Livrables (« What you submit »)

| # | Livrable demandé | Échéance du guide | Document | État |
|---|---|---|---|---|
| 1 | Note de caractérisation datée d'octobre, avec critères de réussite | fin octobre | [01_caracterisation_probleme.md](01_caracterisation_probleme.md) | ✅ rédigée ; problème mesuré le 17/10, note du 12/11 (**en retard**), **validation mentor à obtenir** |
| 2 | Description du dispositif de veille : sources, outils, automatisations, rythme, répartition | fin octobre | [02_dispositif_veille.md](02_dispositif_veille.md) + [`scripts/veille/`](../../../../scripts/veille/) + [`veille.yml`](../../../../.github/workflows/veille.yml) | ✅ outillage testé ; ⚠️ rythme et répartition **à valider en équipe** ; webhook Discord à configurer |
| 3 | Journal de veille complet | octobre → mai, ≥ 20 entrées | [03_journal_veille.md](03_journal_veille.md) | 🟡 11 / 20 ; à alimenter chaque semaine |
| 4 | Comptes rendus des présentations internes | dès novembre, 6 sur l'année | [04_presentations_internes.md](04_presentations_internes.md) | 🔴 0 / 6 ; rituel et planning définis, 1re présentation à faire en novembre ; 5 éléments réutilisés listés (3 exigés) |
| 5 | Note d'état de l'art, comparaison d'au moins 3 approches, sources citées | fin décembre | [05_etat_de_l_art.md](05_etat_de_l_art.md) | ✅ 7 approches, 4 comparées par la mesure |
| 6 | Code et protocole des 2 prototypes, avec les mesures | décision : fin janvier ; exploration : février–avril | [06_prototype_decision.md](06_prototype_decision.md), [07_prototype_exploration.md](07_prototype_exploration.md) ; code : [`libs/ferrum_alloc/`](../../../../libs/ferrum_alloc/), [`research/heap-allocator/bench/`](../../../../research/heap-allocator/bench/) | ✅ mesures sur l'hôte et sous QEMU, 3 exécutions chacune |
| 7 | Note de décision technique | fin février | [08_note_decision.md](08_note_decision.md) | ✅ proposée ; **à valider en équipe** |
| 8 | Preuve d'intégration : commit ou merge, démonstration, tests | fin avril | [09_integration.md](09_integration.md) | 🟡 intégré et testé (QEMU + unitaires) ; **manque : PR relue et fusionnée, run CI vert, capture de la démo** |
| 9 | Mesures avant / après sur les critères | fin avril | [09_integration.md §3](09_integration.md) | ✅ |
| 10 | Liens et captures des contributions aux communautés | novembre → mai | [10_communautes.md](10_communautes.md) | 🔴 0 / 5 ; 2 communautés choisies, 5 brouillons prêts à publier |

✅ fait · 🟡 partiel · 🔴 dépend d'actions de l'équipe qui n'ont pas encore eu lieu

## Ce qu'il reste à faire par l'équipe (rien de cela ne peut être produit sans vous)

1. **Faire valider** la note de caractérisation et ses critères par le mentor au prochain suivi, en assumant le retard.
2. **Valider en réunion** la répartition et le rythme de veille ; créer le webhook Discord (`DISCORD_WEBHOOK_VEILLE`).
3. **Tenir le journal** : environ 2 entrées par mois pour l'équipe, chacune avec une conclusion.
4. **Faire et enregistrer la 1re présentation** (support : notes 06 et 08) avant fin novembre, puis une par mois.
5. **Relire et fusionner la PR** (relecteur ≠ auteur), puis reporter le lien de PR, le hash de merge et le lien du run CI dans `09_integration.md`.
6. **Publier les contributions** préparées dans `10_communautes.md` depuis vos comptes et en archiver liens et captures.

## Retours de relecture du 15 novembre 2026

Chaque retour a été confronté au guide Tech 4 avant d’être appliqué. Simplifier le vocabulaire est possible partout ; retirer une mesure ou une preuve exigée par le guide ne l’est pas, car « an objective with no evidence counts as not validated ».

| Document | Retour | Suite donnée |
|---|---|---|
| Tous | Lexique des mots-clés techniques | ✅ [Lexique commun](lexique.md), lié depuis chaque document |
| 01 | Moins technique, à visée large, parler du projet et pas du code, garder les tableaux et les critères | ✅ Résumé « En bref », énoncé et contraintes reformulés au niveau du projet, critères expliqués « en clair ». Seuils et mesures inchangés |
| 01 | Prendre un problème plus large en exemple (« faire fonctionner les couches entre elles efficacement ») | ⏸️ **À décider en équipe.** Le guide demande **un** problème difficile précis, mesuré avec l’approche simple et doté de critères chiffrés, sur lequel portent ensuite état de l’art, prototypes et intégration. Un sujet aussi large ne se mesure pas. Une version plus large peut exister à côté, comme modèle, sans remplacer cette note |
| 02, 03, 05 | RAS | — (02 : rythme et répartition restent à valider en équipe) |
| 04 | À voir en groupe | ⏸️ Rituel attendu dès novembre : 1re présentation avant fin novembre |
| 06 | Explication trop technique | ✅ Résumé « En bref », comparaison en image des candidats, lecture de chaque tableau en une phrase, commandes et fichiers bruts en annexe. Protocole et mesures côte à côte conservés |
| 07 | Ajouter un second prototype d’exploration et comparer dans un tableau | ⏸️ **À décider en équipe.** Le guide demande un prototype d’exploration, déjà fourni (TLSF, comparé à A et B′ au §4). Un second est possible avant avril, à condition d’écrire et dater son hypothèse **avant** la mesure. Pistes : `talc`, buddy |
| 08 | Ajouter du lexique | ✅ Résumé « En bref » et lexique propre à la note |
| 09 | Vulgariser « Ce qui a été intégré » | ✅ Résumé en clair ; le tableau fichier par fichier, preuve de l’intégration, passe en annexe |

## Arborescence

```
docs/greenlights/objecitf_obligatoire_1/tas_noyau/
├── README.md                       ← ce fichier
├── 01 … 10 *.md                    ← les livrables
├── lexique.md                      ← les mots techniques expliqués simplement
├── experiences/                    ← comptes rendus des expériences 1 à 5 (mesures « avant »), captures et journaux
└── results/
    ├── host/                       ← banc hôte : 3 exécutions (.md + .json)
    └── qemu/                       ← suite QEMU : 3 exécutions × 3 allocateurs (journal, results.json, rapport baseline)
libs/ferrum_alloc/                  ← bibliothèque intégrée au noyau (bump, A, B/B')
research/heap-allocator/bench/      ← banc du prototype de décision + exploration TLSF
scripts/veille/                     ← veille automatique
```

[Retour à l’objectif obligatoire 1](../README.md) · [Index Greenlight](../../README.md)
