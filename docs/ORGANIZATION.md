# FerrumOS — Document d'Organisation Technique

> Document de référence pour le pilotage du projet **FerrumOS** et, avant tout, pour la
> **répartition détaillée du travail** entre les 4 membres de l'équipe.
>
> Il constitue l'enrichissement complet de `docs/eip/repartitions.txt` : chaque périmètre y est
> décliné en tâches concrètes, ordonnées et rattachées à un responsable, en s'appuyant sur
> l'ensemble des livrables du dépôt (dossiers, présentations, PoC).
>
> **Objectif du livrable (premier suivi EIP)** — répondre aux 5 points attendus :
> 1. Pitch du projet · 2. État d'avancement · 3. Objectifs à 1 an ·
> 4. Organisation de l'équipe · 5. Engagements de chacun d'ici le prochain suivi (octobre).
>
> Sources consolidées :
> - `docs/eip/repartitions.txt` → Répartition technique des membres (**base de ce document**)
> - `docs/dossiers/FerrumOS_Dossier_1_Besoins_Détaillé.pdf` → Besoins & périmètre (MVP, MoSCoW)
> - `docs/dossiers/FerrumOS_Dossier_2_Contexte_Détaillé.pdf` → Contexte, choix Rust, sécurité
> - `docs/dossiers/FerrumOS_Dossier_3_Architecture_Détaillé.pdf` → Architecture en couches, mémoire, ordonnancement
> - `docs/dossiers/FerrumOS_Dossier_4_Chiffrage_Détaillé.pdf` → CAPEX/OPEX, ressources
> - `docs/dossiers/FerrumOS_Dossier_5_Risques_Détaillé.pdf` → Risques, Green IT, résilience
> - `docs/presentations/FerrumOS_Presentations_Summary.md` → Présentation générale + EIP Bloc 1
> - `POC/` → 3 proof of concepts (`ferrumos_PoC`, `ferrum_Poc_adaptative_sheduller`, `ferrum_Poc_graphic`)

---

Pour le suivi Greenlight, consulter [l’index des documents](./greenlights/README.md). Les [règles de contribution](../CONTRIBUTING.md) indiquent où documenter un changement et comment joindre ses preuves. L’ordonnancement proposé pour le noyau commun est décrit dans la [fiche à priorités](./greenlights/objecitf_obligatoire_1/ordonnancement_a_priorites.md). La répartition et les jalons historiques du premier suivi figurent ci-dessous.

## 1. Pitch du projet

**FerrumOS** est un système d'exploitation développé **intégralement en Rust**, dans
une logique **pédagogique et expérimentale**. L'objectif n'est pas de concurrencer les
OS existants mais de **maîtriser en profondeur** les couches fondamentales d'un OS
moderne : démarrage (bootloader), gestion mémoire (paging x86_64), interruptions (IDT),
ordonnancement et architecture noyau.

La sécurité mémoire native de Rust (ownership / borrowing) est utilisée comme un
**avantage architectural** dès la conception, avec une limitation stricte des blocs
`unsafe` aux zones critiques (IRQ, changement de contexte, accès mémoire physique).

> **Analogie (vulgarisation)** : FerrumOS aujourd'hui, ce sont les fondations d'une
> maison — béton coulé, murs porteurs montés, charpente posée. Sans ces fondations,
> rien de ce qui vient ensuite ne tient.

---

## 2. État d'avancement actuel

Le projet dispose déjà de **3 Proof of Concepts** fonctionnels qui valident les briques clés :

| PoC | Rôle | Contenu clé |
|-----|------|-------------|
| `ferrumos_PoC/` | **PoC de référence (MVP)** | Boot (`bootloader` + `cargo bootimage`), VGA texte, IDT (timer/clavier), paging + Bump Allocator, shell CLI (`help`, `version`, `mem`) |
| `ferrum_Poc_adaptative_sheduller/` | **Ordonnancement adaptatif** | Base kernel + `scheduler.rs` + `ui.rs`, écran de bienvenue VGA, automatisation QEMU via `Makefile` |
| `ferrum_Poc_graphic/` | **Rendu graphique** | Pipeline moderne `bootloader 0.11` + `bootloader_api`, framebuffer linéaire, desktop, barre de recherche, souris + clavier |

