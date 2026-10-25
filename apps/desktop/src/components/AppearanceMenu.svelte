<script lang="ts">
  import type { AccentPreset, ThemeMode, UiDensity } from "../lib/bindings";
  import {
    ACCENT_OPTIONS,
    DENSITY_OPTIONS,
    THEME_MODE_OPTIONS,
  } from "../lib/theme";

  interface Props {
    themeMode: ThemeMode;
    accent: AccentPreset;
    density: UiDensity;
    launchAtLogin: boolean;
    onchange: (next: {
      themeMode: ThemeMode;
      accent: AccentPreset;
      density: UiDensity;
    }) => void;
    onLaunchAtLogin: (enabled: boolean) => void;
  }

  let {
    themeMode,
    accent,
    density,
    launchAtLogin,
    onchange,
    onLaunchAtLogin,
  }: Props = $props();

  let open = $state(false);

  function commit(partial: {
    themeMode?: ThemeMode;
    accent?: AccentPreset;
    density?: UiDensity;
  }) {
    onchange({
      themeMode: partial.themeMode ?? themeMode,
      accent: partial.accent ?? accent,
      density: partial.density ?? density,
    });
  }
</script>

<div class="relative">
  <button
    type="button"
    class="flex h-7 items-center gap-1.5 rounded-md px-2 text-xs text-runa-muted transition-colors hover:bg-runa-chipHover hover:text-runa-text focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-1 focus-visible:outline-runa-accent"
    title="Apparence"
    aria-label="Apparence"
    aria-expanded={open}
    onclick={(e) => {
      e.stopPropagation();
      open = !open;
    }}
  >
    <svg width="14" height="14" viewBox="0 0 14 14" fill="none" aria-hidden="true">
      <circle cx="7" cy="7" r="5.25" stroke="currentColor" stroke-width="1.3" />
      <path
        d="M7 1.75v10.5A5.25 5.25 0 0 0 7 1.75Z"
        fill="currentColor"
        opacity="0.35"
      />
    </svg>
    Apparence
  </button>

  {#if open}
    <!-- backdrop to close -->
    <button
      type="button"
      class="fixed inset-0 z-40 cursor-default"
      aria-label="Fermer le menu apparence"
      onclick={() => (open = false)}
    ></button>
    <div
      class="absolute left-0 top-full z-50 mt-1 w-56 rounded-lg border border-runa-border bg-runa-panel p-3 shadow-lg"
      role="dialog"
      aria-label="Réglages d’apparence"
    >
      <p class="mb-1.5 text-[10px] uppercase tracking-wider text-runa-muted">Mode</p>
      <div class="mb-3 flex flex-wrap gap-1">
        {#each THEME_MODE_OPTIONS as opt (opt.id)}
          <button
            type="button"
            class="rounded-md px-2 py-1 text-xs {themeMode === opt.id
              ? 'bg-runa-accent text-runa-accent-fg'
              : 'bg-runa-chip text-runa-muted hover:text-runa-text'}"
            onclick={() => commit({ themeMode: opt.id })}
          >
            {opt.label}
          </button>
        {/each}
      </div>

      <p class="mb-1.5 text-[10px] uppercase tracking-wider text-runa-muted">Accent</p>
      <div class="mb-3 flex flex-wrap gap-1">
        {#each ACCENT_OPTIONS as opt (opt.id)}
          <button
            type="button"
            class="rounded-md px-2 py-1 text-xs {accent === opt.id
              ? 'bg-runa-accent text-runa-accent-fg'
              : 'bg-runa-chip text-runa-muted hover:text-runa-text'}"
            onclick={() => commit({ accent: opt.id })}
          >
            {opt.label}
          </button>
        {/each}
      </div>

      <p class="mb-1.5 text-[10px] uppercase tracking-wider text-runa-muted">Densité</p>
      <div class="mb-3 flex flex-wrap gap-1">
        {#each DENSITY_OPTIONS as opt (opt.id)}
          <button
            type="button"
            class="rounded-md px-2 py-1 text-xs {density === opt.id
              ? 'bg-runa-accent text-runa-accent-fg'
              : 'bg-runa-chip text-runa-muted hover:text-runa-text'}"
            onclick={() => commit({ density: opt.id })}
          >
            {opt.label}
          </button>
        {/each}
      </div>

      <p class="mb-1.5 text-[10px] uppercase tracking-wider text-runa-muted">Système</p>
      <label
        class="flex cursor-pointer items-center gap-2 rounded-md px-1 py-1 text-xs text-runa-text"
      >
        <input
          type="checkbox"
          class="accent-runa-accent"
          checked={launchAtLogin}
          onchange={(e) => onLaunchAtLogin(e.currentTarget.checked)}
        />
        Lancer au démarrage
      </label>
    </div>
  {/if}
</div>
