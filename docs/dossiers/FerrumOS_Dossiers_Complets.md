# FerrumOS — Dossiers Détaillés (Compilation)

> Compilation des 5 dossiers détaillés du projet **FerrumOS**, un système d'exploitation développé intégralement en Rust dans une logique pédagogique et expérimentale.

## Table des matières

1. [Dossier 1 — Besoins Clients & Périmètre](#dossier-1--besoins-clients--périmètre)
2. [Dossier 2 — Contexte Technologique & Conformité](#dossier-2--contexte-technologique--conformité)
3. [Dossier 3 — Spécifications & Architecture](#dossier-3--spécifications--architecture)
4. [Dossier 4 — Chiffrage & Dimensionnement Technique](#dossier-4--chiffrage--dimensionnement-technique)
5. [Dossier 5 — Impacts, Risques & Responsabilité](#dossier-5--impacts-risques--responsabilité)

---

## Dossier 1 — Besoins Clients & Périmètre

### 1. Vision et Objectifs Stratégiques

FerrumOS est un système d'exploitation développé intégralement en Rust dans une logique pédagogique et expérimentale.

L'objectif est de maîtriser les couches fondamentales d'un OS moderne : démarrage, gestion mémoire, interruptions, ordonnancement et architecture noyau.

Le projet vise une compréhension profonde plutôt qu'une logique concurrentielle vis-à-vis des systèmes existants.

### 2. Analyse des Parties Prenantes

- Les utilisateurs principaux sont des étudiants en informatique systèmes et des développeurs bas niveau.
- Les enseignants et évaluateurs constituent une partie prenante indirecte.
- Les attentes portent sur la lisibilité, la justification des choix techniques et la rigueur méthodologique.

### 3. Personas Détaillés

- **Persona 1** : Étudiant en systèmes souhaitant comprendre les mécanismes internes d'un OS.
- **Persona 2** : Développeur expérimenté désirant expérimenter des modules noyau et algorithmes d'ordonnancement.

Les besoins incluent documentation claire, modularité et environnement testable.

### 4. Backlog Produit et Priorisation

La méthode **MoSCoW** a été utilisée pour prioriser les fonctionnalités.

- **Must Have** : Bootloader, Kernel minimal, Paging mémoire, Interruptions, Shell minimal.
- **Should Have** : Logs, gestion multitâche avancée.
- **Could Have** : Système de fichiers simple.
- **Won't Have** : Interface graphique, multi-utilisateur avancé.

### 5. Définition du MVP

Le MVP consiste en un système capable de démarrer en machine virtuelle, initialiser la mémoire, gérer des interruptions et exécuter un shell minimal.

La validation repose sur la stabilité en environnement QEMU et l'absence d'erreurs mémoire critiques.

### Schéma explicatif — Périmètre du MVP

```
[QEMU / VM]
      |
      v
[Bootloader]
      |
      v
[Kernel minimal]
  |     |      |
  |     |      +--> Interruptions
  |     +---------> Gestion memoire (paging)
  +---------------> Shell minimal (CLI)

Objectif MVP : boot stable + memoire + interruptions + shell
```

### Tableau — Rôle et priorité des blocs

| Bloc | Rôle dans le MVP | Priorité |
| --- | --- | --- |
| Bootloader | Initialise le démarrage | Must Have |
| Kernel minimal | Base d'exécution du système | Must Have |
| Paging mémoire | Gestion mémoire et isolation | Must Have |
| Interruptions | Gestion des événements matériels / système | Must Have |
| Shell minimal | Interaction utilisateur en CLI | Must Have |
| Logs | Observabilité et debug | Should Have |
| Multitâche avancé | Exécution concurrente évoluée | Should Have |
| Système de fichiers simple | Persistance de base | Could Have |
| Interface graphique | Hors périmètre du MVP | Won't Have |

### Schéma de validation du MVP

- **Critère 1** : Démarrage reproductible dans QEMU
- **Critère 2** : Aucune erreur mémoire critique
- **Critère 3** : Shell minimal exécutable après boot

---

## Dossier 2 — Contexte Technologique & Conformité

### 1. Benchmark et État de l'Art

Une analyse comparative conceptuelle avec les systèmes Unix modernes a été réalisée.

FerrumOS adopte une architecture volontairement simplifiée pour favoriser la compréhension.

L'étude montre l'intérêt d'un noyau maîtrisé et modulaire pour l'apprentissage.

### 2. Choix Technologique : Rust

- Rust a été choisi pour sa sécurité mémoire native et son modèle d'ownership.
- La limitation des blocs `unsafe` permet de réduire drastiquement les risques de corruption mémoire.
- Le langage permet un contrôle bas niveau adapté au développement d'un noyau.

### 3. Audit Sécurité

- Isolation stricte espace noyau / utilisateur.
- Audit manuel des zones critiques.
- Tests de robustesse et scénarios de charge.

### 4. Conformité Légale et Accessibilité

- Aucune donnée personnelle collectée : impact RGPD nul.
- Documentation structurée selon principes d'accessibilité numérique.
- Navigation clavier et contraste validés.

### 5. Matrice de Compétences

- **Rust** : niveau intermédiaire avec montée en compétence kernel.
- **Architecture systèmes** : progression continue.
- **CI/CD et gestion de version** : maîtrisés.

### Schéma explicatif — Positionnement Rust dans FerrumOS

```
[Developpement noyau]
        |
        +--> Controle bas niveau
        +--> Performance
        +--> Securite memoire (ownership)
        +--> Reduction des corruptions memoire
                 |
                 +--> Limitation des blocs unsafe
```

### Schéma explicatif — Séparation des espaces

```
[Espace utilisateur]
     | (appels systeme / interface)
     v
----------------------------
[Frontiere de protection]
----------------------------
     |
     v
[Espace noyau]
 - memoire
 - interruptions
 - ordonnanceur
 - drivers
```

### Tableau — Axes de conformité et sécurité

| Axe | Mesure citée | Effet attendu |
| --- | --- | --- |
| Sécurité | Isolation noyau / utilisateur | Réduction du risque de propagation |
| Qualité | Audit manuel zones critiques | Vérification des points sensibles |
| Robustesse | Tests de charge | Détection de comportements instables |
| Conformité | Pas de données personnelles | Impact RGPD nul |
| Accessibilité | Navigation clavier + contraste | Documentation plus inclusive |

---

## Dossier 3 — Spécifications & Architecture

### 1. Architecture Générale (C4 Model)

- Architecture en couches : **Boot → Kernel → Drivers → Services → Programmes**.
- Le noyau constitue le cœur décisionnel (CPU, mémoire, interruptions).
- Les services étendent progressivement les fonctionnalités.

### 2. Gestion Mémoire

- Implémentation du paging x86_64.
- Isolation stricte des espaces mémoire.
- Allocateur personnalisé et contrôlé.

### 3. Ordonnancement

- Algorithme **Round Robin** pour le MVP.
- Perspectives d'évolution vers priorités et multiprocesseur.

### 4. Design et Parcours Utilisateur

- Shell CLI minimaliste.
- Parcours : **Boot → Kernel → Shell → Commandes**.
- Approche volontairement sobre pour clarté pédagogique.

### Schéma explicatif — Architecture en couches

```
+---------------------------+
|        Programmes         |
+---------------------------+
|         Services          |
+---------------------------+
|          Drivers          |
+---------------------------+
|          Kernel           |
| CPU | Memoire | IRQ | RR  |
+---------------------------+
|        Bootloader         |
+---------------------------+
```

### Schéma explicatif — Gestion mémoire (paging, vue simplifiée)

```
Processus / Kernel
        |
        v
[Adresse virtuelle]
        |
        v
[Tables de pages x86_64]
        |
        v
[Adresse physique]

Contraintes MVP :
- isolation stricte
- allocateur controle
- acces memoire maitrises
```

### Schéma explicatif — Ordonnancement Round Robin (MVP)

```
File des taches : [T1] -> [T2] -> [T3] -> [T1] -> ...

Principe :
1) Temps CPU attribue a T1
2) Expiration du quantum
3) Passage a la tache suivante
4) Rotation continue
```

### Tableau — États et évolutions prévues

| Composant | État MVP | Évolution prévue |
| --- | --- | --- |
| Paging x86_64 | Cible d'implémentation | Optimisations futures |
| Isolation mémoire | Exigence forte | Renforcement continu |
| Ordonnancement | Round Robin | Priorités + multiprocesseur |
| Interface | Shell CLI minimaliste | Fonctions CLI enrichies |

---

## Dossier 4 — Chiffrage & Dimensionnement Technique

### 1. Ressources Techniques

- Développement via QEMU.
- Gestion de version Git et intégration continue.
- Hébergement documentation et dépôt distant.

### 2. Analyse CAPEX / OPEX

- **CAPEX** : 0 € (outils open-source).
- **OPEX** : environ 26 €/mois (hébergement, domaine).
- **Projection version serveur** : +20 €/mois.

### 3. Viabilité et Scalabilité

- Architecture modulaire favorisant l'évolution.
- Projection de coûts maîtrisés à l'échelle.
- Possibilité de migration vers support bare-metal.

### Schéma explicatif — Environnement technique

```
[Poste de dev]
   |
   +--> QEMU (tests VM)
   +--> Git (versioning)
   +--> CI/CD (validation)
   +--> Depot distant + documentation
```

### Tableau — CAPEX / OPEX

| Catégorie | Valeur source | Commentaire explicatif |
| --- | --- | --- |
| CAPEX | 0 € | Outils open-source, pas de dépense initiale matérielle |
| OPEX actuel | ~26 €/mois | Hébergement + domaine |
| Projection serveur | +20 €/mois | Extension de capacité / services |

### Schéma explicatif — Projection de coûts

```
OPEX mensuel actuel          : ########################## 26
Projection + version serveur : ############################################## 46
```

**Lecture :**
- Base actuelle = 26 €/mois
- Avec version serveur = 46 €/mois estimé

### Scalabilité

- La modularité réduit le coût de changement.
- La migration bare-metal est présentée comme une possibilité d'évolution technique.

---

## Dossier 5 — Impacts, Risques & Responsabilité

### 1. Matrice des Risques

- **Bug mémoire** : impact critique, mitigation via tests intensifs.
- **Complexité technique** : documentation continue.
- **Montée en compétence** : progressive.

### 2. Green IT & RSE

- Code minimaliste.
- Optimisation des ressources.
- Absence de dépendances inutiles.

### 3. Résilience & Continuité

- Sauvegardes Git régulières.
- Images VM stables.
- Pipeline CI/CD pour validation continue.

### Tableau — Risques et impacts

| Risque / axe | Impact | Mesure / observation |
| --- | --- | --- |
| Bug mémoire | Critique | Mitigation via tests intensifs |
| Complexité technique | Élevé (implicite) | Documentation continue |
| Montée en compétence | Progressive | Approche incrémentalement maîtrisable |
| Green IT & RSE | Positif | Code minimaliste, optimisation, peu de dépendances |
| Continuité | Positif | Sauvegardes Git, images VM, CI/CD |

### Schéma explicatif — Boucle de résilience / continuité

```
[Developpement]
      |
      v
[Commit Git] -> [Sauvegarde]
      |
      v
[Pipeline CI/CD]
      |
      v
[Validation continue]
      |
      v
[Image VM stable]
      |
      +--> reprise rapide / continuation
```

### Schéma explicatif — Logique Green IT

```
Moins de dependances
        +
Code minimaliste
        +
Optimisation des ressources
        =
Empreinte logicielle mieux maitrisee
```
