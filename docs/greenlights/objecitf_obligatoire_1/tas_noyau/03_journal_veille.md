# 03 · Journal de veille

> **Objectif obligatoire 1**, livrable « Complete research log ». Alimenté d'octobre à mai, **20 entrées minimum**, réparties dans le temps.
> Règle : une entrée sans conclusion ne compte pas. « Rien à faire, parce que… » est une conclusion.

## Compteur

| | |
|---|---|
| Entrées au 12/11/2026 | **11** (dont 4 reconstituées depuis des commits, marquées ⓡ) |
| Objectif | 20 d'ici mai 2027 |
| Utilisées dans le projet | 7 (colonne « Suite donnée » = **adopté**) |

Les entrées ⓡ ont été écrites le 12/11/2026 à partir de traces datées du dépôt (commit, rapport). Leur date est celle de la trace, pas de l'écriture.

## Entrées

| # | Date | Qui | Source | Sujet | Ce que ça change pour FerrumOS | Suite donnée |
|---|---|---|---|---|---|---|
| 1 ⓡ | 2026-10-17 | Rayan | P. Oppermann, *Writing an OS in Rust : Testing* (os.phil-opp.com) ; trace : commit `702fb41` | Framework de test `no_std` : `custom_test_frameworks`, sortie QEMU par le port `isa-debug-exit`, sortie série | Permet de tester le noyau sous QEMU au lieu de regarder l'écran | **Adopté** : harnais de test du PoC ordonnanceur (`testing.rs`, `test-success-exit-code = 33`) |
| 2 ⓡ | 2026-10-17 | Rayan | Notes de version de `bootimage` (0.10.5) et de `x86_64` 0.14 ; trace : `test_environment.md` | `bootimage` ≥ 0.10.5 passe `-Zjson-target-spec`, inconnu de notre nightly ; `x86_64` 0.14 casse quand le trait `Step` change | Une mise à jour silencieuse de la toolchain casse le build | **Adopté** : versions épinglées (`nightly-2026-01-15`, `bootimage 0.10.3`) |
| 3 ⓡ | 2026-10-17 | Benjamin | Documentation de pacman, Guix et pkgutils (Redox) ; trace : branche `docs/package-management-research` | Systèmes de paquets : signatures, générations, retour arrière | Pistes pour installer des programmes sur FerrumOS plus tard ; prototype hôte, 14 expériences | Différé : outils natifs non testés, à reprendre quand FerrumOS exécutera des ELF (voir `README.txt` de la branche) |
| 4 ⓡ | 2026-11-11 | Elias | Documentation GitHub Actions ; actions `dtolnay/rust-toolchain`, `Swatinem/rust-cache` | Épingler une nightly précise en CI, mettre en cache la compilation | Les tests QEMU tournent en CI avec la même toolchain qu'en local | **Adopté** : `.github/workflows/kernel-tests.yml` (commit `c88bde4`) |
| 5 | 2026-11-12 | Mathias | Documentations de Redox, Asterinas, Theseus, Tock, Hubris ; trace : `docs/greenlights/Objectif obligatoire 2/` (branche `main`) | Frontières de protection et découpage des OS Rust | Choix d'un noyau monolithique modulaire, code matériel isolé dans des adaptateurs | **Adopté** : note de décision d'architecture (objectif 2) |
| 6 | 2026-11-12 | Elias | P. Oppermann, *Allocator Designs* (os.phil-opp.com/allocator-designs) | Trois designs : bump, liste chaînée, blocs fixes | Base des candidats A et B ; le texte signale lui-même que les blocs fixes gardent la mémoire dans les classes | **Adopté** : design de B (`fixed_block.rs`), puis variante B' avec récupération après mesure |
| 7 | 2026-11-12 | Elias | crates.io : `linked_list_allocator` 0.10.6, `talc` 5.1.1, `rlsf` 0.2.3, `buddy_system_allocator` 0.13.0, `slabmalloc` 0.11.0 | Maturité et maintenance des allocateurs `no_std` | `slabmalloc` n'a plus de version depuis août 2022 : piste slab écartée pour la maintenance ; les autres sont actifs en 2026 | Comparaison dans `05_etat_de_l_art.md` |
| 8 | 2026-11-12 | Elias | Masmano et al., *TLSF*, ECRTS 2004 ; crate `rlsf` | Allocation en O(1) garanti | Hypothèse : meilleur p99 que la liste chaînée | **Adopté** comme prototype d'exploration ; résultat négatif (pas de gain mesurable, 33 % de capacité perdue sur les blocs de 64 o) |
| 9 | 2026-11-12 | Elias | Redox, MR 76 « Use slab allocator for kernel heap » (gitlab.redox-os.org) | Redox a envisagé de passer de la liste chaînée à des blocs fixes jusqu'à 4 Kio | Même dilemme que le nôtre, chez un projet plus avancé | A conforté le choix des deux candidats ; la MR ne dit pas si le changement a été fusionné, à vérifier |
| 10 | 2026-11-12 | Elias | Docs Hermit (`hermit-kernel` 0.11.0, `mm/allocator`) et Asterinas (`ostd` 0.18.0, `osdk-heap-allocator`) | Allocateurs choisis par deux noyaux Rust actifs : Talc pour Hermit, slab par CPU pour Asterinas | Une alternative en crate existe et est utilisée en production (`talc`) | Rien à faire maintenant : notre implémentation suffit (`08_note_decision.md`) ; `talc` ajouté aux sources suivies |
| 11 | 2026-11-12 | Elias | Paquet QEMU Ubuntu (10.2.1) contre 11.1.0 du `test_environment.md` | Version de l'émulateur de mesure | Les valeurs déterministes d'octobre sont reproduites à l'identique en 10.2.1 ; les cycles ne sont pas comparables entre versions | **Adopté** : version notée dans `test_environment.md` et dans `results.json` de chaque exécution |

## Modèle d'entrée

```
| n | AAAA-MM-JJ | Qui | Source (lien, version, date de publication) | Sujet | Ce que ça change ou pourrait changer | Suite donnée : adopté (où) / différé (jusqu'à quand) / écarté (pourquoi) |
```

Point d'attention pour la soutenance : les entrées 5 à 11 datent toutes du 12 novembre. Le jury attend des entrées **réparties dans le temps** ; le rythme hebdomadaire du dispositif (`02_dispositif_veille.md`) doit corriger cela dès maintenant.
