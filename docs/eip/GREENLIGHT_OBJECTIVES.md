# FerrumOS - Objectifs pour les Greenlights

Proposition préparée le 22 septembre 2026 pour le parcours Technical Track, Tech 4, année 2026/2027.

Le guide prévoit **deux objectifs obligatoires et deux objectifs complémentaires**, choisis avec le mentor à la troisième séance, en octobre, puis conservés pour l'année. Les complémentaires retenus pour cette proposition sont **C : travailler avec des experts techniques** et **D : mesurer et optimiser les performances**. Ce choix est préparé à la demande de l'équipe et sera validé ultérieurement avec le mentor. Les autres possibilités du guide sont A : protéger et valoriser la technologie, et B : ouvrir le projet aux contributions extérieures.

**Pourquoi C et D pour FerrumOS ?** Le projet concentre des choix bas niveau exigeants sur la mémoire, les interruptions et l'ordonnancement : un regard expert peut améliorer ces décisions. Il contient aussi un allocateur et des compteurs à faire évoluer, ce qui offre un terrain concret pour mesurer et démontrer des progrès. Ces deux objectifs correspondent aux périmètres déjà répartis entre les quatre membres.

Les nombres de livrables et les échéances scolaires viennent du guide. Les critères techniques propres à FerrumOS et la répartition des pilotes sont des propositions : ils ne constituent ni des résultats déjà obtenus ni des engagements déjà validés.

## Liste synthétique à présenter

| Objectif | Application à FerrumOS | Résultat à démontrer au jury |
|---|---|---|
| **1. Assurer une veille et résoudre le verrou technique du projet** - obligatoire | Étudier, comparer et intégrer un ordonnancement adapté à un noyau Rust, capable de faire progresser les tâches tout en préservant la réactivité. | Une comparaison mesurée, un choix justifié et une solution intégrée au noyau avec tests et démonstration. |
| **2. Structurer, documenter et fiabiliser l'architecture technique** - obligatoire | Faire converger les prototypes vers un socle commun, fiabiliser boot, mémoire, interruptions et shell, puis rendre le projet testable et reproductible. | Une image de référence, une architecture documentée, des vérifications bloquantes et un lancement réussi par une personne extérieure. |
| **3. Confronter nos choix à des experts techniques** - complémentaire C proposé | Faire examiner l'ordonnancement, la gestion mémoire et les zones critiques en Rust par des spécialistes externes. | Deux échanges effectifs, deux modifications traçables et une recommandation écartée avec justification. |
| **4. Mesurer et optimiser la gestion mémoire** - complémentaire D proposé | Instrumenter l'allocateur, mesurer ses limites et améliorer la réutilisation de la mémoire sur des charges reproductibles. | Deux cycles d'optimisation accompagnés de mesures avant/après et de contrôles de non-régression. |

## Point de départ observé dans le dépôt

Cette préparation repose sur la lecture de la documentation, du code et de la configuration locale. Aucune compilation ni exécution sous QEMU n'a été effectuée pour cette analyse.

- Trois prototypes distincts sont présents : noyau de référence, ordonnanceur adaptatif et interface graphique. Les chaînes de démarrage diffèrent : `bootloader 0.9` pour le noyau de référence et `bootloader 0.11` pour le prototype graphique.
- Le prototype d'ordonnancement contient des files de priorités, des quantums, un mécanisme de vieillissement et les commandes `tasks` et `stats`. La progression des tâches repose sur `simulate_tick()` ; elle ne démontre pas encore l'exécution de plusieurs tâches avec sauvegarde et restauration de leur contexte processeur.
- Les statistiques d'attente doivent être fiabilisées avant toute comparaison : le compteur d'attente est remis à zéro lors de la sélection d'une tâche, puis utilisé pour les statistiques à sa terminaison.
- L'allocateur Bump réutilise le tas lorsque toutes les allocations ont été libérées. La conservation d'une allocation persistante peut donc empêcher la récupération des espaces temporaires libérés.
- La configuration CI prévoit des compilations et des contrôles de qualité, mais neutralise les échecs de formatage et de Clippy. Aucun lancement de tests n'y est configuré. L'historique distant et les protections de branche restent à vérifier.

## Objectif 1 - Veille et résolution du verrou technique

**Formulation à présenter :** « Nous voulons concevoir et intégrer un ordonnancement adapté à FerrumOS, en nous appuyant sur une veille organisée et sur des expérimentations comparables, afin de préserver la réactivité sans empêcher les autres tâches de progresser. »

