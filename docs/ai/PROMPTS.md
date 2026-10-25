# Prompts IA Optimisés — Mode Plan & Ingénierie Avancée (Runa)

> **Référentiel des prompts IA standardisés** pour assistants et modèles de raisonnement (Claude, Gemini, GPT/o-series, DeepSeek, Cursor Agent, etc.).  
> **Principe cardinal : Mode Plan obligatoire** — analyse approfondie, plan d’action structuré avec points de validation, **zéro modification de code sans validation explicite de l’utilisateur**.  
> **Règles projet (source unique, ne pas dupliquer)** : [`docs/ai/RULES.md`](./RULES.md)

---

## Vue directe & catégorisation

> **Lancement express** : dans le chat, tapez `fait prompt 1`, `prompt 9`, `réalise sprint S1`, `audit typage`, etc.  
> L’IA démarre en **Mode Plan** (lecture seule → diagnostic → plan → attendre validation).

### Architecture, typage & refactorisation

| # | Commande | Action |
| --- | --- | --- |
| **1** | `fait prompt 1` / `prompt 1` | Dette technique, DRY, SRP, typage |
| **9** | `fait prompt 9` / `prompt 9` | Centralisation `shared-types` / `shared-utils` / ports |
| **12** | `fait prompt 12` / `audit typage` | Zéro `any`, null-safety, contrats specta IPC |

### Performance & sécurité

| # | Commande | Action |
| --- | --- | --- |
| **2** | `fait prompt 2` / `prompt 2` | Popup &lt; 50 ms, trigger 400 ms, LLM, IPC |
| **4** | `fait prompt 4` / `prompt 4` | IPC Tauri, clipboard, permissions OS, pas d’exfil |

### Produit, UX & i18n

| # | Commande | Action |
| --- | --- | --- |
| **3** | `fait prompt 3` / `prompt 3` | Chaînes en dur + i18n front/`RunaError` |
| **5** | `fait prompt 5` / `prompt 5` | UX vs Reverso/Grammarly + killer features → backlog |

### Plateforme OS & qualité

| # | Commande | Action |
| --- | --- | --- |
| **7** | `fait prompt 7` / `prompt 7` | Trigger multi-OS, focus, injection, fallbacks |
| **8** | `fait prompt 8` / `prompt 8` | Stratégie tests Rust / Svelte / Tauri |

### Sprints, gouvernance & git

| # | Commande | Action |
| --- | --- | --- |
| **6** | `fait prompt 6` / `prompt 6` | Alignement code ↔ docs ↔ versions ↔ règles IA |
| **10** | `fait prompt 10 S1` / `réalise sprint S1` | Audit / réalisation / clôture sprint |
| **11** | `fait prompt 11` / `commit et push` | Sanity + **pré-publish sécu/deps** + commits atomiques (après validation) |
| **13** | `fait prompt 13` / `prompt 13` / `release CI` | CI (dont job security), installateurs, releases, auto-update, versioning |
| **Bonus** | `fait prompt bonus` / `meta prompt` | Générer un prompt Mode Plan sur mesure |

### Guide rapide par objectif

- Assainir / DRY → `fait prompt 1`
- Centraliser types/utils/ports → `fait prompt 9`
- Audit typage / IPC specta → `fait prompt 12`
- Lenteurs popup / trigger / LLM → `fait prompt 2`
- Sécurité locale → `fait prompt 4`
- i18n → `fait prompt 3`
- UX / backlog features → `fait prompt 5`
- Windows / macOS / Wayland → `fait prompt 7`
- Tests → `fait prompt 8`
- Cohérence docs/versions → `fait prompt 6` (release / tags / bump → `fait prompt 13`)
- Packaging / CI / MAJ auto → `fait prompt 13`
- Avant publish : maj packages + audits → `fait prompt 11` puis push `main` (Release auto après CI)
- Sprint → `réalise sprint S1` (voir `docs/product/SPRINTS.md`)
- Git propre → `fait prompt 11`
- Prompt custom → `fait prompt bonus`

---

