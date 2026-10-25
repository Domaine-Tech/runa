# Agents — Runa

**Source unique des règles :** [`docs/ai/RULES.md`](docs/ai/RULES.md)

Ouvre et suis ce fichier avant toute modification. Ne duplique pas les règles ici.

## Raccourcis

| Sujet | Doc |
| --- | --- |
| Règles IA (obligatoire) | [docs/ai/RULES.md](docs/ai/RULES.md) |
| Prompts Mode Plan | [docs/ai/PROMPTS.md](docs/ai/PROMPTS.md) (`fait prompt 13` = CI / release / MAJ) |
| Architecture | [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) |
| Standards code | [docs/CODING_STANDARDS.md](docs/CODING_STANDARDS.md) |
| Sprints | [docs/product/SPRINTS.md](docs/product/SPRINTS.md) |
| Backlog features | [docs/product/BACKLOG.md](docs/product/BACKLOG.md) |
| Commandes | [docs/COMMANDS.md](docs/COMMANDS.md) |
| Svelte AI | MCP local `@sveltejs/mcp` (`.cursor/mcp.json`) + règle / skills / agent — [doc](https://svelte.dev/docs/ai/overview) · plugin marketplace : `/add-plugin svelte` |

## Impératifs (résumé — détail dans RULES.md)

1. Hexagonale : types → `shared-types`, utils → `shared-utils`, pas de helpers locaux.
2. Pas de `.unwrap()` / `.expect()` hors tests ; erreurs = `RunaError`.
3. Idées hors sprint → `docs/product/BACKLOG.md`, pas de scope creep.
4. Français si l’utilisateur écrit en français ; diffs minimaux ; pas de commit sans demande.
