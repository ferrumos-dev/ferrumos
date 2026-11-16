# Registre des décisions techniques

Consigner une décision au moment de sa validation. Pour chaque entrée, garder le contexte, les options réellement étudiées, les conséquences et les preuves. Une proposition conserve ce statut jusqu’à l’arbitrage de l’équipe.

| Sujet | Statut | Proposition et alternative à comparer | Preuve à joindre après arbitrage |
| --- | --- | --- | --- |
| Architecture du noyau | Proposée | Monolithique modulaire pour intégrer les PoC. Micronoyau pour déplacer des services derrière des frontières de protection. | [Note de décision](../pdf/01_FerrumOS_Note_de_decision.pdf), choix daté, coût accepté et schéma mis à jour |
| Politique de sélection | Proposée | Priorités fixes et FIFO à priorité égale. Comparer le quantum par priorité et le calcul adaptatif. | [Spécification et scénarios S01 à S07](../../objecitf_obligatoire_1/ordonnancement_a_priorites.md), résultats et politique retenue |
| Allocateur du tas noyau | Proposée | Liste chaînée triée par adresse avec fusion (A). Comparée au bump existant, aux blocs fixes avec et sans récupération (B, B′) et à TLSF, selon les critères C1 à C7. | [Note de décision](../../objecitf_obligatoire_1/tas_noyau/08_note_decision.md), [mesures](../../objecitf_obligatoire_1/tas_noyau/06_prototype_decision.md), [preuve d’intégration](../../objecitf_obligatoire_1/tas_noyau/09_integration.md), PR relue et exécution CI |
| Validation en CI | Proposée | Contrôles séparés pour qualité, compilation et QEMU. Comparer avec la validation limitée à la compilation des PoC. | [Plan CI](./ci_cd.md), lien des exécutions nominale et en échec, conditions de fusion |

## Entrées

### ADR-002 · Allocateur du tas noyau

- 2026-11-12 · Proposée, à valider en réunion d’équipe et en revue de PR
- Contexte. Avec l’allocateur bump, l’expérience 2 s’arrête à 99 itérations sur 10 000 : la mémoire libérée n’est réutilisée que lorsque tout est libéré. [Caractérisation et critères C1 à C7](../../objecitf_obligatoire_1/tas_noyau/01_caracterisation_probleme.md), fixés avant la mesure des candidats.
- Options. Bump (référence), A liste chaînée avec fusion, B blocs fixes, B′ blocs fixes avec récupération, TLSF en exploration. Mêmes scénarios sur l’hôte et sous QEMU, trois exécutions chacune.
- Option choisie. A. A et B′ atteignent tous les seuils ; A est retenu pour son pire cas (22–77 ns contre 243–343 µs verrou tenu pour B′), le refus des doubles libérations et sa taille.
- Conséquences acceptées. Coût en O(nombre de trous), granularité de 16 o, code maison avec des blocs `unsafe` à relire, verrou sans masquage des interruptions.
- Fichiers. `libs/ferrum_alloc/src/linked_list.rs`, `POC/ferrum_Poc_adaptative_sheduller/src/allocator.rs` (feature Cargo `heap-linked-list` par défaut).
- Preuves. [Mesures](../../objecitf_obligatoire_1/tas_noyau/06_prototype_decision.md) et [intégration](../../objecitf_obligatoire_1/tas_noyau/09_integration.md). Lien de PR, révision fusionnée et exécution CI à renseigner après revue.
- Réexamen. p99 de `long_run` au-dessus de 500 ns, fragmentation au-dessus de 500 ‰, ou contention révélée par un vrai changement de contexte.

### ADR-003 · Démarrage BIOS et pilotes du projet commun

