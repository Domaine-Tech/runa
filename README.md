# Runa

Application bureau **100 % locale** : correction, traduction et reformulation via un LLM local (GGUF). Daemon tray + **popup rapide** (`Ctrl+C+C`) + **fenêtre principale**.

Pas de cloud d’inférence. Les mises à jour téléchargent uniquement des binaires signés et les métadonnées de release — jamais le texte utilisateur.

## Plateformes GA (v1.0)

| OS | Statut |
| --- | --- |
| Windows 10/11 | Supporté (installeur NSIS) |
| Linux | Supporté (`.deb` + AppImage) |
| macOS | Différé (sprint S6) |

## Installation

Téléchargez la dernière release : [GitHub Releases](https://github.com/Domaine-Tech/runa/releases/latest).

### Windows

1. Exécuter l’installeur **NSIS** (`.exe`).
2. Au premier lancement, Windows SmartScreen peut avertir (binaire non Authenticode — voir backlog BL-015). Choisir « Informations complémentaires » → exécuter quand même si vous faites confiance à la source.
3. **WebView2** est requis (bootstrap souvent inclus / déjà présent sur Windows 11).
4. Ouvrir Runa depuis le tray → fenêtre principale → **Télécharger le modèle** (GGUF, hash blake3). Le modèle n’est **pas** dans l’installeur.

### Linux

- **AppImage** : `chmod +x Runa_*.AppImage` puis lancer. Autonome pour la plupart des distros.
- **`.deb`** (Debian/Ubuntu) : `sudo dpkg -i runa_*.deb` puis installer les dépendances WebKit si besoin (`libwebkit2gtk-4.1-0`, etc.). Hors dépôt apt officiel → avertissement « paquet non authentifié » possible.

Permissions / comportements utiles :

| Besoin | Windows | Linux |
| --- | --- | --- |
| Trigger `Ctrl+C+C` | Listener clipboard | Wayland data-control / X11 / portail raccourci |
| Remplacement 1 clic | `SendInput` Ctrl+V | X11 XTest ; Wayland → collage manuel (Ctrl+V) |
| Evdev (opt-in) | — | Config `linux_evdev_opt_in` uniquement |

## Mises à jour

- Au démarrage, Runa **vérifie** silencieusement s’il existe une version plus récente (log seulement).
- Pour installer : tray → **Vérifier les mises à jour** → confirmer le dialogue.
- Signature **minisign** (pas Authenticode). Endpoint : GitHub `latest.json` uniquement.

## Développement

Voir [`docs/COMMANDS.md`](docs/COMMANDS.md).

```powershell
# Windows — feature llama par défaut
.\scripts\dev.ps1
```

```bash
# Linux / macOS
./scripts/dev.sh
```

Règles agents / architecture : [`docs/ai/RULES.md`](docs/ai/RULES.md) · [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) · sprints [`docs/product/SPRINTS.md`](docs/product/SPRINTS.md).

## Licence

MIT — voir les métadonnées du workspace Cargo.
