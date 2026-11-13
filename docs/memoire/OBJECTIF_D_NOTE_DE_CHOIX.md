# Objectif D : note de choix de l'axe et des indicateurs

> **Objectif complémentaire D, mesurer, tester et optimiser les performances techniques.**
> Projet FerrumOS. Note rédigée le 13 novembre 2026.
> Livrable attendu avant la fin décembre 2026.
> Dépôt : `EpitechPGE3-2025/G-EIP-601-ALL-6-1-eip-81`, branche `test/memory`.
> Mesures de référence : `docs/memoire/COMPTE_RENDU_EXPERIENCES_1_ET_2.md` et
> `docs/memoire/COMPTE_RENDU_EXPERIENCES_3_4_ET_5.md`.
> Protocole et environnement : `POC/ferrum_Poc_adaptative_sheduller/TEST_ENVIRONMENT.md`.

---

## 0. Synthèse

| Élément | Valeur |
|---|---|
| Axe prioritaire retenu | Consommation mémoire |
| Indicateur principal | Taux de réutilisation du tas |
| Référence mesurée | **0 %**, soit 99 itérations sur 10 000 avant refus |
| Cible de fin d'année | **au moins 95 %**, soit 10 000 itérations sur 10 000 |
| Garde fous | capacité en blocs vivants, coût d'un cycle allocation et libération |
| Boucles d'optimisation prévues | 2, échéances fin mars et fin mai 2027 |

---

## 1. Axe retenu et justification

L'axe retenu est la **consommation mémoire**.

FerrumOS est un noyau : il n'y a aucun allocateur système en dessous de lui.
Toute la mémoire dynamique du système passe par son propre allocateur, un
allocateur à pointeur de 100 Kio qui avance dans le tas à chaque allocation et
ne revient en arrière que lorsque le compteur de blocs vivants retombe à zéro.

Or un noyau ne retombe jamais à zéro. Le shell, l'ordonnanceur et les pilotes
gardent des structures vivantes en permanence. La conséquence est directe :
dans un noyau en fonctionnement, **chaque allocation consomme définitivement du
tas**, et la mémoire libérée n'est jamais rendue.

### Ce qui se passe si l'axe n'est pas traité

Le système s'arrête après un nombre fini d'opérations, et ce nombre est petit.
La mesure du 17 octobre 2026 le donne : **99 itérations sur 10 000**. Il n'y a
ni dégradation progressive ni récupération possible, l'allocateur passe du
fonctionnement normal au refus sec. Toute fonctionnalité qui alloue dans la
durée, un espace utilisateur, des processus, une pile réseau, un système de
fichiers, est hors d'atteinte tant que ce point n'est pas réglé.

### Pourquoi cet axe plutôt qu'un autre de la liste

| Axe candidat | Raison de l'écarter |
|---|---|
| Temps de réponse ou d'exécution | QEMU tourne ici sans accélération matérielle. La dispersion mesurée sur les cycles va de moins 13 % à plus 13 % sur l'expérience 1, et atteint un facteur 2 sur l'expérience 2 pour un nombre d'itérations constant. Un gain inférieur au bruit ne serait pas démontrable. |
| Comportement sous charge | Un noyau qui ne sort pas de la machine virtuelle n'a pas de charge extérieure à simuler, et l'ordonnanceur n'exécute pas encore de vraies tâches. |
| Temps de chargement côté client, coût d'exploitation | Sans objet pour un système d'exploitation sans service en ligne. |
| Taux d'erreur et résilience | Dépend d'abord de la mémoire : aujourd'hui le mode de défaillance dominant est l'épuisement du tas. |
| Sécurité | Traitée par la conception du langage, avec les blocs `unsafe` cantonnés aux zones critiques. Ce n'est pas un axe chiffrable en l'état. |

La mémoire se compte en octets et en blocs. Ce sont des grandeurs
déterministes, et elles le sont vérifiablement : les trois exécutions de chaque
expérience donnent exactement le même nombre d'itérations, le même nombre de
blocs et le même nombre d'octets.

---

## 2. Indicateurs définis

### I1, taux de réutilisation du tas, indicateur principal

| | |
|---|---|
| **Formule** | octets libérés puis réalloués ÷ octets libérés |
| **Outil** | `tests/experiment_2.rs`, relevé `RECORD name=exp2_long_lived_vs_churn` |
| **Conditions** | tas de 100 Kio, 4 blocs long lived de 64 octets conservés vivants, boucle allouant puis libérant 1 Kio, graine aléatoire fixe, interruptions masquées, noyau fraîchement démarré, 3 exécutions |
| **Référence** | **0 %** |
| **Cible** | **au moins 95 %** |

