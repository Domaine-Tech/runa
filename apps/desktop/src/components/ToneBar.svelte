<script lang="ts">
  import type { Tone, WritingContext } from "../lib/bindings";
  import { CONTEXT_LABELS, TONE_LABELS } from "../lib/theme";

  interface Props {
    tone: Tone;
    context: WritingContext;
    ontone: (tone: Tone) => void;
    oncontext: (context: WritingContext) => void;
    onrun: () => void;
  }

  let { tone, context, ontone, oncontext, onrun }: Props = $props();

  const tones: Tone[] = ["formal", "concise", "fluid"];
  const contexts: WritingContext[] = ["mail", "chat", "note"];

  const chip = (active: boolean) =>
    `rounded-lg px-3 py-1 text-xs focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-1 focus-visible:outline-runa-accent ${
      active
        ? "bg-runa-accent text-runa-accent-fg"
        : "bg-runa-chip text-runa-muted"
    }`;
</script>

<div class="mb-2 flex flex-col gap-1">
  <div class="flex flex-wrap gap-1">
    {#each contexts as c (c)}
      <button type="button" class={chip(context === c)} onclick={() => oncontext(c)}>
        {CONTEXT_LABELS[c]}
      </button>
    {/each}
  </div>
  <div class="flex flex-wrap gap-1">
    {#each tones as t (t)}
      <button type="button" class={chip(tone === t)} onclick={() => ontone(t)}>
        {TONE_LABELS[t]}
      </button>
    {/each}
    <button
      type="button"
      class="ml-auto rounded-lg bg-runa-chip px-3 py-1 text-xs text-runa-text hover:bg-runa-chipHover focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-1 focus-visible:outline-runa-accent"
      onclick={onrun}
    >
      Lancer
    </button>
  </div>
</div>
