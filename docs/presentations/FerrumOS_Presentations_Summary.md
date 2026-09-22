# FerrumOS — Presentations Summary

> Combined content from:
> - **FerrumOS_EIP_Bloc1.pdf** — EIP Bloc 1 RNCP • Soutenance 2025–2026
> - **Présentation - FerrumOS.pdf** — General Project Presentation

---

## 1. Presentation Générale (Présentation - FerrumOS)

### Auteurs
- Rayan Benhabiles
- Mathias Desgranges
- Benjamin Ale

---

### Qu'est-ce qu'un OS ?
La couche logicielle essentielle entre le matériel et les applications. Un système d'exploitation (OS) est une couche logicielle qui gère le matériel, les processus et les ressources, facilitant ainsi l'interaction entre l'utilisateur et l'ordinateur.

### Pourquoi partir de zéro ?
Importance d'une approche innovante et libre.

- **Comprendre le fonctionnement interne** : En développant un système d'exploitation de zéro, nous pouvons analyser profondément les interactions internes entre le matériel et les logiciels.
- **Éviter les contraintes héritées** : Créer un OS depuis le début permet d'éviter les limitations des systèmes existants, favorisant ainsi l'innovation et la flexibilité dans le développement.

### Pourquoi Rust ?
Sécurité mémoire par ownership et borrowing dans le code. Rust offre une gestion de la mémoire sécurisée grâce à la propriété et l'emprunt, permettant d'éviter les erreurs courantes dans d'autres langages de programmation.

#### La croissance rapide de Rust dans l'industrie
- **20%** de croissance annuelle
- Rust connaît une adoption croissante en raison de sa sécurité et de sa fiabilité, attirant de plus en plus de développeurs chaque année.

### Objectifs du projet
Approche collaborative pour un développement efficace.

- **Apprentissage collectif** : L'objectif principal est de favoriser l'apprentissage collectif, permettant aux membres de l'équipe de partager leurs connaissances et d'améliorer continuellement leurs compétences.
- **Architecture modulaire** : Nous visons à créer une architecture modulaire qui facilite l'intégration et la mise à jour des composants, réduisant ainsi la complexité du développement et de la maintenance.
- **Documentation rigoureuse** : Une documentation rigoureuse est essentielle pour assurer la clarté et la compréhension du projet, garantissant que chaque membre de l'équipe puisse suivre l'évolution et les décisions prises.

### Architecture Globale

| Niveau | Rôle |
|--------|------|
| **Programmes** | Applications utilisateur |
| **Services** (pilotes/réseau) | Ajout progressif de fonctions (fichiers, réseau). Testé en VM et documenté. |
| **Noyau FerrumOS** | Arbitre CPU, mémoire et événements. |
| **Démarrage (boot)** | Initialisation matérielle |

### Priorisation MoSCoW

| Priorité | Éléments |
|----------|----------|
| **Must Have** | Bootloader et noyau — cruciaux au démarrage et au fonctionnement de FerrumOS. |
| **Should Have** | Journaux système — suivre les événements et erreurs, garantissant une meilleure gestion et débogage. |

### Gestion mémoire

- **Paging x86_64** : Gestion efficace de la mémoire, garantissant l'isolation entre les processus et optimisant l'utilisation des ressources système.
- **Allocateur personnalisé** : Conçu pour répondre aux besoins spécifiques de FerrumOS, améliorant la performance de la gestion de mémoire tout en minimisant les erreurs de mémoire.

### Ordonnancement

- **Round Robin** : Méthode d'ordonnancement simple qui permet de gérer efficacement les processus en leur attribuant un temps de traitement égal.
- **Scheduler adaptatif** : Ajuste dynamiquement la priorité des tâches en fonction de leur comportement, optimisant ainsi l'utilisation des ressources et réduisant la latence.

### Roadmap — 3 Phases

| Phase | Description |
|-------|-------------|
| **Fondations** | Développement des bases du système FerrumOS. |
| **Noyau** | Création d'un noyau fonctionnel et performant. |
| **Maturité** | Stabilisation et ajout de fonctionnalités avancées. |

