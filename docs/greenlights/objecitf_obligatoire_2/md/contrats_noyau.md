# Contrats du noyau

Les contrats du noyau définissent les unités, les cas d’erreur et les responsabilités de synchronisation à la frontière entre la logique portable et les adaptateurs matériels. Les signatures de `ferrumos-kernel-core` dans `foundation/` servent de référence pour la couche `no_std` sans `unsafe`.

## Interfaces Rust de l’archive

| Module | Interface | Résultat et cas limites |
| --- | --- | --- |
| `memory` | `MemorySnapshot::new(capacity, used)` et `MemoryReporter::memory_snapshot(&self)` | Les valeurs sont en octets (`u64`). Le constructeur renvoie `None` si l’utilisation dépasse la capacité. Le contrat de lecture renvoie `None` si un relevé cohérent est indisponible. `heap_free_bytes()` calcule `capacity − used`. `(0, 0)` est un relevé valide |
| `interrupts` | `TimerTickSink::on_timer_tick(&mut self)` | Signale un tick. Un appel depuis le gestionnaire d’interruption exige un traitement borné et non bloquant. L’adaptateur gère les masques, l’acquittement, l’ordre des opérations et la réentrance |
| `scheduler` | `RoundRobin<N>::register`, `remove`, `select_next`, `runnable_count` | `TaskId(u64)` identifie une tâche prête. `register` renvoie `Duplicate` ou `Full` sans modifier la file en cas d’erreur. `remove` renvoie `false` si l’identifiant est absent. `select_next` avance le curseur et renvoie `None` à vide. L’exécuteur gère les transitions d’état et les contextes CPU |
| `telemetry` | `Counters`, `KernelSnapshot` et `KernelReporter::kernel_snapshot(&self)` | Les compteurs `u64` saturent. `on_timer_tick` compte les ticks et `record_context_switch` s’appelle après une commutation effective. Le relevé expose `memory`, `runnable_tasks` et `counters`. Un tick est une unité d’événement |

## Exemple de sélection sur l’hôte

```rust
use ferrumos_kernel_core::scheduler::{RegisterError, RoundRobin, TaskId};

fn main() {
    let mut ready = RoundRobin::<1>::new();
    assert_eq!(ready.register(TaskId(7)), Ok(()));
    assert_eq!(ready.register(TaskId(7)), Err(RegisterError::Duplicate));
    assert_eq!(ready.register(TaskId(8)), Err(RegisterError::Full));
    assert_eq!(ready.select_next(), Some(TaskId(7)));
    assert!(ready.remove(TaskId(7)));
    assert_eq!(ready.select_next(), None);
}
```

## Raccordement au prototype

- `allocator::init_heap` initialise le tas du PoC et renvoie une erreur de mappage, notamment `FrameAllocationFailed` si aucune frame n’est disponible. L’adaptateur mémoire devra produire un relevé cohérent sous les protections de l’allocateur, puis le transmettre au shell et au monitoring en lecture.
- `interrupts.rs` contient un gestionnaire timer qui appelle `scheduler::tick()` et acquitte le PIC. Le shell et l’écran d’accueil appellent aussi `tick()` pour faire avancer la simulation. À l’intégration, définir la source de ticks retenue et séparer la sélection d’une tâche de sa commutation effective.
- Pour `KernelReporter`, sérialiser les accès partagés et capturer la mémoire, le nombre de tâches prêtes et les compteurs dans un relevé cohérent. Le shell ou l’interface consomme le relevé après la section critique.

## Contrat de priorité proposé

Le PoC adaptatif simule trois niveaux de priorité. Pour le noyau commun, valider l’ordre des priorités, la règle à égalité, le traitement d’une file pleine et la progression des tâches en attente. Préserver une politique Round Robin comme référence de comparaison. Documenter les transitions et les tests associés avant de stabiliser l’interface.

Sources — archive FerrumOS, `foundation/kernel-core/src/`. Dépôt FerrumOS, `POC/ferrum_Poc_adaptative_sheduller/src/`. Technical Track – Student Guide, Tech 4, pp. 8–11.

