# Automatisation des tests noyau sous QEMU

> **Objectif complémentaire 2** (suivi du 17 octobre 2026) : préparer l'automatisation
> des tests sous QEMU et la conservation des résultats afin de faciliter les
> comparaisons et de détecter les régressions.
>
> Responsable : Elias Lebrun (CI/CD). S'appuie sur le harnais de test de Rayan
> Benhabiles (branche `test/memory`, `POC/ferrum_Poc_adaptative_sheduller`).

---

## 1. Ce que fait le système, en une phrase

À chaque modification du noyau, la CI démarre toutes les images de test dans QEMU,
enregistre les mesures, les compare à des valeurs de référence (la **baseline**) et
au run précédent, et **échoue si quelque chose s'est dégradé**.

```
 push / PR
    │
    ▼
 cargo test --no-fail-fast        7 images QEMU : tests de capacité + expériences 1 à 6
    │  (sortie série)
    ▼
 results.py collect  ──►  results.json   statut de chaque image + lignes RECORD
    │
    ▼
 results.py compare  ──►  report.md      baseline + run précédent → ✅ / ❌
    │
    ├─► résumé affiché sur la page du run GitHub Actions
    ├─► artefact téléchargeable (90 jours)
    └─► sur main : commit dans la branche test-results (historique permanent)
```

## 2. Lancer les tests

### En local

Prérequis : l'environnement de `POC/ferrum_Poc_adaptative_sheduller/test_environment.md`
(Rust `nightly-2026-01-15`, `bootimage` 0.10.3, QEMU) et Python 3.

```sh
cd POC/ferrum_Poc_adaptative_sheduller
make test
```

ou, depuis la racine du dépôt :

```sh
bash scripts/kernel_tests/run.sh
```

Les résultats sont écrits dans `kernel-test-results/` (ignoré par git) :

| Fichier | Contenu |
|---|---|
| `kernel-tests.log` | sortie complète de `cargo test` (compilation + sortie série de chaque image) |
| `results.json` | résultats structurés : statut de chaque image, mesures `RECORD`, versions des outils |
| `report.md` | comparaison avec la baseline, lisible tel quel |

Pour comparer avec un run précédent : `bash scripts/kernel_tests/run.sh --previous ancien/results.json`.

Code de sortie : `0` si tout passe, `1` en cas de test échoué ou de régression, `2` si un outil manque.

### En CI

Workflow `.github/workflows/kernel-tests.yml`, déclenché par tout push ou pull request
qui touche le PoC ordonnanceur, les scripts de test ou le workflow lui-même (et à la
main via *Run workflow*).

| Job | Rôle |
|---|---|
| `results-tool` | auto-tests de `results.py` (sans QEMU, quelques secondes) |
| `qemu-tests` | installe QEMU et la toolchain figée, lance `run.sh`, publie le rapport dans le résumé du run et l'artefact `kernel-test-results-<commit>` |
| `store-results` | uniquement sur `main` : ajoute le run à la branche `test-results` |

## 3. La baseline : comment une régression est détectée

`POC/ferrum_Poc_adaptative_sheduller/tests/baseline.json` donne, pour chaque mesure,
une règle (`min`, `max` ou `equals`). Exemple :

```json
"exp2_long_lived_vs_churn": {
  "iterations": { "min": 99 },
  "errors":     { "max": 0 },
  "used_after": { "max": 101632 }
}
```

Le run échoue si :

- une image de test échoue, plante ou ne termine pas (`[failed]`, timeout, triple faute) ;
- une image attendue (`expected_suites`) ne s'est pas exécutée ;
- une mesure sort de sa règle (ex. : exp2 retombe sous 99 itérations) ;
- une mesure de la baseline n'apparaît plus dans la sortie.

**Pourquoi c'est nécessaire.** Les expériences vérifient déjà `errors == 0` et l'absence
de cadres perdus, mais l'expérience 2 *réussit* aujourd'hui en ne tenant que 99
itérations sur 10 000 : elle mesure, elle ne juge pas. Sans baseline, une modification
qui la ferait tomber à 10 itérations passerait inaperçue.

**Seules les valeurs déterministes ont une règle** (itérations, erreurs, mémoire
utilisée). Les cycles sont émulés par QEMU sans accélération matérielle et varient
jusqu'à 20 % entre deux exécutions identiques : ils sont affichés avec leur écart,
jamais bloquants.

### Mettre à jour la baseline

Quand une amélioration est voulue (cycle d'optimisation de l'allocateur, objectif D),
la baseline est relevée **dans la même pull request** : par exemple
`"iterations": { "min": 10000 }` pour l'expérience 2. Le nouveau niveau devient le
minimum à conserver. Le rapport de la PR montre l'écart avant/après, qui sert de
preuve de mesure.

### Ajouter une mesure

1. Dans le noyau, émettre une ligne avec `testing::Record { .. }.report()` :
   `RECORD name=<nom> iterations=… errors=… used_before=… used_after=… cycles=…`
2. Ajouter sa règle dans `baseline.json` (sinon elle est signalée « sans règle »).
3. Une nouvelle image de test (`tests/experiment_N.rs`) doit être ajoutée à `expected_suites`.

## 4. Conservation des résultats

| Où | Durée | Contenu |
|---|---|---|
| Artefact du run GitHub Actions | 90 jours | `kernel-tests.log`, `results.json`, `report.md` |
| Branche `test-results` | permanente | `runs/<date>_<commit>/` pour chaque push sur `main`, `history.csv`, `latest.json` |

`history.csv` contient une ligne par mesure et par run
(`date, commit, branch, status, record, iterations, errors, used_before, used_after, cycles`) :
il s'ouvre dans un tableur pour tracer l'évolution d'une mesure dans le temps.

`latest.json` est le dernier run sur `main` ; chaque run de CI (y compris sur une PR)
s'y compare pour afficher les écarts.

La branche `test-results` ne contient que des résultats, jamais de code, et n'est
écrite que par la CI.

## 5. Limites connues

- **Environnement CI ≠ environnement de référence.** Les runners GitHub utilisent la
  version de QEMU d'Ubuntu, pas 11.1.0. Les valeurs déterministes n'en dépendent pas ;
  les cycles, si. Les versions réelles sont enregistrées dans chaque `results.json`.
- **Seul le PoC ordonnanceur est testé** : c'est le seul à avoir un harnais. Après la
  fusion des PoC (jalon M1), il faudra pointer `POC_DIR` vers la base commune.
- **Pas de concurrence réelle** : tant que l'ordonnanceur ne fait pas de changement de
  contexte, la partie « plusieurs tâches » de l'expérience 5 ne peut pas être testée.
- **La CI de l'organisation doit autoriser l'écriture** par `GITHUB_TOKEN` pour que
  `store-results` puisse pousser sur `test-results` (réglage *Settings → Actions →
  Workflow permissions*). Sans ce droit, seuls les artefacts sont conservés.

## 6. Fichiers

| Fichier | Rôle |
|---|---|
| `scripts/kernel_tests/run.sh` | point d'entrée unique, local et CI |
| `scripts/kernel_tests/results.py` | `collect` (log → JSON), `compare` (baseline → rapport), `history` (JSON → CSV) |
| `scripts/kernel_tests/test_results.py` | auto-tests du parseur et de la comparaison |
| `scripts/kernel_tests/store_history.sh` | CI : commit des résultats dans `test-results` |
| `POC/ferrum_Poc_adaptative_sheduller/tests/baseline.json` | valeurs de référence |
| `.github/workflows/kernel-tests.yml` | workflow CI |