**Verrou proposé :** passer du démonstrateur d'ordonnancement simulé à une gestion de tâches réellement exécutées par le noyau, avec une politique de sélection mesurée, des changements de contexte sûrs et des interactions maîtrisées avec les interruptions.

### Travail prévu

1. Décrire le problème, les contraintes et les limites d'une première approche, avec une mesure initiale avant fin octobre.
2. Comparer au moins **trois approches** : Round Robin, priorités avec vieillissement, et files multiniveaux adaptatives (MLFQ). Comparer réactivité, équité, coût mémoire et complexité d'intégration.
3. Construire le **prototype de décision** : implémenter deux solutions candidates sur le même scénario et conserver leur code, leur protocole et leurs mesures. Le prototype actuel peut servir de point de départ après correction de l'instrumentation.
4. Réaliser un **prototype exploratoire distinct**, par exemple sur l'ajustement du quantum à partir du comportement observé des tâches. Écrire l'hypothèse avant l'expérience et documenter le résultat, même négatif.
5. Décider sur la base des mesures, documenter les options écartées et intégrer la solution retenue à la branche principale. Montrer son fonctionnement avec des tâches noyau effectives ; distinguer les résultats en simulation de ceux obtenus dans le noyau.

### Critères techniques proposés

- Exécuter au moins **trois tâches noyau distinctes**, avec progression observable et changements de contexte vérifiés.
- Sur une charge mixte fixée avant comparaison, viser une réduction d'au moins **20 % du 95e percentile du temps d'attente des tâches interactives** par rapport au Round Robin, sans baisse de débit supérieure à **5 %**.
- Vérifier qu'aucune tâche prête n'est perdue et que chaque tâche prête progresse pendant le scénario ; mesurer également le délai maximal entre deux passages en exécution.

Les seuils de 20 % et 5 % sont des hypothèses de cadrage, pas des gains constatés. Leur faisabilité doit être examinée avec une première mesure et le mentor avant de figer les critères en octobre. Les mesures utiliseront une horloge définie et stable, indépendante du nombre de passages dans la boucle du shell.

### Preuves attendues par le guide

- Un dispositif de veille démontrable : sources justifiées, outils ou alertes, rythme et domaines répartis entre les quatre membres.
- Au moins **20 entrées de veille datées**, réparties d'octobre à mai, chacune avec une conclusion pour le projet.
- Un partage interne au moins mensuel à partir de novembre, **6 présentations internes** tracées et **3 éléments de veille effectivement réutilisés**.
- Un état de l'art, le code des prototypes, les protocoles, les mesures, la décision technique et les preuves d'intégration : commits, tests et démonstration.
- Une participation à **2 communautés techniques**, **5 contributions publiques** documentées et au moins **1 contribution externe**. Ces activités restent à réaliser et à tracer par l'équipe.

**Pilote proposé : Mathias.** Benjamin accompagne l'intégration bas niveau, Rayan les métriques et Elias leur restitution dans le shell. La veille concerne toute l'équipe.

**Échéances du guide :** problème et veille en octobre ; état de l'art en décembre ; prototype de décision en janvier ; décision en février ; exploration entre février et avril ; intégration avant fin avril ; bilan de veille et de communauté en mai.

## Objectif 2 - Architecture fiable, documentée et reproductible

**Formulation à présenter :** « Nous voulons transformer nos prototypes en un système cohérent, documenté et testable, qu'une personne extérieure puisse compiler et lancer sous QEMU à partir de nos instructions. »

### Travail prévu

1. Choisir une chaîne de démarrage de référence et intégrer progressivement boot, mémoire, interruptions, ordonnanceur, shell et supervision sur un socle commun. Justifier la place du prototype graphique dans cette architecture.
2. Produire un schéma d'architecture et un schéma des données internes : tâches, files, états, pages, allocations et métriques. Préciser les responsabilités, échanges, capacités et modes de défaillance.
3. Documenter les API internes noyau/interface, la compilation, les tests, les journaux, les erreurs et la procédure de diagnostic sous QEMU.
4. Exécuter formatage, analyse statique, tests et compilation à chaque changement couvert par la CI ; rendre les échecs bloquants pour la branche principale et conserver l'historique.
5. Tester le parcours critique **démarrage → initialisation mémoire et interruptions → lancement des tâches → commande du shell → affichage des informations**. Couvrir aussi les erreurs : entrée invalide, capacité des files atteinte, allocation impossible et exception mémoire.
6. Faire compiler et lancer FerrumOS par une personne extérieure avec la documentation seule ; enregistrer durée, blocages et corrections apportées.

