import { getCurrentWindow } from "@tauri-apps/api/window";
import type { AccentPreset, ThemeMode, UiConfig, UiDensity } from "./bindings";

export type ResolvedAppearance = "dark" | "light";

export type ThemeUi = {
  theme_mode: ThemeMode;
  accent: AccentPreset;
  density: UiDensity;
};

let mediaListener: ((e: MediaQueryListEvent) => void) | null = null;
let mediaQuery: MediaQueryList | null = null;

function systemAppearance(): ResolvedAppearance {
  if (typeof window === "undefined" || !window.matchMedia) {
    return "dark";
  }
  return window.matchMedia("(prefers-color-scheme: light)").matches
    ? "light"
    : "dark";
}

function resolveAppearance(mode: ThemeMode): ResolvedAppearance {
  if (mode === "system") {
    return systemAppearance();
  }
  return mode;
}

async function syncNativeTheme(mode: ThemeMode): Promise<void> {
  try {
    const win = getCurrentWindow();
    if (mode === "system") {
      await win.setTheme(null);
    } else {
      await win.setTheme(mode);
    }
  } catch {
    // Permission ou runtime hors Tauri — ignorer.
  }
}

function attachSystemListener(mode: ThemeMode, accent: AccentPreset, density: UiDensity) {
  if (mediaQuery && mediaListener) {
    mediaQuery.removeEventListener("change", mediaListener);
    mediaQuery = null;
    mediaListener = null;
  }
  if (mode !== "system" || typeof window === "undefined" || !window.matchMedia) {
    return;
  }
  mediaQuery = window.matchMedia("(prefers-color-scheme: light)");
  mediaListener = () => {
    applyThemeAttributes(mode, accent, density);
  };
  mediaQuery.addEventListener("change", mediaListener);
}

function applyThemeAttributes(
  mode: ThemeMode,
  accent: AccentPreset,
  density: UiDensity,
): void {
  const root = document.documentElement;
  root.dataset.appearance = resolveAppearance(mode);
  root.dataset.accent = accent;
  root.dataset.density = density;
}

/** Normalise une UiConfig partielle (configs TOML anciennes / export specta). */
export function normalizeThemeUi(
  ui: Partial<ThemeUi> | UiConfig | null | undefined,
): ThemeUi {
  return {
    theme_mode: ui?.theme_mode ?? "dark",
    accent: ui?.accent ?? "indigo",
    density: ui?.density ?? "comfortable",
  };
}

/**
 * Applique le thème sur `<html>` et aligne le thème natif de la fenêtre.
 * La popup force toujours `compact`.
 */
export function applyTheme(
  ui: ThemeUi | UiConfig,
  options?: { forceCompact?: boolean },
): void {
  const normalized = normalizeThemeUi(ui);
  const density: UiDensity =
    options?.forceCompact === true ? "compact" : normalized.density;
  applyThemeAttributes(normalized.theme_mode, normalized.accent, density);
  attachSystemListener(normalized.theme_mode, normalized.accent, density);
  void syncNativeTheme(normalized.theme_mode);
}

export const THEME_MODE_OPTIONS: { id: ThemeMode; label: string }[] = [
  { id: "system", label: "Système" },
  { id: "dark", label: "Sombre" },
  { id: "light", label: "Clair" },
];

export const ACCENT_OPTIONS: { id: AccentPreset; label: string }[] = [
  { id: "indigo", label: "Indigo" },
  { id: "slate", label: "Ardoise" },
  { id: "emerald", label: "Émeraude" },
  { id: "amber", label: "Ambre" },
];

export const DENSITY_OPTIONS: { id: UiDensity; label: string }[] = [
  { id: "comfortable", label: "Confortable" },
  { id: "compact", label: "Compact" },
];

export const TONE_LABELS: Record<"formal" | "concise" | "fluid", string> = {
  formal: "Formel",
  concise: "Concis",
  fluid: "Fluide",
};

export const CONTEXT_LABELS: Record<"mail" | "chat" | "note", string> = {
  mail: "Mail",
  chat: "Chat",
  note: "Note",
};
