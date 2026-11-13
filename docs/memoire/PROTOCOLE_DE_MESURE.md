# Protocole de mesure mémoire, procédure à suivre

> **À qui s'adresse ce document** : à un membre de l'équipe qui n'a jamais lancé
> ces tests, et qui doit rejouer les mesures pour vérifier qu'elles sont
> reproductibles.
>
> **Pourquoi** : l'objectif complémentaire D exige qu'une autre personne puisse
> refaire la même mesure et obtenir un résultat comparable. Tant que personne
> d'autre que l'auteur n'a lancé le protocole, cette case n'est pas cochée.
>
> **Ce n'est pas un test d'amélioration.** Le code de l'allocateur n'a pas
> changé depuis le 17 octobre 2026. Si tu obtiens les mêmes chiffres, c'est
> précisément le résultat attendu.
>
> Environnement de référence : `POC/ferrum_Poc_adaptative_sheduller/TEST_ENVIRONMENT.md`
> Note de choix de l'axe et des indicateurs : `docs/memoire/OBJECTIF_D_NOTE_DE_CHOIX.md`

---

## 1. Ce que tu vas mesurer

Trois indicateurs. Les deux premiers sont déterministes : ils doivent tomber sur
le chiffre exact. Le troisième est bruité : un écart est normal.

| Indicateur | Ce que c'est | Valeur attendue |
|---|---|---|
| I1 | itérations d'allocation et libération avant le premier refus | **99** sur 10 000, exactement |
| I2 | blocs de 1 Kio vivants simultanément | **100**, exactement |
| I3 | cycles processeur par itération | entre **134 000 et 175 000**, large tolérance |

Plus les contrôles d'intégrité, qui doivent tous être à zéro : erreurs de
corruption, erreurs d'alignement, erreurs de traduction d'adresse, cadres
physiques perdus.

---

## 2. Prérequis

### QEMU

Nécessaire pour démarrer le noyau. Version de référence : **11.1.0**.

Sous Windows, installeur officiel depuis `qemu.weilnetz.de`, puis vérifier que
le dossier d'installation est bien dans le `PATH` :

```
qemu-system-x86_64 --version
```

Si la commande n'est pas trouvée, ajoute `C:\Program Files\qemu` au `PATH` et
ouvre un nouveau terminal.

**Si ta version n'est pas 11.1.0, ne la change pas.** Lance les tests avec ce
que tu as et note ta version dans le compte rendu. Une différence de
comportement entre deux versions de QEMU serait un résultat intéressant.

### Rust

La version du compilateur est épinglée dans `rust-toolchain.toml`, donc rustup
installe la bonne tout seul au premier `cargo`. Il faut juste rustup :

```
rustup --version
```

### bootimage

Construit l'image bootable. La version est imposée, les plus récentes ne
fonctionnent pas avec ce projet :

```
cargo install bootimage --version 0.10.3
```

---

## 3. Récupérer le code

**Place-toi sur le commit exact, pas seulement sur la branche.** La branche
`test/memory` va continuer d'évoluer, et le but est de mesurer l'état de
référence.

```
git clone https://github.com/EpitechPGE3-2025/G-EIP-601-ALL-6-1-eip-81.git
cd G-EIP-601-ALL-6-1-eip-81
git checkout b0f8753
cd POC/ferrum_Poc_adaptative_sheduller
```

---

## 4. Lancer la mesure

Le protocole demande **3 exécutions**. Garde la sortie complète dans un
fichier, c'est elle qui sert de preuve, pas la capture d'écran.

```
cargo test 2>&1 | tee resultat-TONPRENOM-passe1.log
cargo test 2>&1 | tee resultat-TONPRENOM-passe2.log
cargo test 2>&1 | tee resultat-TONPRENOM-passe3.log
```

La première exécution est longue : elle compile le noyau, la bibliothèque
standard pour une cible sur mesure, et construit sept images bootables. Les
suivantes sont rapides.

Chaque exécution démarre sept machines virtuelles : une pour les neuf tests de
capacité du harnais, et une par expérience. C'est voulu : chaque expérience
tourne dans un noyau fraîchement démarré, sur un tas intact, pour que rien ne
circule d'une expérience à l'autre.