### Sources
- wiki.osdev.org
- os.phil-opp.com
- rust-lang.org
- survey.stackoverflow.co

---

## 2. EIP — Bloc 1 RNCP • Soutenance 2025–2026

### Présentation du PoC

**FerrumOS** — Un système d'exploitation pédagogique écrit en Rust.

| Composant | Description |
|-----------|-------------|
| Bootloader x86_64 | Démarrage du système |
| Paging mémoire | Gestion mémoire paginée |
| Shell CLI interactif | Interface en ligne de commande |
| OS Kernel | Noyau pédagogique |
| Gestion interruptions | IDT et handlers |
| Allocateur Bump | Allocation mémoire simplifiée |

---

### Bloc 1 — RNCP — Structure de la présentation

#### A1 — Identification du besoin client
- B1-C01.1 Analyse des besoins clients
- B1-C01.2 Analyse des besoins – accessibilité
- B1-C02-1 Compte-rendu d'audit
- B1-C02-2 Approche méthodologique de l'audit

#### A2 — Traduction technique du besoin
- B1-C03-1 Documentation des spécifications
- B1-C03-2 Spécifications – accessibilité
- B1-C03-3 Documents accessibilité
- B1-C04-1 Analyse financière
- B1-C04-2 Analyse financière – benchmarks
- B1-C05-1 Étude prospective
- B1-C05-2 Vulgarisation prospective

---

### A1 — IDENTIFICATION DU BESOIN CLIENT

#### B1-C01.1 — Analyse des besoins clients
Recenser les besoins en observant et en échangeant avec les parties prenantes.

##### Parties prenantes identifiées
- Étudiants en systèmes
- Développeurs bas niveau expérimentés
- Enseignants & évaluateurs (pédagogie, rigueur)

##### Priorisation MoSCoW

| Niveau | Éléments |
|--------|----------|
| **Must Have** | Bootloader · Kernel · Paging · IDT · Shell |
| **Should Have** | Logs système · Multitâche avancé |
| **Could Have** | Système de fichiers simple |
| **Won't Have** | Interface graphique · Multi-utilisateur |

##### 🎯 MVP validé
- Boot stable reproductible sous QEMU
- Initialisation mémoire sans erreur critique
- Interruptions matérielles opérationnelles
- Shell CLI avec commandes help / version / mem

---

#### B1-C01.2 — Analyse des besoins — Accessibilité
Prise en compte des normes pour les personnes en situation de handicap.

##### Accessibilité numérique dans un projet OS
Contexte spécifique : FerrumOS est un OS pédagogique — son interface est exclusivement CLI (shell texte). L'accessibilité s'évalue donc sur les outils, la documentation et la lisibilité du code, non sur une interface graphique.

##### Interface CLI inclusive
- Texte pur : compatible avec lecteurs d'écran
- Pas de dépendance à la souris ou aux couleurs
- Sortie VGA lisible avec polices à fort contraste

##### Documentation accessible
- Code commenté en anglais clair
- README structuré avec titres hiérarchiques
- Schémas ASCII inclus dans les dossiers

##### Rust et accessibilité logicielle
- Sécurité mémoire réduit les crashes inattendus
- Comportement prévisible = meilleure UX
- Dépendances minimales = moins de surface à risque

---

#### B1-C02-1 — Compte-rendu d'audit technique
Audit technique, fonctionnel et de sécurité de l'environnement projet.

##### Environnement technique audité

| Outil | Rôle | Choix justifié |
|-------|------|----------------|
| Rust nightly | Compilation noyau | Accès fonctionnalités no_std expérimentales |
| QEMU x86_64 | Exécution & validation | Isolation, reproductibilité, debug facilité |
| GDB | Débogage noyau | Inspection registres, mémoire, IDT en live |
| Git + CI/CD | Versioning & intégration | Traçabilité par composant, validation automatique |
| cargo-bootimage | Image bootable | Génération ISO bootable depuis le source Rust |

##### Contraintes de sécurité identifiées
- Blocs unsafe limités aux zones critiques (IRQ, changement de contexte, accès mémoire physique)
- Revue systématique du code unsafe — politique ownership Rust stricte — tests sous QEMU

