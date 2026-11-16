# Ordonnanceur à priorités

## Politique de sélection

Une politique à priorités sélectionne la tâche prête de niveau le plus élevé. Le contrat définit aussi le traitement des priorités égales, le réveil, la cession volontaire et la saturation. Les politiques Unix ci-dessous fournissent des références pour comparer ces choix.

- **Comparaison Round Robin.** Rejouer la même séquence avec le sélecteur de l’archive `foundation/kernel-core/src/scheduler.rs`. Les journaux `resultats/qemu_*.stdout.log` établissent le périmètre de référence du banc autonome.
- **Comparaison adaptative.** Le simulateur `POC/ferrum_Poc_adaptative_sheduller/src/scheduler.rs` utilise trois files avec des quanta de 2, 4 et 8 ticks. `Task::simulate_tick` mesure une progression simulée. Pour comparer une exécution CPU, utiliser les traces de changement de contexte de l’image intégrée.

## Trois approches de référence

| Approche | Règle de sélection | Conséquence pour FerrumOS |
| --- | --- | --- |
| Priorités fixes et FIFO, inspirées de `SCHED_FIFO` | Sélectionner la tête de la file non vide de plus haute priorité. À priorité égale, une tâche réveillée entre en fin de file et une cession volontaire remet la tâche en fin de file. | Un ordre explicite sans quantum rend la règle testable dans la couche portable. Une tâche prête de faible priorité peut attendre indéfiniment. |
| Priorités fixes avec quantum, inspirées de `SCHED_RR` | Garder le même ordre entre niveaux et faire tourner les tâches d’un même niveau à l’expiration de leur quantum. | Partager le CPU entre tâches de même priorité exige de suivre le temps d’exécution. Les tâches moins prioritaires peuvent toujours rester en attente. |
| Priorités calculées pour le temps partagé, inspirées d’ULE de FreeBSD | Estimer l’interactivité d’après le temps de sommeil volontaire et le temps d’exécution. Ce score intervient dans le calcul de priorité des tâches en temps partagé. | Examiner cette approche si la réactivité du shell et la progression des tâches de fond deviennent des critères mesurés. Elle demande des données d’exécution supplémentaires. |

