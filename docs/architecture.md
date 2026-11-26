# Architecture des sources FerrumOS

FerrumOS est organisé comme un noyau monolithique modulaire, selon la proposition
des [schémas T02/T04](./greenlights/objecitf_obligatoire_2/pdf/02_FerrumOS_Architecture_T02_T04.pdf).
Le projet commun utilise un paquet Cargo `ferrumos`, une bibliothèque `no_std`,
un exécutable de démarrage et un répertoire `src/` à la racine. Les configurations
de compilation et la chaîne Rust se trouvent également à la racine.

Le démarrage BIOS, l'affichage framebuffer, les diagnostics série et la saisie
clavier utilisent ces modules. Le démarrage mappe un tas réutilisable puis installe
l'IDT et le timer PIT. La boucle de démarrage confie la saisie à une tâche
coopérative de priorité haute. Une tâche de diagnostic de priorité normale termine
après son premier passage. La console édite et reproduit les lignes.
L'espace utilisateur, les appels système et l'isolation restent à concevoir et à
valider. Le [guide de démarrage](./boot.md) précise la configuration testée.

## Carte du code

```text
src/
├── lib.rs
├── main.rs
├── boot/
│   ├── mod.rs
│   └── runtime.rs
├── arch/
│   ├── mod.rs
│   └── x86_64/
│       ├── mod.rs
│       ├── cpu.rs
│       ├── io.rs
│       ├── memory.rs
│       ├── memory/       frames.rs, paging.rs et heap.rs
│       ├── interrupts.rs
│       └── interrupts/   hardware.rs et entry.S
├── kernel/
│   ├── mod.rs
│   ├── memory/mod.rs
│   ├── interrupts/mod.rs
│   ├── scheduler/mod.rs
│   ├── executor/mod.rs
│   └── telemetry/mod.rs
├── drivers/
│   ├── mod.rs
│   ├── console.rs
│   ├── serial/           mod.rs et uart.rs
│   ├── display/mod.rs
│   ├── input/            mod.rs et ps2.rs
│   ├── storage/mod.rs
│   └── network/mod.rs
├── services/
│   ├── mod.rs
│   ├── filesystem/mod.rs
│   └── network/mod.rs
├── ui/
│   ├── mod.rs
│   ├── shell/mod.rs
│   └── monitor/mod.rs
└── programs/mod.rs
```

| Couche | Responsabilité actuelle ou à développer |
| --- | --- |
| `main.rs`, `boot` | Demander au bootloader le mapping physique, recevoir le `BootInfo`, initialiser COM1, le tas, le framebuffer, PS/2 et le timer, puis exécuter les tâches coopératives. Le point d'entrée inclut le traitement terminal des paniques. |
| `arch/x86_64` | Valider les régions physiques, attribuer leurs frames entières de 4 Kio et mapper le tas. Protéger l'allocateur, installer l'IDT et le PIC/PIT, acquitter IRQ0, préserver le drapeau IF dans les sections critiques et attendre une interruption avec `sti; hlt`. |
| `kernel/memory` | `MemorySnapshot` valide une capacité et une utilisation en octets. `MemoryReporter` fournit un relevé cohérent ou `None` quand il est indisponible. |
| `kernel/interrupts` | `TimerTickSink` décrit la réception directe d'événements, notamment en simulation. `TickSnapshot` convertit un relevé et la fréquence rationnelle du timer en durée entière. |
| `kernel/scheduler` | `PriorityScheduler<N>` choisit `High`, puis `Normal`, puis `Low`. À priorité égale, l'ordre FIFO évolue sur cession ou réveil. Les doublons et dépassements de capacité sont refusés sans mutation. |
| `kernel/executor` | `CooperativeExecutor<C, N>` conserve les fonctions et leur état. Un pas rend `Yield`, `Blocked` ou `Complete`. L'appel emprunte le contexte et utilise la pile du CPU appelant. |
| `kernel/telemetry` | Conserver des comptes distincts pour les appels aux tâches, les interruptions timer livrées et les bascules CPU. Le fournisseur de `KernelSnapshot` assemble un relevé copié. `CapturedKernel` capture les sources réelles une fois, `UnwiredKernel` répond tant qu’aucune source n’est raccordée. |
| `drivers/serial`, `display`, `input`, `console.rs` | Piloter COM1 et PS/2, rendre le texte dans le framebuffer et conserver une ligne UTF-8 de capacité fixe. Les opérations matérielles restent hors des tests hôte. |
| `drivers/storage`, `network` | Réserver les pilotes de stockage par blocs et de carte réseau. |
| `services` | Construire les fonctionnalités de fichiers et de réseau à partir des interfaces du noyau et des pilotes. |
| `ui` | Fournir le shell et le moniteur de tâches et de ressources. Le moniteur rend le relevé en texte, distingue une mesure indisponible d’un compteur à zéro et le sert au shell par `StatsProvider`. |
| `programs` | Accueillir les futurs programmes ; leur format et leur modèle d'exécution seront définis avec l'espace utilisateur. |