---

#### B1-C02-2 — Approche méthodologique de l'audit
Moyens d'investigation, collecte de retours, benchmark.

| Étape | Description |
|-------|-------------|
| **01 — Benchmark État de l'Art** | Analyse comparative : Linux, Unix, micro-noyaux expérimentaux. Identification des principes clés : modularité, séparation des responsabilités, robustesse. |
| **02 — Investigation Technique** | Lecture des guides de référence (OSDev Wiki, phil-opp blog). Analyse des dépendances et contraintes du développement bare-metal en Rust. |
| **03 — Collecte de Retours** | Échanges avec utilisateurs cibles (étudiants, enseignants). Itérations sur la documentation et la lisibilité du code en fonction des retours. |
| **04 — Synthèse & Opportunités** | Identification de l'opportunité : un noyau lisible et instrumentable à des fins pédagogiques, sans ambition concurrentielle. |

---

### A2 — TRADUCTION TECHNIQUE DU BESOIN FONCTIONNEL
Spécifications, chiffrage et vision prospective — 7 observables : B1-C03 · B1-C04 · B1-C05

#### B1-C03-1 — Documentation des spécifications
Spécifications techniques et fonctionnelles couvrant les contraintes identifiées.

##### Architecture en couches — Modèle C4

```
Programmes
    ↑
Services
    ↑
Drivers
    ↑
Kernel (CPU · Mémoire · IRQ · Scheduler)
    ↑
Bootloader
```

##### Gestion Mémoire — Paging x86_64
- Tables de pages 4 niveaux
- Isolation espaces noyau / futurs processus
- Allocateur Bump personnalisé et contrôlé

##### Ordonnancement — Round Robin
- Quantum CPU par tâche
- Préemption via interruption timer
- Base pour évolution vers priorités dynamiques

---

#### B1-C03-2 — Spécifications — Accessibilité
Prise en compte des problématiques d'accessibilité numérique dans les spécifications.

Accessibilité intégrée dès les spécifications. Choix architectural : interface CLI texte seul → accessibilité native, sans effort supplémentaire.

##### CLI & Lecteurs d'écran
- Shell texte = sortie lisible par NVDA, Orca
- Aucune dépendance couleur pour fonctionner
- Contraste VGA : blanc sur noir par défaut

##### Documentation structurée
- Titres H1/H2/H3 hiérarchisés
- Pas d'information portée uniquement par la couleur
- Schémas ASCII lisibles hors contexte visuel

##### Code & API noyau
- Nommage explicite des fonctions
- Commentaires en anglais clair
- Pas de variables nommées par couleur

---

#### B1-C03-3 — Documents accessibles
Les documents du dossier respectent les recommandations d'accessibilité.

##### Conformité documentaire — bonnes pratiques appliquées

| Statut | Critère |
|--------|---------|
| ✓ | Titres hiérarchiques H1 → H3 dans tous les dossiers (PDF, Markdown, README) |
| ✓ | Langue principale déclarée — documents en français, code commenté en anglais |
| ✓ | Texte alternatif pour les schémas (descriptions ASCII incluses dans les dossiers) |
| ✓ | Contraste suffisant dans les documents PDF : noir sur blanc |
| ✓ | Police lisible : Calibri / sans-serif, corps 11pt minimum |
| ✓ | Structure sémantique : paragraphes nommés, listes à puces cohérentes |
| △ | WCAG 2.1 niveau AA — objectif de conformité partielle (projet pédagogique, hors périmètre UI web) |

---

#### B1-C04-1 — Analyse financière
CAPEX / OPEX — optimisation des coûts de production et d'exploitation.

##### Modèle économique — Open Source & Coûts maîtrisés

| Métrique | Valeur |
|----------|--------|
| CAPEX total | **0 €** (outils open-source) |
| OPEX / mois | **~10 €** (hébergement + domaine) |
| OPEX annuel estimé | **~120 €** (scénario full-web) |
| Effort humain estimé | **480 h** (dev + doc + tests) |

##### Ressources & Coûts

