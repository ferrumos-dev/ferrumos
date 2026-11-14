# 04 · Partage interne : présentations et ce qui a été réutilisé

> **Objectif obligatoire 1**, livrable « Records of the internal presentations ».
> Exigences : rituel en place **dès novembre**, au moins mensuel ; **6 présentations** enregistrées sur l'année ; **3 éléments** de la veille réellement utilisés dans le projet.
>
> Mots techniques : voir le [lexique](lexique.md).

## 1. Rituel

- **10 minutes « veille »** à chaque réunion d'équipe : chacun dit ce qu'il a lu dans le digest de la semaine (`02_dispositif_veille.md`).
- **Une présentation de 15 minutes par mois**, avec 10 minutes de questions, **enregistrée** (enregistrement Discord ou OBS, ou au minimum ce compte rendu signé par les présents).
- Le digest automatique du lundi arrive dans les issues GitHub « Veille du … » (et sur Discord si le webhook est configuré).

## 2. Comptes rendus

**Aucune présentation n'a encore eu lieu au 12/11/2026.** Le tableau ci-dessous est le planning proposé ; une ligne ne passe en « faite » qu'avec la date réelle, les présents et le lien de l'enregistrement.

| # | Date | Présentateur | Sujet | Public | Enregistrement | Ce qu'on en a tiré | État |
|---|---|---|---|---|---|---|---|
| 1 | novembre 2026 (avant le prochain suivi) | Elias | Allocateurs de tas : 99 → 10 000, liste chaînée contre blocs fixes, pourquoi B' a été écarté | équipe | — | — | **à faire** |
| 2 | décembre 2026 | Rayan | Protocole de mesure mémoire et limites des tests sous QEMU | équipe | — | — | prévu |
| 3 | janvier 2027 | Mathias | Architecture des OS Rust (Redox, Asterinas, Theseus) | équipe | — | — | prévu |
| 4 | février 2027 | Benjamin | Migration `bootloader` 0.9 → 0.11 | équipe | — | — | prévu |
| 5 | mars 2027 | à définir | sujet issu du digest | équipe | — | — | prévu |
| 6 | avril 2027 | à définir | sujet issu du digest | équipe | — | — | prévu |
| 7 | mai 2027 | à définir | bilan de la veille de l'année | équipe | — | — | prévu |

Support prêt pour la présentation 1 : `06_prototype_decision.md` et `08_note_decision.md` (tableaux directement projetables).

Modèle de compte rendu :

```
### Présentation n — AAAA-MM-JJ
- Présentateur :
- Présents :
- Sujet et sources :
- Enregistrement : <lien>
- Questions posées :
- Décision ou action qui en sort (avec responsable) :
```

## 3. Ce qui, issu de la veille, est utilisé dans le projet

Exigence : au moins 3. Chaque ligne renvoie à une entrée du journal (`03_journal_veille.md`).

| # | Élément | Source d'origine | Ce que ça a changé | Où le voir |
|---|---|---|---|---|
| 1 | Framework de test `no_std` sous QEMU (`isa-debug-exit`, sortie série) | blog phil-opp, *Testing* (entrée 1) | Les expériences mémoire sont des tests automatiques au lieu d'observations à l'écran | `POC/ferrum_Poc_adaptative_sheduller/src/testing.rs` |
| 2 | Toolchain et `bootimage` épinglés | notes de version `bootimage`, `x86_64` (entrée 2) | Le build ne casse plus quand une nightly sort | `rust-toolchain.toml`, `test_environment.md` |
| 3 | Design des blocs de taille fixe | blog phil-opp, *Allocator Designs* (entrée 6) | Candidat B du prototype de décision ; sa limite mesurée a mené à B' puis à la décision | `libs/ferrum_alloc/src/fixed_block.rs` |
| 4 | Comparaison des OS Rust | docs Redox, Asterinas, Theseus, Tock, Hubris (entrée 5) | Architecture monolithique modulaire retenue | `docs/greenlights/Objectif obligatoire 2/` (branche `main`) |
| 5 | Épinglage de la toolchain en CI | actions `dtolnay/rust-toolchain`, `Swatinem/rust-cache` (entrée 4) | Tests QEMU automatiques à chaque push | `.github/workflows/kernel-tests.yml` |