[Schémas T02/T04](../pdf/02_FerrumOS_Architecture_T02_T04.pdf) · [Stratégie de tests](./strategie_de_tests.md)

## Mettre en œuvre les contrats dans le code

Dans une copie de travail de `foundation/`, placer l’exemple ci-dessous dans `kernel-core/examples/reporter_probe.rs`. Ce workspace provient du dossier de preuves. Le fichier crée un relevé mémoire synthétique, compte un tick et sélectionne un identifiant. Il utilise les interfaces Rust présentes dans l’archive. Il peut servir de point de départ à un adaptateur de télémétrie sur l’hôte.

```rust
use ferrumos_kernel_core::interrupts::TimerTickSink;
use ferrumos_kernel_core::memory::{MemoryReporter, MemorySnapshot};
use ferrumos_kernel_core::scheduler::{RoundRobin, TaskId};
use ferrumos_kernel_core::telemetry::{Counters, KernelReporter, KernelSnapshot};

struct Probe {
    ready: RoundRobin<2>,
    counters: Counters,
}

impl MemoryReporter for Probe {
    fn memory_snapshot(&self) -> Option<MemorySnapshot> {
        MemorySnapshot::new(16, 6)
    }
}

impl KernelReporter for Probe {
    fn kernel_snapshot(&self) -> KernelSnapshot {
        KernelSnapshot {
            memory: self.memory_snapshot(),
            runnable_tasks: self.ready.runnable_count(),
            counters: self.counters,
        }
    }
}

fn main() {
    let mut probe = Probe {
        ready: RoundRobin::new(),
        counters: Counters::default(),
    };
    probe.ready.register(TaskId(7)).unwrap();
    probe.counters.on_timer_tick();
    assert_eq!(probe.ready.select_next(), Some(TaskId(7)));

    let snapshot = probe.kernel_snapshot();
    assert_eq!(snapshot.memory.map(MemorySnapshot::heap_free_bytes), Some(10));
    assert_eq!(snapshot.runnable_tasks, 1);
    assert_eq!(snapshot.counters.timer_ticks(), 1);
    assert_eq!(snapshot.counters.context_switches(), 0);
}
```

Depuis la copie de travail de `foundation/`, avec Rust 1.90.0, rustfmt et Clippy installés.

```sh
cargo +1.90.0 fmt --all -- --check
cargo +1.90.0 clippy --workspace --all-targets --offline --locked -- -D warnings
cargo +1.90.0 run --locked --offline --example reporter_probe
```

Les `assert_eq!` documentent les valeurs attendues. `unwrap()` se limite à la donnée fixe de l’exemple. Dans l’adaptateur réel, traiter explicitement `Duplicate`, `Full` et l’absence de relevé. Ne publier `Some(MemorySnapshot)` qu’après capture d’une capacité et d’une utilisation cohérentes. Le shell affiche alors une valeur ou un état « indisponible » selon le résultat.

### Chemins du raccordement sur l’image commune

Pour un raccordement au prototype d’ordonnancement, utiliser les points d’entrée de `POC/ferrum_Poc_adaptative_sheduller/src/`. Produire les relevés mémoire sous la protection de l’allocateur, près de `init_heap` dans `allocator.rs`. Dans `interrupts.rs`, conserver l’acquittement du PIC sur le chemin du timer. Définir une source de ticks par scénario, notamment quand `shell.rs` ou `ui.rs` font avancer une simulation. Appeler `record_context_switch()` après une commutation effective.

Appliquer les normes Rust au changement. Conserver `no_std` et `#![forbid(unsafe_code)]` dans la politique portable. Justifier chaque bloc `unsafe` restant dans l’adaptateur matériel avec un commentaire `SAFETY` qui décrit ses invariants. Borner le travail en IRQ et garder l’affichage hors section critique. Faire relire les unités, les erreurs et les responsabilités de verrouillage avec les responsables du shell et des métriques.

Références. `foundation/kernel-core/src/` dans l’archive et les chemins du PoC ci-dessus. [Cible Rust sans OS](https://doc.rust-lang.org/rustc/platform-support/x86_64-unknown-none.html).