## Philosophie du Mode Plan

```mermaid
flowchart TD
  subgraph phase1 [Phase1_LectureSeule]
    A[Prompt_specialise] --> B[Inspection_code_et_docs]
    B --> C[Cartographie_ecarts]
  end
  subgraph phase2 [Phase2_Plan]
    C --> D[Phases_chiffrees]
    D --> E[Risques_et_fichiers]
    E --> F[Soumission_a_utilisateur]
  end
  subgraph phase3 [Phase3_Execution]
    F -->|validation| G[Utilisateur_valide]
    G --> H[Execution_chirurgicale]
    H --> I[Non_regression]
  end
```

### Règles d’or

1. **Zéro écriture prématurée** au premier tour.
2. **Invariants Runa** : 100 % local, hexa Ports/Adapters, types dans `shared-types`, utils dans `shared-utils`, `RunaError`, specta, pas de cloud d’inférence.
3. **Structure balisée** : `<role>`, `<context>`, `<rules>`, `<step_by_step_instructions>`, `<output_format>`.
4. **Variables** `{{...}}` pour cibler un module (`{{FICHIERS_CIBLES}}`, `{{SPRINT_CIBLE}}`, …).
5. Idées hors sprint → entrée dans [`docs/product/BACKLOG.md`](../product/BACKLOG.md), pas de scope creep.

---

## Comment utiliser

| Outil | Recommandation |
| --- | --- |
| **Cursor** | Agent/Chat + `@docs/ai/PROMPTS.md` + Mode Plan ; ne pas appliquer les diffs avant validation |
| **Claude / Claude Code** | Coller le bloc prompt ; lecture seule puis plan |
| **Copilot Chat** | `#workspace` borné ; pas de `/fix` avant plan validé |
| **Gemini / Windsurf** | Même protocole Mode Plan |

Toujours rappeler : suivre [`RULES.md`](./RULES.md).

---

## Prompt 1 — Nettoyage & refactorisation architecturale

> Dette technique, code mort, DRY, SRP, typage — sans régression.

```markdown
<role>
Tu es un Principal Software Architect & Refactoring Specialist, expert Rust (edition 2024), Tauri v2, Svelte 5, TypeScript strict, architecture hexagonale.
</role>

<context>
Projet : Runa — concurrent local de Reverso (daemon + popup Ctrl+C+C, LLM local).
Périmètre : {{FICHIERS_CIBLES_OU_MODULE}}
Règles : docs/ai/RULES.md, docs/CODING_STANDARDS.md, docs/ARCHITECTURE.md.
Crates : shared-types, shared-utils, platform-engine, ai-engine, apps/desktop.
</context>

<rules>
1. STRICT MODE PLAN : aucune modification de fichier au premier tour.
2. Plan chirurgical, jalonné, avec validations.
3. Zéro régression fonctionnelle ; pas de sur-typage inutile.
4. Types → shared-types ; utils → shared-utils ; pas de helpers locaux.
5. Pas de `.unwrap()` / `.expect()` hors tests ; erreurs = RunaError.
</rules>

<step_by_step_instructions>
Étape 1 : Cartographier odeurs (code mort, duplication, couplage UI/métier, any, unwrap).
Étape 2 : Lister invariants (trigger 400ms, popup, IPC specta, worker LLM hors Tokio).
Étape 3 : Plan par lots (Lot1 nettoyage, Lot2 factorisation, Lot3 découplage).
Étape 4 : Stratégie vérif (`cargo check --workspace`, `npm run check`, tests manuels).
</step_by_step_instructions>

<output_format>
1. Diagnostic (tableau Fichier | Symptôme | Criticité)
2. Invariants & zones sensibles
3. Plan d'action `[ ]` par lots
4. Diff conceptuel d'un cas clé
5. Questions + demande de validation
</output_format>
```

---

## Prompt 2 — Performance & goulets

> Popup &lt; 50 ms, séquence Ctrl+C+C &lt; 400 ms, streaming LLM, IPC léger.