Indicateur dérivé, plus lisible en séance : le nombre d'itérations atteintes
avant le premier refus d'allocation. Référence **99 sur 10 000**, cible
**10 000 sur 10 000**.

### I2, capacité en blocs vivants simultanés, garde fou

| | |
|---|---|
| **Formule** | nombre maximal de blocs de 1 Kio alloués avant le premier refus |
| **Outil** | `tests/experiment_3.rs`, relevé `RECORD name=exp3_exhaustion_and_recovery` |
| **Conditions** | tas de 100 Kio, remplissage par blocs de 1 Kio jusqu'au refus, libération totale, 3 tours par exécution, 3 exécutions |
| **Référence** | **100 blocs**, soit 102 400 octets sur un tas de 102 400 octets, donc 100 % du tas effectivement utilisable |
| **Seuil de non régression** | **au moins 90 blocs** |

Cet indicateur existe parce que la solution envisagée coûtera des métadonnées
par bloc. Le guide demande d'annoncer l'arbitrage, et c'est ici qu'il se lira.

### I3, coût d'un cycle allocation et libération, garde fou

| | |
|---|---|
| **Formule** | médiane des cycles TSC ÷ nombre d'itérations, sur 10 000 itérations |
| **Outil** | `tests/experiment_1.rs`, relevé `RECORD name=exp1_alloc_free_sequence`, compteur `timing::cycles()` |
| **Conditions** | tailles de 8 à 512 octets et alignements de 1 à 64 tirés sur graine fixe, écriture puis vérification du motif, 3 exécutions, médiane retenue |
| **Référence** | **154 429 cycles par itération**, dispersion mesurée de moins 13 % à plus 13 % |
| **Seuil de non régression** | **au plus plus 25 %**, soit 193 036 cycles par itération |

Le seuil est posé au delà de la dispersion mesurée, sinon il déclencherait sur
du bruit. Cet indicateur ne sert jamais à démontrer un gain, uniquement à
détecter une dégradation franche.

---

## 3. Chiffre cible de fin d'année et sur quoi il repose

**Cible : I1 d'au moins 95 %, soit l'expérience 2 portée de 99 à 10 000
itérations, sans que I2 descende sous 90 blocs ni que I3 dépasse plus 25 %.**

Le chiffre de 10 000 n'est pas arbitraire, il est imposé par l'arithmétique du
protocole. Dix mille itérations de 1 Kio demandent 10 Mio de mémoire fraîche,
sur un tas qui en contient 100 Kio, soit cent fois moins. Aucune marge ne peut
produire ce résultat : le seul moyen de tenir la boucle est de réutiliser la
mémoire libérée. Avec une réutilisation complète, l'empreinte vivante reste à
1 Kio pour le bloc en cours plus 256 octets pour les 4 blocs long lived, quel
que soit le nombre d'itérations.

Atteindre 10 000 est donc la preuve que la réutilisation existe, et pas
seulement qu'on a gagné de la marge.

---

## 4. Protocole de mesure

Le protocole complet est versionné dans
`POC/ferrum_Poc_adaptative_sheduller/TEST_ENVIRONMENT.md`. Résumé des points
qui rendent une mesure comparable à une autre :

| Élément | Valeur figée |
|---|---|
| Rust | `nightly-2026-01-15`, épinglé par `rust-toolchain.toml` |
| `bootimage` | 0.10.3 |
| `bootloader` | 0.9.34, épinglé par `Cargo.lock` |
| QEMU | 11.1.0, machine et mémoire par défaut, sans accélération matérielle |
| Cible | `x86_64-ferrumos.json`, virgule flottante logicielle |
| Tas | 100 Kio à l'adresse `0x_4444_4444_0000` |
| Jeu de données | tailles, alignements et motifs tirés par un générateur xorshift à graine fixe |
| Isolation | une image bootable par expérience, `harness = false`, noyau fraîchement démarré, rien ne circule d'une expérience à l'autre |
| Répétitions | 3 exécutions par expérience, valeur médiane retenue pour les cycles, valeur exacte pour les grandeurs déterministes |
| Moment de la mesure | compteurs relevés avant et après chaque expérience, chronomètre arrêté avant l'impression des résultats |