### Critères et preuves attendus

- **4 décisions d'architecture** datées, chacune comparant au moins deux options, dont une décision réexaminée au cours de l'année avec justification.
- Au moins **1 test d'intégration complet**, une couverture mesurée avec le périmètre du chemin critique explicité, et **3 régressions réellement détectées par les tests**, avec dates et preuves. Les limites de mesure propres au noyau devront être exposées.
- **15 demandes de fusion relues** par un autre membre, avec commentaires ; **3 cas** où la revue entraîne une modification avant fusion.
- Au moins **1 changement défectueux bloqué par la CI**, documenté.
- Un bilan initial et final de qualité du code, les exceptions aux règles justifiées et un registre de dette technique avec coût estimé.
- Un rapport daté du test de reproductibilité externe et une documentation corrigée en conséquence.

**Pilotes proposés : Benjamin pour le socle et le démarrage, Elias pour les interfaces et la documentation.** Mathias prend en charge les tests du noyau ; Rayan fournit les vues de supervision et vérifie la cohérence des métriques. Les revues sont réparties entre tous.

**Échéances du guide :** première architecture en novembre ; documentation, CI et mesure initiale de qualité en décembre ; couverture du chemin critique et test externe avant fin mars ; bilan final de qualité en mai. Décisions et revues sont tracées toute l'année.

## Objectif 3 - Travailler avec des experts techniques

**Complémentaire C proposé, à valider avec le mentor.**

**Formulation à présenter :** « Nous voulons confronter les choix techniques sensibles de FerrumOS à des spécialistes extérieurs et transformer leurs retours en améliorations concrètes et justifiées. »

### Sujets proposés

- Ordonnancement et changements de contexte : préemption, équité et gestion des interruptions.
- Gestion mémoire : limites de l'allocateur, récupération des blocs et évolution de l'isolation.
- Sécurité du code bas niveau : invariants des blocs `unsafe`, accès matériel et verrouillage.

### Critères et preuves attendus

- Définir **2 à 3 sujets** avec questions précises et options encore ouvertes avant fin décembre.
- Rechercher et solliciter au moins **6 experts** entre janvier et avril ; conserver un registre daté et relancer chaque sollicitation sans réponse.
- Réaliser au moins **2 échanges en visioconférence ou en présentiel**, le premier avant fin mars et le second avant fin mai.
- Envoyer le dossier préparatoire **48 heures avant** chaque échange et rédiger le compte rendu **dans les 48 heures après**.
- Mettre en œuvre au moins **2 changements concrets**, chacun relié à un retour d'expert et à un commit.
- Justifier au moins **1 recommandation non suivie** et produire un bilan en mai.

**Répartition proposée : Elias coordonne le suivi ; Benjamin et Mathias préparent les questions bas niveau ; Rayan prépare les résultats et mesures à discuter.** Les contacts et échanges sont des actions futures de l'équipe.

## Objectif 4 - Mesurer et optimiser la gestion mémoire

**Complémentaire D proposé, à valider avec le mentor.**

**Formulation à présenter :** « Nous voulons mesurer les limites de la gestion mémoire actuelle de FerrumOS, puis améliorer la réutilisation du tas en démontrant les effets de chaque changement avec un protocole reproductible. »

L'axe prioritaire est la **consommation et la réutilisation de la mémoire**. Il complète le travail sur l'ordonnancement en traitant un autre problème déjà visible dans le code : la réutilisation limitée du tas Bump lorsqu'une allocation persistante reste active.

### Protocole et indicateurs proposés

Fixer la version du noyau, la configuration QEMU, la taille du tas, la suite de tailles et d'alignements des allocations, leur ordre de libération et le nombre de répétitions. Conserver une allocation persistante et enchaîner des allocations temporaires, avec une quantité de données vivantes inférieure à la capacité du tas. Prévoir un cas séparé de saturation volontaire.

