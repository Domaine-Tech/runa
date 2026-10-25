<script lang="ts">
  interface Props {
    /** Classes Tailwind pour la taille (défaut h-4 w-4). */
    class?: string;
  }

  let { class: className = "h-4 w-4 shrink-0 text-runa-accent" }: Props = $props();
</script>

<svg
  xmlns="http://www.w3.org/2000/svg"
  viewBox="0 0 128 128"
  fill="none"
  class={className}
  aria-hidden="true"
>
  <path
    stroke="currentColor"
    stroke-width="12"
    stroke-linecap="round"
    stroke-linejoin="round"
    d="M64 24 L104 64 L64 104 L24 64 Z"
  />
</svg>