**MVP validé** (Dossier 1 + EIP) :
- Boot stable et reproductible sous QEMU
- Initialisation mémoire sans erreur critique
- Interruptions matérielles opérationnelles
- Shell CLI avec commandes `help` / `version` / `mem`

> Les 3 PoC sont aujourd'hui séparés. Un premier chantier transverse consiste à
> **converger vers une base de code unique** (kernel + scheduler + entrées) sur laquelle
> chaque pôle greffera son périmètre — voir §5.

---

## 3. Objectifs à 1 an

Aligné sur l'étude prospective (EIP B1-C05-1), horizon **H2 (6–18 mois)** :

- Espace utilisateur isolé (Ring 3)
- Table de syscalls de type POSIX
- Support ELF pour charger des binaires
- Driver réseau minimal (e1000)
- Scheduler à priorités dynamiques (au-delà du Round Robin du MVP)
- Système de fichiers FAT16 minimal
- Gestionnaire de tâches temps réel + statistiques ressources (CPU/RAM/stockage)
- Tests unitaires kernel automatisés en CI/CD

> Objectif image : un espace utilisateur isolé, des processus qui tournent côte à côte,
> un vrai loader de binaires → **FerrumOS devient expérimentable**.

---

## 4. Stack technique & architecture

| Outil | Rôle | Justification |
|-------|------|---------------|
| **Rust nightly** | Langage & compilation noyau | Sécurité mémoire (ownership), `no_std`, `build-std`, cible `x86_64-unknown-none` |
| **cargo-bootimage / bootloader** | Image bootable | Génération d'image bootable depuis le source Rust |
| **QEMU x86_64** | Exécution & validation | Isolation, reproductibilité, debug facilité |
| **GDB** | Débogage noyau | Inspection registres, mémoire, IDT en live |
| **Git + GitHub + CI/CD** | Versioning & intégration | Traçabilité par composant, validation automatique |
| **Make** | Automatisation build/boot | Cibles `install`, `boot`, `build`, `run`, `run-debug`, `run-serial` |
| **Target custom** | `x86_64-ferrumos.json` | Spécification bare-metal du noyau |

**Architecture en couches (modèle C4)** — chaque couche est rattachée à un pôle (voir §5) :

```
+---------------------------+
|        Programmes         |  Applications utilisateur
+---------------------------+
|         Services          |  Fichiers, réseau (ajout progressif)      → Pôle 1 / 2
+---------------------------+
|          UI / Shell       |  Shell CLI, gestionnaire de tâches, stats  → Pôle 3 / 4
+---------------------------+
|          Drivers          |  Clavier, souris, VGA, framebuffer         → Pôle 1
+---------------------------+
|          Kernel           |  CPU | Mémoire (paging) | IRQ | Scheduler   → Pôle 2
+---------------------------+
|        Bootloader         |  Initialisation matérielle                 → Pôle 1
+---------------------------+
```

- **Mémoire** : paging x86_64, tables 4 niveaux, isolation stricte, allocateur Bump personnalisé (→ évolution allocateur plus riche).
- **Ordonnancement** : Round Robin (quantum + préemption timer) → évolution priorités dynamiques.

---

## 5. Organisation de l'équipe & répartition détaillée du travail

L'équipe compte **4 membres**, chacun responsable d'un **pôle technique** dérivé de
`docs/eip/repartitions.txt`. Cette section décline chaque pôle en **tâches concrètes**,
priorisées et testables, et explicite les **collaborations** entre pôles.

### 5.1 Principes de répartition

- **Un responsable unique par périmètre** pour éviter les chevauchements, mais des
  **interfaces claires** entre pôles (voir §5.6 collaborations).
