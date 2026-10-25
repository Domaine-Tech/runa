<script lang="ts">
  import type { Language } from "../lib/bindings";

  interface Props {
    target: Language;
    onchange: (lang: Language) => void;
    onrun: () => void;
  }

  let { target, onchange, onrun }: Props = $props();

  const languages: { id: Language; label: string }[] = [
    { id: "auto", label: "Auto" },
    { id: "fr", label: "FR" },
    { id: "en", label: "EN" },
    { id: "es", label: "ES" },
    { id: "de", label: "DE" },
  ];
</script>

<div class="mb-2 flex flex-wrap items-center gap-1">
  {#each languages as lang}
    <button
      type="button"
      class="rounded-lg px-3 py-1 text-xs focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-1 focus-visible:outline-runa-accent {target ===
      lang.id
        ? 'bg-runa-accent text-runa-accent-fg'
        : 'bg-runa-chip text-runa-muted'}"
      onclick={() => onchange(lang.id)}
    >
      {lang.label}
    </button>
  {/each}
  <button
    type="button"
    class="ml-auto rounded-lg bg-runa-chip px-3 py-1 text-xs text-runa-text hover:bg-runa-chipHover focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-1 focus-visible:outline-runa-accent"
    onclick={onrun}
  >
    Traduire
  </button>
</div>
