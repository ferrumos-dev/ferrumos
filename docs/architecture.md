# Architecture des sources FerrumOS

FerrumOS est organisé comme un noyau monolithique modulaire, selon la proposition
des [schémas T02/T04](./greenlights/objecitf_obligatoire_2/pdf/02_FerrumOS_Architecture_T02_T04.pdf).
Le projet commun utilise un paquet Cargo `ferrumos`, une bibliothèque `no_std`,
un exécutable de démarrage et un répertoire `src/` à la racine. Les configurations
de compilation et la chaîne Rust se trouvent également à la racine.

Le démarrage BIOS, l'affichage framebuffer, les diagnostics série et la saisie
clavier sont implémentés dans ces modules. Les autres couches restent des points
d'intégration. La console édite et reproduit les lignes sans exécuter de commandes.
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
│       └── interrupts.rs
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
| `main.rs`, `boot` | Recevoir le `BootInfo` de `bootloader_api 0.11.15`, initialiser COM1, le framebuffer puis PS/2, et exécuter la boucle de saisie. Le point d'entrée inclut le traitement terminal des paniques. |
| `arch/x86_64` | Encapsuler actuellement les ports d'E/S, la désactivation des interruptions et l'arrêt du CPU. Les tables CPU, contextes, tables de pages et handlers IRQ restent à intégrer. |
| `kernel/memory` | Définir les contrats et les relevés mémoire cohérents, avec des valeurs en octets et une absence explicite de mesure. |
| `kernel/interrupts` | Définir les événements et le contrat portable de réception des ticks. |
| `kernel/scheduler` | Sélectionner les tâches prêtes selon une politique documentée, avec des cas limites et erreurs explicites. |
| `kernel/executor` | Gérer les états et la durée de vie des tâches ; demander les bascules de contexte aux mécanismes matériels. |
| `kernel/telemetry` | Définir les compteurs et assembler les relevés destinés aux consommateurs du noyau. |
| `drivers/serial`, `display`, `input`, `console.rs` | Piloter COM1 et PS/2, rendre le texte dans le framebuffer et conserver une ligne UTF-8 de capacité fixe. Les opérations matérielles restent hors des tests hôte. |
| `drivers/storage`, `network` | Réserver les pilotes de stockage par blocs et de carte réseau. |
| `services` | Construire les fonctionnalités de fichiers et de réseau à partir des interfaces du noyau et des pilotes. |
| `ui` | Fournir le shell et le moniteur de tâches et de ressources. |
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

La boucle de démarrage conserve les interruptions masquables désactivées et lit
le clavier par scrutation. Elle n'installe ni timer ni ordonnanceur : cette étape
sera raccordée par le pôle noyau. Les détails de cette transition figurent dans
les [interfaces de reprise](./boot.md#interfaces-pour-la-suite-du-projet).

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
et leurs résultats restent des références pour les prochaines implémentations.
Leurs huit tests ne constituent pas des tests exécutés sur le noyau courant.

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