- **Périmètres bas niveau autonomes** (boot, drivers, kernel) isolés des périmètres
  UI/monitoring, qui se branchent dessus via des API stables.
- **Travail en binôme** sur les briques transverses (IA d'optimisation, gestionnaire de
  tâches) pour sécuriser la montée en compétence et la qualité.

### 5.2 Pôle 1 — Benjamin Ale · Cœur système bas niveau & compatibilité

> Périmètre `repartitions.txt` : *Technique kernel, bootloader, drivers, compatibilité VM, IA d'opti.*

| # | Tâche | Livrable / critère de validation |
|---|-------|----------------------------------|
| 1.1 | Consolider le pipeline de boot (`bootloader` / `bootimage`, cible custom) | Boot reproductible sous QEMU, doc de build |
| 1.2 | Drivers d'entrée : clavier PS/2, souris | Événements clavier/souris fiables dans le shell/desktop |
| 1.3 | Drivers d'affichage : VGA texte + framebuffer linéaire | Sortie stable en mode texte et graphique |
| 1.4 | Base driver disque (préparation FAT16) + amorce driver réseau (e1000) | Lecture bloc disque de test ; détection carte e1000 |
| 1.5 | Compatibilité VM & bare-metal : matrice QEMU, cibles Make (`run`, `run-debug`, `run-serial`) | Matrice de compatibilité documentée |
| 1.6 | Support ELF / loader de binaires (partie bas niveau) | Chargement d'un binaire ELF minimal |
| 1.7 | Revue et documentation des blocs `unsafe` (IRQ, context switch, accès phys.) | Inventaire `unsafe` justifié |
| 1.8 | Instrumentation bas niveau (compteurs perf/métriques kernel) pour l'IA d'opti | Hooks exposés au Pôle 4 |

### 5.3 Pôle 2 — Mathias Desgranges · Kernel & fondations techniques

> Périmètre `repartitions.txt` : *Technique bas niveau kernel, autre technique.*

| # | Tâche | Livrable / critère de validation |
|---|-------|----------------------------------|
| 2.1 | Gestion mémoire : paging 4 niveaux, isolation noyau/utilisateur | Isolation vérifiée, pas de fuite entre espaces |
| 2.2 | Évolution de l'allocateur (Bump → linked-list / fixed-size blocks) | Alloc/désalloc dynamique testée |
| 2.3 | Interruptions : IDT, PIC/APIC, timer, gestion robuste des exceptions | Aucune triple-fault sur cas nominaux |
| 2.4 | Scheduler : Round Robin → **priorités dynamiques** (quantum adaptatif) | Ordonnancement priorisé démontré sous QEMU |
| 2.5 | Framework de **tests unitaires kernel** (`no_std`) intégré à la CI | Tests verts en CI/CD |
| 2.6 | Appui technique transverse aux autres pôles (revue bas niveau) | Revues de code régulières |

### 5.4 Pôle 3 — Elias Lebrun  · UI système haut niveau (Shell) & liaison

> Périmètre `repartitions.txt` : *UI technique non bas niveau (shell, etc.).*

