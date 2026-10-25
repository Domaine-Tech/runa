# Standards de code — Runa

Règles strictes du dépôt. Tout écart doit être justifié en revue et, de préférence, corrigé plutôt que documenté comme exception.

## 1. Architecture et DRY

1. **Types** : uniquement dans `crates/shared-types`. Interdiction de redéfinir DTO, événements IPC, enums de domaine ou erreurs ailleurs.
2. **Utilitaires** : uniquement dans `crates/shared-utils` (temps, debounce / séquence, hash, diff, logging, config). Interdiction de helpers locaux dans les fichiers métiers.
3. **Ports OS** : traits + adapters dans `crates/platform-engine`.
4. **Ports IA** : traits + worker + prompts dans `crates/ai-engine`.
5. **Composition** : seule `apps/desktop/src-tauri` injecte les dépendances et parle à Tauri.

## 2. Typage Rust ↔ TypeScript

- Tous les types IPC dérivent `Serialize`, `Deserialize`, `Clone`, `Debug`, `specta::Type`.
- Génération via `tauri-specta` vers `apps/desktop/src/lib/bindings.ts`.
- Interdiction d’interfaces TypeScript manuelles qui dupliquent les **commandes** IPC.
- **Exception événements** : tant que les payloads d’événements (`TriggerEvent`, `InferenceEvent`, `ModelDownloadEvent`) ne sont pas exportés dans `bindings.ts`, le front peut les re-déclarer dans `src/lib/events.ts` en miroir strict de `shared-types` (pas de champs inventés).
- Interdiction de `any` / `unknown` non affiné dans le frontend applicatif.
- `tsconfig` : `strict`, `noUncheckedIndexedAccess`, `exactOptionalPropertyTypes`.

## 3. Erreurs

- Un seul enum : `shared_types::error::RunaError` (`thiserror`).
- Conversion IPC : sérialisation de `RunaError` (specta) ; pas de `String` opaque comme type d’erreur public.
- Clippy workspace : `unwrap_used` et `expect_used` en **deny**.
- Autorisés uniquement dans les modules `#[cfg(test)]`.
- Pas de `panic!` pour le contrôle de flux.

## 4. Nommage

| Élément | Convention |
| --- | --- |
| Crates | `kebab-case` (`shared-types`, `ai-engine`) |
| Modules Rust | `snake_case` |
| Traits | PascalCase **sans** suffixe `Trait` (`ClipboardWatcher`, pas `ClipboardWatcherTrait`) |
| Structs / enums | PascalCase |
| Fonctions / champs | `snake_case` |
| Constantes | `SCREAMING_SNAKE_CASE` |
| Features Cargo | `snake_case` (`llama`) |
| Événements IPC | `snake_case` documenté (`trigger_fired`, `inference`) |

## 5. Async et concurrence

- Tokio pour IPC, timers, portails.
- Thread OS dédié pour clipboard Windows et pour `llm-worker`.
- Pas d’appel bloquant LLM sur le runtime Tokio.
- Canaux typés ; pas de `Mutex` autour du modèle partagé entre Tokio et le worker (le worker est le seul propriétaire).

## 6. Logging

- `tracing` exclusivement (pas de `println!` / `dbg!` en production).
- Initialisation centralisée dans `shared_utils::logging`.
- Niveaux : `error` / `warn` / `info` / `debug` / `trace`.
- Pas de contenu presse-papiers en `info` (PII) ; hash ou longueur en `debug` si besoin.

## 7. Tests

- Tests unitaires à côté des utilitaires purs (`SequenceDetector`, diff).
- Pas d’accès réseau dans les tests par défaut.
- Les adapters OS : tests d’intégration manuels documentés dans `OS_COMPATIBILITY.md`.

## 8. Frontend

- Svelte 5 runes / store centralisé.
- Composants atomiques dans `apps/desktop/src/components`.
- Styles via Tailwind ; pas de CSS global ad hoc non nécessaire.
- **Couleurs** : uniquement via tokens `runa.*` (variables `--runa-*` dans `src/theme/tokens.css`). Interdiction de hex / `zinc-*` / `emerald-*` / `red-*` en dur dans les composants.
- Popup frameless : ne pas ajouter de chrome système dans `tauri.conf.json`.

## 9. Git et secrets

- Pas de modèle GGUF ni de secrets dans le dépôt.
- `.gitignore` : `target/`, `node_modules/`, `dist/`, `*.gguf`, `.env`.

## 10. Checklist avant PR

- [ ] `cargo check --workspace`
- [ ] `cargo clippy --workspace -- -D warnings` (si CI)
- [ ] `npm run check` dans `apps/desktop` (`svelte-check`)
- [ ] Aucun nouveau type IPC hors `shared-types`
- [ ] Aucun helper dupliqué hors `shared-utils`
- [ ] Bindings specta régénérés si les types ont changé
