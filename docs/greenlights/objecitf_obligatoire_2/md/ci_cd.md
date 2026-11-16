# Intégration continue et livraison

**Objectif obligatoire 2 · Validation continue et livraison**

Configurer dans `.github/workflows/ci.yml` les contrôles de qualité, de compilation et d’exécution QEMU. Associer chaque verdict à la révision testée.

## Garanties à respecter dans le workflow

| Garantie | Mise en œuvre |
| --- | --- |
| Propagation des échecs | Conserver le code de sortie de `rustfmt` et de Clippy. Éviter `|| echo ...` et `|| true` sur ces contrôles. Vérifier la règle avec un défaut volontaire. |
| Construction reproductible | Employer la même cible JSON et les mêmes options `build-std` en local et en CI. Valider la commande sur un environnement propre. |
| Artefact vérifiable | Publier l’image produite à un chemin stable. Configurer `if-no-files-found: error`. |
| Essais d’exécution séparés | Exécuter QEMU dans un job dédié. Conserver le verdict et les journaux avec l’identifiant de l’image testée. |

## Contrôles et critères de réussite

| Contrôle | Mise en œuvre proposée | Condition de réussite |
| --- | --- | --- |
| Qualité et build | Déclencher les jobs pour `pull_request` et `push`. Épingler la version Rust validée. | Les commandes réussissent et les images sont disponibles. |
| Bibliothèque portable | Exécuter les tests hôte et compiler pour `x86_64-unknown-none`. | Les tests réussissent et Cargo produit la bibliothèque pour la cible. |
| Banc QEMU indépendant | Épingler l’outillage, la machine et le délai. Exécuter le cas nominal et le contrôle négatif. | Les codes et les journaux correspondent au contrat du banc. |
| Image FerrumOS intégrée | Tester le démarrage et les jalons observables. | L’image issue de la révision testée émet les marqueurs série attendus dans le délai. |

