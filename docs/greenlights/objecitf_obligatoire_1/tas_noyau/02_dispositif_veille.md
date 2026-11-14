# 02 · Dispositif de veille

> **Objectif obligatoire 1**, livrable « Description of your research setup: sources, tools, automations, rhythm, split ».
> Version du **12 novembre 2026** (Elias Lebrun). Le guide l'attendait décrit avant fin octobre : **retard à signaler au mentor**.
> Statut : outillage en place et testé ; **rythme et répartition à valider en réunion d'équipe**.

## 1. Sources suivies

Liste outillée : [`scripts/veille/sources.json`](../../../../scripts/veille/sources.json) (vérifiée automatiquement chaque semaine). Chaque source a une raison d'être et un responsable.

| Source | Type | Ce qu'elle apporte | Pourquoi on la garde | Resp. |
|---|---|---|---|---|
| `linked_list_allocator` | crate (crates.io) | allocateur de référence de notre choix A | comparer, récupérer des correctifs | Rayan |
| `talc` | crate | allocateur du noyau Hermit | alternative mature à notre code maison | Rayan |
| `rlsf` | crate | TLSF en Rust | prototype d'exploration | Rayan |
| `buddy_system_allocator` | crate | buddy (rCore) | piste pour les cadres physiques | Rayan |
| `rust-osdev/x86_64` | releases GitHub | dépendance du noyau | la 0.14 casse avec les nightlies récentes | Benjamin |
| `rust-osdev/bootloader` | releases GitHub | chargeur des PoC | migration 0.9 → 0.11 à prévoir | Benjamin |
| `rust-osdev/bootimage` | releases GitHub | outil de build/test | épinglé en 0.10.3, la 0.10.5 casse | Elias |
| `asterinas/asterinas` | releases GitHub | OS Rust, tas slab par CPU | référence de la note d'architecture | Mathias |
| `theseus-os/Theseus` | commits GitHub | OS Rust à composants | référence de la note d'architecture | Mathias |
| Blog phil-opp | RSS | base des PoC, design de B | nouvelle édition = impact direct | Benjamin |
| Redox OS | RSS | micronoyau Rust | comparaison d'architecture | Mathias |
| QEMU | RSS | émulateur de toutes nos mesures | une version change nos cycles | Elias |
| This Week in Rust | RSS | nightlies, crates `no_std` | toolchain épinglée à surveiller | Elias |
| Blog officiel Rust | RSS | stabilisations, cibles | idem | Elias |
| LWN | RSS | noyau Linux (SLUB, Rust for Linux) | état de l'art allocateurs | Mathias |

Sources **suivies à la main** (pas de flux exploitable) :

| Source | Pourquoi | Resp. |
|---|---|---|
| Forum et wiki OSDev (`forum.osdev.org`, `wiki.osdev.org`) | questions bas niveau, communauté où contribuer ; le flux du forum refuse les clients automatiques (HTTP 403 au 12/11) | Benjamin |
| Discord / Zulip rust-osdev | discussions sur `x86_64`, `bootloader` | Benjamin |
| Publications (ECRTS, USENIX ATC, OSDI) | allocateurs, ordonnanceurs | Mathias |

## 2. Outils et automatisations

| Outil | Ce qu'il fait | Démonstration |
|---|---|---|
| [`scripts/veille/check_sources.py`](../../../../scripts/veille/check_sources.py) | interroge crates.io, l'API GitHub et les flux RSS/Atom, compare à l'état précédent, écrit un **digest Markdown groupé par responsable** ; bibliothèque standard Python seulement | `python3 scripts/veille/check_sources.py --state veille-state.json --digest digest.md` ; testé le 12/11/2026 : 15 sources jointes, détection des nouveautés vérifiée en vieillissant l'état de 3 sources (`bootloader v0.11.17`, *This Week in Rust 676*, `talc 5.0.2` remontés) |
| [`.github/workflows/veille.yml`](../../../../.github/workflows/veille.yml) | **chaque lundi 6 h UTC** : lance le script, publie le digest dans le résumé du run, **ouvre une issue « Veille du … »** s'il y a du nouveau, garde l'état en cache | onglet Actions → *Veille* → *Run workflow* (après merge) |
| Salon Discord automatique | si le secret `DISCORD_WEBHOOK_VEILLE` est défini, le digest est aussi posté sur un salon `#veille` de l'équipe | à configurer : Paramètres du salon → Intégrations → Webhook, puis secret du dépôt |
| Journal partagé | [`03_journal_veille.md`](03_journal_veille.md) dans le dépôt : versionné, relu en PR | historique Git |

## 3. Rythme (proposition)

| Quoi | Quand | Durée | Qui |
|---|---|---|---|
| Digest automatique | chaque lundi | — | CI |
| Lecture du digest, tri de sa partie, entrées au journal | dans la semaine | **1 h / semaine / membre** | chacun pour son domaine |
| Point veille en réunion d'équipe | chaque réunion hebdomadaire | 10 min | tour de table |
| Présentation interne enregistrée | **une par mois** de novembre à mai (7 au total, 6 exigées) | 15 min + 10 min de questions | rotation, voir `04_presentations_internes.md` |

Objectif chiffré : **≥ 20 entrées** au journal d'ici mai, au moins **2 par mois** pour toute l'équipe, chaque entrée avec une conclusion.

## 4. Répartition

| Membre | Domaine de veille | Lien avec son pôle |
|---|---|---|
| **Rayan Benhabiles** | allocateurs, mémoire, mesures de performance | auteur des expériences mémoire |
| **Benjamin Ale** | chargeur, `x86_64`, pilotes, OSDev | pôle 1 (boot, drivers) |
| **Mathias Desgranges** | architecture des OS Rust, publications | auteur de la note d'architecture |
| **Elias Lebrun** | toolchain Rust, QEMU, CI, outillage de veille | rôle CI/CD |

Cette répartition reprend les rôles techniques de `docs/ORGANIZATION.md`. **Elle doit être confirmée par chacun** : une veille assignée sans accord ne sera pas tenue.