| # | Tâche | Livrable / critère de validation |
|---|-------|----------------------------------|
| 3.1 | Parser de commandes robuste (historique, tokenisation, gestion d'erreurs) | Shell tolérant aux entrées invalides |
| 3.2 | Enrichir le jeu de commandes (`help`, `version`, `mem`, `clear`, `echo`, `uptime`, `ps`…) | Nouvelles commandes documentées |
| 3.3 | Commandes de supervision reliées au Pôle 4 (`ps`, `top`-like) | Affichage des stats via le shell |
| 3.4 | Définir l'**API de liaison kernel ↔ UI** (mémoire, scheduler, drivers) | Interface stable réutilisée par Pôles 2 & 4 |
| 3.5 | Documentation & accessibilité (doc structurée, code commenté en anglais, CLI accessible) | Doc conforme aux critères d'accessibilité (§annexe) |

### 5.5 Pôle 4 — Rayan Benhabiles · Monitoring dynamique & IA d'optimisation

> Périmètre `repartitions.txt` : *UI technique mémoire dynamique (gestionnaire de tâches / performances avec mémoire optimisée et stats CPU/GPU/RAM/stockage), IA d'opti.*

| # | Tâche | Livrable / critère de validation | Binôme |
|---|-------|----------------------------------|--------|
| 4.1 | Gestionnaire de tâches : liste temps réel des processus/threads | Affichage dynamique rafraîchi | Mathias (données scheduler) |
| 4.2 | Statistiques ressources : CPU, RAM, stockage (GPU à terme) | Compteurs affichés en continu | Benjamin (hooks bas niveau) |
| 4.3 | Suivi mémoire optimisé (usage allocateur, fragmentation) | Métriques mémoire lisibles | Mathias (allocateur) |
| 4.4 | Intégration à l'UI/shell | Vue moniteur accessible depuis le shell | Elias (UI) |
| 4.5 | Prototype **IA d'optimisation** (heuristique ordonnancement/mémoire sur métriques) | Démonstrateur d'optimisation | Benjamin (métriques) |

### 5.6 Matrice de responsabilités (RACI simplifiée)

| Composant | Benj | Mathias | Elias | Rayan |
|-----------|:----:|:-------:|:-----:|:-----:|
| Bootloader | **R** | C | I | I |
| Kernel bas niveau (mémoire/IRQ) | C | **R** | I | I |
| Drivers / Compatibilité VM | **R** | C | I | I |
| Scheduler (RR → priorités) | C | **R** | I | C |
| Support ELF / loader | **R** | C | I | I |
| Shell / CLI | I | C | **R** | C |
| API liaison kernel ↔ UI | C | C | **R** | C |
| Gestionnaire de tâches & stats (CPU/RAM/stockage) | C | C | C | **R** |
| IA d'optimisation | C | C | I | **R** |
| Tests unitaires kernel & CI/CD | C | **R** | C | I |
| Documentation & accessibilité | C | C | **R** | C |

> **R** = Responsable · **C** = Contributeur/Consulté · **I** = Informé

### 5.7 Collaborations & binômes clés

- **Rayan ↔ Benjamin** : hooks/métriques bas niveau → stats & IA d'opti.
- **Rayan ↔ Mathias** : données scheduler/allocateur → gestionnaire de tâches.
- **Rayan ↔ Elias** : intégration du moniteur dans l'UI/shell.
- **Elias ↔ Mathias** : API de liaison kernel ↔ UI.
- **Benjamin (autonome)** : boot, drivers, compatibilité VM.

---

## 6. Engagements par membre — jusqu'au prochain suivi EIP (octobre)

Objectifs concrets et **testables sous QEMU** d'ici la prochaine soutenance.

| Membre | Engagements d'ici octobre |
|--------|---------------------------|
| **Benjamin** | Consolider bootloader + drivers (clavier/souris/affichage), garantir la compatibilité VM (matrice QEMU + cibles Make), amorcer le loader ELF et exposer les premiers hooks de métriques ; revue des blocs `unsafe`. |
| **Mathias** | Faire évoluer le scheduler vers des priorités dynamiques, renforcer paging/isolation, faire évoluer l'allocateur, et mettre en place les tests unitaires kernel automatisés en CI. |
| **Elias** | Enrichir le shell (parser + nouvelles commandes), définir l'API de liaison kernel ↔ UI, améliorer documentation et accessibilité. |
| **Rayan** | Prototyper le gestionnaire de tâches et l'affichage des stats (CPU/RAM/stockage) en binôme, poser les bases de l'IA d'optimisation à partir des métriques collectées. |

---

## 7. Timeline (Roadmap)

### 7.1 Roadmap projet — 3 phases

| Phase | Description |
|-------|-------------|
| **Fondations** | Bases du système (boot, VGA, mémoire) — *réalisé via PoC* |
| **Noyau** | Noyau fonctionnel et performant (IRQ, paging, scheduler) — *en cours* |
| **Maturité** | Stabilisation et fonctionnalités avancées (userspace, syscalls, réseau) |

### 7.2 Horizons prospectifs (EIP B1-C05-1)

| H1 — Court terme (0–6 mois) | H2 — Moyen terme (6–18 mois) | H3 — Long terme (18+ mois) |
|-----------------------------|------------------------------|----------------------------|
| Scheduler priorités dynamiques | Espace utilisateur isolé (Ring 3) | Shell POSIX compatible |
| Système de fichiers FAT16 minimal | Syscall table POSIX-like | Port d'outils Unix (busybox-like) |
| Support multicœur (SMP basique) | Driver réseau minimal (e1000) | Packaging & distribution pédagogique |
| Tests unitaires kernel automatisés | Support ELF (chargement binaires) | Contribution communauté OSDev |

### 7.3 Jalon immédiat

- **Prochain suivi EIP : octobre** → livrables selon les engagements de la section 6.

---

## Annexe — Éléments consolidés des dossiers 1 à 5

> Éléments de cadrage repris des dossiers pour référence ; ils appuient la répartition
> du travail sans en constituer le cœur.

### A. Priorisation MoSCoW (Dossier 1)

| Priorité | Éléments |
|----------|----------|
| **Must Have** | Bootloader · Kernel minimal · Paging mémoire · Interruptions · Shell minimal |
| **Should Have** | Logs système · Multitâche avancé |
| **Could Have** | Système de fichiers simple |
| **Won't Have** | Interface graphique (hors MVP) · Multi-utilisateur avancé |

### B. Chiffrage & ressources (Dossier 4)

| Catégorie | Valeur | Commentaire |
|-----------|--------|-------------|
| **CAPEX** | **0 €** | Outils 100 % open-source, aucune dépense matérielle initiale |
| **OPEX** | **~10 €/mois** (~120 €/an) | Hébergement documentation + domaine |
| **Effort humain estimé** | **~480 h** | Dev + documentation + tests |

**Scénario retenu** : *Full Open Source* (stack gratuite intégrale, communauté active),
choisi face au VPS dédié (~360 €/an) et au SaaS premium jugés surdimensionnés/inadaptés
au projet pédagogique.

### C. Risques & méthodologie (Dossier 5)

| Risque / axe | Impact | Mitigation |
|--------------|--------|------------|
| Bug mémoire | Critique | Tests intensifs, Rust ownership, revue `unsafe` |
| Complexité technique | Élevé | Documentation continue, architecture modulaire |
| Montée en compétence | Progressive | Approche incrémentale, pair-programming, revue de code |
| Green IT & RSE | Positif | Code minimaliste, peu de dépendances, optimisation ressources |
| Continuité | Positif | Sauvegardes Git, images VM stables, pipeline CI/CD |

**Boucle de résilience** :
`Développement → Commit Git → Sauvegarde → CI/CD → Validation continue → Image VM stable → reprise rapide`

### D. Conformité & accessibilité (Dossiers 2 & EIP)

- **RGPD** : aucune donnée personnelle collectée → impact nul.
- **Accessibilité** : interface CLI (texte) → compatible lecteurs d'écran (NVDA, Orca), pas de dépendance couleur/souris, contraste VGA blanc/noir.
- **Documentation** : titres hiérarchiques H1→H3, code commenté en anglais clair, schémas ASCII décrits.
- **Sécurité** : isolation stricte espace noyau / utilisateur, audit manuel des zones critiques.

---

## Sources & références

- Répartition de base : `docs/eip/repartitions.txt`
- Documents internes : `docs/dossiers/` (Dossiers 1 à 5)
- Présentations : `docs/presentations/FerrumOS_Presentations_Summary.md`
- PoCs : `POC/ferrumos_PoC`, `POC/ferrum_Poc_adaptative_sheduller`, `POC/ferrum_Poc_graphic`
- Target spec : `targets/x86_64-ferrumos.json`
- Externes : `wiki.osdev.org`, `os.phil-opp.com`, `rust-lang.org`
