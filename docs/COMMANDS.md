# Commandes développeur — Runa

Guide des commandes utiles pour développer, vérifier et lancer l’application.

## Prérequis

| Outil | Version minimale | Rôle |
| --- | --- | --- |
| Rust (`rustup`) | stable (edition 2024) | Workspace Cargo + Tauri |
| Node.js + npm | Node 20+ | Frontend Svelte / Vite |
| (optionnel) `cargo-watch` | latest | Watch Rust plus confortable sous Linux/macOS |

```bash
# Optionnel Linux/macOS
cargo install cargo-watch
# ou
cargo install watchexec-cli
```

## Démarrage rapide (recommandé)

Les scripts installent `node_modules` si besoin, lancent **Tauri v2 en mode dev**, et **relancent** le process quand les crates Rust / `src-tauri` changent. Le frontend (Svelte/Vite) reste en HMR sans relancer toute l’app.

### Windows (PowerShell)

```powershell
# Depuis la racine du dépôt (feature llama activée par défaut)
.\scripts\dev.ps1

# Sans llama (stub IA, compile plus léger)
.\scripts\dev.ps1 -NoLlama

# Vite seul (UI sans shell Tauri)
.\scripts\dev.ps1 -Frontend

# Un seul lancement (watch Tauri/Vite natif uniquement)
.\scripts\dev.ps1 -Once

# Debounce personnalisé
.\scripts\dev.ps1 -DebounceSeconds 2
```

Si l’exécution de scripts est bloquée :

```powershell
Set-ExecutionPolicy -Scope CurrentUser RemoteSigned
```

### Linux / macOS

```bash
chmod +x scripts/dev.sh
./scripts/dev.sh              # feature llama par défaut
./scripts/dev.sh --no-llama   # stub IA

./scripts/dev.sh --frontend
./scripts/dev.sh --once
```

Le script choisit automatiquement, dans l’ordre : `cargo-watch` → `watchexec` → `inotifywait` → polling.

## Commandes manuelles

### Frontend (`apps/desktop`)

```bash
cd apps/desktop
npm install
npm run dev          # Vite — http://localhost:1420
npm run build        # Build production frontend
npm run check        # svelte-check (types Svelte/TS)
npx tsc --noEmit     # TypeScript strict
```

### Tauri (app complète)

```bash
cd apps/desktop
npm run tauri -- dev     # Dev (Vite + recompile Rust)
npm run tauri -- build   # Bundle installable
```

Équivalent Cargo depuis la racine :

```bash
cargo run -p runa-desktop
```

### Workspace Rust

```bash
# Depuis la racine
cargo check --workspace
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo fmt
```

Tests ciblés :

```bash
cargo test -p shared-utils
cargo test -p ai-engine
```

Feature LLM (`llama-cpp-2`) — **activée par défaut** dans les scripts `dev` :

```bash
# Prérequis Windows : CMake, MSVC Build Tools, LLVM (libclang pour bindgen)
#   scoop install cmake llvm
#   $env:LIBCLANG_PATH = "$env:USERPROFILE\scoop\apps\llvm\current\bin"
cargo check -p ai-engine --features llama
# Optionnel GPU Vulkan :
cargo check -p ai-engine --features llama-vulkan
# Desktop avec inférence réelle :
cargo check -p runa-desktop --features llama
# Dev UI (llama par défaut) :
.\scripts\dev.ps1
./scripts/dev.sh
# Dev UI sans llama (stub) :
.\scripts\dev.ps1 -NoLlama
./scripts/dev.sh --no-llama
# ou depuis apps/desktop :
npm run dev:app          # + llama
npm run dev:no-llama     # stub
```

## Structure utile

```text
scripts/dev.ps1     Watch + relance (Windows) ; llama par défaut ; -NoLlama pour stub
scripts/dev.sh      Watch + relance (Unix) ; llama par défaut ; --no-llama pour stub
docs/               Architecture, OS, IA, standards, commandes
crates/             Domaine / utils / platform / ai
apps/desktop/       Svelte 5 + src-tauri
```

