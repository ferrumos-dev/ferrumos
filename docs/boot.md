# Démarrer et vérifier FerrumOS

Le noyau du projet racine démarre dans QEMU, affiche une console texte dans le
framebuffer et reçoit les touches d'un clavier PS/2. Le code exécuté vient de
`src/` ; aucun exécutable des PoC n'est utilisé pour construire cette image.

La console affiche les caractères saisis et permet de corriger une ligne. Elle
ne possède pas encore de parser ni de commandes de shell. Le jalon couvert ici
est le démarrage, l'affichage, les diagnostics et la saisie.

## Préparer les outils

Installer `rustup`, Python **3.11 ou plus récent** et QEMU x86_64. Depuis la racine
du dépôt, préparer les chaînes épinglées :

```sh
rustup toolchain install 1.94.0 --profile minimal --component rustfmt,clippy --target x86_64-unknown-none
rustup toolchain install nightly-2026-03-07 --profile minimal --component rust-src,llvm-tools-preview,rustfmt,clippy
```

La première construction télécharge également les dépendances verrouillées dans
les deux `Cargo.lock`. Elle nécessite le réseau. Les versions ont des rôles
distincts :

| Composant | Chaîne et dépendances |
| --- | --- |
| Bibliothèque et noyau dans `src/` | Rust `1.94.0`, cible intégrée `x86_64-unknown-none` |
| Interface de démarrage du noyau | `bootloader_api = 0.11.15` |
| Constructeur d'image hôte dans `tools/image-builder/` | `nightly-2026-03-07`, `bootloader = 0.11.15`, support BIOS uniquement |
| Affichage et décodage clavier | `font8x8 = 0.3.1`, `pc-keyboard = 0.7.0` |

Le constructeur d'image possède son propre manifeste, lockfile et fichier de
chaîne Rust. Il est exclu du workspace racine, comme les PoC, `libs/ferrum_alloc`
et le banc `research/heap-allocator/bench`. Les commandes du lanceur sélectionnent
explicitement la chaîne attendue pour chaque étape ; une chaîne héritée du terminal
ne remplace pas celle du noyau ou du constructeur.

Sur Linux, installer le paquet `qemu-system-x86` de la distribution. Sous Windows,
le lanceur cherche `qemu-system-x86_64` dans `PATH`, puis à l'emplacement
`%ProgramFiles%\qemu\qemu-system-x86_64.exe`. Pour une installation différente,
définir la variable `QEMU` avec le chemin de l'exécutable, sans arguments :

```powershell
$env:QEMU = 'D:\Outils\qemu\qemu-system-x86_64.exe'
```

Sur un système POSIX :

```sh
export QEMU=/chemin/vers/qemu-system-x86_64
```

## Construire et lancer

Depuis la racine du dépôt :

```sh
python scripts/boot.py build
python scripts/boot.py run
```

Utiliser `python3` si c'est le nom de Python 3 sur le poste. `build` construit le
noyau puis l'image `target/boot/ferrumos-bios.img`. `run` reconstruit également
l'image avant de lancer QEMU. Une étape en échec interrompt la commande ; le lanceur
ne réutilise pas une ancienne image après une erreur de compilation.

La fenêtre affiche la version de FerrumOS et une invite `>`. Cliquer à l'intérieur
pour saisir du texte. `Ctrl+Alt` libère la capture clavier/souris de QEMU.

| Saisie | Comportement |
| --- | --- |
| Lettres et symboles | Disposition **AZERTY française**, caractères UTF-8 affichables, Shift, Caps Lock et AltGr |
| Retour arrière | Retire le dernier caractère complet, y compris un caractère accentué |
| Échap | Efface la ligne en cours |
| Entrée | Termine la ligne, écrit son contenu dans le journal série et affiche une nouvelle invite |
| Ligne pleine | Les caractères supplémentaires sont ignorés ; les caractères déjà saisis restent intacts |

La ligne contient au maximum **128 octets UTF-8**, donc éventuellement moins de
128 caractères. Les touches de navigation, raccourcis de contrôle et touches de
fonction ne sont pas interprétés comme des commandes. La console ne traite pas
les touches mortes/compositions. Tab n'insère pas de caractère dans la ligne.
La disposition US existe dans l'API du pilote ; le démarrage sélectionne AZERTY.
La police couvre ASCII et Latin-1. Un autre caractère, comme le symbole euro,
apparaît comme `?` à l'écran tout en restant intact dans la ligne UTF-8 et le
journal série.

## Chemin de démarrage

```text
scripts/boot.py
  -> noyau ELF : src/main.rs, compilé avec Rust 1.94.0
  -> image BIOS : tools/image-builder, compilé avec la nightly épinglée
  -> QEMU -> bootloader 0.11.15 -> BootInfo -> boot::run
  -> COM1 -> framebuffer -> clavier PS/2 -> boucle d'édition de ligne
```

Le bootloader fournit le contexte initial, la pile, la carte mémoire et le
framebuffer. `src/main.rs` déclare l'ABI attendue par `bootloader_api`, ainsi que
le traitement terminal des paniques. `src/boot/runtime.rs` possède les pilotes
pendant toute la boucle d'entrée.

- COM1 émet à **115200 bauds, 8 bits, sans parité, 1 bit d'arrêt**. Les attentes
  matérielles sont bornées et les fins de ligne série utilisent CRLF.
- La console framebuffer valide format, dimensions, stride et longueur avant
  d'écrire. Elle prend en charge RGB, BGR et niveaux de gris 8 bits, avec défilement
  et effacement du caractère précédent.
