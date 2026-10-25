<script lang="ts">
  import { onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import ActionBar from "./components/ActionBar.svelte";
  import AppearanceMenu from "./components/AppearanceMenu.svelte";
  import AppHeader from "./components/AppHeader.svelte";
  import LanguageBar from "./components/LanguageBar.svelte";
  import ResultPanel from "./components/ResultPanel.svelte";
  import ToneBar from "./components/ToneBar.svelte";
  import {
    commands,
    type AccentPreset,
    type ThemeMode,
    type UiConfig,
    type UiDensity,
  } from "./lib/bindings";
  import type {
    InferenceEvent,
    ModelDownloadEvent,
    TriggerEvent,
  } from "./lib/events";
  import { store } from "./lib/store.svelte";
  import { applyTheme, normalizeThemeUi } from "./lib/theme";

  let themeMode = $state<ThemeMode>("dark");
  let accent = $state<AccentPreset>("indigo");
  let density = $state<UiDensity>("comfortable");
  let launchAtLogin = $state(true);
  let modelPath = $state("");
  let downloadBusy = $state(false);
  let downloadLabel = $state<string | null>(null);

  function formatError(err: {
    kind: string;
    message?: string;
  }): string {
    if (err.kind === "cancelled") return "annulé";
    if (typeof err.message === "string") return err.message;
    return String(err.kind);
  }

  function applyUiConfig(ui: UiConfig) {
    const normalized = normalizeThemeUi(ui);
    themeMode = normalized.theme_mode;
    accent = normalized.accent;
    density = normalized.density;
    applyTheme(normalized, { forceCompact: store.windowMode === "popup" });
  }

  async function refreshCanReplace() {
    const result = await commands.hasReplaceOrigin();
    if (result.status === "ok") {
      store.canReplace = result.data;
    }
  }

  async function refreshDiff() {
    if (!store.clipboardText || !store.resultText) {
      store.setDiff([]);
      return;
    }
    if (store.clipboardText === store.resultText) {
      store.setDiff([]);
      return;
    }
    const result = await commands.wordDiffCmd(
      store.clipboardText,
      store.resultText,
    );
    if (result.status === "ok") {
      store.setDiff(result.data);
    }
  }

  async function persistTheme(next: {
    themeMode: ThemeMode;
    accent: AccentPreset;
    density: UiDensity;
  }) {
    const result = await commands.setUiTheme(
      next.themeMode,
      next.accent,
      next.density,
    );
    if (result.status === "ok") {
      applyUiConfig(result.data);
    } else {
      store.errorMessage = formatError(result.error);
    }
  }

  async function persistLaunchAtLogin(enabled: boolean) {
    const previous = launchAtLogin;
    launchAtLogin = enabled;
    const result = await commands.setLaunchAtLogin(enabled);
    if (result.status === "ok") {
      launchAtLogin = result.data;
    } else {
      launchAtLogin = previous;
      store.errorMessage = formatError(result.error);
    }
  }

  onMount(() => {
    let unlistenTrigger: (() => void) | undefined;
    let unlistenInference: (() => void) | undefined;
    let unlistenTheme: (() => void) | undefined;
    let unlistenDownload: (() => void) | undefined;
    const win = getCurrentWindow();
    const label = win.label;
    store.windowMode = label === "main" ? "main" : "popup";

    void (async () => {
      const cfg = await commands.getConfig();
      if (cfg.status === "ok") {
        applyUiConfig(cfg.data.ui);
        modelPath = cfg.data.ai.model_path;
        launchAtLogin = cfg.data.launch_at_login ?? true;
      } else {
        applyTheme(
          { theme_mode: "dark", accent: "indigo", density: "comfortable" },
          { forceCompact: store.windowMode === "popup" },
        );
      }

      unlistenTheme = await listen<UiConfig>("ui_theme_changed", (event) => {
        applyUiConfig(event.payload);
      });

      if (store.windowMode === "popup") {
        // Show/focus : uniquement côté Rust (bridge) — pas de second show ici.
        unlistenTrigger = await listen<TriggerEvent>("trigger_fired", (event) => {
          store.onTrigger(event.payload);
          // Preview (texte vide) : hydrate l'UI sans lancer l'inférence.
          const text = event.payload.clipboard_text.trim();
          if (text.length === 0) return;
          if (store.activeTab === "translate") {
            void runTranslate();
          } else {
            void runCorrect();
          }
        });
      } else {
        void refreshCanReplace();
        unlistenDownload = await listen<ModelDownloadEvent>(
          "model_download",
          (event) => {
            const p = event.payload;
            switch (p.type) {
              case "started":
                downloadBusy = true;
                downloadLabel = "Téléchargement…";
                break;
              case "progress": {
                downloadBusy = true;
                const total = p.total_bytes;
                if (total && total > 0) {
                  const pct = Math.min(
                    100,
                    Math.round((p.downloaded_bytes / total) * 100),
                  );
                  downloadLabel = `Téléchargement ${pct} %`;
                } else {
                  downloadLabel = `Téléchargement ${Math.round(p.downloaded_bytes / (1024 * 1024))} Mo`;
                }
                break;
              }
              case "done":
                downloadBusy = false;
                downloadLabel = "Modèle prêt";
                modelPath = p.path;
                break;
              case "error":
                downloadBusy = false;
                downloadLabel = null;
                store.errorMessage = formatError(p.error);
                break;
              default: {
                const _exhaustive: never = p;
                return _exhaustive;
              }
            }
          },
        );
      }

      unlistenInference = await listen<InferenceEvent>("inference", async (event) => {
        store.onInference(event.payload);
        if (
          event.payload.type === "done" &&
          (store.activeGenerationId === null ||
            event.payload.generation_id === store.activeGenerationId)
        ) {
          await refreshDiff();
        }
      });
    })();

    return () => {
      unlistenTrigger?.();
      unlistenInference?.();
      unlistenTheme?.();
      unlistenDownload?.();
    };
  });

  async function runCorrect() {
    store.errorMessage = null;
    const result = await commands.correct(
      store.clipboardText,
      store.tone,
      store.context,
    );
    if (result.status === "ok") {
      store.beginGeneration(result.data);
    } else {
      store.errorMessage = formatError(result.error);
    }
  }

  async function runTranslate() {
    store.errorMessage = null;
    const result = await commands.translate(
      store.clipboardText,
      store.targetLang,
    );
    if (result.status === "ok") {
      store.beginGeneration(result.data);
    } else {
      store.errorMessage = formatError(result.error);
    }
  }

  async function onEnter() {
    store.errorMessage = null;
    const result = await commands.applyReplacement(
      store.resultText || store.clipboardText,
    );
    if (result.status === "error") {
      store.errorMessage = formatError(result.error);
    }
  }

  async function onCopy() {
    store.errorMessage = null;
    const text = store.resultText || store.clipboardText;
    const result = await commands.copyText(text);
    if (result.status === "error") {
      store.errorMessage = formatError(result.error);
    }
  }

  async function onEscape() {
    const result = await commands.dismiss();
    if (result.status === "error") {
      store.errorMessage = formatError(result.error);
    }
  }

  async function onCancel() {
    const result = await commands.cancelInference();
    if (result.status === "error") {
      store.errorMessage = formatError(result.error);
    }
  }

  async function onDownloadModel() {
    store.errorMessage = null;
    downloadBusy = true;
    downloadLabel = "Démarrage…";
    const result = await commands.downloadModel();
    if (result.status === "error") {
      downloadBusy = false;
      downloadLabel = null;
      store.errorMessage = formatError(result.error);
    }
  }

  function onKeydown(e: KeyboardEvent) {
    if (store.windowMode === "main") {
      return;
    }
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      void onEnter();
    }
    if (e.key === "Escape") {
      e.preventDefault();
      void onEscape();
    }
  }

  async function dragWindow(e: MouseEvent) {
    if (e.button !== 0) return;
    const target = e.target as HTMLElement | null;
    if (target?.closest("button, a, input, textarea, select")) return;
    await getCurrentWindow().startDragging();
  }

  async function minimizeWindow() {
    await getCurrentWindow().minimize();
  }

  async function toggleMaximize() {
    await getCurrentWindow().toggleMaximize();
  }

  async function hideMainWindow() {
    await getCurrentWindow().hide();
  }
