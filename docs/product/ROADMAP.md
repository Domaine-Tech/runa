# Roadmap produit — Runa

**Objectif produit :** concurrent 100 % local de Reverso — daemon léger avec **deux surfaces** :
1. **Popup rapide** après `Ctrl+C+C` (&lt; 400 ms), près du texte sélectionné, actions rapides + remplacement 1 clic.
2. **Fenêtre principale** (clic tray / app en tâche de fond) : saisie libre, correction / reformulation / traduction, copier / coller.

Inférence 100 % locale (LLM), pas de cloud.

Ce document fixe les **versions**. Le détail d’exécution est dans [SPRINTS.md](./SPRINTS.md). Les idées hors scope courant vont dans [BACKLOG.md](./BACKLOG.md).

## Statut des versions

| Version | Nom | Statut | Objectif |
| --- | --- | --- | --- |
| **0.1.0** | Foundation | **Livré** | Architecture hexa, docs, stubs OS/IA, shell Tauri |
| **0.2.0** | Trigger Windows | **Livré** | `Ctrl+C+C` ouvre réellement la popup sur Windows 11 |
| **0.3.0** | Replace + Dual UI | **Livré** | Focus + collage + onglets + fenêtre principale tray |
| **0.3.1** | Thème + UI minimale | **Livré** (non tagué) | Tokens CSS, presets, UI découpée — sous workspace 0.5.0 ; pas de tag `v0.3.1` |
| **0.4.0** | Linux | **Code livré** (S3-5 ouvert) | Chaîne Wayland/X11/evdev/portail ; clôture bloquée par tests manuels Fedora |
| **0.5.0** | Local AI | **Livré** | Inférence GGUF streaming (feature `llama`) ; workspace / npm / Tauri = 0.5.0 |
| **1.0.0** | GA | **Livré** | CI + installateurs + release auto + MAJ + polish **Windows + Linux** |
| **1.1.0** | macOS | Différé | Adapters + tests + bundle macOS (après GA) |

## Définition de « Done » par version

### 0.1.0 — Foundation

- [x] Workspace Cargo multi-crates + Clean Architecture
- [x] Docs architecture / OS / IA / standards / commandes
- [x] Traits OS + stubs `Unsupported`
- [x] Shell Tauri v2 + Svelte 5 + tray + popup préchargée
- [x] Scripts `dev.ps1` / `dev.sh`

### 0.2.0 — Trigger Windows

- [x] `AddClipboardFormatListener` + `SequenceDetector` (smoke auto)
- [x] Capture HWND avant `show()`
- [x] Affichage immédiat du texte presse-papiers **via Ctrl+C+C réel** (S1-4)
- [x] Tests manuels double Ctrl+C OK sur Windows (S1-5)

### 0.3.0 — Replace + Dual UI

- [x] Onglets Correction (tons) + Traduction (UI complète)
- [x] Diff visuel via `word_diff`
- [x] `apply_replacement` : write clipboard → hide → restore focus → paste
- [x] Repli presse-papiers si injection refusée (`InjectionFallback` visible)
- [x] Fenêtre principale au clic tray (saisie + résultat + copier)
- [x] Popup positionnée près de la sélection (`GetGUIThreadInfo` / caret)

### 0.3.1 — Thème + UI minimale

- [x] Tokens CSS `--runa-*` + Tailwind (zéro couleur hors thème)
- [x] Presets : mode (`system` / `dark` / `light`), accent, densité
- [x] `UiConfig` étendu + `set_ui_theme` persisté (TOML)
- [x] Composants Svelte découpés ; menu apparence fenêtre principale
- [x] Popup : actions Copier / Remplacer / Masquer

**Note versions :** code livré sous le workspace **0.5.0** (pas de tag Git `v0.3.1` séparé). Entrées CHANGELOG sous `[Unreleased]` / notes 0.3.1.

### 0.4.0 — Linux

- [x] Chaîne data-control → XFixes → evdev opt-in → portail (code S3 ; validation manuelle ouverte)
- [ ] Injection / repli documentés **et testés** GNOME/KDE (repli Wayland documenté ; XTest X11)
- [ ] Matrice Linux dans `OS_COMPATIBILITY.md` cochée

**Note versions :** pas de bump / tag `0.4.0` tant que S3-5 n’est pas validé ; le workspace reste en **0.5.0**.

### 0.5.0 — Local AI

- [x] Feature `llama` + chargement GGUF mmap (code ; CMake/LLVM requis)
- [x] Streaming tokens → événement `inference`
- [x] Keep-alive / annulation
- [x] Téléchargement modèle explicite (hash blake3)
- [x] Validation manuelle inférence + bump version 0.5.0 (2026-10-25)

### 1.0.0 — GA (Win + Linux)

- [x] CI PR (`cargo check` / `test` + `npm run check`)
- [x] Installateurs Windows (NSIS) / Linux (`.deb` et AppImage) — artefacts produits par `release.yml`
- [x] Pipeline release : tag auto `vX.Y.Z` + bump **patch** si collision ; MAJOR.MINOR = sprint / dev
- [x] Sync versions (Cargo workspace, `package.json`, `tauri.conf.json`, CHANGELOG)
- [x] Auto-update (`tauri-plugin-updater` + signatures) — endpoint release uniquement, pas d’exfil texte
- [x] Cold start popup &lt; 50 ms (mesure via log `show_ms` ; pas de seuil flaky CI)
- [x] Pas de fuite PII dans les logs `info`
- [x] Doc utilisateur (install, permissions, MAJ)
- [x] Checklist sécurité locale (pas de télémétrie d’inférence)

### 1.1.0 — macOS (après GA)

- [ ] Poll `changeCount` + Accessibilité
- [ ] Remplacement Cmd+V / repli
- [ ] Bundle macOS + checklist `OS_COMPATIBILITY.md`

## Hors scope volontaire (jusqu’à décision backlog)

- Cloud / API distantes pour l’inférence
- Sync multi-appareils
- Extension navigateur
- Plugin IDE

Pour proposer une feature : voir [BACKLOG.md](./BACKLOG.md).
