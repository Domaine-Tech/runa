<script lang="ts">
  import type { TabId } from "../lib/store.svelte";
  import RunaMark from "./RunaMark.svelte";
  import TabBar from "./TabBar.svelte";
  import WindowControls from "./WindowControls.svelte";

  interface Props {
    mode: "popup" | "main";
    activeTab: TabId;
    /** Version runtime Tauri (ex. 1.0.1), affichée à côté du titre. */
    version?: string;
    ontab: (tab: TabId) => void;
    ondrag?: (e: MouseEvent) => void;
    ondblclick?: () => void;
    onminimize?: () => void;
    onmaximize?: () => void;
    onclose?: () => void;
    children?: import("svelte").Snippet;
  }

  let {
    mode,
    activeTab,
    version = "",
    ontab,
    ondrag,
    ondblclick,
    onminimize,
    onmaximize,
    onclose,
    children,
  }: Props = $props();
</script>

{#if mode === "main"}
  <header
    class="relative flex h-12 shrink-0 cursor-default items-center gap-3 border-b border-runa-border/60 bg-runa-bg px-4 select-none"
  >
    <!-- zone de drag (évite les listeners a11y sur <header>) -->
    <div
      class="absolute inset-x-0 top-0 h-12"
      role="presentation"
      onmousedown={(e) => ondrag?.(e)}
      ondblclick={() => ondblclick?.()}
    ></div>
    <div class="relative z-10 flex items-center gap-2">
      <RunaMark />
      <h1 class="text-sm font-semibold tracking-wide text-runa-text">Runa</h1>
      {#if version}
        <span class="text-[11px] tabular-nums text-runa-muted" title="Version">
          v{version}
        </span>
      {/if}
    </div>
    <div class="relative z-10">
      <TabBar active={activeTab} onchange={ontab} />
    </div>
    <div class="relative z-10 ml-auto flex items-center gap-2">
      {#if children}
        {@render children()}
      {/if}
      <WindowControls
        onminimize={() => onminimize?.()}
        onmaximize={() => onmaximize?.()}
        onclose={() => onclose?.()}
      />
    </div>
  </header>
{:else}
  <header class="mb-0 flex items-center justify-between px-4 pt-4">
    <div class="flex items-center gap-2">
      <RunaMark />
      <h1 class="text-sm font-semibold tracking-wide text-runa-text">Runa</h1>
      {#if version}
        <span class="text-[11px] tabular-nums text-runa-muted" title="Version">
          v{version}
        </span>
      {/if}
    </div>
    <TabBar active={activeTab} onchange={ontab} />
  </header>
{/if}
