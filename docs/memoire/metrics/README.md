# Historique de suivi des indicateurs, objectif D

`history.csv` conserve une ligne par exécution de la suite de tests. C'est le
livrable « historique de suivi des indicateurs » de l'objectif D : il sert à
distinguer un gain réel du bruit de mesure.

## Comment ajouter une mesure

```
cargo test 2>&1 | tee docs/memoire/logs/<nom>.log
python -I scripts/metrics_history.py docs/memoire/logs/<nom>.log \
  --date AAAA-MM-JJ --commit <hash court> --label "<ce que teste cette passe>" \
  --allocator bump
```

`--allocator` passera à `free-list` à la boucle 1. `--dry-run` affiche le
résumé sans rien écrire. Le protocole demande 3 exécutions par mesure, donc
trois lignes par session.

## Colonnes

| Colonne | Signification |
|---|---|
| `date` | date de l'exécution |
| `commit` | hash court du dépôt au moment de la mesure |
| `label` | ce que cette session de mesure cherchait à établir |
| `run` | numéro de l'exécution dans la session, de 1 à 3 |
| `allocator` | allocateur en place : `bump`, puis `free-list` |
| `i1_reuse_pct` | **I1**, taux de réutilisation du tas, en pourcentage |
| `i1_iterations` | indicateur dérivé de I1 : itérations atteintes avant le refus, sur 10 000 |
| `i2_live_blocks` | **I2**, blocs de 1 Kio vivants simultanément, le minimum des 3 tours |
| `i3_cycles_per_iter` | **I3**, cycles TSC par itération, expérience 1 |
| `exp1_errors` … `exp5_errors` | erreurs de corruption ou d'alignement, doivent rester à 0 |
| `exp4_frames_lost` | cadres physiques perdus sur 500 cycles de page, doit rester à 0 |
| `source_log` | log brut d'où la ligne est extraite |

## Formule de I1

```
octets libérés   = (itérations - 1) x 1024
résident attendu = 256 (4 blocs long lived) + 1024 (bloc en cours)
octets réutilisés = octets libérés - (used_after - résident attendu)
I1 = octets réutilisés / octets libérés
```

Avec l'allocateur à pointeur : `used_after = 101 632`, `itérations = 99`, donc
`98 x 1024 = 100 352` octets libérés et `101 632 - 1 280 = 100 352` octets de
croissance. Les deux nombres sont égaux, donc **I1 = 0 %**. Aucun octet libéré
n'est réutilisé, et le résultat ne dépend d'aucune interprétation.

## Seuils

| Indicateur | Référence | Cible ou seuil |
|---|---|---|
| I1 | 0 % | cible : au moins 95 % |
| I1 dérivé | 99 / 10 000 | cible : 10 000 / 10 000 |
| I2 | 100 blocs | seuil de non régression : au moins 90 |
| I3 | 154 429 cycles | seuil de non régression : au plus plus 25 %, soit 193 036 |

I3 ne sert jamais à démontrer un gain. La dispersion observée entre sessions de
mesure va de 150 305 à 169 974 cycles par itération sans aucun changement de
code, soit un écart de 13 %.