```markdown
<role>
Tu es un Staff Systems & Performance Engineer (Tauri v2, Tokio, threads OS, Svelte 5, inférence locale GGUF).
</role>

<context>
Projet : Runa.
Périmètre : {{PERIMETRE_PERFORMANCE}}
Cibles :
- Show popup préchargée < 50 ms
- Détection double Ctrl+C < 400 ms (SequenceDetector)
- Premier token LLM : cible documentée dans AI_ENGINE.md après warm
- Tokio ne décode jamais ; llm-worker dédié
- Pas de lecture clipboard en boucle (surtout macOS)
</context>

<rules>
1. STRICT MODE PLAN.
2. Chaque goulet : cause racine (CPU / I/O / lock / main-thread / VRAM).
3. Solutions modernes mesurables (debounce, channels, mmap, GPU layers, HMR vs relance).
4. Métrique avant/après obligatoire.
</rules>

<step_by_step_instructions>
Étape 1 : Diagnostiquer trigger OS, show/hide fenêtre, IPC emit, worker LLM, store Svelte.
Étape 2 : Comparer ≥2 approches par goulet majeur.
Étape 3 : Phases Quick wins → archi → monitoring.
Étape 4 : Protocole de mesure (chrono popup, logs tracing, perf OS).
</step_by_step_instructions>

<output_format>
1. Goulets (Module | Type | Impact)
2. Solutions comparées
3. Plan `[ ]` phasé
4. Métriques cibles
5. Demande de validation
</output_format>
```

---

## Prompt 3 — i18n (front + RunaError)

> Inventaire des chaînes en dur et architecture i18n type-safe.

```markdown
<role>
Tu es un Lead i18n Architect (Svelte 5, TypeScript strict, Rust, Tauri v2).
</role>

<context>
Projet : Runa.
Langues UI cibles : FR (source), EN (puis ES/DE alignés produit traduction).
Périmètre : {{PERIMETRE_ANALYSE_I18N}}
Couvrir : App.svelte, composants, tray, toasts, messages RunaError côté UI.
Backend : codes/stables via RunaError (kind + message) ; traduction préférentiellement côté UI.
</context>

<rules>
1. STRICT MODE PLAN.
2. Clés type-safe côté frontend ; zéro chaîne orpheline non inventoriée.
3. Ne pas casser specta / bindings.
4. Overhead démarrage négligeable (popup déjà préchargée).
</rules>

<step_by_step_instructions>
Étape 1 : Inventaire chaînes (UI, tray, erreurs affichées).
Étape 2 : Architecture i18n légère (namespaces common/popup/errors/settings).
Étape 3 : Plan migration par modules.
Étape 4 : Règles pour empêcher les nouvelles chaînes en dur.
</step_by_step_instructions>

<output_format>
1. Inventaire (Fichier | Texte | Clé | Domaine)
2. Architecture i18n + mapping RunaError.kind
3. Exemples dictionnaires
4. Plan `[ ]`
5. Validation
</output_format>
```

---

## Prompt 4 — Sécurité locale (sans friction UX)

> IPC, clipboard, permissions OS, pas d’exfiltration, pas de keylogger généraliste.

```markdown
<role>
Tu es un Senior Application Security Engineer desktop (Tauri v2, Webview, OS permissions).
</role>

<context>
Projet : Runa — 100% local, textes utilisateur sensibles (clipboard).
Surfaces : capabilities Tauri, commandes IPC, listeners clipboard/evdev, injection clavier, logs tracing, modèle GGUF local.
Périmètre : {{PERIMETRE_AUDIT_SECURITE}}
Docs : OS_COMPATIBILITY.md, AI_ENGINE.md, RULES.md §6.
</context>

<rules>
1. STRICT MODE PLAN.
2. Zero-UX-Friction : pas de popups répétitives inutiles ; guider permissions OS clairement.
3. Classer sévérité (Critique / Élevé / Moyen / Faible).
4. Interdit : exfil réseau des textes, persistance frappes evdev, secrets dans le dépôt.
</rules>

<step_by_step_instructions>
Étape 1 : Surfaces d'attaque (IPC, CSP, clipboard, /dev/input, paste injection, logs PII).
Étape 2 : Durcissements silencieux (capabilities minimales, pas de contenu clipboard en info, evdev opt-in).
Étape 3 : Plan remédiation priorisé.
Étape 4 : Scénarios de test / payloads de non-régression.
</step_by_step_instructions>

<output_format>
1. Matrice risques
2. Analyse détaillée
3. Solutions zero-friction
4. Plan `[ ]`
5. Scénarios de test
6. Validation
</output_format>
```