## Frontières entre modules

Le module `kernel` interdit le code `unsafe`. Les accès matériels sont réservés
aux adaptateurs d'architecture et aux pilotes, derrière des interfaces dont les
invariants, la propriété des ressources et la synchronisation devront être
explicites. Les adaptateurs `arch::x86_64` ne sont exposés que sur cette architecture.

Le chemin matériel de démarrage est compilé uniquement pour `x86_64-unknown-none`.
La feature `baremetal` active l'exécutable ; la bibliothèque et les éléments
portables restent testables sur l'hôte. Le constructeur du clavier et celui du
port série exigent un propriétaire exclusif. Le framebuffer est emprunté de façon
mutable depuis le `BootInfo`, sans singleton global.

Le démarrage conserve IF à zéro pendant l'initialisation des ressources partagées.
Il installe ensuite l'IDT, reprogramme les PIC et le PIT, démasque uniquement IRQ0,
puis active les interruptions. Le handler timer incrémente un compteur atomique
saturant et acquitte le PIC maître. Il garde l'allocation, les verrous et l'affichage
dans l'exécution normale. Le clavier reste scruté par la tâche de saisie. Après chaque
tour coopératif, `sti; hlt` attend le prochain réveil matériel.

Les composants appartiennent aujourd'hui à une même bibliothèque. La séparation
en modules organise les responsabilités ; elle ne constitue ni une frontière de
protection mémoire, ni une séparation en paquets Cargo. Les règles de dépendance
doivent être conservées lors des prochaines implémentations :

- Les politiques et les contrats du noyau ne dépendent pas des interfaces
  utilisateur ni des détails de registre ou de périphérique.
- L'ordonnanceur choisit une tâche ; l'exécuteur gère sa progression et demande
  une bascule à `arch`. Un tick, une sélection et une bascule effective sont trois
  événements distincts.
- Les pilotes gèrent les périphériques. `drivers/network` concerne la carte
  réseau, `services/network` les protocoles et services. De même,
  `drivers/storage` fournit l'accès au périphérique et `services/filesystem`
  organise les fichiers.
- Le shell et le moniteur consomment les contrats des services et du noyau. Les
  relevés doivent être capturés de façon cohérente avant leur affichage, hors des
  sections critiques.

## Correspondance avec la documentation historique

Les documents Greenlight font référence à une archive de preuves organisée sous
`foundation/kernel-core/src/`. Cette archive décrivait une bibliothèque portable
et ses tests ; elle n'est pas importée dans le projet commun. La structure
ci-dessous reprend ses responsabilités avec les chemins du projet actuel.

| Référence historique ou bloc T02/T04 | Emplacement actuel |
| --- | --- |
| `kernel-core/src/memory.rs` | `src/kernel/memory/mod.rs` |
| `kernel-core/src/interrupts.rs` | `src/kernel/interrupts/mod.rs` |
| `kernel-core/src/scheduler.rs` | `src/kernel/scheduler/mod.rs` |
| `kernel-core/src/telemetry.rs` | `src/kernel/telemetry/mod.rs` |
| Démarrage et pilotes | `src/boot/` et `src/drivers/` |
| Adaptateurs x86_64 | `src/arch/x86_64/` |
| Exécuteur CPU | `src/kernel/executor/`, avec les mécanismes dans `src/arch/x86_64/cpu.rs` |
| Shell et monitoring | `src/ui/shell/` et `src/ui/monitor/` |

Les [contrats archivés](./greenlights/objecitf_obligatoire_2/md/contrats_noyau.md)
servent de référence pour les unités mémoire et les observations. Les modules
de `src/kernel/` possèdent leurs tests dans le paquet racine. La politique du
noyau commun utilise des priorités fixes.

## Cycle d'une tâche coopérative

L'exécuteur enregistre la tâche avec un identifiant, une priorité et une fonction
`fn(&mut C) -> TaskStatus`. Il choisit un identifiant prêt et appelle la fonction.
`Yield` conserve la tâche en fin de file de sa priorité. `Blocked` retire la tâche
de la file prête et conserve sa fonction. `wake` la réinscrit en fin de file.
`Complete` libère l'identifiant et l'emplacement du registre.

Une fonction doit rendre la main après un travail borné. Une tâche haute qui
reste prête peut retarder les priorités basses. La boucle de démarrage réveille
la tâche clavier à chaque tour de scrutation et borne le nombre de dispatchs du
tour. Le timer réveille le CPU entre les tours. Chaque appel incrémente
`dispatches`, tandis que les compteurs de ticks et de bascules CPU gardent leur
sens propre.

Les régions mémoire physiques proviennent du `BootInfo`. Le résumé de démarrage
compte leur capacité initiale après validation des bornes et des recouvrements.
Le curseur de frames attribue chaque page utilisable entière une fois pendant
l'initialisation du tas. `OffsetPageTable` accède aux tables actives via CR3 et le
mapping physique fourni par le bootloader. Le tas réserve 1 Mio à l'adresse
virtuelle `0x4444_4444_0000`, avec des pages présentes, inscriptibles et non
exécutables. Un échec partiel conserve les frames et mappings déjà attribués et
interdit une seconde initialisation jusqu'au redémarrage.

