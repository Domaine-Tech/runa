<script lang="ts">
  interface Props {
    onminimize: () => void;
    onmaximize: () => void;
    onclose: () => void;
  }

  let { onminimize, onmaximize, onclose }: Props = $props();
</script>

<div class="flex gap-0.5 rounded-lg bg-runa-chip p-0.5">
  <button
    type="button"
    class="flex h-7 w-7 items-center justify-center rounded-md text-runa-muted transition-colors hover:bg-runa-chipHover hover:text-runa-text focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-1 focus-visible:outline-runa-accent"
    title="Réduire"
    aria-label="Réduire"
    onclick={onminimize}
  >
    <svg width="12" height="12" viewBox="0 0 12 12" fill="none" aria-hidden="true">
      <path d="M2.5 6h7" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" />
    </svg>
  </button>
  <button
    type="button"
    class="flex h-7 w-7 items-center justify-center rounded-md text-runa-muted transition-colors hover:bg-runa-chipHover hover:text-runa-text focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-1 focus-visible:outline-runa-accent"
    title="Agrandir"
    aria-label="Agrandir"
    onclick={onmaximize}
  >
    <svg width="12" height="12" viewBox="0 0 12 12" fill="none" aria-hidden="true">
      <rect
        x="2.5"
        y="2.5"
        width="7"
        height="7"
        rx="1.2"
        stroke="currentColor"
        stroke-width="1.4"
      />
    </svg>
  </button>
  <button
    type="button"
    class="flex h-7 w-7 items-center justify-center rounded-md text-runa-muted transition-colors hover:bg-runa-chipHover hover:text-runa-text focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-1 focus-visible:outline-runa-accent"
    title="Fermer"
    aria-label="Fermer"
    onclick={onclose}
  >
    <svg width="12" height="12" viewBox="0 0 12 12" fill="none" aria-hidden="true">
      <path
        d="M3 3l6 6M9 3L3 9"
        stroke="currentColor"
        stroke-width="1.4"
        stroke-linecap="round"
      />
    </svg>
  </button>
</div>