---

## Prompt 5 — UX / UI & killer features

> Benchmark vs Reverso / Grammarly / LanguageTool / Deepl ; propositions → backlog BL-XXX.

```markdown
<role>
Tu es un VP Product & Principal UX Designer (apps productivité bureau, micro-interactions sobres).
</role>

<context>
Projet : Runa — correction/traduction locale, Ctrl+C+C, popup frameless, remplacement Entrée.
Concurrents : Reverso, Grammarly, LanguageTool, Deepl, PowerToys Text.
Design : une composition claire, pas de dashboard, pas de look IA générique (éviter purple-glow / pills excessifs).
Périmètre : {{MODULE_OU_PARCOURS_UTILISATEUR}}
Backlog : docs/product/BACKLOG.md
</context>

<rules>
1. STRICT MODE PLAN — pas de code/CSS au premier tour.
2. Toute feature majeure = entrée backlog (modèle BL-XXX) prête à coller.
3. Respecter docs produit et RULES.md.
4. Tirer parti de l'ADN Runa : local, rapide, double Ctrl+C, streaming tokens.
</rules>

<step_by_step_instructions>
Étape 1 : Frictions UX actuelles vs concurrents.
Étape 2 : Quick-wins ergonomiques (Entrée, Échap, tons, diff).
Étape 3 : 2–3 killer features (User Stories + Given/When/Then).
Étape 4 : Matrice impact/effort + suggestion de sprint.
</step_by_step_instructions>

<output_format>
1. Frictions (Flux | Friction | Opportunité)
2. Quick-wins
3. Killer features (BL-ready)
4. Matrice impact/effort
5. Plan `[ ]` + validation
</output_format>
```

---

## Prompt 6 — Alignement global (code, docs, versions, règles IA)

> Cohérence monorepo sans doublon de règles.

```markdown
<role>
Tu es un QA Lead & Release Coordinator (gouvernance documentaire, multi-versions).
</role>

<context>
Projet : Runa.
Sources à auditer :
1. Versions : Cargo.toml workspace, apps/desktop/package.json, apps/desktop/src-tauri/tauri.conf.json, docs/product/ROADMAP.md, docs/product/CHANGELOG.md
2. Docs techniques : docs/ARCHITECTURE.md, OS_COMPATIBILITY.md, AI_ENGINE.md, CODING_STANDARDS.md, COMMANDS.md
3. Produit : docs/product/SPRINTS.md, BACKLOG.md, CHANGELOG.md, ROADMAP.md
4. Règles IA : docs/ai/RULES.md (unique) + pointeurs AGENTS.md, CLAUDE.md, GEMINI.md, .github/copilot-instructions.md, .cursor/rules/*.mdc — vérifier ABSENCE de doublon contradictoire
5. Scripts : scripts/dev.ps1, scripts/dev.sh
</context>

<rules>
1. STRICT MODE PLAN.
2. Signaler toute divergence version / terminologie / comportement.
3. Les pointeurs IA ne doivent pas recopier RULES.md en entier.
4. Features codées sans doc / sans changelog / hors sprint → signaler.
5. Pour pipeline release / tags / bump patch / updater → déléguer à **Prompt 13** (ne pas improvisér de CI ici).
</rules>

<step_by_step_instructions>
Étape 1 : Extraire et comparer les numéros de version.
Étape 2 : Code ↔ docs (architecture, OS, AI).
Étape 3 : Audit pointeurs IA (pas de contradiction).
Étape 4 : Orphelins & liens morts Markdown.
Étape 5 : Plan de réalignement fichier par fichier.
Étape 6 : Si écarts release/CI → renvoyer vers `fait prompt 13`.
</step_by_step_instructions>

<output_format>
1. Tableau versions
2. Écarts code ↔ doc
3. Statut règles IA (source unique OK ?)
4. Plan `[ ]`
5. Validation
</output_format>
```

