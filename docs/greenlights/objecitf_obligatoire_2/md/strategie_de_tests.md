# Stratégie de tests et preuves

Valider séparément les contrats de la bibliothèque sur l’hôte, le banc autonome sous QEMU x86_64 et le chemin critique de l’image intégrée. Associer chaque verdict à la révision et à l’environnement testés. Documenter les mesures de performance dans un protocole distinct.

La matrice définit les contrôles à appliquer selon le périmètre de la PR. Un résultat de formatage, de compilation ou de lint complète la validation fonctionnelle. Conserver les journaux de ces contrôles avec les résultats de tests.

## Résultats de référence

La campagne archivée d’octobre 2026 compare `foundation_avant/` à `foundation/`. La variante après optimisation retourne immédiatement `None` si la liste du Round Robin est vide. Les deux variantes réussissent les huit tests sur macOS x86_64 via Rosetta et sur le banc autonome QEMU x86_64. Le banc passe en mode long et exécute les mêmes assertions. Le contrôle négatif déclenche une panique après les huit assertions. Le formatage, la vérification Cargo, Clippy et la compilation de la bibliothèque pour `x86_64-unknown-none` réussissent également dans la campagne de référence.

| ID | Module et scénario | Critère d’acceptation | Preuve archivée |
| --- | --- | --- | --- |
| T01 | Mémoire, relevé incohérent | Une utilisation de 17 pour une capacité de 16 est rejetée | 8/8 avant et après sur les deux cibles |
| T02 | Mémoire, calcul de l’espace libre | `(16, 6)` donne 10 et `(0, 0)` donne 0 | Idem |
| T03 | Télémétrie, compteurs distincts | Deux ticks et un changement de contexte déclarés donnent 2 et 1 | Idem |
| T04 | Télémétrie, saturation | Un compteur à `u64::MAX` conserve sa valeur | Idem |
| T05 | Ordonnanceur, rotation | Les tâches prêtes sont sélectionnées dans l’ordre 1, 2, 3, 1, 2, 3 | Idem |
| T06 | Ordonnanceur, vide et capacité nulle | La sélection à vide renvoie `None` et la capacité nulle refuse l’insertion | Idem |
| T07 | Ordonnanceur, erreurs d’inscription | Doublon et dépassement refusés, nombre de tâches inchangé | Idem |
| T08 | Ordonnanceur, retrait | La tâche retirée est sautée et son emplacement est réutilisable | Idem |

T01 à T08 désignent ici les cas de test. Les sorties Cargo et les lignes `PASS` de la console QEMU donnent les noms de fonctions correspondants. Le corpus archivé établit le comportement de la sélection Round Robin et des relevés simulés. L’allocateur réel, les interruptions matérielles, les changements de contexte, la préemption et l’isolation utilisateur restent à couvrir sur le prototype commun.

Le microbanc sous Rosetta mesure le cas vide de 61,733 à 0,463 ns par sélection, médianes des moyennes de boucle. Les charges non vides restent proches entre les deux variantes. Les valeurs décrivent la sélection en laboratoire, sur le poste de référence.

## Matrice de tests à mettre en place

P0 désigne un contrôle bloquant proposé avant fusion ou avant activation du composant concerné. P1 désigne une extension à planifier quand les interfaces et la charge de test seront stabilisées.

| Priorité | Niveau et scénario | Critère d’acceptation et trace attendue |
| --- | --- | --- |
| P0 | Hôte, formatage, compilation, Clippy et T01–T08 | Contrôles au vert et journaux associés à la révision |
| P0 | QEMU autonome, succès et contrôle négatif | Mode long, huit `PASS` nommés et résumé `passed=8 failed=0` avec code 33. Panique volontaire marquée, code 35 et absence de résumé de succès pour le contrôle négatif |
| P0 | QEMU intégré, démarrage → timer → sélection → tâche | L’image atteint le noyau. Un tick est traité et acquitté. Deux tâches montrent qu’elles progressent. Les compteurs distinguent ticks et changements de contexte réellement exécutés. Conserver la console série et les assertions |
| P0 | QEMU intégré, allocation → épuisement → libération | L’épuisement suit l’erreur définie par le contrat. Après libération, le bloc peut être réalloué et le relevé mémoire reste cohérent. Conserver les relevés avant, après et après récupération. Couvert sur le PoC d’ordonnancement par `experiment_2` et `experiment_3` avec l’allocateur à liste chaînée : [mesures avant](../../objecitf_obligatoire_1/tas_noyau/experiences/compte_rendu_experiences_3_4_5.md), [intégration](../../objecitf_obligatoire_1/tas_noyau/09_integration.md) et [automatisation QEMU](./automatisation_tests_qemu.md) |
| P1 | QEMU intégré, faute en espace utilisateur | Quand Ring 3 est disponible, la tâche fautive est arrêtée et ses ressources sont récupérées. Le noyau et une autre tâche restent actifs. Un pointeur utilisateur invalide suit le chemin d’erreur prévu. Conserver le journal et les compteurs |

Pour une politique d’ordonnancement, vérifier l’ordre de sélection, les priorités égales, la progression des tâches moins prioritaires et la saturation. Fixer les critères avant de mesurer. Les traces doivent identifier les ticks simulés et les changements de contexte effectivement exécutés.

## Exécution, traçabilité et régressions