## Que surveille le watch ?

| Chemin | Effet |
| --- | --- |
| `crates/**` (`*.rs`, `*.toml`, …) | Relance `tauri dev` |
| `apps/desktop/src-tauri/**` | Relance `tauri dev` |
| `apps/desktop/src/**` | HMR Vite (pas de relance process) |
| `target/`, `node_modules/`, `*.gguf` | Ignorés |

## Dépannage

| Symptôme | Piste |
| --- | --- |
| `cargo` introuvable | Installer [rustup](https://rustup.rs) et rouvrir le terminal |
| `npm` introuvable | Installer Node 20+ |
| Relances trop fréquentes | Augmenter le debounce (`-DebounceSeconds` / attendre 1–2 s) |
| Popup invisible | Tray « Afficher » ; fenêtre créée avec `visible: false` |
| Ctrl+C+C n’ouvre rien | Tester dans **Notepad** (pas Cursor/VS Code) : deux copies &lt; 400 ms. Logs attendus : `impulsion Ctrl+C armée` puis `TriggerFired` / `bridge: TriggerEvent`. Même sélection dans un IDE peut ne pas renvoyer `WM_CLIPBOARDUPDATE`. |
| Stub IA / compile sans CMake | `.\scripts\dev.ps1 -NoLlama` ou `./scripts/dev.sh --no-llama` (sinon llama est le défaut) |
| Échec build llama | CMake + LLVM (`LIBCLANG_PATH`) ; ou relancer avec `-NoLlama` / `--no-llama` |
| Modèle manquant | Fenêtre principale → « Télécharger le modèle » (blake3 vérifié, pas de fetch silencieux) |

## Release, versions et auto-update (S5)

Politique SemVer figée pour la GA (détail sprint : [`docs/product/SPRINTS.md`](./product/SPRINTS.md) S5 ; Mode Plan : `fait prompt 13`).

### Workflows

| Fichier | Déclencheur | Rôle |
| --- | --- | --- |
| [`.github/workflows/ci.yml`](../.github/workflows/ci.yml) | push / PR | check / test / `npm run check` (Win + Ubuntu) ; job `llama` ; job **`security`** (`cargo clippy`, `cargo audit`, `npm audit --audit-level=high`) |
| [`.github/workflows/release.yml`](../.github/workflows/release.yml) | **Auto** après CI verte sur **push `main`** (+ `workflow_dispatch` manuel) | `prepare-version` → tag `vX.Y.Z` → build NSIS / deb / AppImage (`--features llama`) → GitHub Release + `latest.json` |

Chaque **push sur `main`** dont la CI réussit déclenche une Release.  
- **MAJOR.MINOR** : fixés par le **dev avant le push** (liés aux sprints / jalons ROADMAP) — la CI ne les change pas.  
- Si version dépôt = dernière release → **bump patch** auto seulement (`1.0.0` → `1.0.1`).  
Les PR ne publient pas. Secours manuel : Actions → Release → Run workflow.

**Dépôt public requis pour les MAJ clients :** l’updater Tauri télécharge `latest.json` et les binaires **sans authentification**. Un repo GitHub privé bloque en pratique les mises à jour des apps installées. Runa doit rester **public** une fois la GA publiée (secrets updater restent dans GitHub Secrets, jamais dans le git).

### Pré-publish (avant push sur `main` qui doit sortir)

Recommandé avant un push `main` destiné à être publié (agents : **Prompt 11** / **Prompt 13**). La CI `security` rejoue audits + clippy ; la **maj des packages** reste manuelle (pas de `cargo update` / `npm update` automatique en CI).

```powershell
# Depuis la racine du dépôt
cargo update
cargo audit
cargo clippy --workspace --locked
cargo check --workspace --locked
cargo test --workspace --locked

cd apps/desktop
npm outdated
npm update
npm audit --audit-level=high
npm run check
cd ../..
```

Puis :

1. Relire le diff des lockfiles (`Cargo.lock`, `package-lock.json`) — pas de bump majeur surprise.
2. Réévaluer [`.cargo/audit.toml`](../.cargo/audit.toml) (ignores RUSTSEC) ; retirer dès qu’un patch existe (voir [BL-016](./product/BACKLOG.md)).
3. Commit atomique éventuel `chore(deps): …` + push sur `main`.
4. Attendre CI verte (y compris job `security`) → **Release démarre toute seule**.
5. Secours : **Actions → Release → Run workflow** si besoin de republier hors CI.

Sanity sans viser une release : branche / PR (la Release ne part pas sur les PR).

### Fichiers de version (doivent rester alignés)

| Fichier | Champ |
| --- | --- |
| `Cargo.toml` (workspace) | `version` |
| `apps/desktop/package.json` | `version` |
| `apps/desktop/src-tauri/tauri.conf.json` | `version` (source runtime) |
| `docs/product/CHANGELOG.md` | entrée versionnée à la release |
| `docs/product/ROADMAP.md` / `SPRINTS.md` | jalon livré (manuel) |

Script : [`scripts/prepare_version.py`](../scripts/prepare_version.py)

```bash
# Décision sans écriture (sans gh : --skip-gh --latest X.Y.Z)
python scripts/prepare_version.py --dry-run --skip-gh
python scripts/prepare_version.py --dry-run --skip-gh --latest 1.0.0
```

### Règles CI release

1. Tag Git **`vX.Y.Z`** créé **automatiquement** par le workflow release (pas de republish d’un même tag → échec).
2. Job **`prepare-version`** :
   - Lit la version du dépôt (les trois fichiers doivent être identiques, sinon échec).
   - Compare à la **dernière GitHub Release**.
   - Version **nouvelle** (ou aucune release) → tag `vX.Y.Z` (si absent) + build + upload.
   - Version **égale** à la dernière release → **bump patch** `X.Y.Z` → `X.Y.(Z+1)`, sync Cargo / package.json / tauri.conf / CHANGELOG, commit `chore(release): …`, puis tag + release.
   - Version **inférieure** à la dernière release → **échec** (pas de downgrade).
3. Exemples : dernière release `v1.0.0`, dépôt encore `1.0.0` → `1.0.1` / tag `v1.0.1`. Nouveau sprint → le dev bump à `1.1.0` (ou `2.0.0`) **avant** le push ; la CI publie cette version telle quelle.
4. Artefacts : NSIS (Windows), `.deb` + AppImage (Linux). GGUF **non** inclus (`createUpdaterArtifacts` signe les bundles, pas le modèle).
5. Auto-update : `tauri-plugin-updater` + endpoint `…/releases/latest/download/latest.json` + signatures minisign. Dialogue au démarrage **et** tray (« Installer » / « Plus tard ») ; si inférence en cours → seconde confirmation ; jamais d’install silencieuse.
6. Authenticode / notarisation : hors bloquant → [BL-015](./product/BACKLOG.md).

### Secrets GitHub (jamais commités)

| Secret | Rôle |
| --- | --- |
| `TAURI_SIGNING_PRIVATE_KEY` | Clé privée minisign (contenu ou chemin) — **obligatoire** pour `release.yml` |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | Mot de passe de la clé (vide accepté si la clé n’en a pas) |
| `RELEASE_TOKEN` | PAT optionnel si `GITHUB_TOKEN` ne peut pas pousser sur `main` protégée |

Génération locale (hors dépôt) :

```powershell
cd apps/desktop
# Avec mot de passe (recommandé) — ne pas coller le mdp dans un chat
npx tauri signer generate -w "$env:USERPROFILE\.tauri\runa.key" -p "TON_MDP" -f --ci
```

La clé publique déjà embarquée dans `tauri.conf.json` correspond à la paire générée pour S5 (fichier privé typique : `%USERPROFILE%\.tauri\runa.key`).  
Mettre le **contenu** de la clé privée dans le secret GitHub `TAURI_SIGNING_PRIVATE_KEY`.  
Si la clé est perdue : régénérer, remplacer `plugins.updater.pubkey`, republier une release signée avec la nouvelle paire.  
`*.key` / `*.key.pub` sont dans `.gitignore`.

**Rotation :** une release encore signée avec l’ancienne clé doit embarquer la **nouvelle** clé publique dans le binaire ; sinon les clients installés rejettent les MAJ suivantes.

`bundle.createUpdaterArtifacts: true` : un `tauri build` local **exige** `TAURI_SIGNING_PRIVATE_KEY` (contrairement à `tauri dev`).

### Perf popup (S5-5)

Budget cold `show()` preview **&lt; 50 ms**. Mesure : dans les logs au niveau `info`, chercher `bridge: popup show() preview` et le champ `show_ms` (pas de seuil flaky en CI). Timings associés : `capture_ms`, `clipboard_ready_ms` (sans PII).

### Logs PII (S5-6)

**Interdit** de logger le contenu presse-papiers / texte utilisateur en `info`. Autorisé : `text_len`, timings, motifs OS. Détails éventuels en `debug` seulement.

### Checklist sync avant / pendant une release

- [ ] Versions Cargo / `package.json` / `tauri.conf.json` identiques
- [ ] `CHANGELOG.md` : section Unreleased → version datée (fait par `prepare_version` si bump / première pub)
- [ ] Tag `vX.Y.Z` = version syncée
- [ ] ROADMAP / SPRINTS mis à jour si jalon (0.5.0, 1.0.0, …)
- [ ] Pré-publish : maj deps + `cargo audit` / `npm audit` / clippy ; lockfiles commités si besoin
- [ ] CI verte dont job **`security`**
- [ ] Secrets updater configurés ; pas de `.gguf` / `.key` dans le commit
- [ ] Push `main` → CI verte → Release auto (ou `workflow_dispatch` secours)

## Svelte AI (MCP)

Installé dans le dépôt (stdio local via `@sveltejs/mcp`) :

| Fichier | Rôle |
| --- | --- |
| [`.cursor/mcp.json`](../.cursor/mcp.json) | Serveur MCP `svelte` |
| [`.cursor/rules/svelte-mcp-tools.mdc`](../.cursor/rules/svelte-mcp-tools.mdc) | Règle always-on |
| [`.cursor/skills/`](../.cursor/skills/) | Skills `svelte-code-writer` + `svelte-core-bestpractices` |
| [`.cursor/agents/svelte-file-editor.md`](../.cursor/agents/svelte-file-editor.md) | Agent dédié fichiers `.svelte` |

Smoke test CLI :

```bash
npx -y @sveltejs/mcp list-sections
npx -y @sveltejs/mcp svelte-autofixer ./apps/desktop/src/components/ToneBar.svelte --svelte-version 5.17.0
```

Après clone : recharger Cursor (ou **View → Open MCP Settings**) et activer le serveur `svelte` si demandé. Optionnel marketplace : `/add-plugin svelte` ([doc](https://svelte.dev/docs/ai/cursor-plugin)).

## Documentation liée

- [ARCHITECTURE.md](./ARCHITECTURE.md)
- [OS_COMPATIBILITY.md](./OS_COMPATIBILITY.md)
- [AI_ENGINE.md](./AI_ENGINE.md)
- [CODING_STANDARDS.md](./CODING_STANDARDS.md)
- Produit : [ROADMAP](./product/ROADMAP.md) · [SPRINTS](./product/SPRINTS.md) · [BACKLOG](./product/BACKLOG.md)
- Règles IA (tous LLM) : [ai/RULES.md](./ai/RULES.md) · [ai/README.md](./ai/README.md)
- Prompts Mode Plan : [ai/PROMPTS.md](./ai/PROMPTS.md) (`fait prompt 1`, `fait prompt 13`, `réalise sprint S1`, …)
- Svelte AI : [svelte.dev/docs/ai](https://svelte.dev/docs/ai/overview)
