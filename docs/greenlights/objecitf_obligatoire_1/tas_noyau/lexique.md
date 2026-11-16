# Lexique · tas du noyau (objectif obligatoire 1)

Ce lexique explique les mots techniques utilisés dans les documents 01 à 10. Il vise un lecteur qui ne connaît ni Rust ni le fonctionnement interne d’un système d’exploitation. Chaque définition dit ce que le mot veut dire **pour FerrumOS**, sans chercher l’exhaustivité.

[Retour au dossier](README.md)

## Le système

| Terme | En clair |
| --- | --- |
| Noyau | Le cœur du système d’exploitation, qui gère la mémoire, le processeur et le matériel pour tous les autres programmes. Une erreur dans le noyau arrête toute la machine. |
| Noyau monolithique | Noyau dans lequel tous les services (mémoire, pilotes, ordonnanceur) partagent le même espace. C’est rapide, mais une corruption touche tout le système. |
| PoC | *Proof of Concept*, prototype qui montre qu’une idée fonctionne. Le noyau mesuré ici est celui du PoC d’ordonnancement. |
| Ordonnanceur | La partie du noyau qui décide quelle tâche utilise le processeur, et pendant combien de temps. |
| Changement de contexte | Le moment où le processeur passe réellement d’une tâche à une autre. Le PoC ne sait pas encore le faire, ce qui limite les tests de concurrence. |
| Contention | Plusieurs traitements attendent le même verrou en même temps. |
| Shell | L’interface en ligne de commande du système. |
| Pilote | Le code qui fait fonctionner un périphérique (clavier, écran, port série). |
| Interruption | Un signal du matériel (minuterie, clavier) qui interrompt le programme en cours pour exécuter un court traitement, le gestionnaire d’interruption. |
| Verrou (verrou tournant) | Un mécanisme qui empêche deux traitements de modifier la même donnée en même temps. Un verrou tournant attend en boucle qu’il se libère, ce qui bloque tout s’il ne se libère jamais. |
| Page, cadre, mappage | La mémoire est découpée en pages de 4 Kio. Un cadre est une page de mémoire physique. Mapper, c’est associer une adresse utilisée par le programme à un cadre réel. |
| Panique | Arrêt volontaire du noyau quand il rencontre une situation qu’il ne sait pas gérer. |

## La mémoire et les allocateurs

| Terme | En clair |
| --- | --- |
| Tas | La réserve de mémoire où le noyau prend de la place quand il en a besoin, puis la rend. Dans FerrumOS, elle fait 100 Kio. |
| Allocation, libération | Prendre un morceau du tas (allocation) puis le rendre (libération). |
| Allocateur | Le code qui décide quel morceau du tas donner à chaque demande, et qui récupère les morceaux rendus. C’est lui que ce dossier remplace. |
| Bloc | Un morceau de mémoire donné par l’allocateur. Un bloc *vivant* est un bloc pris et pas encore rendu. |
| o, Kio | Octet, et kibioctet (1 Kio = 1 024 octets). |
| Allocateur à pointeur (bump) | L’allocateur d’origine. Il avance un curseur à chaque demande et ne revient au début que lorsque **tout** a été rendu. Il est très rapide mais ne réutilise jamais la mémoire rendue une à une. |
| Liste chaînée (candidat A) | Allocateur qui tient la liste des zones libres, rangées par adresse. C’est la solution retenue. |
| Trou | Une zone libre du tas, dans la liste chaînée. |
| Premier ajustement | Règle de choix : servir une demande avec la première zone libre assez grande. |
| Fusion | Quand un bloc rendu touche une zone libre, les deux sont réunis en une seule zone plus grande. C’est ce qui évite l’émiettement du tas. |
| Blocs de taille fixe, classe (candidats B et B′) | Allocateur qui range les demandes par tailles standard (16, 32, 64… 2 048 o), comme des cases de tailles différentes. Chaque taille est une classe. |
| Récupération (*reclaim*, B′) | Ajout fait à B : quand la mémoire manque, les cases vides de toutes les classes sont rendues et réunies, puis la demande est retentée. |
| TLSF, buddy, slab | Trois autres familles d’allocateurs étudiées dans l’état de l’art. TLSF vise un temps constant, buddy découpe la mémoire en puissances de deux, slab garde des réserves par type d’objet. |
| Alignement | Certaines données doivent commencer à une adresse multiple de 8, 16 ou 64. L’allocateur doit le respecter sans perdre de place. |
| Métadonnées, en-tête | Les informations que l’allocateur garde pour s’organiser. Un en-tête placé devant chaque bloc coûte de la place. Les candidats A et B rangent leurs informations dans la mémoire libre, donc ne coûtent rien. |
| Fragmentation, ‰ | La mémoire libre est éparpillée en petits morceaux : il peut rester assez de place au total, mais pas d’un seul tenant. Elle est mesurée en pour mille (‰). 0 ‰ signifie que toute la mémoire libre est d’un seul morceau. |
| Granularité | La plus petite quantité distribuée. Avec une granularité de 16 o, une demande de 8 octets en reçoit 16. |
| Épuisement | Le tas ne peut plus servir une demande. |
| Double libération | Rendre deux fois le même bloc, une erreur de programmation. La liste chaînée la détecte et la refuse. |

## Mesures et tests