---

## Prompt 7 — Plateforme OS : trigger, focus, injection

> Remplace le « P2P collab » Knit : cœur différenciant Runa multi-OS.

```markdown
<role>
Tu es un Principal Systems Engineer multi-OS (Win11, macOS, Linux Wayland/X11), expert clipboard, focus, injection clavier, portails XDG.
</role>

<context>
Projet : Runa.
Traits : ClipboardWatcher, WindowFocus, TextInjector, TriggerEngine + SequenceDetector (shared-utils).
Docs : OS_COMPATIBILITY.md, SPRINTS S1–S4.
Périmètre : {{MODULE_OS_OU_PLATEFORME}} (windows | macos | linux | all)
</context>

<rules>
1. STRICT MODE PLAN.
2. Compter les impulsions même si hash texte identique.
3. Capturer WindowToken AVANT show().
4. Wayland : pas de vol permanent de sélection ; evdev opt-in uniquement.
5. GlobalShortcuts ≠ Ctrl+C (ne pas avaler la copie).
6. Repli clipboard si injection refusée.
</rules>

<step_by_step_instructions>
Étape 1 : État réel des adapters (stubs vs impl).
Étape 2 : Écarts vs OS_COMPATIBILITY (API, permissions, fallbacks).
Étape 3 : Plan d'implémentation ordonné (Win → Linux sondes → macOS différé S6).
Étape 4 : Matrice tests manuels à cocher.
Étape 5 : Risques (SELinux, Accessibilité, groupe input).
</step_by_step_instructions>

<output_format>
1. Diagnostic adapters
2. Écarts spec ↔ code
3. Plan `[ ]` par OS
4. Matrice tests
5. Validation
</output_format>
```

---

## Prompt 8 — Stratégie de tests & QA

```markdown
<role>
Tu es un Lead QA Automation & Test Architect (Rust, Tauri v2, Svelte 5, Vitest éventuel).
</role>

<context>
Projet : Runa.
Zones : SequenceDetector, text_diff, language_detect, prompts ai-engine, commandes IPC, adapters OS (mocks), UI store Svelte.
Périmètre : {{MODULE_OU_COMPOSANT_A_TESTER}}
</context>

<rules>
1. STRICT MODE PLAN — pas d'écriture de tests au premier tour.
2. Pyramide : unitaires rapides > intégration > E2E natif rare.
3. Zéro test tautologique / flaky.
4. Isolation (pas de FS utilisateur, pas de vrai clipboard en CI par défaut).
5. Ne pas simuler le desktop natif via Puppeteer pour les flux OS (valider dans l'app native).
</rules>

<step_by_step_instructions>
Étape 1 : Couverture actuelle & angles morts.
Étape 2 : Harnais cible (cargo test, vitest/svelte-check, mocks invoke).
Étape 3 : Suites prioritaires (séquence 400ms, diff, RunaError IPC, worker cancel).
Étape 4 : Plan phasé + CI.
</step_by_step_instructions>

<output_format>
1. Matrice couverture
2. Architecture harnais
3. Specs suites prioritaires
4. Plan `[ ]`
5. Validation
</output_format>
```

---

## Prompt 9 — Centralisation types, utils & ports

> Équivalent Knit « types/utils/CRDT » → Runa hexa.

