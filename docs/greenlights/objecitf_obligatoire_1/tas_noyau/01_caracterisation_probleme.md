# 01 · Caractérisation du problème difficile : la réutilisation du tas noyau

> **Objectif obligatoire 1** du guide Tech 4, livrable « Characterization note on the hard problem, dated October, with the success criteria ».
>
> | | |
> |---|---|
> | Problème identifié et mesuré | **17 octobre 2026** (Rayan Benhabiles, [compte rendu des expériences 1 et 2](experiences/compte_rendu_experiences_1_2.md)) |
> | Mesures complétées | 19 octobre 2026 (expériences 3 à 5) |
> | Note formalisée | **12 novembre 2026** (Elias Lebrun), avant toute mesure des solutions candidates |
> | Reformulation pour un public large | 15 novembre 2026 : vocabulaire simplifié, **critères et seuils inchangés** |
> | Validation mentor | **à obtenir** au prochain suivi (le guide l’attend avant fin octobre : retard à assumer devant le mentor) |
>
> Mots techniques : voir le [lexique](lexique.md).

---

## En bref

- **Ce que FerrumOS doit faire.** Tourner longtemps : ses différentes parties (shell, ordonnanceur, pilotes) prennent sans cesse de la mémoire puis la rendent.
- **Ce qu’il ne sait pas faire.** Réutiliser la mémoire rendue. Elle reste perdue tant qu’il reste quelque chose en cours d’utilisation, c’est-à-dire en pratique toujours.
- **Ce que donne la solution simple en place.** Un test qui prend et rend 1 Kio en boucle s’arrête au **99e tour sur 10 000** : la réserve est vide alors que presque rien n’est réellement utilisé.
- **Quand le problème sera résolu.** Quand les **sept critères chiffrés** du §4 seront atteints, par exemple 10 000 tours sur 10 000 dans ce même test.

## 1. Énoncé du problème

Le noyau de FerrumOS dispose d’une petite réserve de mémoire, le **tas**, de 100 Kio. Toutes ses parties y puisent : le shell pour le texte tapé, l’ordonnanceur pour la liste des tâches, les pilotes pour leurs files d’attente. Chacune prend de la mémoire, s’en sert, puis la rend.

Pour tourner longtemps, le noyau doit **réutiliser la mémoire rendue**. La quantité totale qu’il utilise au cours de sa vie ne doit pas être limitée par la taille de la réserve, seulement la quantité utilisée à un instant donné.

Aujourd’hui, ce n’est pas le cas : la mémoire rendue n’est réutilisée que lorsque **tout** a été rendu en même temps. Un noyau garde toujours quelque chose en cours d’utilisation, donc en pratique la mémoire rendue n’est jamais réutilisée. Le système s’arrête quand la réserve a été traversée une fois.

## 2. Ce qui le rend difficile

| Contrainte | Ce qu’elle impose | Conséquence sur la solution |
|---|---|---|
| **Ressources** | Une réserve petite et fixe, sans autre système en dessous pour fournir de la mémoire | L’organisation de la mémoire doit tenir dans la mémoire elle-même, sans en gaspiller |
| **Fiabilité** | Une erreur dans la gestion de la mémoire abîme tout le noyau, donc toute la machine | Prouver par des tests que rien n’est abîmé, avant de chercher la vitesse |
| **Temps** | Le noyau doit répondre vite, y compris à terme pendant les interruptions du matériel | Un coût par opération borné et mesuré, y compris dans les cas lents, pas seulement en moyenne |
| **Compatibilité** | Le langage Rust impose la forme des demandes, avec des contraintes d’alignement | Respecter ces contraintes sans perdre de place |
| **Concurrence** | Plusieurs traitements peuvent demander de la mémoire, et une interruption peut survenir en plein milieu | Ne jamais bloquer le système ; garder la protection actuelle par verrou |
| **Mesure** | Les temps mesurés dans la machine simulée varient de 15 à 20 % d’une exécution à l’autre | Juger d’abord sur des mesures qui ne varient pas (nombre de tours, d’erreurs, d’octets) |

## 3. Ce que donne l’approche simple, mesuré

L’approche simple est celle en place : un allocateur « à pointeur » (*bump*). Il avance d’un cran à chaque demande, comme une pile de feuilles qu’on prend sans jamais remettre. Il ne revient au début que si **toutes** les demandes ont été rendues.

Mesures du 17 et du 19 octobre 2026, 3 exécutions identiques dans la machine simulée QEMU. Elles ont été **reproduites à l’identique le 12 novembre 2026** (voir [`results/qemu/`](results/qemu/)).

