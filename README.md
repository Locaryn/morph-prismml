# morph-prismml

Le runtime, le catalogue et les réglages recommandés des modèles [Bonsai](https://docs.prismml.com) de PrismML, pour Locaryn. (Anciennement `morph-bonsai`.)

Bonsai est une famille de modèles en **1 bit** et **ternaires** : un 27 milliards de paramètres tient en 3,8 Go. Les formats ternaires (Ternary Bonsai 2) exigent le fork `prism` de llama.cpp, que le llama.cpp ordinaire ne lit pas : ce morph l'apporte.

## Ce que ça fait

- **Un moteur** (Réglages → Moteur → *PrismML*). Au premier démarrage, le lanceur télécharge le runtime depuis [github.com/PrismML-Eng/llama.cpp](https://github.com/PrismML-Eng/llama.cpp) (version épinglée), **vérifie son empreinte SHA-256** (il refuse un fichier dont GitHub ne publie pas l'empreinte), l'installe dans le dossier privé du morph, puis lance `llama-server` sur `127.0.0.1` uniquement.
- **Le bon backend** : CUDA 12.4 avec une carte NVIDIA et un pilote 551 ou plus ; Vulkan sinon (AMD, Intel, et processeur en dernier recours) ; Metal sur Mac.
- **Un catalogue** : Bonsai 27B, Ternary Bonsai 2 27B, 8B, 4B et 1.7B, avec le projecteur d'images quand le modèle en a un. Chaque adresse est vérifiée à la publication.
- Le serveur **ne survit pas au lanceur** : si Locaryn s'arrête, la mémoire vidéo est rendue (objet *job* sous Windows, `PDEATHSIG` sous Linux).

## Mesuré sur une RTX 4050 portable (6 Go)

| Modèle | Mémoire vidéo | Génération | Notes |
| --- | --- | --- | --- |
| Bonsai 27B, 1 bit (3,8 Go) | 4,3 Go à 8 192 de contexte | ≈ 30 jetons/s | Chargé en 12 s ; appels d'outils fonctionnels |

Le 27B 1 bit répond en bon français avec des écarts occasionnels (un mot anglais au milieu d'une phrase) : c'est le prix du 1 bit. **Ternary Bonsai 2 27B** (5,95 Go au mieux) ne tient pas dans 6 Go avec un contexte utile : prévoir 8 Go ou plus pour le garder entièrement sur la carte. Les chiffres de qualité de l'éditeur ne sont pas vérifiés par Locaryn.

## Réglages

| Réglage | Défaut | Rôle |
| --- | --- | --- |
| `context_size` | 8192 | Contexte, en jetons |
| `thinking` | `off` | Raisonnement : `off`, `on`, `auto`. Activé, Bonsai raisonne longuement et chaque réponse prend plusieurs fois plus de temps |
| `vision` | `false` | Charge le projecteur d'images (≈ 0,6 Go de mémoire vidéo) |
| `backend` | `auto` | `auto`, `cuda`, `vulkan`, `cpu` |
| `gpu_layers` | 0 | 0 : automatique (tout sur la carte si le modèle tient largement dans la mémoire libre, sinon llama.cpp répartit) ; un nombre impose le choix |
| `parallel` | 1 | Requêtes en parallèle ; chaque emplacement a son cache |

## Lanceur en ligne de commande

```bash
locaryn-bonsai-launch status    # backend choisi, runtime installé ou non (JSON)
locaryn-bonsai-launch install   # télécharge le runtime sans rien lancer
locaryn-bonsai-launch serve --port 8189 --model chemin/vers/modele.gguf
```

`LOCARYN_EXTENSION_DATA_DIR` désigne où le runtime s'installe, `LOCARYN_EXTENSION_CONFIG_FILE` le fichier de réglages.

## Limites

- Les binaires ne sont pas signés.
- Linux CUDA : le runtime CUDA du système doit être présent (le build Linux n'embarque pas ses bibliothèques CUDA comme le fait celui de Windows).
- Seul Windows x86_64 + CUDA a été éprouvé de bout en bout ; Vulkan, Linux et macOS reposent sur les mêmes archives officielles mais n'ont pas été lancés ici.

## Réglages recommandés

Chaque modèle du catalogue déclare l'échantillonnage que PrismML recommande ([docs.prismml.com/run/llamacpp](https://docs.prismml.com/run/llamacpp)) : température 0,5, top-p 0,85, top-k 20, min-p 0, sans pénalité de répétition. Locaryn l'applique au modèle actif tant que vous n'avez pas changé ces réglages vous-même (panneau « Paramètres du modèle »), et le serveur de ce morph les prend comme valeurs par défaut.
