# Démarrer et vérifier FerrumOS

Le noyau du projet racine démarre dans QEMU, affiche une console texte dans le
framebuffer et reçoit les touches d'un clavier PS/2. Le code exécuté vient de
`src/` ; aucun exécutable des PoC n'est utilisé pour construire cette image.

La console affiche les caractères saisis et permet de corriger une ligne.
Le chemin de démarrage initialise aussi le tas réutilisable et le timer
périodique. Le parser de commandes de shell reste à intégrer.

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
| Pages du tas et sections critiques | `x86_64 = 0.15.5`, `spin = 0.12.3`, crate locale `ferrum_alloc` |

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
  -> COM1 -> carte mémoire physique -> pages du tas -> vérification allocate/free
  -> framebuffer -> clavier PS/2 -> exécuteur coopératif -> IDT et PIC/PIT
  -> activation IF -> tâches de saisie et diagnostic -> attente du timer
```

Le bootloader fournit le contexte initial, la pile, la carte mémoire, le mapping
physique demandé dans `BootloaderConfig` et le framebuffer. `src/main.rs` déclare
l'ABI attendue par `bootloader_api`, ainsi que
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
- Le tas mappe 1 Mio à `0x4444_4444_0000` à partir de frames utilisables entières.
  `LinkedListHeap` réutilise et fusionne les blocs libérés. Les allocations sont
  des handles RAII, avec restitution automatique au `drop` et relevés cohérents
  protégés par le même verrou. Les erreurs d'initialisation partielle imposent
  un redémarrage avant une nouvelle tentative.
- L'IDT route IRQ0 depuis le PIC maître vers le handler PIT. Seule cette IRQ est
  démasquée. Le PIT utilise le diviseur `11_932` pour une entrée `1_193_182` Hz.
  Le handler enregistre un tick puis acquitte le PIC. Le démarrage active IF
  après cette installation. `sti; hlt` attend ensuite un réveil à chaque tour.

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

1. attend `FERRUMOS_BOOT_OK`, `MEMORY_MAP_READY`, `HEAP_READY`,
   `HEAP_ALLOC_FREE_OK`, `DISPLAY_READY`, `KEYBOARD_READY`, `TIMER_READY`, `HEAP_IRQ_SAFE_OK`,
   `INPUT_TASK_READY` et `KERNEL_PRIORITY_OK`, puis vérifie les totaux mémoire
   pour la machine de 128 Mio et `HEAP_SAMPLE capacity=1048576 used=0 free=1048576`.
   Le contrôle noyau alloue deux blocs, écrit et relit leur contenu, vérifie leur
   indépendance, libère et réutilise un bloc, puis provoque un refus d'allocation
   trop grande avant de vérifier la récupération complète. Il répète cette
   vérification après activation des IRQ et contrôle que les verrous ont
   restauré le drapeau IF de l'appelant avant `HEAP_IRQ_SAFE_OK`.
2. exige deux lignes `TIMER_SAMPLE` avec des ticks strictement croissants et une
   conversion en millisecondes conforme à la fraction du PIT. Une IRQ unique ou
   un compteur figé échoue à ce contrôle.
3. injecte les touches physiques `t`, `e`, `s`, `x`, Retour arrière, `t`, Entrée
   via le moniteur QMP ;
4. attend l'acquittement exact `INPUT_LINE: test`, issu du clavier PS/2 et de la
   tâche coopérative d'édition exécutée dans le noyau ;
5. capture le framebuffer et vérifie ses dimensions, sa complétude et la présence
   de plusieurs couleurs ;
6. termine QEMU proprement et écrit le verdict.

Une panique, un marqueur manquant, une image uniforme, une fermeture prématurée
de QEMU ou un délai dépassé font échouer la commande. Le contrôle de l'image
détecte un écran vide ; il ne constitue pas une reconnaissance de texte ni une
validation de la lisibilité de tous les glyphes.

Les tests Rust sur l'hôte couvrent les limites du framebuffer, le défilement,
l'effacement, les dispositions et modificateurs clavier, les erreurs PS/2 et la
ligne UTF-8. Ils couvrent aussi les erreurs de carte mémoire, les invariants des
relevés, les priorités, l'ordre FIFO, le blocage/réveil des tâches et les compteurs.
Les tests hôte du tas utilisent une région possédée pour vérifier allocation,
zéro-initialisation, libération, réutilisation, épuisement et refus d'une seconde
initialisation. Les conversions d'horloge couvrent les paramètres invalides et
les bornes arithmétiques.
Les tests Python contrôlent notamment le lanceur et ses chemins
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
| `target/boot/results/summary.json` | Verdict, empreinte SHA-256 de l'image, commande QEMU, délai, durée, mémoire physique, relevé du tas, deux observations du timer et framebuffer |

Pour une PR, joindre les diagnostics de la révision testée ou le lien de l'artefact
CI. Un succès de compilation seul ne prouve pas que les pilotes fonctionnent.
Les journaux d'une ancienne image ne remplacent pas une nouvelle exécution après
modification du démarrage ou des pilotes.

## Interfaces pour la suite du projet

| Responsable | Point d'intégration |
| --- | --- |
| Benjamin | `src/boot/` conserve l'ordre d'initialisation ; `TextConsole`, `SerialPort` et `Ps2Keyboard` possèdent leurs ressources. Les accès aux ports passent par `arch::x86_64::io`. |
| Elias | Consommer `InputEvent` et `InputLine<N>` pour le futur shell, puis utiliser `core::fmt::Write` pour les sorties. Le parser ne doit pas relire directement les ports clavier. |
| Mémoire et timer | `arch::x86_64::memory::heap()` rend le tas initialisé. Ses allocations sont possédées et libérées au `drop`. L'adaptateur timer possède l'IDT et les PIC/PIT. Étendre les IRQ après définition de leurs propriétaires et de leur acquittement. |
| Monitoring | Importer `kernel::memory::MemoryReporter` et lire `heap.memory_snapshot()`. `heap.stats()` expose aussi les tailles demandées vivantes, allocations et échecs. Pour le temps, capturer `arch::x86_64::interrupts::clock_snapshot()` puis afficher les valeurs du même `TickSnapshot`, hors IRQ et hors verrou. |

`Ps2Keyboard::poll_event` consomme au plus un octet disponible par appel et produit
des événements typés. Le décodage clavier et `InputLine` sont réutilisables sans
accès matériel. `TextConsole` implémente `core::fmt::Write` sur un emprunt mutable
du framebuffer ; transmettre cet emprunt ou une interface de sortie explicite
aux consommateurs plutôt que créer un second propriétaire global.

## Périmètre actuel

La configuration de référence est **BIOS, machine QEMU `pc`, accélération TCG,
un CPU, 128 Mio de RAM et affichage `std`**. Le clavier reste scruté par les tâches
coopératives, tandis que le timer réveille le CPU en attente. Les sections
critiques préservent l'état antérieur d'IF. Le handler timer garde les allocations,
verrous et sorties console dans l'exécution normale.

Le noyau exécute les fonctions de tâches sur une pile partagée. Le tas a une
capacité fixe de 1 Mio. Sa mesure `used` inclut l'arrondi et l'alignement, tandis
que `HeapStats::live_bytes` suit les tailles demandées encore vivantes. La durée
du timer est dérivée des IRQ livrées, avec arrondi à la milliseconde inférieure.
La préemption, les piles de tâche indépendantes, le shell à commandes, les services
de fichiers/réseau, l'UEFI, le multiprocesseur et le matériel réel demandent des
scénarios supplémentaires.

Références techniques : [API de démarrage](https://docs.rs/bootloader_api/0.11.15/bootloader_api/),
[constructeur BIOS](https://docs.rs/bootloader/0.11.15/bootloader/),
[décodage PS/2](https://docs.rs/pc-keyboard/0.7.0/pc_keyboard/)
et [architecture du projet](./architecture.md).