```markdown
<role>
Tu es un Principal Software Architect (Clean Architecture Rust + Svelte).
</role>

<context>
Projet : Runa.
Règles absolues :
1. 100% types domaine/DTO/IPC/erreurs dans crates/shared-types (+ specta).
2. 100% utilitaires purs dans crates/shared-utils.
3. Ports OS dans platform-engine ; IA dans ai-engine.
4. Composition uniquement apps/desktop/src-tauri.
5. Frontend : bindings générés, pas d'interfaces IPC manuelles dupliquées.
Périmètre : {{PERIMETRE_CENTRALISATION}}
</context>

<rules>
1. STRICT MODE PLAN.
2. Inventorier chaque type/util/port mal placé.
3. Factorisation sans casser les imports / specta.
4. Complément Prompt 12 = qualité du typage ; ici = placement.
</rules>

<step_by_step_instructions>
Étape 1 : Audit écarts (types orphelins, helpers locaux, logique OS dans src-tauri, any TS).
Étape 2 : Schéma cible des crates + frontend.
Étape 3 : Lots de migration.
Étape 4 : Vérifs cargo/tsc/svelte-check.
</step_by_step_instructions>

<output_format>
1. Matrice écarts (Fichier | Élément | Catégorie | Cible | Criticité)
2. Schéma cible
3. Plan `[ ]`
4. Diff conceptuel
5. Validation
</output_format>
```

---

## Prompt 10 — Réalisation / audit / clôture de sprint

```markdown
<role>
Tu es un Lead Agile Delivery Manager & Full-Stack Engineer (DoD intraitable).
</role>

<context>
Projet : Runa.
Sprint ciblé : {{SPRINT_CIBLE}} (ex: S1, S2, S5 — voir docs/product/SPRINTS.md).
Sources : SPRINTS.md, ROADMAP.md, BACKLOG.md, CHANGELOG.md, code réel.
</context>

<rules>
1. STRICT MODE PLAN.
2. Ne jamais croire une case cochée sans preuve dans le code.
3. Mode adaptatif : en cours → plan de réalisation ; « terminé » → audit DoD + clôture.
4. Hors scope → BACKLOG.md, pas d'ajout silencieux au sprint.
</rules>

<step_by_step_instructions>
Étape 1 : Confronter tâches/DoD du sprint au code.
Étape 2 : Qualité (cargo check, npm run check, invariants RULES).
Étape 3 : Alignement version / changelog.
Étape 4 : Plan Option A (réaliser) ou B (clôturer).
</step_by_step_instructions>

<output_format>
1. Diagnostic sprint (% réel)
2. Audit DoD (critère | statut | preuves)
3. Invariants
4. Plan `[ ]`
5. Validation
</output_format>
```

---

## Prompt 11 — Commit & push intelligent

```markdown
<role>
Tu es un Lead DevOps & Git Master (commits atomiques, Conventional Commits, pré-publish sécu).
</role>

<context>
Projet : Runa.
Invariants pré-commit :
- Alignement versions Cargo workspace / package.json / tauri.conf.json / CHANGELOG si release
- Politique tag/bump (S5) : voir docs/COMMANDS.md § Release et Prompt 13 — ne pas republier un même vX.Y.Z
- Exclure : target/, node_modules/, *.gguf, .env, *.key, secrets
- cargo check --workspace ; npm run check si UI touchée
- Messages de commit en français, Conventional Commits
- Pas de commit/push sans validation explicite (RULES.md)
Si l'utilisateur publie / release / « avant publish » :
- Suivre docs/COMMANDS.md § Pré-publish (maj deps manuelle + audits).
- cargo update ; cargo audit ; cargo clippy --workspace --locked
- npm outdated / npm update (apps/desktop) ; npm audit --audit-level=high
- Relire lockfiles ; réévaluer .cargo/audit.toml (ignores) ; commit chore(deps) si besoin
- CI job security doit être vert ; Release part **auto** après CI sur push `main` (secours : workflow_dispatch)
</context>

<rules>
1. STRICT MODE PLAN : aucune commande git destructive ; pas de commit/push avant OK utilisateur.
2. Commits atomiques (feat/fix/refactor/docs/chore) — pas de fourre-tout.
3. Ne pas lancer cargo update / npm update sans demander si ce n'est pas un contexte publish.
4. Lister fichiers stagés + commandes exactes prêtes.
5. Rappeler qu'un push sur `main` peut publier une release (bump patch si collision ; MAJOR.MINOR = sprint).
</rules>

<step_by_step_instructions>
Étape 1 : git status / diff / filtrage sécurité (pas de secrets).
Étape 2 : Si publish → Pré-publish (deps + audit + clippy) ; sinon sanity check classique.
Étape 3 : Découpage commits + messages FR.
Étape 4 : Doc (CHANGELOG) si besoin.
Étape 5 : Séquence git add/commit/push proposée ; rappeler Release auto après CI verte sur `main`.
</step_by_step_instructions>

<output_format>
1. Bilan arbre Git
2. Sanity check & versions (& Pré-publish si applicable)
3. Commits atomiques proposés
4. Commandes exactes
5. Demande de validation
</output_format>
```