| Terme | En clair |
| --- | --- |
| QEMU | Un logiciel qui simule un ordinateur complet. Le noyau y démarre et y est testé comme sur une vraie machine. |
| Banc hôte, banc noyau | Deux façons de mesurer avec les mêmes scénarios. Le banc hôte lance les allocateurs comme un programme ordinaire sur le PC, ce qui donne des temps précis. Le banc noyau les fait tourner dans le vrai noyau, sous QEMU. |
| Image amorçable | Un fichier qui contient un noyau prêt à démarrer. Chaque expérience a la sienne, pour partir d’un tas neuf. |
| Harnais de test | Le code qui lance les tests dans le noyau, affiche leurs résultats et indique à QEMU si tout a réussi. |
| Graine | Le nombre de départ d’une suite pseudo-aléatoire. La même graine redonne exactement la même suite de demandes, ce qui rend les mesures rejouables. |
| Grandeur déterministe | Une mesure qui donne toujours le même résultat (nombre d’itérations, d’erreurs, d’octets). On juge d’abord sur elles, car les temps varient. |
| ns, µs, cycles | Nanoseconde (milliardième de seconde), microseconde (millionième). Les cycles comptent les tops d’horloge du processeur simulé ; sous QEMU ils varient de 15 à 20 %. |
| p50, p99 | Médiane, et valeur sous laquelle se trouvent 99 % des mesures. Le p99 décrit les cas lents, pas la moyenne. |
| Pire cas | L’opération la plus lente observée. Pour un noyau, c’est souvent plus important que la moyenne. |
| O(1), O(n), O(n²) | La façon dont le coût grandit avec la taille du problème : constant, proportionnel, ou proportionnel au carré (un problème deux fois plus gros coûte quatre fois plus). |
| Baseline | Les valeurs de référence enregistrées. Une mesure qui en sort fait échouer la vérification automatique. |
| Régression | Un retour en arrière involontaire : quelque chose qui marchait ne marche plus. |
| `RECORD` | La ligne de résultat que chaque expérience écrit, que les scripts lisent pour comparer les mesures. |

## Rust et outils de développement

| Terme | En clair |
| --- | --- |
| Rust | Le langage de programmation de FerrumOS, conçu pour éviter les erreurs de mémoire. |
| `no_std` | Code Rust écrit sans la bibliothèque standard, parce qu’il n’y a pas de système d’exploitation en dessous. C’est le cas du noyau et de l’allocateur. |
| Crate | Une bibliothèque Rust. `ferrum_alloc` est la nôtre. `linked_list_allocator`, `talc` et `rlsf` sont des crates publiques étudiées. |
| `unsafe` | Partie du code où Rust ne peut pas vérifier lui-même la sécurité de la mémoire. Chaque bloc porte un commentaire `SAFETY` qui explique pourquoi il est correct. |
| `GlobalAlloc`, `Layout` | L’interface que Rust impose à tout allocateur, et la description d’une demande (taille et alignement). |
| `Box`, `Vec`, `String` | Les structures de Rust qui prennent de la mémoire dans le tas : une valeur, une liste, un texte. |
| Feature Cargo | Une option de compilation. Elle permet de choisir l’allocateur du noyau, dont l’ancien pour rejouer le « avant ». |
| Nightly, toolchain | La toolchain est l’ensemble des outils qui compilent le code. Nightly désigne la version expérimentale de Rust dont le noyau a besoin ; elle est figée à une date pour que tout le monde obtienne le même résultat. |
| Release | Compilation optimisée, celle qui sert à mesurer les temps. |
| CI | Intégration continue : à chaque modification, GitHub compile, teste et mesure automatiquement le projet. |
| Commit, PR, merge, issue | Un commit enregistre une modification. Une PR (*pull request*) propose de l’intégrer et permet de la relire. Le merge l’intègre à la branche principale. Une issue est un ticket de suivi. |
| ADR | *Architecture Decision Record* : une entrée du registre des décisions techniques. |

## Veille

| Terme | En clair |
| --- | --- |
| Veille | Le suivi organisé de ce qui se publie sur notre sujet : nouvelles versions, articles, discussions. |
| Digest | Le résumé hebdomadaire des nouveautés repérées automatiquement dans les sources suivies. |
| RSS | Un format de flux qui permet de recevoir automatiquement les nouveaux articles d’un site. |
| Webhook | Une adresse qui permet à un programme de poster automatiquement un message, ici dans le salon Discord de l’équipe. |

## Repères propres au dossier

| Repère | Sens |
| --- | --- |
| C1 à C7 | Les sept critères de réussite fixés dans la note 01 avant toute mesure des solutions. |
| exp1 à exp6 | Les six expériences mémoire. exp1 : suite de 10 000 demandes variées. exp2 : la mémoire rendue est-elle réutilisée ? exp3 : remplir, tout rendre, recommencer. exp4 : cycles de pages. exp5 : sous interruptions. exp6 : changement de profil, des petits blocs puis des gros. |
| `long_run` | Scénario du banc hôte : 100 000 demandes sans jamais tout rendre, pour imiter un noyau qui tourne longtemps. |
| A, B, B′, X | Les candidats comparés : A liste chaînée, B blocs de taille fixe, B′ blocs fixes avec récupération, X TLSF pour l’exploration. |
