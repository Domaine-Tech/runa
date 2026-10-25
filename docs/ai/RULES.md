# Règles IA — Runa (source unique)

**Ceci est la seule source de vérité** pour tous les agents (Cursor, Claude, Copilot, Codex, Gemini, Continue, Aider, etc.).

Les fichiers à la racine (`AGENTS.md`, `CLAUDE.md`, …) et `.cursor/rules/` sont des **pointeurs** : ne pas y dupliquer le contenu métier.

Avant toute modification de code : lire aussi, selon le besoin :
- [ARCHITECTURE.md](../ARCHITECTURE.md)
- [CODING_STANDARDS.md](../CODING_STANDARDS.md)
- [OS_COMPATIBILITY.md](../OS_COMPATIBILITY.md)
- [AI_ENGINE.md](../AI_ENGINE.md)
- [product/SPRINTS.md](../product/SPRINTS.md) / [product/BACKLOG.md](../product/BACKLOG.md)
- Prompts standardisés Mode Plan : [PROMPTS.md](./PROMPTS.md) (`fait prompt N`)

---

## 1. Mission produit

Construire **Runa** : app bureau 100 % locale, daemon tray + **popup rapide** (`Ctrl+C+C` &lt; 400 ms) + **fenêtre principale** (saisie libre), correction / traduction / reformulation via LLM local, remplacement ou copier/coller. Pas de cloud d’inférence.

## 2. Architecture (non négociable)

1. Hexagonale : ports = traits Rust ; adapters dans `platform-engine` / `ai-engine`.
2. **Types uniquement** dans `crates/shared-types` (DTO, events, `RunaError`, config).
3. **Utils uniquement** dans `crates/shared-utils` (pas de helpers locaux dispersés).
4. Composition root **uniquement** : `apps/desktop/src-tauri`.
5. Frontend : types via **specta / tauri-specta** → `bindings.ts` ; pas d’interfaces IPC manuelles ; pas de `any`.
6. Erreurs : `RunaError` seul ; **interdit** `.unwrap()` / `.expect()` hors `#[cfg(test)]`.
7. Traits **sans** suffixe `Trait` (`ClipboardWatcher`, pas `ClipboardWatcherTrait`).

## 3. Concurrence

- Tokio : IPC, timers, portails — **jamais** le décodage LLM.
- Thread dédié `llm-worker` : seul propriétaire du modèle.
- Windows clipboard : thread message-only + pompe `GetMessage`.
- Capturer la fenêtre active **avant** `show()` de la popup.

## 4. Produit / sprints

1. Travailler le sprint **Doing** / prochain `Todo` dans `docs/product/SPRINTS.md`.
2. Toute idée hors scope → ajouter une entrée dans `docs/product/BACKLOG.md` (modèle BL-XXX), **ne pas** gonfler le sprint.
3. Mettre à jour `CHANGELOG.md` / `ROADMAP.md` quand une version est livrée.
4. Feature `llama` = sprint S4 ; adapters macOS = S6. Ne pas improviser hors sprint actif, sauf demande explicite.

## 5. Style de travail agent

1. Réponses en **français** si l’utilisateur écrit en français.
2. Diffs **minimaux** ; pas de refactor opportuniste.
3. Pas de fichiers markdown non demandés (sauf docs produit/règles quand c’est la tâche).
4. Pas de commit / push / `--force` sans demande explicite.
5. Pas de secrets, pas de `.gguf` dans le dépôt.
6. Vérifier : `cargo check --workspace` ; frontend `npm run check` si UI touchée.
7. Avant **publish / Release** : passe sécu + maj deps (voir `docs/COMMANDS.md` § Pré-publish ; Prompt 11) — audits, clippy ; push `main` → CI → **Release auto**. MAJOR.MINOR = sprint / dev avant push ; collision → bump **patch** seulement (`1.0.0` → `1.0.1`).
8. Logs : `tracing` ; **jamais** le contenu presse-papiers en niveau `info`.

## 6. Interdits sécurité / éthique (projet)

- Pas d’exfiltration réseau des textes utilisateur.
- Fallback evdev Linux : opt-in, filtre Ctrl+C, aucune persistance des frappes.
- Pas de keylogger généraliste « pour debug ».

## 7. Checklist avant de considérer une tâche terminée

- [ ] Types / utils au bon endroit (pas de duplication)
- [ ] `RunaError` propagé (pas de panic de contrôle de flux)
- [ ] Bindings specta régénérés ou `bindings.ts` cohérent si IPC changé
- [ ] Docs produit mises à jour si jalon / backlog touché
- [ ] `cargo check --workspace` OK
- [ ] Si release : Pré-publish (audits + maj deps + CI `security` verte) OK
