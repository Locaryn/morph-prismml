---
name: bonsai
description: Moteur d'inférence Bonsai (modèles 1-bit et ternaires de PrismML) fourni par le morph morph-prismml.
---

# Moteur Bonsai

Ce morph n'ajoute pas d'outil à appeler : il fournit un **moteur** (Réglages → Moteur → « PrismML ») et des modèles à son catalogue. Quand il est actif et qu'un modèle Bonsai est choisi, c'est `llama-server` (fork PrismML de llama.cpp) qui répond, en local.

Pour l'utilisateur qui demande pourquoi une réponse est lente ou ne démarre pas :

- le **premier démarrage** télécharge le runtime (jusqu'à 700 Mo avec CUDA) : c'est normal, la progression est dans le journal du moteur (`engine-bonsai.log`) ;
- un **27B ternaire** ne tient pas dans une carte de 6 Go : il passe en partie sur le processeur et ralentit ; le **27B en 1 bit** (3,8 Go) tient ;
- le **raisonnement** est coupé par défaut (réglage « Raisonnement ») : l'activer allonge beaucoup les réponses.