- 2026-11-16 · Proposée, implémentée sur la branche de travail et à relire en équipe.
- Contexte. Le projet commun sous `src/` doit produire sa propre image et fournir affichage et saisie, avec des modules réutilisables pour le shell et le noyau.
- Options examinées. Reprendre la chaîne `bootloader 0.9`/VGA des PoC, ou utiliser `bootloader 0.11.15` avec son ABI framebuffer. Les critères sont la séparation des responsabilités, la maîtrise des versions et la possibilité de tester les pilotes sur l'hôte puis dans l'image réelle.
- Choix implémenté. `bootloader_api 0.11.15` dans le noyau compilé avec Rust `1.94.0`, et `bootloader 0.11.15` dans un constructeur d'image hôte isolé, épinglé sur `nightly-2026-03-07`. Le framebuffer et les événements clavier sont exposés par les pilotes ; une ligne UTF-8 de capacité fixe prépare leur utilisation par le shell.
- Conséquences. Deux chaînes Rust à préparer, interface de boot à maintenir avec des versions identiques, clavier PS/2 scruté sur un seul CPU avec les interruptions désactivées. Le test couvre BIOS/QEMU ; ni l'UEFI ni le matériel réel ne sont validés. La scrutation occupe le CPU jusqu'à l'intégration des IRQ.
- Fichiers. `src/main.rs`, `src/boot/`, `src/arch/x86_64/`, `src/drivers/`, `tools/image-builder/`, `scripts/boot.py` et `.github/workflows/kernel.yml`.
- Preuves. Tests hôte des pilotes et de la ligne ; `python scripts/boot.py test --timeout 30` sur l'image intégrée, avec marqueurs série, saisie/correction PS/2 et capture framebuffer non uniforme. Joindre le `summary.json`, les journaux et la révision de la PR. [Protocole et limites](../../../boot.md#contrôles-et-preuves).
- Réexamen. Intégration du timer/IRQ et de l'ordonnanceur, besoin de plusieurs CPU, nouveau protocole de boot ou exigences dépassant la console initiale.

## Modèle d’entrée

```markdown
### Titre de la décision

- Date et statut
- Contexte et problème
- Options examinées et comparaison selon les mêmes critères
- Option choisie et raison
- Conséquences acceptées et condition de réexamen
- Liens vers le schéma, la PR et les tests concernés
```

Pour une décision révisée, conserver l’entrée initiale. Ajouter la décision qui la remplace et le fait ou la mesure qui justifie le changement.

## Rédiger et utiliser une décision dans une PR

Écrire l’entrée dans ce registre au moment où la PR introduit le choix. Reprendre son identifiant dans la description de la PR et dans la stratégie de tests. Utiliser une date `AAAA-MM-JJ`, le statut `Proposée`, `Acceptée` ou `Remplacée` et au moins deux options évaluées sur les mêmes critères. Indiquer le fichier touché, la preuve à obtenir et les conditions qui justifieraient une révision.

### Exemple à adapter pour la politique de priorité

Dans `docs/greenlights/objecitf_obligatoire_2/md/registre_decisions.md`, ajouter une entrée comme celle-ci lorsque l’équipe ouvre la discussion. Compléter la date, le statut et les liens dès qu’une décision ou un résultat est disponible.

```markdown
### ADR-001 · Règle de choix de la prochaine tâche

- Date à renseigner lors de la revue
- Statut proposé
- Contexte. Le PoC adaptatif simule la progression des tâches.
- Option A. Priorité fixe, ordre FIFO à priorité égale.
- Option B. Priorité par niveau avec un quantum pour départager les tâches de même niveau.
- Critères. Ordre observé, attente maximale et coût de la sélection.
- Proposition. A, pour tester un contrat déterministe à partir de séquences d’événements.
- Contrepartie. Une charge haute continue peut retarder les tâches basses.
- Fichier. POC/ferrum_Poc_adaptative_sheduller/src/scheduler.rs, pour lire le simulateur.
- Preuves. S01 à S06 sur le sélecteur portable, puis S07 sur l’image intégrée.
- Liens. Issue, PR, révision, journal et schéma à renseigner après validation.
- Réexamen. Attente mesurée incompatible avec le besoin du shell ou des tâches de fond.
```

### Transformer le choix en vérifications de code

Associer chaque règle à une observation distincte. La règle « priorité haute d’abord » correspond à S02 et à l’identifiant retourné. La règle FIFO correspond à S03 et S04, avec l’ordre avant et après cession ou réveil. La preuve d’une exécution CPU correspond à S07 et à une trace de bascule. Relier les scénarios à la PR et préciser le périmètre de chaque preuve.

Pour relever la révision et vérifier les fichiers avant revue, lancer depuis la racine du dépôt.

```sh
git rev-parse HEAD
git diff --check
git diff -- 'docs/greenlights/objecitf_obligatoire_2/md/registre_decisions.md'
```

**Critère de revue.** Le relecteur peut relier la règle retenue à un fichier, une modification et un scénario d’acceptation. Lorsqu’une mesure ou un test motive un changement, conserver les deux entrées et relier la preuve à la décision qui remplace l’ancienne.
