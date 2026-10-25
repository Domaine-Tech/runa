<script lang="ts">
  import type { TabId } from "../lib/store.svelte";

  interface Props {
    active: TabId;
    onchange: (tab: TabId) => void;
  }

  let { active, onchange }: Props = $props();
</script>

<div class="flex gap-0.5 rounded-lg bg-runa-chip p-0.5 text-xs">
  <button
    type="button"
    class="rounded-md px-2.5 py-1.5 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-1 focus-visible:outline-runa-accent {active ===
    'correct'
      ? 'bg-runa-accent text-runa-accent-fg'
      : 'text-runa-muted hover:text-runa-text'}"
    onclick={() => onchange("correct")}
  >
    Correction
  </button>
  <button
    type="button"
    class="rounded-md px-2.5 py-1.5 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-1 focus-visible:outline-runa-accent {active ===
    'translate'
      ? 'bg-runa-accent text-runa-accent-fg'
      : 'text-runa-muted hover:text-runa-text'}"
    onclick={() => onchange("translate")}
  >
    Traduction
  </button>
</div>