Pour relancer une seule expérience :

```
cargo test --test experiment_2
```

---

## 5. Lire les résultats

Chaque expérience imprime une ligne de relevé qui commence par `RECORD`. Pour
les retrouver dans ton log :

```
grep RECORD resultat-TONPRENOM-passe1.log
```

Voici ce que tu dois voir, et où regarder :

### Expérience 1, correction de l'allocateur

```
RECORD name=exp1_alloc_free_sequence iterations=10000 errors=0 used_before=0 used_after=0 cycles=...
```

`iterations` doit être 10000 et `errors` doit être 0. Divise `cycles` par 10000
pour obtenir I3.

### Expérience 2, réutilisation de la mémoire, c'est la mesure clé

```
RECORD name=exp2_long_lived_vs_churn iterations=99 errors=0 used_before=0 used_after=101632 cycles=...
```

`iterations=99` est **le chiffre central de tout l'objectif D**. La boucle
devait faire 10 000 tours, elle s'arrête à 99 parce que l'allocateur ne réutilise
jamais la mémoire libérée. `used_after=101632` doit aussi être exact.

### Expérience 3, épuisement et reprise

```
capacity per round: 100 / 100 / 100 blocks of 1024 B
RECORD name=exp3_exhaustion_and_recovery iterations=3 errors=0 ...
```

Les trois tours doivent donner 100 blocs.

### Expérience 4, cycles de pages

```
frames free before the loop: N after 500 cycles: N (lost: 0)
translation errors: 0
```

`lost` doit être 0 et les deux nombres de cadres doivent être identiques.

### Expérience 5, allocateur sous interruptions

```
RECORD name=exp5_under_interrupts iterations=10200 errors=0 ...
```

`errors` doit être 0.

### Code de sortie

En fin d'exécution, `cargo test` doit indiquer que les sept images sont passées.
Un code de sortie 33 signifie que tous les tests ont réussi, 35 qu'un test a
échoué.

---

## 6. Ce que tu me renvoies

Les captures d'écran ne suffisent pas. Il faut ces sept éléments :

1. ton prénom et la date de l'exécution
2. ta version de QEMU, telle que `qemu-system-x86_64 --version` l'affiche
3. le temps écoulé entre le clonage et ton premier résultat
4. l'endroit où tu as bloqué, et ce qui manquait dans la documentation
5. les trois fichiers `resultat-TONPRENOM-passeN.log` complets
6. une capture de la fin de la sortie, pour le dossier de preuves
7. le formulaire ci-dessous rempli

### Formulaire à remplir

```
Prénom                        :
Date de l'exécution           :
Système                       :
Version de QEMU               :
Version de Rust (rustc -V)    :
Temps jusqu'au 1er résultat   :

                                 passe 1    passe 2    passe 3
I1, itérations avant refus     :
I2, blocs vivants              :
I3, cycles par itération       :
Erreurs totales                :
Cadres perdus (expérience 4)   :

Blocages rencontrés            :


Ce qui manquait dans la doc    :


```

---

## 7. Et si tu n'obtiens pas les mêmes chiffres

**Ne corrige rien et ne relance pas en changeant quelque chose.** Préviens
d'abord, avec ton log complet.

| Cas | Interprétation |
|---|---|
| I3 différent de plus ou moins 15 % | normal, QEMU tourne sans accélération matérielle. Rien à signaler |
| I1 différent de 99 | **important.** Quelque chose diffère dans l'environnement ou la taille du tas. C'est un résultat en soi, pas une erreur de ta part |
| I2 différent de 100 | même chose, à signaler tel quel |
| Une erreur ou un cadre perdu non nul | **le plus important de tous.** Cela voudrait dire que le harnais laisse passer un défaut selon la machine |
| Le noyau ne démarre pas, triple faute | probablement une version de bootimage ou de Rust différente. Vérifie les prérequis et note ce que tu avais |

Un désaccord entre deux machines est une information utile pour l'objectif D.
Un désaccord qu'on efface en bidouillant ne l'est pas.
