# Objectif complémentaire D : mesurer, tester et optimiser les performances techniques

Responsable : Rayan Benhabiles.
Axe retenu : **consommation mémoire**. Choix arrêté, il ne change plus.

C'est le seul objectif de la filière dont le résultat est un chiffre : soit la
mesure existe et l'écart se lit, soit rien n'est démontré. Ce dossier contient
donc la chaîne complète, protocole, référence, diagnostic, boucles
d'optimisation et non régression.

## Contenu

| Fichier | Rôle | Jalon du guide |
|---|---|---|
| `OBJECTIF_D_NOTE_DE_CHOIX.md` | axe retenu, justification, les indicateurs avec formule et outil, le chiffre cible, le diagnostic et les boucles prévues | axe et indicateurs, avant fin décembre |
| `PROTOCOLE_DE_MESURE.md` | la procédure à suivre pour rejouer les mesures, destinée à quelqu'un qui n'a jamais lancé les tests | protocole de mesure, avant fin janvier |
| `compte_rendu_experiences_1_2.md` | expériences 1 et 2, mesures de référence du 17 octobre 2026 | référence et diagnostic, avant fin février |
| `compte_rendu_experiences_3_4_5.md` | expériences 3, 4 et 5, mesures de référence du 19 octobre 2026 | référence et diagnostic |
| `metrics/history.csv` | une ligne par exécution, pour distinguer un gain réel du bruit de mesure | historique de suivi des indicateurs |
| `metrics/README.md` | colonnes de l'historique, formule de l'indicateur I1, seuils de non régression | |
| `images/` | 8 captures de la sortie des tests, preuves des relevés | relevés datés avec captures |
| `logs/` | 4 logs bruts, sortie série complète des exécutions | relevés datés avec exports |

## Les indicateurs

| | Indicateur | Référence | Cible ou seuil |
|---|---|---|---|
| I1 | taux de réutilisation du tas | 0 % avec l'allocateur à pointeur | au moins 95 % |
| I1 dérivé | itérations avant refus d'allocation | 99 sur 10 000 | 10 000 sur 10 000 |
| I2 | blocs de 1 Kio vivants simultanément | 100 | au moins 90 |
| I3 | cycles par cycle allocation et libération | 154 429 | au plus plus 25 % |

## Outillage

| | |
|---|---|
| Protocole et environnement figé | `POC/ferrum_Poc_adaptative_sheduller/test_environment.md` |
| Extraction des indicateurs et contrôle de régression | `scripts/metrics_history.py` |
| Expériences | `POC/ferrum_Poc_adaptative_sheduller/tests/experiment_1.rs` à `experiment_5.rs`, une image bootable par expérience |

## Rapport avec l'objectif obligatoire 1

L'objectif obligatoire 1 et cet objectif portent tous les deux sur le tas du
noyau. Ils ne couvrent pas le même travail :

- **Objectif obligatoire 1** : caractériser le verrou technique, établir l'état
  de l'art, comparer des solutions candidates, décider et intégrer. Dossier
  dans `docs/greenlights/objecitf_obligatoire_1/`.
- **Objectif complémentaire D** : la chaîne de mesure. Le protocole
  reproductible, les indicateurs chiffrés, le relevé de référence daté, les
  boucles d'optimisation avec écart avant et après, la non régression et
  l'historique.

Les mesures de référence d'octobre 2026 réunies dans ce dossier servent aux deux
objectifs, et sont citées comme telles de part et d'autre.