</script>

<svelte:window onkeydown={onKeydown} />

<main
  class="flex h-full w-full flex-col bg-runa-bg text-runa-text"
  class:main-window={store.windowMode === "main"}
>
  <AppHeader
    mode={store.windowMode}
    activeTab={store.activeTab}
    ontab={(tab) => (store.activeTab = tab)}
    ondrag={(e) => void dragWindow(e)}
    ondblclick={() => void toggleMaximize()}
    onminimize={() => void minimizeWindow()}
    onmaximize={() => void toggleMaximize()}
    onclose={() => void hideMainWindow()}
  >
    {#snippet children()}
      {#if store.windowMode === "main"}
        <AppearanceMenu
          {themeMode}
          {accent}
          {density}
          {launchAtLogin}
          onchange={(next) => void persistTheme(next)}
          onLaunchAtLogin={(enabled) => void persistLaunchAtLogin(enabled)}
        />
      {/if}
    {/snippet}
  </AppHeader>

  <div
    class="flex min-h-0 flex-1 flex-col p-4"
    class:pt-3={store.windowMode === "popup"}
  >
    {#if store.activeTab === "correct"}
      <ToneBar
        tone={store.tone}
        context={store.context}
        ontone={(t) => (store.tone = t)}
        oncontext={(c) => (store.context = c)}
        onrun={() => void runCorrect()}
      />
    {:else}
      <LanguageBar
        target={store.targetLang}
        onchange={(lang) => (store.targetLang = lang)}
        onrun={() => void runTranslate()}
      />
    {/if}

    {#if store.windowMode === "main"}
      <div
        class="mb-3 flex flex-wrap items-center gap-2 rounded-lg border border-runa-border bg-runa-panel px-3 py-2 text-xs"
      >
        <span class="text-runa-muted">Modèle local</span>
        <span class="min-w-0 flex-1 truncate text-runa-text" title={modelPath}>
          {modelPath || "non installé"}
        </span>
        <button
          type="button"
          class="rounded-lg bg-runa-chip px-3 py-1.5 text-runa-text hover:bg-runa-chipHover focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-1 focus-visible:outline-runa-accent disabled:opacity-50"
          disabled={downloadBusy}
          onclick={() => void onDownloadModel()}
        >
          {downloadBusy ? "Téléchargement…" : "Télécharger le modèle"}
        </button>
        {#if downloadLabel}
          <span class="w-full text-runa-muted">{downloadLabel}</span>
        {/if}
      </div>
      <label
        class="mb-1 text-xs uppercase tracking-wider text-runa-muted"
        for="source">Source</label
      >
      <textarea
        id="source"
        class="mb-3 min-h-[120px] flex-none resize-y rounded-lg border border-runa-border bg-runa-panel p-3 text-sm leading-relaxed text-runa-text outline-none focus:border-runa-accent focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-1 focus-visible:outline-runa-accent"
        bind:value={store.clipboardText}
        placeholder="Saisissez ou collez un texte…"
      ></textarea>
    {/if}

    <ResultPanel
      showSource={store.windowMode === "popup"}
      sourceText={store.clipboardText}
      resultText={store.resultText}
      streaming={store.streaming}
      diff={store.diff}
    />

    {#if store.errorMessage}
      <p class="mt-2 text-xs text-runa-danger">{store.errorMessage}</p>
    {/if}

    <ActionBar
      mode={store.windowMode}
      canReplace={store.canReplace}
      streaming={store.streaming}
      oncopy={() => void onCopy()}
      onreplace={() => void onEnter()}
      ondismiss={() => void onEscape()}
      oncancel={() => void onCancel()}
    />
  </div>
</main>