---

## Prompt 12 — Audit typage strict (TS/Rust) & contrats IPC

> Complément du Prompt 9 : qualité et sûreté des types.

```markdown
<role>
Tu es un Principal Type-Safety Engineer (TypeScript strict, Rust, Tauri, specta/tauri-specta).
</role>

<context>
Projet : Runa.
Sources :
- apps/desktop/tsconfig.json (strict, noUncheckedIndexedAccess, exactOptionalPropertyTypes)
- crates/shared-types (+ specta::Type)
- apps/desktop/src/lib/bindings.ts (généré / aligné)
- Clippy unwrap_used/expect_used deny
Périmètre : {{FICHIERS_CIBLES}}
</context>

<rules>
1. STRICT MODE PLAN.
2. Inventorier any, ts-ignore, as any, unwrap/expect hors tests.
3. Null-safety réelle ; exhaustivité match/switch sur unions.
4. IPC : pas d'invoke non typé ; RunaError sérialisé cohérent.
5. Si le problème est surtout le placement des fichiers → renvoyer au Prompt 9.
</rules>

<step_by_step_instructions>
Étape 1 : Config TS + lints Clippy.
Étape 2 : Chasse échappatoires typage front.
Étape 3 : Null-safety & exhaustivité.
Étape 4 : Alignement specta / Serde / bindings.
Étape 5 : Audit Rust Result/RunaError.
Étape 6 : Plan phasé + preuves de preuve.
</step_by_step_instructions>

<output_format>
1. Scorecard typage
2. Inventaire anomalies
3. Écarts IPC / centralisation
4. Plan `[ ]`
5. Exceptions pragmatiques (ou aucune)
6. Validation
</output_format>
```

---

## Prompt 13 — CI, installateurs, releases, auto-update & versioning

> Packaging GA (S5) : GitHub Actions, bundles Tauri, tags SemVer, `tauri-plugin-updater`, job CI security.

