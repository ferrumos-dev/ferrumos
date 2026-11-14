# 07 · Prototype d'exploration : TLSF (Two-Level Segregated Fit)

> **Objectif obligatoire 1**, livrable « Exploration prototype: code, hypothesis stated beforehand, result including a negative one ».
> Pas de décision attachée : il s'agit de tester une piste que l'équipe ne maîtrise pas.
>
> Mots techniques : voir le [lexique](lexique.md).

## 1. Pourquoi cette piste

Repérée pendant l'état de l'art (`05_etat_de_l_art.md`, entrée du journal du 12/11/2026) : TLSF est l'allocateur des systèmes temps réel (Masmano et al., ECRTS 2004). Il promet allocation et libération en **temps constant**, là où nos deux candidats ont chacun un chemin en O(n) (parcours de liste pour A, vidange des listes pour B'). Personne dans l'équipe n'a implémenté ni utilisé un allocateur à deux niveaux de listes ségrégées et bitmaps.

Pour explorer sans tout réécrire, le prototype utilise la crate **`rlsf` 0.2.3** (yvt, MIT/Apache-2.0, publiée le 27/07/2026), branchée dans le même banc que les candidats.

## 2. Hypothèse, écrite avant la mesure (12 novembre 2026)

> **H1.** TLSF tient les mêmes critères de réutilisation que la liste chaînée : exp2 ≥ 10 000, exp1 sans épuisement, `long_run` sans refus, 0 erreur.
>
> **H2.** Son p99 par opération est **inférieur** à celui de la liste chaînée sur `long_run` (charge fragmentée), parce qu'il ne parcourt pas de liste.
>
> **H3.** Contrepartie attendue : un en-tête par bloc (*boundary tag*), donc une capacité exp3 **inférieure à 100** blocs de 1 Kio, que nous estimons entre 96 et 99.
>
> **Ce qui ferait abandonner la piste** : H3 sous 95 blocs (critère C4 manqué), ou un p99 qui n'est pas meilleur que celui de A ou B' (aucun gain sur le seul point où TLSF devrait l'emporter).

## 3. Protocole

Identique au prototype de décision (`06_prototype_decision.md`) : même tas de 100 Kio aligné sur 4 Kio, mêmes scénarios, mêmes graines, même machine, même compilation (`--release`), tas neuf pour chaque scénario.

Différence assumée : `rlsf` n'expose pas de statistiques (`used`, fragmentation). Ces colonnes valent « n/a » ; C5 et C7 ne sont donc pas évaluables pour TLSF.

Code : `research/heap-allocator/bench/src/heaps.rs` (type `Tlsf`), configuration `Tlsf<'static, u16, u16, 12, 16>`.

## 4. Résultat (mesuré le 12 novembre 2026, après l'écriture de l'hypothèse)

Fichiers bruts : `results/host/bench_run{1,2,3}_2026-11-12.{md,json}`, ligne « X tlsf ».

| Mesure | TLSF | A · liste chaînée | B' · blocs fixes + reclaim |
|---|---|---|---|
| exp2 itérations | 10 000 | 10 000 | 10 000 |
| exp1 épuisements, 6 graines | 0 | 0 | 0 |
| `long_run` refus | 0 | 0 | 0 |
| erreurs, tous scénarios | 0 | 0 | 0 |
| exp3 capacité (blocs de 1 Kio) | **96** / 96 / 96 | 100 | 100 |
| blocs de 64 o avant saturation (changement de profil) | **1 066** | 1 600 | 1 600 |
| p99 exp1 (ns, 3 exécutions) | 49 / 48 / 48 | 52 / 54 / 51 | 28 / 28 / 29 |
| p99 `long_run` (ns) | 49 / 47 / 48 | 52 / 51 / 51 | 30 / 29 / 29 |
| 1re allocation après changement de profil (ns) | 37 / 26 / 26 | 77 / 22 / 28 | 243 266 / 342 751 / 242 589 |

| Hypothèse | Verdict |
|---|---|
| **H1** mêmes critères de réutilisation que A | **Confirmée.** |
| **H2** p99 inférieur à celui de A sur `long_run` | **Non démontrée.** 47–49 ns contre 51–52 ns : 2 à 5 ns d'écart, du même ordre que la variation entre exécutions et que le coût du chronomètre. Sur une charge de 48 blocs vivants, la liste de A reste trop courte pour que le temps constant de TLSF se voie. |
| **H3** capacité exp3 entre 96 et 99 | **Confirmée, à la borne basse : 96.** L'en-tête par bloc coûte 4 % sur des blocs de 1 Kio, et **33 %** sur des blocs de 64 o (1 066 au lieu de 1 600), ce que nous n'avions pas anticipé. |

## 5. Conclusion : piste écartée pour l'instant

La condition d'abandon fixée à l'avance est remplie : aucun gain de p99 mesurable sur A, et un coût mémoire réel sur les petits blocs, qui sont la majorité des allocations d'un noyau. **TLSF n'est pas retenu.**

Ce que l'exploration apporte quand même :

- le seul allocateur testé dont le pire cas reste petit **partout**, y compris sur le changement de profil ; c'est l'argument à ressortir si FerrumOS ajoute un jour des contraintes temps réel ;
- la confirmation chiffrée que les en-têtes par bloc coûtent cher avec un tas de 100 Kio, ce qui a conforté le choix d'allocateurs sans en-tête (A et B stockent leurs métadonnées dans les blocs libres seulement) ;
- une prise en main de `rlsf` : 30 lignes d'adaptation, aucune difficulté de compilation `no_std`.

À rouvrir si : un tas beaucoup plus grand (la part de l'en-tête baisse), ou une contrainte de latence au pire cas.