Toute extension de l'allocation physique doit préserver la propriété des frames
déjà consommées par le tas et ses nouvelles tables de pages. Elle doit reprendre
leur état d'attribution ou les exclure explicitement de son pool. Le résumé
`BootMemorySummary` demeure une capacité initiale de la carte bootloader, et le
relevé du tas décrit uniquement sa région de 1 Mio. Une mesure de RAM physique
libre courante exigera le suivi complet de ces attributions.

## Consommer le tas et l'horloge

`arch::x86_64::memory::heap()` fournit le tas partagé après son initialisation.
`KernelHeap::allocate(Layout)` rend un `HeapBlock` dont les octets sont initialisés
à zéro. `as_mut_slice()` exige un emprunt mutable du handle. `Drop` restitue le
pointeur et le layout d'origine à `ferrum_alloc::LinkedListHeap`. Plusieurs handles
peuvent vivre simultanément. Chaque verrou du tas désactive temporairement les
interruptions et restaure le drapeau IF de l'appelant.

`MemoryReporter::memory_snapshot()` capture capacité, utilisation et espace libre
sous ce verrou. L'utilisation inclut l'arrondi de l'allocateur et l'alignement.
`KernelHeap::stats()` rend `Option<HeapStats>`, dont `live_bytes` donne séparément
les tailles demandées encore vivantes. Capturer les valeurs, libérer le verrou,
puis formater le relevé.
Le résumé physique initial et le relevé du tas conservent leurs unités et leur
périmètre respectifs.

`arch::x86_64::interrupts::clock_snapshot()` capture le nombre de ticks livrés et
la configuration du PIT dans un `TickSnapshot`. La fréquence vaut exactement
`1_193_182 / 11_932` Hz, proche de 100 Hz. `uptime_millis()` utilise cette fraction
et arrondit vers le bas. Une longue période avec IF à zéro peut perdre des
événements matériels. Le compteur représente donc une durée issue du timer,
avec les contraintes de livraison de ses IRQ.

Le compteur atomique du handler IRQ0, accessible par `tick_count()`, est la
source unique des ticks du noyau intégré. Pour `stats`, le fournisseur lit
`clock_snapshot()` une fois, puis construit les compteurs du `KernelSnapshot`
avec `executor.counters().with_timer_ticks(clock.ticks())`. Cette opération
copie le total observé et préserve les compteurs d'exécution. Le fournisseur
capture les données sous sa synchronisation, puis le moniteur les formate hors
section critique. Il utilise la fréquence d'entrée et le diviseur du même
`TickSnapshot` pour afficher la durée associée aux ticks du relevé. Un total de
zéro est valide lorsque le timer est configuré et n'a encore livré aucune IRQ.

La boucle de démarrage applique cette règle après les deux relevés périodiques.
Elle construit un `CapturedKernel` depuis le tas, l'ensemble des tâches prêtes et
ces compteurs, puis écrit le rapport entre les marqueurs `STATS_BEGIN` et
`STATS_END` par l'interface `StatsProvider` du shell. Le test de démarrage y
exige sept mesures, aucune valeur indisponible, la capacité du tas mappé et une
durée conforme au total de ticks du rapport lui-même.

## Compilation et intégration progressive

Le [README racine](../README.md#getting-started) contient les commandes de
compilation et les contrôles. Rust 1.94.0 compile la bibliothèque et le noyau pour
la cible intégrée `x86_64-unknown-none`. Le programme hôte indépendant
`tools/image-builder/`, épinglé sur `nightly-2026-03-07`, construit l'image BIOS
avec `bootloader 0.11.15`. `scripts/boot.py` orchestre la construction et QEMU.
Les fichiers générés vont dans `target/`, ignoré par Git.

Le [workflow du projet](../.github/workflows/kernel.yml) vérifie le formatage,
la compilation, Clippy, la documentation, les tests portables et le démarrage
avec saisie sous QEMU. Les critères du test matériel et ses limites sont décrits
dans le [guide de démarrage](./boot.md#contrôles-et-preuves).

Les trois PoC, `libs/ferrum_alloc`, `research/heap-allocator/bench` et le constructeur
d'image restent exclus du workspace racine. Chacun conserve son manifeste et ses
contrôles. Les PoC utilisent leurs configurations nightly sous `POC/` ; les
configurations locales du scheduler et du prototype graphique restent prioritaires.
Leur code pourra être intégré module par module, avec les tests des contrats puis
les scénarios matériels correspondants.

Références : [organisation technique](./ORGANIZATION.md),
[normes Rust](./greenlights/objecitf_obligatoire_2/md/normes_rust.md) et
[stratégie de tests](./greenlights/objecitf_obligatoire_2/md/strategie_de_tests.md).
