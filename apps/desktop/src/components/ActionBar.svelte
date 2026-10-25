<script lang="ts">
  interface Props {
    mode: "popup" | "main";
    canReplace: boolean;
    streaming: boolean;
    oncopy: () => void;
    onreplace: () => void;
    ondismiss?: () => void;
    oncancel?: () => void;
  }

  let {
    mode,
    canReplace,
    streaming,
    oncopy,
    onreplace,
    ondismiss,
    oncancel,
  }: Props = $props();
</script>

<footer class="mt-3 flex items-center justify-between gap-2 text-[11px] text-runa-muted">
  {#if mode === "popup"}
    <span>Entrée = remplacer · Échap = fermer</span>
    <div class="flex gap-2">
      {#if streaming}
        <button
          type="button"
          class="rounded-lg bg-runa-chip px-3 py-1.5 text-xs text-runa-danger hover:bg-runa-chipHover focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-1 focus-visible:outline-runa-accent"
          onclick={() => oncancel?.()}
        >
          Annuler
        </button>
      {/if}
      <button
        type="button"
        class="rounded-lg bg-runa-chip px-3 py-1.5 text-xs text-runa-text hover:bg-runa-chipHover focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-1 focus-visible:outline-runa-accent"
        onclick={oncopy}
      >
        Copier
      </button>
      <button
        type="button"
        class="rounded-lg px-3 py-1.5 text-xs focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-1 focus-visible:outline-runa-accent {canReplace
          ? 'bg-runa-accent text-runa-accent-fg hover:opacity-90'
          : 'bg-runa-chip text-runa-muted opacity-50'}"
        disabled={!canReplace}
        title={canReplace
          ? "Remplacer dans l’app d’origine"
          : "Aucune origine — utilisez Ctrl+C+C d’abord"}
        onclick={onreplace}
      >
        Remplacer
      </button>
      <button
        type="button"
        class="rounded-lg bg-runa-chip px-3 py-1.5 text-xs text-runa-text hover:bg-runa-chipHover focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-1 focus-visible:outline-runa-accent"
        onclick={() => ondismiss?.()}
      >
        Masquer
      </button>
    </div>
  {:else}
    <span>Fenêtre principale</span>
    <div class="flex gap-2">
      {#if streaming}
        <button
          type="button"
          class="rounded-lg bg-runa-chip px-3 py-1.5 text-xs text-runa-danger hover:bg-runa-chipHover focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-1 focus-visible:outline-runa-accent"
          onclick={() => oncancel?.()}
        >
          Annuler
        </button>
      {/if}
      <button
        type="button"
        class="rounded-lg bg-runa-chip px-3 py-1.5 text-xs text-runa-text hover:bg-runa-chipHover focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-1 focus-visible:outline-runa-accent"
        onclick={oncopy}
      >
        Copier
      </button>
      <button
        type="button"
        class="rounded-lg px-3 py-1.5 text-xs focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-1 focus-visible:outline-runa-accent {canReplace
          ? 'bg-runa-accent text-runa-accent-fg hover:opacity-90'
          : 'bg-runa-chip text-runa-muted opacity-50'}"
        disabled={!canReplace}
        title={canReplace
          ? "Remplacer dans l’app d’origine"
          : "Aucune origine HWND — utilisez Ctrl+C+C d’abord"}
        onclick={onreplace}
      >
        Remplacer
      </button>
    </div>
  {/if}
</footer>