Donner aux contrôles des noms distincts et les exécuter à chaque PR concernée avant de les rendre obligatoires. Ajouter QEMU comme contrôle requis après validation du lanceur sur le runner. Exécuter systématiquement chaque contrôle requis pour que son statut corresponde à une vérification réelle. Vérifier la règle de fusion avec une PR de test qui échoue volontairement. [Syntaxe GitHub Actions](https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax) · [Statuts des contrôles GitHub](https://docs.github.com/en/pull-requests/reference/status-checks)

## Commandes confirmées dans le PoC

Depuis `foundation/` dans le dossier du PoC extrait. Les quatre premières commandes sont celles du README. La dernière reprend le contrôle de compilation du script de campagne avec `Cargo.lock`. Installer au préalable `rustfmt`, Clippy et la cible bare metal pour la version utilisée.

```sh
cargo fmt --all -- --check
cargo check --workspace --offline
cargo test --workspace --offline
cargo clippy --workspace --all-targets --offline -- -D warnings
cargo build --lib --offline --locked --target x86_64-unknown-none
```

Le job hébergé sur Linux exécuterait les tests hôte Linux. Le script complet `run_campaign.py` demande macOS et utilise `x86_64-apple-darwin` pour la partie hôte et le benchmark. Extraire la partie QEMU dans un lanceur dédié est donc un préalable pour la CI Linux. Depuis la racine du dossier du PoC, `python3 verifier_resultats.py` contrôle les fichiers d’une campagne archivée. Cette commande vérifie des preuves existantes. [Cible `x86_64-unknown-none`](https://doc.rust-lang.org/rustc/platform-support/x86_64-unknown-none.html)

## Contrat QEMU et conservation des preuves

- **Nominal.** Pour le banc archivé, attendre le code QEMU `33`, le marqueur `arch=x86_64 word_bits=64 long_mode=1`, les huit assertions et le résumé `SUMMARY passed=8 failed=0`. Si la suite évolue, adapter le nombre attendu à la liste des tests de la version.
- **Contrôle négatif.** Attendre le code `35` et le marqueur `intentional_failure_for_exit_validation`. Le lanceur traite cette issue comme un contrôle réussi uniquement si les conditions prévues sont remplies. Une autre panique, un journal incomplet ou un dépassement de délai fait échouer le job.
- **Artefacts.** Conserver les journaux série et stderr, le résumé de chaque exécution, les versions d’outillage, la configuration et le SHA de la révision réellement testée. Publier les fichiers produits même si les tests échouent, puis conserver le statut d’échec du contrôle. L’absence de preuves attendues doit être visible. [Artefacts de workflow GitHub](https://docs.github.com/en/actions/tutorials/store-and-share-data) · [Fichiers attendus dans `upload-artifact`](https://github.com/actions/upload-artifact#customization-if-no-files-are-found)

**Validation du pipeline.** Conserver le lien d’une exécution nominale, le lien d’une exécution où un défaut volontaire déclenche un échec et les journaux associés. Conserver les mesures Rosetta avec le benchmark et les images de test avec les artefacts du pipeline.

[Workflow GitHub Actions](../../../../.github/workflows/ci.yml) · [Qualité Rust](./normes_rust.md) · [Stratégie de tests](./strategie_de_tests.md)

## Configurer les jobs GitHub Actions

Placer les extraits dans `.github/workflows/ci.yml` et valider les jobs dans une PR. Donner à chaque contrôle un nom qui indique ce qu’il vérifie.

### Rendre le lint bloquant

Définir `jobs.lint` avec le répertoire du PoC de référence. Les options `build-std` de l’exemple reprennent `POC/ferrumos_PoC/build_and_run_ferrumos.bat`. Valider cette combinaison avec Rust nightly et épingler la version testée avant de rendre le job obligatoire.

```yaml
  lint:
    name: Rustfmt et Clippy (PoC de référence)
    runs-on: ubuntu-latest
    defaults:
      run:
        working-directory: POC/ferrumos_PoC
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@nightly
        with:
          components: rust-src, rustfmt, clippy
      - name: Vérifier le formatage
        run: cargo fmt -- --check
      - name: Analyser la cible personnalisée
        run: |
          cargo clippy --locked --target x86_64-ferrumos.json \
            -Zbuild-std=core,alloc,compiler_builtins \
            -Zbuild-std-features=compiler-builtins-mem \
            -Zjson-target-spec -- -D warnings
```

Dans une PR de vérification, introduire un écart de formatage et constater que `lint` échoue. Corriger le format puis introduire séparément un avertissement reproductible pour vérifier la propagation de l’échec de Clippy. Retirer les changements volontaires avant fusion. Un avertissement accepté se traite selon la norme Rust, au plus près du code concerné. [Clippy en CI](https://doc.rust-lang.org/clippy/continuous_integration/index.html)

### Aligner la construction de l’image de référence

Configurer la construction dans le job `build-reference-poc`, avec un répertoire de travail explicite. `test -s` fait échouer l’étape lorsque l’image manque ou est vide. Utiliser le chemin vérifié dans l’étape d’envoi et ajouter `if-no-files-found: error` dans `with`.

```yaml
      - name: Construire l’image de référence
        working-directory: POC/ferrumos_PoC
        run: |
          cargo bootimage --target x86_64-ferrumos.json \
            -Zbuild-std=core,alloc,compiler_builtins \
            -Zbuild-std-features=compiler-builtins-mem -Zjson-target-spec
          test -s target/x86_64-ferrumos/debug/bootimage-ferrumos.bin
```

### Publier l’image réellement produite par le PoC graphique

Configurer les étapes de `build-graphics-poc` après l’installation de Rust. Le script `POC/ferrum_Poc_graphic/build.rs` écrit `ferrumos_graphic-bios.img` dans `OUT_DIR`. Isoler le répertoire Cargo pour identifier l’image produite par cette construction.

```yaml
      - name: Installer la cible du noyau graphique
        run: rustup target add --toolchain nightly x86_64-unknown-none
      - name: Construire et préparer l’image graphique
        working-directory: POC/ferrum_Poc_graphic
        shell: bash
        run: |
          export CARGO_TARGET_DIR="$RUNNER_TEMP/ferrumos-graphics-target"
          make build
          mapfile -t images < <(find "$CARGO_TARGET_DIR" -name ferrumos_graphic-bios.img)
          test "${#images[@]}" -eq 1
          test -s "${images[0]}"
          cp "${images[0]}" "$RUNNER_TEMP/ferrumos_graphic-bios.img"
      - uses: actions/upload-artifact@v4
        with:
          name: ferrumos-graphics-bootimage
          path: ${{ runner.temp }}/ferrumos_graphic-bios.img
          if-no-files-found: error
```

Contrôler la provenance de l’image et vérifier que son absence fait échouer l’envoi. Pour l’ordonnanceur, utiliser `make build` dans `POC/ferrum_Poc_adaptative_sheduller/`. [Gestion des artefacts](https://docs.github.com/en/actions/tutorials/store-and-share-data)

### Ajouter le contrôle QEMU en deux étapes

Créer `scripts/test_qemu_banc.py` à partir de la compilation bare metal et de l’exécution QEMU de `run_campaign.py`. Passer les images nominale et négative en paramètres. Lancer `subprocess.run` avec `check=False`, capture de la console et délai de 30 secondes. Écrire un journal par image, puis vérifier le code et les marqueurs selon le contrat ci-dessus. Un délai dépassé ou un résultat inattendu renvoie un statut non nul.

Une fois le lanceur présent et validé, ajouter un job dédié à `.github/workflows/ci.yml`. Il prépare le dossier de preuves avant le test, lance le contrôle sans tolérance d’échec, puis publie les traces même si le contrôle échoue. Le contrat de démarrage de l’image intégrée doit faire l’objet d’un autre job, avec ses propres marqueurs. Les codes du banc indépendant restent propres à son dispositif de sortie.

```yaml
      - name: Publier les traces QEMU
        if: ${{ !cancelled() }}
        uses: actions/upload-artifact@v4
        with:
          name: ferrumos-qemu-traces
          path: resultats-ci/
          if-no-files-found: error
```

Rattacher le verdict et les journaux à la révision testée. Documenter le scénario, le code de sortie et les marqueurs attendus pour chaque essai. [Conditions GitHub Actions](https://docs.github.com/en/actions/reference/workflows-and-actions/expressions#status-check-functions)