Sources primaires. [Linux man-pages 6.19, sched(7), sections SCHED_FIFO et SCHED_RR](https://man7.org/linux/man-pages/man7/sched.7.html). [Jeff Roberson, ULE, BSDCon 2003, sections 3.2–3.4](https://papers.freebsd.org/2003/BSDCon/jeff-ule_scheduler.files/jeff-ule_scheduler-paper.pdf).

## Décision d’architecture proposée

**Statut de la politique décrite.** Proposition. Consigner la règle retenue et les résultats de validation dans le registre des décisions.

**Proposition.** Adopter une priorité fixe stricte avec ordre FIFO entre tâches de même niveau. Pour le premier essai, utiliser trois niveaux nommés `Haute`, `Normale` et `Basse`, dans cet ordre, et une capacité totale fixe `N`. La priorité est fixée à l’inscription. Une tâche nouvellement exécutable rejoint la fin de la file de son niveau. La sélection consulte la tête de la meilleure file et la laisse en place. Une cession volontaire déplace la tâche en fin de file. L’exécuteur retire les tâches bloquées ou terminées. Une tâche bloquée rejoint la fin de sa file lorsqu’elle redevient exécutable.

**Raison et limite.** La règle permet de comparer les choix sur une séquence d’événements reproductible, avec la capacité fixe et les identifiants uniques du PoC. Une charge continue en haute priorité peut priver de CPU les niveaux inférieurs. Une tâche qui reste en tête peut aussi retarder ses pairs. Revoir la discipline si les traces d’intégration montrent une attente incompatible avec les besoins du shell ou des tâches de fond.

**Périmètre.** La politique portable choisit un `TaskId` parmi les tâches exécutables. L’exécuteur gère les états, la synchronisation et le changement de contexte. Le timer comptabilise ses ticks. Le compteur de changements de contexte évolue après une bascule effective. L’intégration x86_64 devra fixer les points de réévaluation et les conditions de préemption. Le prototype adaptatif sert de référence pour la gestion des files et des événements. Lors du raccordement, traiter explicitement les refus d’insertion, notamment quand une file de destination est pleine.

## Scénarios d’acceptation à exécuter

| ID | Mise en situation | Résultat attendu |
| --- | --- | --- |
| S01 | Liste vide, puis tentative d’inscription avec capacité nulle. | Sélection `None` et refus de l’inscription. |
| S02 | Inscrire `A` en priorité basse, `B` en normale, `C` en haute. | Sélection `C`. Après retrait de `C`, sélection `B`, puis `A` après retrait de `B`. |
| S03 | Inscrire `B` puis `A` au même niveau. Appeler deux fois la sélection. | `B` aux deux appels. Après cession de `B`, sélection `A`. |
| S04 | Inscrire `A` puis `B` au même niveau. Retirer `A` pour simuler un blocage, puis la réinscrire. | `B` reste en tête. `A` redevient sélectionnable après le départ de `B`. |
| S05 | Inscrire `A` en haute priorité, puis tenter d’inscrire le même `A` en basse. Remplir la capacité totale `N` et tenter d’ajouter un identifiant absent. | Refus du doublon, puis refus pour capacité atteinte. Taille et ordre conservés après chaque refus. |
| S06 | Garder une tâche haute et une basse éligibles pendant plusieurs décisions. Retirer la haute. | La haute est choisie à chaque appel jusqu’à son retrait. La basse est alors sélectionnée. |
| S07 | Sur l’image intégrée, enregistrer un tick, une sélection sans bascule, puis un changement de contexte effectif. | Les traces distinguent les trois événements. Seule la bascule incrémente le compteur de changements de contexte. |

## Enregistrer les résultats

Rejouer les mêmes arrivées de tâches sur la rotation de référence et la politique proposée. Consigner les identifiants retournés, la révision testée et les résultats de S01 à S06. Après raccordement de l’exécuteur, conserver séparément les traces QEMU de S07 et la version de l’image intégrée. Reporter la règle validée, les points de réévaluation et les liens vers les preuves dans la décision d’architecture et les fiches T02/T04.

[Registre des décisions](../objecitf_obligatoire_2/md/registre_decisions.md) · [Stratégie de tests](../objecitf_obligatoire_2/md/strategie_de_tests.md)

## Mise en œuvre proposée dans le dépôt

**Isoler la sélection portable.** L’exemple utilise `foundation/kernel-core/`, conformément au workspace du banc de référence. Une PR sur la politique peut se limiter à la bibliothèque et à ses tests hôte. Recenser les appels au sélecteur avant de modifier son interface.

| Emplacement | Travail attendu |
| --- | --- |
| `foundation/kernel-core/Cargo.toml` | Déclarer la bibliothèque Rust 2021 sans dépendance. |
| `foundation/kernel-core/src/lib.rs` | Déclarer `no_std`, interdire `unsafe` et exporter le module `scheduler`. |
| `foundation/kernel-core/src/scheduler.rs` | Ajouter le contrat de sélection, le type de priorité, les erreurs et les tests du module. |
| `POC/ferrum_Poc_adaptative_sheduller/src/scheduler.rs` | Relire les appels `spawn`, `tick` et `snapshot`. Ils servent de points de repère pour la PR d’intégration. Le prototype exécute `Task::simulate_tick` sur trois files et mesure ses événements simulés. |
| `POC/ferrum_Poc_adaptative_sheduller/src/interrupts.rs` | Examiner l’appel `scheduler::tick()` dans le gestionnaire timer et l’acquittement. Documenter l’ordre retenu avant toute modification matérielle. |

### Contrat du premier incrément

La capacité `N` est globale et fixée à la compilation. Les `len` premières cases contiennent les tâches éligibles dans l’ordre d’inscription ou de réinscription. Les suivantes sont libres. Chaque `TaskId` apparaît une seule fois. La valeur numérique `0` correspond au niveau `High`, qui précède `Normal` puis `Low`. Un identifiant dupliqué ou une file pleine renvoie une erreur et conserve l’état. Pour `N = 0`, une inscription échoue et la sélection retourne `None`.

Le tableau conserve l’ordre FIFO à chaque niveau. `min_by_key` retourne le premier élément lorsque les clés minimales sont égales. La sélection conserve l’ordre. Le retrait décale les cases restantes. Lors d’un réveil, `register` ajoute la tâche en fin de tableau. Une cession volontaire déplace la tâche en fin de tableau et garde son niveau. Avec une tâche haute continuellement prête, une tâche basse peut rester en attente. Les opérations de sélection, de retrait et de cession parcourent au plus `N` éléments. Consigner cette limite et mesurer le coût avant tout appel depuis un chemin d’interruption. [Documentation Rust de `min_by_key`](https://doc.rust-lang.org/core/iter/trait.Iterator.html#method.min_by_key)

### Fichiers de départ à copier dans la branche

Dans `foundation/kernel-core/Cargo.toml`.

```toml
[package]
name = "ferrumos-kernel-core"
version = "0.1.0"
edition = "2021"
```

Dans `foundation/kernel-core/src/lib.rs`.

```rust
#![no_std]
#![forbid(unsafe_code)]

pub mod scheduler;
```

Dans `foundation/kernel-core/src/scheduler.rs`. Conserver les éventuels autres modules de la bibliothèque lorsqu’ils existent dans la branche.

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TaskId(pub u64);

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Priority {
    High = 0,
    Normal = 1,
    Low = 2,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RegisterError {
    Duplicate,
    Full,
}

#[derive(Clone, Copy)]
struct ReadyTask {
    id: TaskId,
    priority: Priority,
}

/// Liste de capacité globale N, en ordre FIFO au sein de chaque priorité.
pub struct PriorityScheduler<const N: usize> {
    ready: [Option<ReadyTask>; N],
    len: usize,
}

impl<const N: usize> Default for PriorityScheduler<N> {
    fn default() -> Self {
        Self {
            ready: [None; N],
            len: 0,
        }
    }
}

impl<const N: usize> PriorityScheduler<N> {
    /// Ajoute une tâche en fin de file et conserve l'état en cas d'erreur.
    pub fn register(&mut self, id: TaskId, priority: Priority) -> Result<(), RegisterError> {
        if self.index_of(id).is_some() {
            return Err(RegisterError::Duplicate);
        }
        if self.len == N {
            return Err(RegisterError::Full);
        }
        self.ready[self.len] = Some(ReadyTask { id, priority });
        self.len += 1;
        Ok(())
    }

    /// Retourne l'identifiant de la prochaine tâche éligible sans modifier la file.
    pub fn select_next(&self) -> Option<TaskId> {
        self.ready[..self.len]
            .iter()
            .flatten()
            .min_by_key(|task| task.priority)
            .map(|task| task.id)
    }

    /// Retire la tâche après blocage ou fin. Retourne false si elle est absente.
    pub fn remove(&mut self, id: TaskId) -> bool {
        let Some(index) = self.index_of(id) else {
            return false;
        };
        self.ready.copy_within(index + 1..self.len, index);
        self.len -= 1;
        self.ready[self.len] = None;
        true
    }

    /// Replace la tâche qui cède en fin de sa priorité.
    pub fn yield_now(&mut self, id: TaskId) -> bool {
        let Some(index) = self.index_of(id) else {
            return false;
        };
        let task = self.ready[index];
        self.ready.copy_within(index + 1..self.len, index);
        self.ready[self.len - 1] = task;
        true
    }

    fn index_of(&self, id: TaskId) -> Option<usize> {
        self.ready[..self.len]
            .iter()
            .position(|slot| slot.is_some_and(|task| task.id == id))
    }
}
```

Ajouter ces premiers tests en bas du même fichier. Poursuivre avec les scénarios S01 à S06 de la fiche principale.

```rust
#[cfg(test)]
mod tests {
    use super::{Priority, PriorityScheduler, RegisterError, TaskId};

    #[test]
    fn higher_priority_preserves_fifo_of_other_tasks() {
        let mut scheduler = PriorityScheduler::<3>::default();
        let first = TaskId(2);
        let second = TaskId(1);
        let urgent = TaskId(3);
        assert_eq!(scheduler.register(first, Priority::Normal), Ok(()));
        assert_eq!(scheduler.register(second, Priority::Normal), Ok(()));
        assert_eq!(scheduler.select_next(), Some(first));
        assert_eq!(scheduler.register(urgent, Priority::High), Ok(()));
        assert_eq!(scheduler.select_next(), Some(urgent));
        assert!(scheduler.remove(urgent));
        assert_eq!(scheduler.select_next(), Some(first));
        assert_eq!(scheduler.select_next(), Some(first));
        assert!(scheduler.yield_now(first));
        assert_eq!(scheduler.select_next(), Some(second));
    }

    #[test]
    fn capacity_and_duplicate_failures_preserve_the_queue() {
        let mut scheduler = PriorityScheduler::<1>::default();
        let existing = TaskId(1);
        let rejected = TaskId(2);
        assert_eq!(scheduler.register(existing, Priority::Low), Ok(()));
        let duplicate = scheduler.register(existing, Priority::High);
        assert_eq!(duplicate, Err(RegisterError::Duplicate));
        assert_eq!(scheduler.register(rejected, Priority::High), Err(RegisterError::Full));
        assert_eq!(scheduler.select_next(), Some(existing));
        assert!(scheduler.remove(existing));
        assert_eq!(scheduler.select_next(), None);
        assert_eq!(scheduler.register(rejected, Priority::High), Ok(()));
        assert_eq!(scheduler.select_next(), Some(rejected));
    }

    #[test]
    fn zero_capacity_has_no_eligible_task() {
        let mut scheduler = PriorityScheduler::<0>::default();
        assert_eq!(scheduler.select_next(), None);
        assert_eq!(scheduler.register(TaskId(1), Priority::Low), Err(RegisterError::Full));
        assert!(!scheduler.remove(TaskId(1)));
        assert!(!scheduler.yield_now(TaskId(1)));
    }
}
```

### Vérification et normes de la PR

Dans `foundation/kernel-core/`, avec Rust, rustfmt et Clippy installés. Utiliser la version de Rust validée par l’équipe. La campagne archivée utilisait Rust 1.90.0. Les commandes ci-dessous portent sur la bibliothèque portable.

```sh
rustc --version
cargo fmt --all
cargo fmt --all -- --check
cargo check --offline
cargo test --offline
cargo clippy --offline --all-targets -- -D warnings
```

Appliquer la proposition T16 du dépôt. Nommer les types en `UpperCamelCase`, les fonctions en `snake_case` et les constantes en `SCREAMING_SNAKE_CASE`. Documenter l’ordre des priorités et le contrat de capacité. Réserver `Result` aux échecs et `Option` à l’absence attendue. Rester en `no_std` dans la bibliothèque. Dans la couche matérielle, justifier chaque `unsafe` par un commentaire `SAFETY` qui indique la précondition. Faire relire la règle, les scénarios d’échec et l’impact sur les appelants. Archiver la révision, la toolchain, la commande et son résultat. [Normes Rust proposées par FerrumOS](../objecitf_obligatoire_2/md/normes_rust.md)

### Raccordement au CPU dans une PR distincte

1. Choisir le PoC cible avec l’équipe. Brancher la bibliothèque comme dépendance locale après la validation des tests hôte. L’exécuteur possède les états, les contextes et la synchronisation. Lui faire appeler le sélecteur aux points définis avec l’équipe, par exemple lors d’un blocage, d’une fin ou d’une cession volontaire.
2. À chaque réveil, réinscrire la tâche selon son niveau. Traiter `Full` et `Duplicate` à l’appel. Pour une réinsertion dans le simulateur, s’assurer que la tâche reste comptabilisée si la destination est pleine. Choisir ensuite le mécanisme de réévaluation en présence d’une nouvelle tâche haute. Le chemin de l’IRQ et son acquittement exigent une revue dédiée.
3. Distinguer dans la télémétrie `tick`, `TaskId` sélectionné et contexte effectivement changé. Un compteur de sélection mesure une décision. Le compteur de changements de contexte avance après la bascule. Préparer le scénario S07 avec traces série de ces trois événements. La preuve CPU dépend de l’intégration de l’exécuteur.
4. Depuis `POC/ferrum_Poc_adaptative_sheduller/`, sur l’environnement Garuda ou Arch configuré selon le README, utiliser `make build`, puis `make run` pour l’affichage VGA. `make run-serial` lance QEMU sans fenêtre et expose le port série. Ajouter une instrumentation série dans la PR d’intégration pour que ce mode puisse fournir les traces S07.

Joindre à la PR la décision d’architecture, les traces de sélection et les résultats des tests portables. Rattacher les traces QEMU à la révision de l’image testée. [Manuel Linux `sched(7)` pour la discipline FIFO](https://man7.org/linux/man-pages/man7/sched.7.html)
