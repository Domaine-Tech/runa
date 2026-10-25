---
name: svelte-core-bestpractices
description: Guidance on writing fast, robust, modern Svelte code. Load this skill whenever in a Svelte project and asked to write/edit or analyze a Svelte component or module. Covers reactivity, event handling, styling, integration with libraries and more.
---

## `$state`

Only use the `$state` rune for variables that should be _reactive_ — in other words, variables that cause an `$effect`, `$derived` or template expression to update. Everything else can be a normal variable.

Objects and arrays (`$state({...})` or `$state([...])`) are made deeply reactive, meaning mutation will trigger updates. In cases where you're dealing with large objects that are only ever reassigned (rather than mutated), use `$state.raw` instead.

## `$derived`

To compute something from state, use `$derived` rather than `$effect`:

```js
// do this
let square = $derived(num * num);

// don't do this
let square;
$effect(() => {
	square = num * num;
});
```

`$derived` is given an expression, _not_ a function. For complex expressions use `$derived.by`.

## `$effect`

Effects are an escape hatch and should mostly be avoided. In particular, avoid updating state inside effects.

- Sync to external libraries → `{@attach ...}`
- User interaction → event handlers / function bindings
- Debug logging → `$inspect`
- Observe something external → `createSubscriber`

Never wrap the contents of an effect in `if (browser) {...}` — effects do not run on the server.

## `$props`

Treat props as though they will change. Values that depend on props should usually use `$derived`.

## Events

Use `onclick={...}` (not `on:click`). For `window` / `document`, prefer `<svelte:window>` / `<svelte:document>`.

## Snippets

Use `{#snippet ...}` and `{@render ...}` instead of slots.

## Each blocks

Prefer keyed `{#each}` blocks. Do not use the index as a key.

## Avoid legacy features

Always use runes mode for new code:

- `$state` instead of implicit reactivity
- `$derived` / `$effect` instead of `$:`
- `$props` instead of `export let`
- `onclick` instead of `on:click`
- snippets instead of slots
- classes with `$state` fields instead of stores when sharing reactivity

## Runa note

Respect project rules in `docs/ai/RULES.md` and `docs/CODING_STANDARDS.md` §8 (tokens `runa.*`, composants dans `apps/desktop/src/components`, types via specta/`bindings.ts`).