| Indicateur | Définition et collecte | Cible technique proposée |
|---|---|---|
| Taux d'échec d'allocation | Nombre de demandes refusées / nombre de demandes ; compteurs dans l'allocateur et export des résultats. | **0 échec sur 10 000 cycles** d'allocation/libération pour la charge nominale définie. |
| Occupation maximale du tas | Maximum d'octets indisponibles pour de nouvelles allocations / capacité du tas ; instrumentation de l'allocateur. | Réduction visée d'au moins **20 %** sur le préfixe de charge que les deux versions peuvent exécuter. |
| Mémoire récupérable après un cycle | Octets disponibles pour de nouvelles allocations à un point fixe après libération ; tenir compte de l'allocation persistante et des métadonnées. | Aucune diminution cumulative entre les cycles ; budget de métadonnées documenté. |

Ces cibles sont à justifier et à figer avant fin décembre après un premier essai de faisabilité ; la mesure de référence formelle et le diagnostic sont attendus avant fin février. Aucun chiffre de performance n'est présenté ici comme acquis. Si la version initiale épuise le tas, comparer les valeurs sur un préfixe commun et présenter séparément le point d'épuisement.

### Démarche et preuves attendues

- Définir les indicateurs et l'axe avant fin décembre, puis livrer le protocole reproductible et l'instrumentation avant fin janvier.
- Produire une mesure initiale datée, un diagnostic fondé sur les données, une hypothèse et une estimation du gain avant fin février.
- Réaliser **2 boucles d'optimisation**, chacune portant sur une modification isolée : par exemple la récupération des blocs libérés, puis une amélioration ciblée de leur organisation si les mesures la justifient.
- Pour chaque boucle : conserver le commit, rejouer exactement le protocole, publier l'écart absolu et en pourcentage, discuter le coût en complexité et vérifier les tests ainsi que les autres indicateurs.
- Conserver les résultats sans gain, l'historique des mesures et les limites du protocole. Automatiser les mesures et la détection de régression lorsque possible.

**Pilote proposé : Rayan pour les mesures, avec Mathias pour l'allocateur.** Benjamin accompagne l'instrumentation ; Elias rend les résultats consultables depuis le shell et la documentation.

**Échéances du guide :** première boucle avant fin mars, seconde avant fin mai ; vérification de non-régression après chacune.

## Jalons de préparation du dossier Greenlight

Le guide ne fournit pas de dates distinctes pour plusieurs jurys Greenlight. Les jalons ci-dessous reprennent ses échéances de travail ; la date du jury reste à confirmer auprès de l'école.

| Période | Livrables prioritaires |
|---|---|
| **Octobre 2026** | Valider les deux complémentaires ; caractériser le verrou et ses critères chiffrés ; organiser la veille et les responsabilités. |
| **Novembre - décembre 2026** | Documenter l'architecture ; activer la CI bloquante ; établir l'état de l'art ; commencer les partages et contributions ; cadrer les sujets experts et les indicateurs mémoire. |
| **Janvier - février 2027** | Comparer les deux candidats d'ordonnancement ; décider ; établir le protocole mémoire, la référence et le diagnostic ; démarrer les sollicitations d'experts. |
| **Mars 2027** | Couvrir le chemin critique par les tests ; réaliser le test de reproductibilité externe ; tenir le premier échange expert ; terminer la première boucle d'optimisation. |
| **Avril 2027** | Intégrer la solution d'ordonnancement et démontrer le résultat ; terminer le prototype exploratoire ; avoir sollicité les six experts. |
| **Mai 2027** | Terminer la seconde boucle et le second échange expert ; consolider les preuves de veille, revues, tests, décisions et mesures ; présenter les écarts restants. |

Pour chaque objectif, le dossier présentera **le résultat visé, l'état réel, les preuves datées, les limites et la prochaine étape**. Une fonctionnalité annoncée dans la roadmap ne vaut pas preuve de validation.

## Sources utilisées

- Guide fourni : *Technical Track - Student Guide - Tech 4*, 2026/2027, pages 2-3 (cadre), 4-7 (objectif 1), 8-11 (objectif 2), 18-19 (C), 20-21 (D).
- Documentation projet : `README.md`, `docs/ORGANIZATION.md`, `docs/eip/TASKS_FOR_FOLLOW_UP_2.md`, `docs/eip/repartitions.txt`, synthèses des dossiers et présentations.
- Vérification du code : `POC/ferrum_Poc_adaptative_sheduller/src/scheduler.rs`, `main.rs`, `interrupts.rs` et `shell.rs` ; `POC/ferrumos_PoC/src/allocator.rs` et `shell.rs` ; manifests et point d'entrée du prototype graphique ; `.github/workflows/ci.yml`.