Reproductibilité : `cargo test` construit et démarre sept images, une pour les
neuf tests de capacité du harnais et une par expérience. Une seule expérience
se relance par `cargo test --test experiment_2`.

---

## 5. Référence datée et diagnostic

### Relevés de référence

| Expérience | Date | Résultat |
|---|---|---|
| 1, correction de l'allocateur sur 10 000 allocations et libérations | 17 octobre 2026 | 0 erreur, 0 chevauchement, 3 exécutions sur 3 |
| 2, réutilisation de la mémoire libérée | 17 octobre 2026 | **refus à la 99e itération sur 10 000**, 3 exécutions sur 3 |
| 3, épuisement et reprise | 19 octobre 2026 | 100 blocs par tour, reprise complète, 0 erreur |
| 4, 500 cycles de page, mappage et démappage | 19 octobre 2026 | 0 cadre perdu, 0 erreur de traduction, 3 cadres de tables de pages jamais rendus |
| 5, allocateur sous interruptions | 19 octobre 2026 | 10 200 itérations, 0 erreur, aucun blocage observé |

Preuves : 8 captures dans `docs/memoire/images/`, 3 logs bruts dans
`docs/memoire/logs/`.

### Diagnostic, établi par l'outillage

Les compteurs du relevé de l'expérience 2 donnent `used_after = 101 632`
octets sur un tas de 102 400. Ce nombre se décompose exactement :

```
4 blocs long lived x 64 octets   =        256 octets
99 itérations x 1024 octets      =    101 376 octets
                                      =============
total                            =    101 632 octets
```

Les 99 Kio libérés par la boucle n'apparaissent nulle part en déduction. Le
pointeur du tas n'a jamais reculé. **Zéro octet libéré n'a été réutilisé**, et
ce n'est pas une impression de lecture : la somme tombe au nombre exact que
rapporte le compteur.

La cause est la condition de retour du pointeur, qui exige que le compteur de
blocs vivants retombe à zéro. Les 4 blocs long lived l'en empêchent pour toute
la durée de l'expérience. L'expérience 3, qui libère tout entre deux tours,
confirme le mécanisme par l'autre côté : la reprise y est complète, parce que le
compteur y repasse bien par zéro.

### Hypothèse explicative

Le blocage ne vient pas de la fragmentation ni d'un défaut de calcul, mais de
l'absence totale de structure de suivi des blocs libres. Un allocateur qui tient
une liste chaînée des blocs libres et la consulte avant d'avancer le pointeur
réutilise la mémoire sans dépendre d'un retour à zéro.

### Gain estimé si l'hypothèse est juste

| Indicateur | Référence | Attendu après la boucle 1 |
|---|---|---|
| I1, taux de réutilisation | 0 % | au moins 95 % |
| I1 dérivé, itérations avant refus | 99 | 10 000 |
| I2, capacité en blocs vivants | 100 | entre 90 et 100, les métadonnées par bloc coûtant de la place |
| I3, coût par cycle | 154 429 cycles | de plus 0 à plus 25 %, le parcours de la liste remplaçant une addition |

Les deux dernières lignes sont des dégradations attendues et assumées. Elles
sont chiffrées à l'avance pour que la remesure les confirme ou les démente, et
non pour les excuser après coup.

---

## 6. Boucles d'optimisation prévues

Règle appliquée aux deux boucles : **un seul changement isolé par boucle**,
remesure dans les conditions exactes du protocole, écart donné en valeur absolue
et en pourcentage, arbitrage consigné, et vérification de non régression. Les
changements qui ne produisent aucun gain sont consignés au même titre que les
autres.

### Boucle 1, échéance fin mars 2027

Changement : remplacer l'allocateur à pointeur par une liste chaînée de blocs
libres, en conservant la même interface et le même tas de 100 Kio.
Validation : I1 au moins 95 %, I2 au moins 90, I3 au plus plus 25 %, et les
cinq expériences plus les neuf tests de capacité toujours au vert.

### Boucle 2, échéance fin mai 2027

Changement à décider **sur les mesures de la boucle 1**, et non à l'avance.
Pistes ouvertes, par ordre de probabilité :

1. fusion des blocs libres adjacents, si la boucle 1 fait apparaître une
   fragmentation qui plafonne I2 ;
2. classes de tailles, si I3 se dégrade au delà du seuil à cause du parcours de
   la liste ;
3. traitement séparé des allocations alignées, si l'expérience 1 montre un
   surcoût concentré sur les alignements élevés.