- Le pilote PS/2 configure la traduction vers le jeu de scan codes 1, désactive
  les IRQ clavier/souris et le port souris, puis active la saisie clavier. Ses
  attentes et tentatives de commande sont bornées.

Les opérations privilégiées sont encapsulées dans `src/arch/x86_64/` et les
pilotes, avec leurs préconditions `SAFETY`. Le module portable `kernel` conserve
son interdiction du code `unsafe`.

## Contrôles et preuves

Le test matériel utilise l'image réelle issue de la même construction :

```sh
python scripts/boot.py test --timeout 30
```

Le délai de 30 secondes couvre l'exécution et les échanges de vérification avec
QEMU, après la construction. Le test démarre QEMU sans fenêtre et :

1. attend les lignes série `FERRUMOS_BOOT_OK`, `DISPLAY_READY` et `KEYBOARD_READY` ;
2. injecte les touches physiques `t`, `e`, `s`, `x`, Retour arrière, `t`, Entrée
   via le moniteur QMP ;
3. attend l'acquittement exact `INPUT_LINE: test`, issu du clavier PS/2 et de la
   boucle d'édition exécutés dans le noyau ;
4. capture le framebuffer et vérifie ses dimensions, sa complétude et la présence
   de plusieurs couleurs ;
5. termine QEMU proprement et écrit le verdict.

Une panique, un marqueur manquant, une image uniforme, une fermeture prématurée
de QEMU ou un délai dépassé font échouer la commande. Le contrôle de l'image
détecte un écran vide ; il ne constitue pas une reconnaissance de texte ni une
validation de la lisibilité de tous les glyphes.

Les tests Rust sur l'hôte couvrent les limites du framebuffer, le défilement,
l'effacement, les dispositions et modificateurs clavier, les erreurs PS/2 et la
ligne UTF-8. Les tests Python contrôlent notamment le lanceur et ses chemins
d'échec. Les commandes figurent dans le [README](../README.md#quality-checks).

Le [workflow racine](../.github/workflows/kernel.yml) exécute ces contrôles sur
Linux et conserve les fichiers de diagnostic. Les contrôles historiques du tas et
du PoC scheduler gardent leur [workflow séparé](../.github/workflows/kernel-tests.yml).

## Diagnostics

Les fichiers générés restent dans `target/`, ignoré par Git.

| Fichier | Contenu |
| --- | --- |
| `target/boot/build.log` | Commandes et sorties de construction du noyau et de l'image |
| `target/boot/ferrumos-bios.img` | Image BIOS à lancer, publiée seulement après construction réussie |
| `target/boot/serial.log` | Journal série d'une session interactive `run` |
| `target/boot/results/serial.log` | Journal série du test automatisé |
| `target/boot/results/qemu.stderr.log` | Sortie et erreurs de QEMU pendant le test |
| `target/boot/results/screen.ppm` | Capture framebuffer utilisée pour la vérification |
| `target/boot/results/summary.json` | Verdict, commande QEMU, délai, durée, empreinte SHA-256 de l'image et résultat observé |

Pour une PR, joindre les diagnostics de la révision testée ou le lien de l'artefact
CI. Un succès de compilation seul ne prouve pas que les pilotes fonctionnent.
Les journaux d'une ancienne image ne remplacent pas une nouvelle exécution après
modification du démarrage ou des pilotes.

## Interfaces pour la suite du projet

| Responsable | Point d'intégration |
| --- | --- |
| Benjamin | `src/boot/` conserve l'ordre d'initialisation ; `TextConsole`, `SerialPort` et `Ps2Keyboard` possèdent leurs ressources. Les accès aux ports passent par `arch::x86_64::io`. |
| Elias | Consommer `InputEvent` et `InputLine<N>` pour le futur shell, puis utiliser `core::fmt::Write` pour les sorties. Le parser ne doit pas relire directement les ports clavier. |
| Mathias | Raccorder mémoire, tas et IRQ après avoir défini leurs invariants. Pour remplacer la scrutation, établir IDT, routage, acquittement et propriété du contrôleur avant d'activer les interruptions. |
| Rayan | Brancher les futurs relevés de télémétrie et le moniteur sur les interfaces du noyau et la sortie texte, hors des sections critiques. Aucun relevé mémoire synthétique n'est publié par cette console. |

`Ps2Keyboard::poll_event` consomme au plus un octet disponible par appel et produit
des événements typés. Le décodage clavier et `InputLine` sont réutilisables sans
accès matériel. `TextConsole` implémente `core::fmt::Write` sur un emprunt mutable
du framebuffer ; transmettre cet emprunt ou une interface de sortie explicite
aux consommateurs plutôt que créer un second propriétaire global.

## Périmètre actuel

La configuration de référence est **BIOS, machine QEMU `pc`, accélération TCG,
un CPU, 128 Mio de RAM et affichage `std`**. Le démarrage conserve les interruptions
masquables désactivées et attend les touches par scrutation, ce qui occupe le CPU.
L'attente bloquante avec réveil matériel dépendra de l'intégration des IRQ.

Le noyau courant n'initialise pas encore de tas, de gestionnaire de paging propre
à FerrumOS, de timer, d'ordonnanceur ni de shell à commandes. Les services de
fichiers/réseau, l'UEFI, le démarrage multiprocesseur et le matériel réel ne sont
pas validés par ce scénario.

Références techniques : [API de démarrage](https://docs.rs/bootloader_api/0.11.15/bootloader_api/),
[constructeur BIOS](https://docs.rs/bootloader/0.11.15/bootloader/),
[décodage PS/2](https://docs.rs/pc-keyboard/0.7.0/pc_keyboard/)
et [architecture du projet](./architecture.md).
