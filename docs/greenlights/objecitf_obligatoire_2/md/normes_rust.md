# Normes Rust et qualité du code

**Convention T16 · Rust, documentation et revue de code**

Appliquer ces conventions à toute modification Rust. Les exemples des PoC et de `ferrumos-kernel-core` illustrent les contrats, les tests unitaires et l’intégration matérielle.

## Règles de code

| Sujet | Règle | Point à vérifier en revue |
| --- | --- | --- |
| Formatage | Utiliser `rustfmt` avec la version Rust retenue pour le projet. Garder les réglages par défaut tant qu’un besoin commun ne justifie pas de configuration. | Le contrôle de formatage passe. Les corrections de style restent limitées aux fichiers concernés. |
| Noms et interfaces | Utiliser `snake_case` pour les fonctions et modules, `UpperCamelCase` pour les types et traits et `SCREAMING_SNAKE_CASE` pour les constantes. Donner aux types le nom du concept représenté, par exemple `TaskId`. | Chaque élément public a un usage identifié. Les unités et les cas d’erreur sont lisibles. |
| Dépendances | Garder la bibliothèque portable en `no_std`, sans `unsafe`. Les adaptateurs possèdent les détails matériels. Justifier toute nouvelle dépendance par son usage, son support de la cible et son coût de maintenance. | La logique de sélection et les contrats restent testables sur l’hôte. Les types propres au matériel restent dans la couche d’intégration. |
| Erreurs et invariants | Utiliser `Result` pour une erreur que l’appelant doit traiter. Réserver `Option` à une absence attendue, comme la sélection dans une file vide. Documenter le traitement des erreurs et les hypothèses de concurrence. | Vérifier que les erreurs laissent l’état cohérent. Pour le chemin timer, identifier la responsabilité de l’acquittement et celle de la synchronisation. |
| Code bas niveau | Encadrer les opérations `unsafe` nécessaires à l’intégration matérielle par de petites interfaces. Consigner l’invariant dans un commentaire `SAFETY` à chaque bloc et documenter la section `Safety` d’une fonction `unsafe` publique. | Le relecteur peut identifier qui établit les préconditions et quand elles cessent d’être vraies. |
| Documentation | Mettre à jour les contrats publics et les exemples avec le changement. Relier une décision structurante au schéma ou à sa fiche de décision. | Le lecteur trouve le comportement en cas normal, les limites et la commande de vérification. |

