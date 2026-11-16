# Architecture des sources FerrumOS

FerrumOS est organisé comme un noyau monolithique modulaire, selon la proposition
des [schémas T02/T04](./greenlights/objecitf_obligatoire_2/pdf/02_FerrumOS_Architecture_T02_T04.pdf).
Le projet commun utilise un paquet Cargo `ferrumos`, une bibliothèque `no_std` et
un répertoire `src/` à la racine. Les configurations de compilation et la chaîne
Rust se trouvent également à la racine.

Cette version prépare la structure du système complet. Les modules sont déclarés
et documentés, sans implémentation fonctionnelle ni point d'entrée de démarrage.
Les programmes futurs ne sont pas encore des exécutables séparés ; l'espace
utilisateur, les appels système et l'isolation restent à concevoir et à valider.

## Carte du code

```text
src/
├── lib.rs
├── boot/mod.rs
├── arch/
│   ├── mod.rs
│   └── x86_64/
│       ├── mod.rs
│       ├── cpu.rs
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
│   ├── serial/mod.rs
│   ├── display/mod.rs
│   ├── input/mod.rs
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

| Couche | Responsabilité à développer |
| --- | --- |
| `boot` | Recevoir les informations du bootloader et coordonner l'initialisation. Fixer l'ABI et l'ordre de démarrage lors de l'intégration d'un PoC. |
| `arch/x86_64` | Encapsuler les registres, les tables CPU, les contextes matériels, les tables de pages, les interruptions et le timer. |
| `kernel/memory` | Définir les contrats et les relevés mémoire cohérents, avec des valeurs en octets et une absence explicite de mesure. |
| `kernel/interrupts` | Définir les événements et le contrat portable de réception des ticks. |
| `kernel/scheduler` | Sélectionner les tâches prêtes selon une politique documentée, avec des cas limites et erreurs explicites. |
| `kernel/executor` | Gérer les états et la durée de vie des tâches ; demander les bascules de contexte aux mécanismes matériels. |
| `kernel/telemetry` | Définir les compteurs et assembler les relevés destinés aux consommateurs du noyau. |
| `drivers` | Piloter les périphériques : console série, affichage, entrées, stockage par blocs et carte réseau. |
| `services` | Construire les fonctionnalités de fichiers et de réseau à partir des interfaces du noyau et des pilotes. |
| `ui` | Fournir le shell et le moniteur de tâches et de ressources. |
| `programs` | Accueillir les futurs programmes ; leur format et leur modèle d'exécution seront définis avec l'espace utilisateur. |

## Frontières entre modules

Le module `kernel` interdit le code `unsafe`. Les accès matériels sont réservés
aux adaptateurs d'architecture et aux pilotes, derrière des interfaces dont les
invariants, la propriété des ressources et la synchronisation devront être
explicites. Les adaptateurs `arch::x86_64` ne sont exposés que sur cette architecture.

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
et ses tests ; elle n'est pas importée dans le squelette courant. La structure
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
Leurs huit tests ne constituent pas des tests exécutés sur ce squelette.

## Compilation et intégration progressive

Le [README racine](../README.md#getting-started) contient les commandes de
compilation et les contrôles. Rust 1.94.0 compile la bibliothèque sur l'hôte et
pour `x86_64-unknown-none`. Les fichiers générés vont dans `target/`, ignoré par Git.
Le [workflow du projet](../.github/workflows/kernel.yml) vérifie le formatage,
la compilation, Clippy et la documentation.

Les trois PoC restent indépendants dans `POC/`, exclus du workspace racine. Leur
chaîne nightly est définie sous `POC/`, avec une configuration propre au prototype
graphique. Leur code pourra être intégré module par module, en ajoutant les tests
des contrats puis les scénarios matériels sous QEMU.

Références : [organisation technique](./ORGANIZATION.md),
[normes Rust](./greenlights/objecitf_obligatoire_2/md/normes_rust.md) et
[stratégie de tests](./greenlights/objecitf_obligatoire_2/md/strategie_de_tests.md).