| Expérience | Résultat | Ce que ça montre |
|---|---|---|
| **exp2** · garder 4 petits blocs, puis prendre et rendre 1 Kio en boucle, objectif 10 000 tours | **99 tours** (1 % de l’objectif), puis refus | La mémoire rendue n’est jamais réutilisée : 99 × 1 Kio, c’est la taille de la réserve |
| exp2 · mémoire comptée comme « utilisée » à la fin | **101 632 o** pour **256 o** réellement utilisés | Le noyau ne sait pas non plus mesurer ce qu’il utilise vraiment |
| **exp1** · 10 000 demandes variées, jamais plus de 16 Kio utilisés à la fois | aucune erreur, mais **13 épuisements** de la réserve | Le test ne survit qu’en rendant tout artificiellement 13 fois ; un vrai noyau ne le peut pas |
| exp3 · remplir de blocs de 1 Kio, tout rendre, 3 tours | 100 / 100 / 100 blocs | La seule qualité de cette approche : elle ne perd aucun octet pour s’organiser |

**Pourquoi ce n’est pas suffisant** : avec une réserve de 100 Kio, un noyau qui prend 1 Kio par seconde s’arrête en moins de deux minutes, même s’il n’utilise jamais plus de 2 Kio à la fois. Toute demande faite après l’épuisement **arrête le noyau**.

## 4. Critères de réussite chiffrés

Fixés le 12 novembre 2026, **avant** toute mesure des solutions candidates. Ils s’appliquent à la solution intégrée au noyau : C1 à C5 sont mesurés dans le noyau sous QEMU, C6 et C7 sur le banc hôte. Ces critères décrivent le problème, pas une solution : n’importe quel allocateur peut être jugé avec eux.

| # | Critère | Seuil de réussite | Avant (bump) | Source de la mesure |
|---|---|---|---|---|
| **C1** | La mémoire libérée est réutilisable | **exp2 ≥ 10 000 itérations** | 99 | expérience 2, sous QEMU |
| **C2** | Plus de remise à zéro artificielle | **exp1 : 0 épuisement** ; scénario `long_run` (100 000 opérations sans libération globale) : **0 refus** | 13 ; à mesurer | expérience 1, sous QEMU ; banc hôte |
| **C3** | Correction | **0 erreur** (chevauchement, alignement, contenu) sur toutes les expériences, exp1 rejouée sur **6 graines** | 0 sur 1 graine | QEMU + banc hôte |
| **C4** | Capacité utile conservée | exp3 **≥ 95 blocs de 1 Kio** par tour (perte de métadonnées ≤ 5 %) ; après un changement de profil (64 o → 1 Kio), **≥ 95 blocs de 1 Kio** | 100 ; à mesurer | expérience 3 + banc hôte |
| **C5** | Retour à vide | `used_after = 0` après libération complète (exp1, exp3) | 0 | QEMU |
| **C6** | Coût par opération borné | sur l'hôte (release), **p99 ≤ 1 000 ns** par allocation ou libération, sur la charge exp1 et sur `long_run` | à mesurer | banc hôte |
| **C7** | Mémoire observable | les statistiques exposent la **mémoire réellement vivante** et la **fragmentation** | non (limites 4 et 5 du compte rendu) | statistiques du tas exposées par le noyau |

**En clair :**

- **C1** : la mémoire rendue sert de nouveau, le test de la boucle va jusqu’au bout.
- **C2** : le noyau n’a plus besoin de tout rendre d’un coup pour survivre.
- **C3** : aucune donnée abîmée, sur plusieurs séries de demandes différentes.
- **C4** : la solution ne gaspille pas plus de 5 % de la réserve pour s’organiser, même quand la taille des demandes change.
- **C5** : quand tout est rendu, la réserve redevient entièrement libre.
- **C6** : chaque demande reste rapide, y compris les plus lentes (moins d’une microseconde pour 99 % d’entre elles).
- **C7** : le noyau sait dire combien de mémoire il utilise réellement et à quel point elle est émiettée.

**Le problème est considéré comme résolu si C1 à C5 sont atteints sous QEMU et C6, C7 sur le banc.** Un critère manqué sera publié avec l’écart, pas reformulé.

## 5. Hors périmètre (assumé)

- **Agrandir la réserve** pendant que le système tourne (limite 6 du compte rendu) : le seuil reste 100 Kio.
- **La mémoire des tables de pages** jamais rendue (limite 7) : c’est un problème de la couche qui gère les pages, pas du tas.
- **La vraie concurrence entre plusieurs tâches** : impossible à tester tant que l’ordonnanceur ne sait pas passer d’une tâche à une autre (expérience 5).

## Sources

- [Compte rendu des expériences 1 et 2](experiences/compte_rendu_experiences_1_2.md) et [des expériences 3, 4 et 5](experiences/compte_rendu_experiences_3_4_5.md) (Rayan Benhabiles, 17 et 19 octobre 2026)
- [Sorties brutes](experiences/logs/) et [environnement de test](../../../../POC/ferrum_Poc_adaptative_sheduller/test_environment.md)
- Reproduction du 12 novembre 2026 : [`results/qemu/bump_run1_2026-11-12.log`](results/qemu/bump_run1_2026-11-12.log) (exécutions 1 à 3) et [`results/qemu/bump_ancienne-baseline_2026-11-12.log`](results/qemu/bump_ancienne-baseline_2026-11-12.log)