| Ressource | Type | Coût |
|-----------|------|------|
| Rust nightly + Cargo | Open-source | 0 € |
| QEMU | Open-source | 0 € |
| GDB | Open-source | 0 € |
| Git + GitHub | Open-source (free tier) | 0 € |
| Hébergement docs | VPS mutualisé | ~6 €/mois |
| Domaine dédié | Registrar | ~12 €/an |

---

#### B1-C04-2 — Analyse financière — Benchmarks
Scénarii de chiffrage appuyés sur des benchmarks de solutions existantes.

##### 3 scénarii comparés — Choix justifié par le benchmark

| | S1 — Full Open Source | S2 — Cloud VPS dédié | S3 — SaaS Premium |
|--|----------------------|-----------------------|-------------------|
| **CAPEX** | 0 € | 0 € | ~500 € |
| **OPEX** | ~72 €/an | ~360 €/an | ~1200 €/an |
| **Avantages** | + Stack gratuite intégrale<br>+ Communauté active | + Isolation complète<br>+ CI/CD managée | + Support professionnel |
| **Limites** | − Support limité<br>− Maintenance manuelle | − Surcoût non justifié<br>− Complexité accrue | − Inadapté au projet péda.<br>− Coût prohibitif |
| **Décision** | ✓ **RETENU** | | |

---

#### B1-C05-1 — Étude prospective
Voies d'évolution et de migration s'appuyant sur l'audit technique.

##### Roadmap d'évolution — 3 horizons

| H1 — Court terme (0-6 mois) | H2 — Moyen terme (6-18 mois) | H3 — Long terme (18+ mois) |
|-----------------------------|------------------------------|----------------------------|
| Scheduler avec priorités dynamiques | Espace utilisateur isolé (Ring 3) | Shell POSIX compatible |
| Système de fichiers FAT16 minimal | Syscall table POSIX-like | Port d'outils Unix (busybox-like) |
| Support multicoeur (SMP basique) | Driver réseau minimal (e1000) | Packaging et distribution pédagogique |
| Tests unitaires kernel automatisés | Support ELF pour charger des binaires | Contribution communauté OSDev |

---

#### B1-C05-2 — Étude prospective — Vulgarisation
Présenter l'évolution de manière synthétique et accessible.

##### FerrumOS : de la fondation à l'écosystème

**Analogie** : FerrumOS aujourd'hui, c'est les fondations d'une maison. On a coulé le béton, monté les murs porteurs, posé la charpente. La maison ne ressemble pas encore à une maison — mais sans ces fondations, rien de ce qui vient ensuite ne tient.

| Étape | Description |
|-------|-------------|
| **Aujourd'hui** | Un kernel qui boot, gère la mémoire, répond aux interruptions et expose un shell. Le minimum viable — mais entièrement maîtrisé. |
| **Dans 1 an** | Un espace utilisateur isolé, des processus qui tournent côte à côte, un vrai loader de binaires. FerrumOS devient expérimentable. |
| **Dans 3 ans** | Un OS pédagogique complet, packagé, documenté, utilisable en cours — une référence open source pour la communauté systèmes. |

---

## Conclusion — Ce que FerrumOS démontre

1. Un projet pédagogique peut être conduit avec la rigueur d'un projet professionnel : audit, specs, chiffrage, risques.
2. La sécurité mémoire n'est pas une contrainte : en Rust, c'est un avantage architectural intégré dès la conception.
3. Avec 0 € de CAPEX, on peut construire une chaîne d'ingénierie complète — CI/CD, tests, documentation, versioning.
4. La roadmap prospective ancre le projet dans une vision à 3 ans, cohérente avec les contraintes identifiées lors de l'audit.

### PoC disponible — `ferrumos_PoC/`

| Fichier | Rôle |
|---------|------|
| `src/main.rs` | Point d'entrée du noyau |
| `src/memory.rs` | Gestion mémoire (paging, allocation) |
| `src/interrupts.rs` | Gestion des interruptions (IDT) |
| `src/shell.rs` | Shell CLI interactif |
| `src/allocator.rs` | Allocateur Bump personnalisé |

---

> **Merci de votre attention — Place aux questions**