---

## 7. Non régression et suivi dans le temps

| Dispositif | État |
|---|---|
| Suite de tests rejouée après chaque boucle : 9 tests de capacité du harnais et 5 expériences | en place |
| Trois indicateurs relevés à chaque exécution, pas seulement celui qu'on optimise | en place |
| Historique daté des valeurs, avec le hash du commit, pour distinguer un gain réel du bruit de mesure | **en place** : `docs/memoire/metrics/history.csv`, alimenté par `scripts/metrics_history.py`, 4 sessions de mesure et 12 exécutions au 13 novembre 2026 |
| Mesure lancée automatiquement par le pipeline, avec échec sur régression de I1 ou I2 | **à créer** |

### Contrôle de stabilité du 13 novembre 2026

La suite complète a été rejouée 3 fois sur le commit `b0f8753`, soit 26 jours
après la mesure de référence et sans aucun changement de code.

| Indicateur | 17 et 19 octobre 2026 | 13 novembre 2026 | Écart |
|---|---|---|---|
| I1, taux de réutilisation | 0 % | 0 % | aucun |
| I1 dérivé, itérations avant refus | 99 | 99 | aucun |
| I2, blocs vivants | 100 | 100 | aucun |
| I3, cycles par itération | 154 429, médiane de 9 exécutions | 160 340, médiane de 3 exécutions | plus 3,8 %, sous le seuil de 25 % |
| Erreurs de corruption, cadres perdus | 0 | 0 | aucun |

Les grandeurs déterministes sont strictement identiques à 26 jours
d'intervalle. Seul I3 bouge, de 3,8 %, ce qui est inférieur à la dispersion de
13 % observée entre deux sessions de mesure d'octobre. La référence est donc
stable, et le bruit de I3 est quantifié.

Attention au vocabulaire : cette passe est un contrôle de stabilité, réalisé
par la même personne sur la même machine. Elle ne vaut pas preuve de
reproductibilité, qui exige qu'un tiers rejoue le protocole. Ce point reste
ouvert.

Note sur l'automatisation : le pipeline devra se fonder sur I1 et I2 pour
accepter ou refuser un changement, jamais sur I3. Une machine de forge tourne
elle aussi sans accélération matérielle, avec une dispersion des cycles au moins
égale à celle mesurée localement.

---

## 8. Calendrier et livrables

| Jalon | Échéance | Livrable | État |
|---|---|---|---|
| Axe et indicateurs choisis | fin décembre 2026 | la présente note | **fait** |
| Protocole de mesure | fin janvier 2027 | `TEST_ENVIRONMENT.md`, outillage en place | **fait**, automatisation à ajouter |
| Référence et diagnostic | fin février 2027 | relevés datés des 17 et 19 octobre, diagnostic et hypothèse ci dessus | **fait** |
| Boucle 1 | fin mars 2027 | commit du changement, remesure, écart observé | à faire |
| Boucle 2 | fin mai 2027 | commit du changement, remesure, écart observé | à faire |
| Non régression | après chaque boucle | résultats de tests, historique des indicateurs | partiellement en place |

---

## 9. Limites reconnues

1. **Les mesures sont prises sur un PoC, pas sur la base commune.** Tout a été
   mesuré sur `POC/ferrum_Poc_adaptative_sheduller`. L'objectif obligatoire 2
   prévoit une base de code unique. Si la fusion intervient entre la référence
   et la boucle 2, la comparaison avant et après perd sa validité. Mesure prise :
   le harnais est un module de la bibliothèque du noyau et migre avec lui, et la
   première mesure sur la base commune sera datée puis comparée explicitement à
   celle d'octobre.
2. **Les durées ne sont pas des durées matérielles.** QEMU tourne sans
   accélération. Les cycles TSC sont émulés et ne servent qu'à comparer deux
   exécutions dans ce même environnement.
3. **L'expérience 5 ne prouve pas la tenue sous interruptions.** Huit
   interruptions seulement sur la durée de l'expérience, et le gestionnaire du
   timer n'alloue pas. L'absence de blocage est un indice, pas une preuve.
4. **Les résultats sont liés à un tas de 100 Kio.** Ils ne s'extrapolent pas à
   une autre taille de tas sans nouvelle mesure.
5. **La moitié multitâche du protocole mémoire reste inexécutable.**
   L'ordonnanceur simule les tâches, sans pile par tâche ni changement de
   contexte. Ce point ne dépend pas de l'objectif D.
