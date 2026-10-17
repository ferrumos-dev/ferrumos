FERRUM OS - VEILLE SUR LES SYSTEMES DE PAQUETS

CONTENU
  FerrumOS_Veille_Paquets.pdf : comparaison, actualites,
    bilan critique, choix proposes, limites et sources.
  poc/run_experiments.py : prototype hote et 14 experiences executables.
  poc/results.json : resultats observes, environnement, empreinte du code.
  poc/review_notes.txt : defaut repere en relecture, correction et limites.
  poc/requirements.txt : version de la bibliotheque cryptographique utilisee.
  build_report.py : source de generation du rapport (ReportLab).

REPRODUIRE LES EXPERIENCES
  Utiliser Linux, macOS ou WSL, Python 3.10 ou plus recent.
  Depuis le dossier de ce fichier :

    python3 -m venv .venv
    .venv/bin/python -m pip install -r poc/requirements.txt
    .venv/bin/python poc/run_experiments.py --output results-new.json

  L'installation de la dependance se fait dans le venv. Les experiences
  elles-memes n'utilisent pas le reseau et ne demandent pas de privileges.
  Le prototype cree un depot et une installation dans un dossier temporaire
  qu'il supprime a la fin. Il ne modifie pas les logiciels du systeme hote.

PORTEE
  Ceci n'est pas pacman, Guix, pkgutils ou un gestionnaire natif Ferrum.
  Les paquets contiennent des fichiers texte de demonstration.
  Le label x86_64-ferrum-demo-abi1 est illustratif : ce n'est pas une cible
  officielle Rust ni une ABI validee pour Ferrum.
  Chaque demande represente le PROFIL COMPLET souhaite. Il ne s'agit pas
  d'une installation additive : les paquets non demandes n'y figurent pas.

  La selection des versions est exacte. Pas de plages de versions, de
  solveur SAT, de paquets virtuels ou de gestion multi-utilisateurs.
  La bascule du lien current utilise os.replace sur le systeme hote.
  Deux lectures distinctes peuvent encadrer la bascule. Cela ne prouve pas
  une transition coherente pour les applications deja lancees.
  Les generations sont des repertoires logiques, pas un store rendu
  immuable par les permissions. Le nettoyage des generations non utilisees
  et la gestion de l'espace disque ne sont pas implementes.

  E12 et E13 injectent des exceptions avant activation et pendant une
  ecriture de preparation. Aucune coupure de courant ni panne du disque
  n'a ete testee ; il n'y a pas de protocole fsync de durabilite.
  Un processus tue peut laisser des generations temporaires inutilisees.

  La cle Ed25519 est generee en memoire pour les essais, puis abandonnee.
  Les empreintes sont liees a un index signe verifie avec la cle de
  confiance du test. Rotation, revocation, expiration et protection contre
  le rejeu d'un ancien index restent hors perimetre.

  E01 teste l'assemblage deterministe de fichiers fixes. Il ne prouve pas
  la reproductibilite d'une compilation Rust. Les tests ne mesurent pas
  les performances des gestionnaires compares et n'executent aucun ELF.
  Le lecteur d'archives est borne au perimetre de petites donnees de test ;
  il ne doit pas etre deploye tel quel sur un depot public non maitrise.

REGENERER LE PDF (facultatif)
  Le script build_report.py necessite reportlab, pypdf et les polices DejaVu Sans
  installees sous /usr/share/fonts/truetype/dejavu (ou adapter FONT_ROOT).
  Versions utilisees : reportlab 4.4.9 et pypdf 6.10.0.
  Depuis le dossier racine : python3 build_report.py
  Le PDF est alors ecrit dans output/pdf/.

SUITE DU TRAVAIL
  Tester les outils upstream dans leurs environnements dedies, puis
  installer/executer de vrais programmes compiles pour Ferrum. Conserver
  les versions et journaux a chaque evolution. Le rapport distingue les
  faits documentes, les essais effectues et les recommandations.

TRACABILITE DE LA REVISION
  Les resultats des essais sont conserves. Seule la docstring du script
  a ete editee ; le code des experiences est inchange. Le JSON conserve
  tested_source_sha256 (script lors des essais) et distributed_source_sha256
  (script fourni). Les horodatages ont ete omis dans le rapport et le journal.
  Les dates internes du ZIP sont normalisees pour le format du conteneur ;
  elles ne representent pas la date des essais.
