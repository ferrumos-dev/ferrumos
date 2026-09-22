**Benjamin :**

* Intégrer le bootloader et les drivers clavier/VGA dans la branche principale.
* Uniformiser les commandes de compilation et de lancement sous QEMU.
* Ajouter des logs de démarrage exploitables.
* Réviser et documenter les principaux blocs `unsafe`.

**Mathias :**

* Intégrer et vérifier le paging, l’allocateur et les interruptions.
* Stabiliser le scheduler Round Robin avec plusieurs tâches simples.
* Exposer les informations mémoire et scheduler aux autres modules.
* Ajouter quelques tests noyau automatisés dans la CI.

**Elias :**

* Améliorer le parser du shell et la gestion des saisies incorrectes.
* Finaliser les commandes `help`, `version`, `clear`, `mem` et `tasks`.
* Définir une API claire entre le noyau et le shell.
* Mettre à jour le README et la documentation d’utilisation.

**Rayan :**

* Afficher la liste et l’état des tâches fournies par le scheduler.
* Ajouter des statistiques simples : uptime, mémoire et changements de contexte.
* Créer une vue de monitoring textuelle accessible depuis le shell.
* Préparer un format de métriques réutilisable plus tard par l’IA d’optimisation. 
