# Règles IA multi-LLM — sans doublon

## Principe

```text
                    ┌─────────────────────┐
                    │  docs/ai/RULES.md   │  ← SEULE source de vérité
                    └──────────┬──────────┘
           ┌───────────────────┼───────────────────┐
           ▼                   ▼                   ▼
      AGENTS.md            CLAUDE.md      .github/copilot-…
           ▼                   ▼                   ▼
   .cursor/rules/*.mdc    (autres outils)     Continuer / Aider…
```

Tout outil LLM doit **pointer** vers [`RULES.md`](./RULES.md).  
Interdiction de recopier les règles dans plusieurs fichiers (divergence garantie).

## Fichiers pointeurs

| Outil | Fichier | Contenu |
| --- | --- | --- |
| Générique / Cursor / Codex | [`AGENTS.md`](../../AGENTS.md) | Lien + résumé 4 lignes max |
| Claude | [`CLAUDE.md`](../../CLAUDE.md) | Lien seul |
| Gemini | [`GEMINI.md`](../../GEMINI.md) | Lien seul |
| GitHub Copilot | [`.github/copilot-instructions.md`](../../.github/copilot-instructions.md) | Lien seul |
| Continue | [`.continue/rules/runa.md`](../../.continue/rules/runa.md) | Lien seul |
| Cursor Rules | [`.cursor/rules/`](../../.cursor/rules/) | Pointeurs + globs, pas de copie longue |

## Modifier une règle

1. Éditer **seulement** `docs/ai/RULES.md`.
2. Si besoin, ajuster le résumé 4 lignes de `AGENTS.md` (sans réécrire tout le doc).
3. Ne pas allonger les `.mdc` Cursor.

## Prompts Mode Plan

Bibliothèque de prompts standardisés (audit, sprints, perf, OS, git…) : [`PROMPTS.md`](./PROMPTS.md)  
Lancement express : `fait prompt 1`, `fait prompt 13` (CI / release / MAJ), `réalise sprint S1`, `audit typage`, etc.

## Produit

Planification versions / sprints / backlog : [`docs/product/`](../product/).
