# 10 · Participation aux communautés techniques

> **Objectif obligatoire 1**, livrable « Links and screenshots of your contributions in the communities ».
> Exigences (novembre à mai) : **2 communautés** au moins, **5 contributions publiques**, dont **1 contribution externe**. Pour chacune : lien, date, contexte, résultat.
>
> Mots techniques : voir le [lexique](lexique.md).

## 1. Communautés visées

| Communauté | Pourquoi nous | Membre référent |
|---|---|---|
| **rust-osdev** (GitHub `rust-osdev/*`, Zulip/Discord rust-osdev) | nous dépendons de `x86_64`, `bootloader`, `bootimage`, `linked_list_allocator` et nous avons un problème reproductible à remonter | Benjamin, Elias |
| **OSDev** (forum.osdev.org, wiki.osdev.org) | communauté historique du développement d'OS ; nos mesures d'allocateurs sous QEMU intéressent la section *OS Design & Theory* | Rayan, Mathias |

## 2. Contributions

**Aucune contribution publique n'a encore été faite au 12/11/2026.** Rien ne doit être ajouté ici sans lien vérifiable et capture d'écran (dans `results/communautes/`).

| # | Date | Qui | Communauté | Type | Lien | Contexte | Résultat | Capture |
|---|---|---|---|---|---|---|---|---|
| | | | | | | | | |

## 3. Contributions prêtes à être publiées

Brouillons rédigés à partir de faits vérifiés dans le dépôt. **À relire, puis à publier par un membre de l'équipe depuis son propre compte** ; reporter ensuite le lien et la date dans le tableau ci-dessus.

### 3.1 Rapport documenté : `bootimage` ≥ 0.10.5 avec une nightly de janvier 2026 (contribution externe)

Où : issues de `rust-osdev/bootimage`, **après avoir vérifié qu'aucune issue existante ne le couvre déjà** (si oui, commenter l'issue existante avec notre reproduction).

> **Title:** `bootimage 0.10.5 passes -Zjson-target-spec, rejected by nightly-2026-01-15`
>
> We build a small `no_std` kernel (bootloader 0.9.34, x86_64 0.14) with a custom JSON target and `build-std`. With `bootimage 0.10.3` everything works; with `0.10.5`, the build fails because `-Zjson-target-spec` is passed and our pinned `nightly-2026-01-15` (rustc 1.94.0-nightly 86a49fd71) does not know this flag.
>
> **Reproduction:** `rust-toolchain.toml` pinned to `nightly-2026-01-15`, `cargo install bootimage --version 0.10.5 --locked`, `cargo bootimage`. The kernel itself compiles, then:
>
> ```
> Building bootloader
> error: unknown `-Z` flag specified: json-target-spec
>
> For available unstable features, see https://doc.rust-lang.org/nightly/cargo/reference/unstable.html
> If you intended to use an unstable rustc feature, try setting `RUSTFLAGS="-Zjson-target-spec"`
> Error: Bootloader build failed.
> ```
>
> **Workaround:** pin `bootimage 0.10.3`. Would it make sense to detect the toolchain before passing the flag, or to document the minimum nightly for 0.10.5 in the changelog?

Erreur reproduite le 12/11/2026 (Elias) sur une copie du PoC au commit `c88bde4`, `bootimage` 0.10.5 installé à part. À faire avant de poster : vérifier qu'aucune issue existante ne couvre déjà le problème.

### 3.2 Expérience partagée : mesures d'allocateurs dans un tas de 100 Kio sous QEMU

Où : forum OSDev, section *OS Design & Theory* ; ou un fil Zulip rust-osdev.

Contenu proposé : le tableau du §3.1 de `06_prototype_decision.md` (bump 99 itérations, liste chaînée et blocs fixes à 10 000, TLSF 96 blocs), le pire cas O(n²) de la récupération des blocs fixes (243–343 µs), et une question ouverte : *comment les autres noyaux amateurs gèrent-ils la mémoire bloquée dans les classes d'un allocateur à blocs fixes ?*

### 3.3 Question avec contexte et reproduction : masquer les interruptions dans `GlobalAlloc`

Où : Zulip/Discord rust-osdev, ou forum OSDev.

Contexte : notre verrou du tas (`spin::Mutex`) ne masque pas les interruptions (limite de `09_integration.md`). Question : vaut-il mieux `without_interrupts` dans `alloc`/`dealloc`, ou interdire l'allocation dans les gestionnaires et le vérifier ? Joindre le lien vers `src/allocator.rs` une fois la PR fusionnée.

### 3.4 Retour d'expérience sur `linked_list_allocator`

Où : discussion ou issue de `rust-osdev/linked-list-allocator`, seulement si nos mesures apportent quelque chose (par exemple le comportement avec des alignements jusqu'à 1 024 o sans perte de padding). Relire d'abord le code de la crate pour ne pas signaler un comportement qu'elle gère déjà.

### 3.5 Réponse à une question d'un autre

À chercher dans le digest hebdomadaire et sur le forum OSDev : une question sur les allocateurs de tas ou les tests sous QEMU à laquelle nos mesures permettent de répondre.