Les conventions de nommage et de documentation s’appuient sur les références Rust. [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/naming.html) · [Rustdoc](https://doc.rust-lang.org/rustdoc/how-to-write-documentation.html)

## Contrôles avant ouverture de la PR

### PoC de référence dans le dépôt

Exécuter le contrôle de formatage dans `POC/ferrumos_PoC/` avec la chaîne Rust retenue pour le projet.

```sh
cargo fmt -- --check
```

Propager le code de sortie de `rustfmt` et de Clippy dans la CI. Valider Clippy avec la cible personnalisée et les options de compilation du PoC, puis ajouter `-D warnings`. Consigner la combinaison validée dans la documentation de build.

### Bibliothèque portable du dossier de preuves

Depuis `foundation/` dans le dossier du PoC extrait, avec Rust, `rustfmt` et Clippy installés. Le README de ce workspace confirme les commandes suivantes.

```sh
cargo fmt --all -- --check
cargo check --workspace --offline
cargo test --workspace --offline
cargo clippy --workspace --all-targets --offline -- -D warnings
```

`cargo fmt --all` applique les corrections avant de relancer le contrôle. Clippy traite les avertissements comme des erreurs sur la version Rust choisie par l’équipe. Une exception à un lint doit être limitée à l’élément concerné et motivée dans le code ou dans la PR. Les options hors ligne sont adaptées au workspace archivé, qui est sans dépendance externe. Pour un workspace ultérieur, l’étape d’installation des dépendances doit précéder toute vérification hors ligne. [rustfmt](https://github.com/rust-lang/rustfmt) · [Clippy en CI](https://doc.rust-lang.org/clippy/continuous_integration/index.html)

## Revue d’une modification

1. L’auteur explique le comportement modifié, le périmètre de la PR, les commandes exécutées et leurs résultats. Il indique la cible et précise si les résultats viennent de l’hôte, du banc QEMU ou du noyau intégré.
2. Le relecteur examine le contrat, les erreurs, la durée de vie des ressources et les cas limites. Pour l’ordonnanceur, une sélection de `TaskId` et un changement effectif de contexte sont deux événements distincts. Pour un changement de contrat mémoire, vérifier l’appelant et les tests associés.
3. L’auteur traite les remarques et ajoute un test quand un nouveau comportement ou une régression le justifie. Une modification du chemin d’intégration matériel exige une preuve sur la cible concernée dès qu’elle est disponible.
4. Avant fusion, un autre membre approuve la PR et les contrôles requis passent pour la révision à fusionner. L’équipe doit valider cette règle et l’activer selon les possibilités du dépôt. [Protections de branche GitHub](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-protected-branches/about-protected-branches)

## Bilan de qualité et dette technique

Pour le bilan initial et le bilan final, conserver la révision, le PoC ou workspace, la cible, la version Rust, la commande et le journal brut. Relever séparément le nombre d’erreurs de compilation, d’avertissements Clippy et de fichiers signalés par `rustfmt`. Comparer le même périmètre avec les mêmes réglages. Si la chaîne ou les règles changent, consigner ce changement avec le résultat. Une commande masquée ou absente laisse la mesure à renseigner.

| Exemple de dette | Impact | Clôture attendue |
| --- | --- | --- |
| Échecs de formatage et de Clippy masqués dans la CI | Un contrôle vert peut cacher un défaut | Sortie de commande propagée et PR de vérification avec un échec visible |
| Refus d’insertion non traités dans des chemins du simulateur d’ordonnancement | Une tâche peut disparaître de la simulation lorsque la file de destination est pleine | Contrat de saturation, résultat d’erreur et scénario correspondant |

Pour chaque dette, consigner l’issue, l’effort estimé et le motif du report. Dans la PR, joindre les résultats de `rustfmt`, de la compilation, des tests et de Clippy. Indiquer les points de revue résolus et les vérifications qui restent à faire.

[Contribuer au dépôt](../../../../CONTRIBUTING.md) · [CI et livraison](./ci_cd.md)

## Mise en pratique dans le code FerrumOS

### Documenter l’accès aux tables de pages

Dans `POC/ferrumos_PoC/src/memory.rs`, la fonction `init` crée le mapper à partir de la table active. Le point d’appel se trouve dans `src/main.rs`, qui lit l’offset de `BootInfo` avant d’initialiser le tas. La proposition suivante documente les préconditions et rend visibles les opérations qui en dépendent. Elle reprend les mêmes appels et ne change pas le comportement attendu.

```rust
/// Construit le mapper pour la table de pages active.
///
/// # Safety
/// `physical_memory_offset` doit mapper la mémoire physique à l'offset donné.
/// L'appelant garantit l'accès mutable exclusif à la table active tant que
/// le mapper existe. Initialiser le mapper une seule fois au démarrage.
pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    // SAFETY: le mapping direct et l'exclusivité sont garantis par l'appelant.
    let level_4_table = unsafe { active_level_4_table(physical_memory_offset) };
    // SAFETY: l'offset décrit le mapping physique utilisé par cette table.
    unsafe { OffsetPageTable::new(level_4_table, physical_memory_offset) }
}
```

Au site d’appel dans `src/main.rs`, ajouter un commentaire `SAFETY` qui rattache l’offset au `BootInfo` et à l’initialisation unique. Examiner aussi le déréférencement dans `active_level_4_table` et les préconditions de `BootInfoFrameAllocator::init` dans le même fichier. Le relecteur vérifie qu’un second mapper ne peut pas être créé pendant que le premier détient la table. Compiler le PoC avec sa cible, puis redémarrer l’image et vérifier que le tas s’initialise toujours. [Rustdoc](https://doc.rust-lang.org/rustdoc/how-to-write-documentation.html)

### Traiter la saturation de la file dans le simulateur

Dans `POC/ferrum_Poc_adaptative_sheduller/src/scheduler.rs`, un retour `false` de `FixedQueue::push` signale une insertion refusée. Le contrat proposé propage l’erreur et conserve la file et le prochain identifiant si l’insertion échoue. Déclarer `SpawnError` au niveau du module, puis mettre à jour `impl Scheduler` et la fonction publique située après le verrou global.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpawnError {
    QueueFull,
    IdExhausted,
}
```

```rust
pub fn spawn(
    &mut self,
    name: &str,
    task_type: TaskType,
    work_ticks: u32,
) -> Result<u32, SpawnError> {
    let id = self.next_id;
    let next_id = id.checked_add(1).ok_or(SpawnError::IdExhausted)?;
    let task = Task::new(id, name, task_type, work_ticks);
    if !self.queues[0].push(task) {
        return Err(SpawnError::QueueFull);
    }
    self.next_id = next_id;
    Ok(id)
}
```

```rust
pub fn spawn(name: &str, task_type: TaskType, work_ticks: u32) -> Result<u32, SpawnError> {
    x86_64::instructions::interrupts::without_interrupts(|| {
        SCHEDULER.lock().spawn(name, task_type, work_ticks)
    })
}
```

Dans `POC/ferrum_Poc_adaptative_sheduller/src/main.rs`, traiter le résultat de chaque inscription de démarrage. Le bloc suivant montre le cas `desktop`. Le contrat de démarrage doit indiquer si un refus interrompt le boot ou laisse l’image démarrer avec un message de diagnostic.

```rust
if let Err(error) = scheduler::spawn("desktop", scheduler::TaskType::Interactive, 1200) {
    println!("Task desktop rejected: {:?}", error);
}
```

Dans un test de la méthode, remplir les 16 places, appeler `spawn` une fois de plus et attendre `Err(SpawnError::QueueFull)` avec la même longueur et le même `next_id`. Tester ensuite le refus lorsque `next_id == u32::MAX`. Les transferts entre files dans `tick` et `drain_aged` demandent une règle séparée sur la conservation de la tâche lors d’une saturation. Ces extraits constituent des propositions à compiler et à relire.

### Vérifier la modification

Depuis la racine du dépôt, cibler le manifest du simulateur pour limiter le formatage aux fichiers de ce PoC. Construire ensuite l’image avec son Makefile existant, sur un environnement équipé selon le README du prototype.

```sh
cargo +nightly fmt --manifest-path POC/ferrum_Poc_adaptative_sheduller/Cargo.toml -- --check
cd POC/ferrum_Poc_adaptative_sheduller
make build
```

Dans la PR, associer chaque changement au fichier, à son invariant et à sa vérification. Faire relire le contrat d’erreur et la précondition `unsafe`. Joindre séparément le résultat de compilation, le scénario de saturation et le contrôle du démarrage.