La CI proposée exécute les contrôles hôte puis QEMU avec un délai explicite. Une erreur de compilation, un délai dépassé, un code incohérent ou un marqueur série manquant fait échouer le contrôle. Pour chaque exécution, conserver la date, la révision ou les empreintes SHA‑256, la configuration, les commandes, les journaux, le code de sortie et le verdict. Les fichiers historiques sont dans `resultats/`, en particulier `manifest.json`, `commandes.json` et `qemu_*.stdout.log`. Le guide d’exploitation décrit les commandes de vérification et de relance.

Les huit succès renseignent le nombre de cas exécutés. Mesurer la couverture du chemin critique quand les tests intégrés seront en place. Pour chaque régression réellement interceptée, consigner la date, le comportement cassé, le test déclenché et le correctif. La panique volontaire qualifie le banc et son verdict.

Sources — archive du PoC FerrumOS x86_64, journaux et manifestes associés. Dépôt FerrumOS, `.github/workflows/ci.yml` et `POC/ferrum_Poc_adaptative_sheduller/src/scheduler.rs`. Technical Track – Student Guide, Tech 4, pp. 9–11 et 20–21.

[Scénarios de priorité S01 à S07](../../objecitf_obligatoire_1/ordonnancement_a_priorites.md#scénarios-dacceptation-à-exécuter) · [Guide QEMU](./exploitation_qemu.md)

## Ajouter un test et l’associer à une preuve

Dans une copie de travail de `foundation/`, ajouter le fichier `kernel-core/tests/round_robin_capacity.rs`. Ce test hôte exerce le contrat public. Il vérifie qu’une inscription refusée laisse le contenu et la rotation exploitables. Après la suppression d’une tâche, le même identifiant refusé peut être inscrit dans l’emplacement libéré.

```rust
use ferrumos_kernel_core::scheduler::{RegisterError, RoundRobin, TaskId};

#[test]
fn accepts_previously_rejected_task_after_slot_is_freed() {
    let mut ready = RoundRobin::<2>::new();
    assert_eq!(ready.register(TaskId(10)), Ok(()));
    assert_eq!(ready.register(TaskId(20)), Ok(()));

    assert_eq!(ready.register(TaskId(30)), Err(RegisterError::Full));
    assert_eq!(ready.runnable_count(), 2);
    assert_eq!(ready.select_next(), Some(TaskId(10)));
    assert_eq!(ready.select_next(), Some(TaskId(20)));

    assert!(ready.remove(TaskId(10)));
    assert_eq!(ready.register(TaskId(30)), Ok(()));
    assert_eq!(ready.runnable_count(), 2);
    assert_eq!(ready.select_next(), Some(TaskId(30)));
    assert_eq!(ready.select_next(), Some(TaskId(20)));
}
```

Depuis `foundation/`, sélectionner le test puis vérifier tout le workspace. Conserver la sortie du test avec le SHA Git ou l’empreinte des sources. Confirmer le nom et le nombre de tests exécutés quand un filtre Cargo est utilisé.

```sh
cargo +1.90.0 test --locked --offline -p ferrumos-kernel-core --test round_robin_capacity
cargo +1.90.0 test --locked --offline --workspace
cargo +1.90.0 fmt --all -- --check
cargo +1.90.0 clippy --workspace --all-targets --offline --locked -- -D warnings
```

Le banc historique extrait huit tests unitaires de `src/`. Le fichier dans `tests/` est exécuté par Cargo sur l’hôte. Pour transposer ce scénario à QEMU, intégrer ses assertions au `mod tests` de `kernel-core/src/scheduler.rs` dans le banc de travail. Dans `run_campaign.py`, adapter le nombre de tests et les résumés attendus au nouveau manifeste. `harness/main.rs.in` calcule déjà le nombre de succès à partir des cas lancés. Garder les sources, empreintes et journaux de la campagne historique comme référence de comparaison.

### Fiche d’acceptation pour le PoC commun

| Champ | Valeur proposée pour le premier scénario intégré |
| --- | --- |
| Identifiant et risque | `INT-TIMER-01` · distinguer un tick traité d’une tâche exécutée |
| Préparation | Révision, empreinte de l’image, commande QEMU, configuration, deux tâches identifiables A et B |
| Action | Démarrer. Attendre le premier tick acquitté. Attendre une preuve d’exécution pour chaque tâche |
| Critère | Console série avec marqueurs d’initialisation, de tick, d’acquittement et de progression de A et B dans le délai convenu |
| Cas d’échec | Supprimer la preuve de progression de B dans une image de test. Le contrôle échoue et conserve les traces |
| État du résultat | À exécuter. Renseigner ensuite la date, le verdict et le lien du journal |

Le contrôle QEMU combine code de sortie, marqueurs série et délai. Dans le banc historique, le code 33 exige huit `PASS` et le résumé de succès. Le code 35 est attendu uniquement pour le contrôle négatif et exige le marqueur de panique volontaire. Sur le PoC commun, définir son protocole avant de reprendre ces valeurs. Un fichier absent ou vide, un délai expiré ou un marqueur manquant doit rendre l’exécution rouge.

Pour la couverture du chemin critique, relier chaque exigence à un test, à ses branches d’erreur et au verdict d’une exécution identifiée. Tenir à jour les cas qui restent à couvrir, dont l’échec d’allocation, l’IRQ et la progression réelle des tâches. Conserver la couverture de code mesurée dans un rapport distinct lorsqu’un outil compatible est disponible.

Références. [Exécution des tests Cargo](https://doc.rust-lang.org/cargo/commands/cargo-test.html). Scripts et journaux de l’archive FerrumOS.