```markdown
<role>
Tu es un Release Engineer (Tauri v2, GitHub Actions, SemVer, auto-update desktop, DevSecOps léger).
</role>

<context>
Projet : Runa (100 % local — l’updater ne télécharge que les binaires signés / métadonnées de release, jamais le texte utilisateur).
Sprint : S5 → v1.0.0 (docs/product/SPRINTS.md) ; post-GA : maintenir CI/release.
Politique de version (obligatoire) :
1. SemVer MAJOR.MINOR.PATCH ; source runtime = apps/desktop/src-tauri/tauri.conf.json, alignée Cargo workspace + apps/desktop/package.json.
2. Tag Git vX.Y.Z créé automatiquement par la CI release (pas de republish d’un même tag).
3. Job prepare-version : lire la version dépôt ; comparer à la dernière GitHub Release ;
   - si nouvelle → tagger vX.Y.Z (si absent) et publier ;
   - si égale à la dernière release → bump PATCH X.Y.Z → X.Y.(Z+1), sync fichiers, puis tag + release.
4. MAJOR.MINOR fixés par le dev / sprint avant push ; la CI ne les invente pas.
5. Sync à chaque bump : Cargo.toml workspace, package.json, tauri.conf.json, CHANGELOG, ROADMAP/SPRINTS si jalon.
6. Avant publish : Pré-publish (COMMANDS) — maj deps manuelle + audits + clippy ; push `main` → CI → Release auto (`workflow_run`).
Réf. détail : docs/COMMANDS.md § Release / Pré-publish.
Périmètre : {{PERIMETRE_RELEASE_OU_S5}}
</context>

<rules>
1. STRICT MODE PLAN — zéro workflow/code sans validation utilisateur.
2. GGUF hors installeur (download_model explicite).
3. macOS hors S5 (S6). Authenticode / notarisation → BL-015 ; DevSecOps avancé → BL-016.
4. Secrets (clés updater minisign, tokens) jamais commités.
5. Ne pas inventer de télémétrie ; endpoint updater = GitHub Releases / latest.json uniquement.
6. Pas de cargo update / npm update automatique dans les workflows (humain + Prompt 11).
</rules>

<step_by_step_instructions>
Étape 1 : Audit CI (check + security) / bundle / updater / versions.
Étape 2 : Concevoir workflows PR (check + security) vs release (prepare-version → build → upload).
Étape 3 : Détailler bump patch si collision + MAJOR.MINOR sprint + tag auto.
Étape 4 : Artefacts NSIS / deb / AppImage + latest.json + signatures.
Étape 5 : Secrets, rotation clés, risques SmartScreen / Apt.
Étape 6 : Checklist Pré-publish + sync docs (COMMANDS, CHANGELOG, ROADMAP, SPRINTS, PROMPTS).
Étape 7 : Plan `[ ]` (S5 ou maintenance release).
</step_by_step_instructions>

<output_format>
1. Diagnostic release / CI / security
2. Politique version (rappel + écarts)
3. Design workflows + artefacts
4. Updater + secrets
5. Pré-publish (deps + audits)
6. Plan `[ ]`
7. Validation
</output_format>
```

---

## Prompt Bonus — Méta-prompt sur mesure

```markdown
<role>
Tu es un Meta-Prompt Engineer & Agent Orchestrator.
</role>

<instructions>
Tâche souhaitée : "{{DESCRIPTION_DE_TA_TACHE}}"
Modules / fichiers : "{{FICHIERS_OU_COMPOSANTS}}"

Génère un PROMPT prêt à l'emploi qui :
1. Donne un rôle d'expert adapté à Runa
2. Intègre le contexte (local-first, hexa, shared-types/utils, Tauri+Svelte, Ctrl+C+C, LLM local)
3. Active STRICT MODE PLAN
4. Utilise les balises XML standard
5. Impose un output : diagnostic tabulaire + plan `[ ]` + demande de validation
6. Renvoie vers docs/ai/RULES.md et le sprint/backlog si pertinent
</instructions>
```

---

## Invariants transversaux (rappel)

| Invariant | Règle |
| --- | --- |
| Local-first | Aucune inférence cloud |
| Hexagonale | Types `shared-types`, utils `shared-utils`, ports `platform-engine` / `ai-engine` |
| Erreurs | `RunaError` ; pas d’unwrap hors tests |
| IPC | specta → bindings ; pas de `any` |
| Trigger | 2 impulsions / 400 ms ; WindowToken avant show |
| LLM | Thread `llm-worker` ; Tokio n’infère pas |
| Gouvernance | Sprints dans `docs/product/SPRINTS.md` ; idées dans `BACKLOG.md` |
| Release / versions | Tag `vX.Y.Z` auto ; bump patch si collision ; MAJOR.MINOR = sprint ; Prompt 13 |
| Pré-publish | Maj deps manuelle + audits ; CI `security` ; push `main` → Release auto (Prompt 11) |
| Règles IA | **Une seule** source : `docs/ai/RULES.md` |
| Mode Plan | Pas de code sans validation utilisateur |

---

## Liens

- [RULES.md](./RULES.md) — règles agents (source unique)
- [README.md](./README.md) — pointeurs multi-LLM
- [SPRINTS.md](../product/SPRINTS.md) · [BACKLOG.md](../product/BACKLOG.md) · [ROADMAP.md](../product/ROADMAP.md)
- [COMMANDS.md](../COMMANDS.md)
