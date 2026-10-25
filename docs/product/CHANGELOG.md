# Changelog — Runa

Format inspiré de [Keep a Changelog](https://keepachangelog.com/).  
Les versions suivent [ROADMAP.md](./ROADMAP.md).

## [Unreleased]

### Added
- CI job `security` : `cargo clippy`, `cargo audit`, `npm audit --audit-level=high`
- Release auto après CI verte sur push `main` (`workflow_run`) ; `workflow_dispatch` en secours
- Docs Pré-publish (COMMANDS) + Prompt 11/13 / RULES : maj deps manuelle avant publish
- `.cargo/audit.toml` : ignores temporaires advisories `zip` (transitif updater) — BL-016

### Changed
- Versioning release : collision → bump **patch** (`1.0.0` → `1.0.1`) ; MAJOR.MINOR = sprint / dev avant push

### Notes

## [1.0.0] — 2026-10-25

### Added
- CI PR : `.github/workflows/ci.yml` (`cargo check` / `test` + `npm run check`, Win + Ubuntu ; job `llama` séparé)
- Release : `.github/workflows/release.yml` + `scripts/prepare_version.py` (bump patch si collision ; MAJOR.MINOR = sprint)
- Bundles GA : NSIS + `.deb` + AppImage (`--features llama`) ; GGUF hors installeur
- Auto-update : `tauri-plugin-updater` + `tauri-plugin-dialog` ; check au démarrage ; tray « Vérifier les mises à jour » avec confirmation
- README utilisateur (install, permissions, MAJ, SmartScreen / apt)
- Thème (S2.1 / 0.3.1) : tokens CSS `--runa-*`, presets mode / accent / densité, `UiConfig` + `set_ui_theme`, composants Svelte, menu apparence, actions popup Copier / Remplacer / Masquer
- Linux : chaîne clipboard `ext-data-control` → `wlr-data-control` → XFixes → evdev opt-in (`linux_evdev_opt_in`)
- Linux : portail `ashpd` GlobalShortcuts (défaut `CTRL+SHIFT+SPACE`, interdit Ctrl+C) → ouverture immédiate
- Linux : focus X11 (`_NET_ACTIVE_WINDOW` / PID) ; injection XTest Ctrl+V en session X11 pure ; Wayland → `InjectionFallback`
- Config : `linux_portal_shortcut` ; sélection backend pure testée (`linux_select`)
- Debounce clipboard partagé (`shared_utils::should_emit_impulse`) ; `ClipboardImpulse.immediate`

### Changed
- `create_platform_stack(&TriggerConfig)` — champs Linux lus au boot
- Linux : portail GlobalShortcuts démarre même sans data-control / X11 ; bind ashpd attendu avant `Ok` ; evdev remonte `PlatformPermission` via canal ready ; restore focus X11 via `_NET_CLIENT_LIST`
- Docs produit / COMMANDS : politique release, secrets updater, mesure `show_ms` (&lt; 50 ms), invariant logs PII
- Perf trigger : preview `TriggerEvent` immédiat + `show()` Rust avant lecture clipboard ; hydration en second événement ; horodatages `capture_ms` / `show_ms` / `clipboard_ready_ms` (sans PII)
- Perf : `show()` / focus uniquement sur la preview ; caret capturé après l'émission preview (hors chemin critique)
- Sécurité : CSP locale stricte ; `devCsp` pour localhost:1420 (retiré du CSP release) ; retrait `tauri-plugin-shell` / `shell:allow-open` ; suppression commande `remember_origin` (origine = bridge uniquement)
- Sécurité : timeout download GGUF 45 min ; evdev n'ouvre que les devices `EV_KEY` ; message stub `-NoLlama` (plus `-Llama`)
- Logs injection : motif OS en `debug` seulement (S5-6)
- Scripts `dev.ps1` / `dev.sh` : feature `llama` **par défaut** ; opt-out via `-NoLlama` / `--no-llama` (npm : `dev:app` + llama, `dev:no-llama` stub)

### Tests
- Unitaires : `language_detect`, `resolve_translate_target`, `text_diff` Delete/equal, bords `SequenceDetector`, serde `RunaError` (tous les kinds), prompts sans markdown, worker cancel précoce / `UnsupportedBackend`
- Mocks trigger : focus avant preview, impulsions contenu identique ; smokes clipboard Windows en `#[ignore]` (hors CI)

### Notes
- S5 GA clôturé : Win + Linux ; macOS → S6 ; Authenticode → BL-015
- S3-5 (tests manuels Fedora) peut rester ouvert ; pas de tag rétroactif `v0.4.0`
- Première publication GitHub : Actions → Release → `workflow_dispatch` (secrets `TAURI_SIGNING_*` requis)

## [0.5.0] — 2026-10-25

### Added
- Feature Cargo `llama` (`llama-cpp-2` 0.1.160) + `llama-vulkan` optionnelle ; streaming tokens → `inference`
- `trim_model_output` ; keep-alive / annulation atomique sur `llm-worker`
- Téléchargement explicite GGUF Qwen2.5-3B Q4_K_M + vérif blake3 ; commande `download_model` + UI fenêtre principale
- Bouton Annuler pendant le streaming
- Axes reformulation Contexte (Mail / Chat / Note) + Ton (prompts enrichis) — BL-014
- `AiConfig.keep_alive_ms` : `#[serde(default)]` pour configs TOML sans la clé

### Changed
- Worker IA : file `std::sync::mpsc` + config `AiConfig` partagée (`parking_lot`)

### Notes
- S4 clôturé : inférence réelle validée (correction / traduction / cancel) sous Windows ; feature Cargo `llama` **off** par défaut (`default = []`, CMake + MSVC / LLVM requis) ; scripts `dev.ps1` / `dev.sh` et npm `dev:app` activent `llama` (opt-out `-NoLlama` / `--no-llama` / `dev:no-llama`)
- Critère « premier token &lt; 500 ms GPU » reste manuel hors CI (`AI_ENGINE.md`)

## [0.3.0] — 2026-10-24

### Added
- `WindowsFocus::restore` : `AttachThreadInput` + `SetForegroundWindow`
- `WindowsInjector` : `SendInput` Ctrl+V (VK virtuels, AZERTY OK)
- `ClipboardWatcher::write_text` + suppression d’impulsions internes (~100 ms)
- Capture caret (`GetGUIThreadInfo`) → position popup près de la sélection
- Fenêtre Tauri `main` (décorée, taskbar) ; tray ouvre `main`, `Ctrl+C+C` ouvre `popup`
- UI : diff mot-à-mot, bascule langues FR/EN/ES/DE/Auto, bouton Copier
- Repli `InjectionFallback` remonte à l’UI si focus / paste refuse
- Commandes `copy_text`, `has_replace_origin` ; `correct`/`translate` renvoient `generation_id`

### Notes
- S2 clôturé : replace + dual UI sur Windows
- Inférence `llama` toujours stub (S4)

## [0.2.0] — 2026-10-24

### Added
- Listener clipboard Windows : thread `win-clipboard`, fenêtre `HWND_MESSAGE`, `AddClipboardFormatListener`
- Debounce 40 ms des `WM_CLIPBOARDUPDATE` (double notif / `EmptyClipboard`+`Set`)
- Capture `GetForegroundWindow` → `WindowToken::Windows` avant `show()` de la popup
- Arrêt propre du `TriggerEngine` sur `RunEvent::Exit`
- Smoke tests auto (contenu différent + identique via `SetClipboardData`)
- Retry lecture clipboard après `TriggerFired` ; export specta BigInt → Number
- Masquer au clic extérieur ; UI popup opaque ; `dev.ps1` nettoie port 1420

### Notes
- S1 clôturé : double Ctrl+C OK sur Windows
- Injection / restauration de focus / fenêtre principale → 0.3.0 (S2)
- macOS / Linux / feature `llama` toujours stubs (S4)

## [0.1.0] — 2026-10-24

### Added
- Workspace Cargo : `shared-types`, `shared-utils`, `platform-engine`, `ai-engine`
- Shell Tauri v2 + Svelte 5 + Tailwind (popup frameless préchargée)
- System tray + commandes IPC specta
- Docs techniques : architecture, OS, AI engine, coding standards, commandes
- Scripts `scripts/dev.ps1` et `scripts/dev.sh` (watch + relance)

### Notes
- Adapters OS et backend `llama` en stubs `Unsupported` (volontaire)
