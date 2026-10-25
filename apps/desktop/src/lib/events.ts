/** Payloads d’événements IPC (alignés sur `shared-types`, hors export commandes specta). */
import type { RunaError } from "./bindings";

/** Miroir de `shared_types::WindowToken` (plus exporté dans bindings après retrait de `remember_origin`). */
export type WindowToken =
  | { platform: "windows"; hwnd: number }
  | { platform: "macos"; pid: number }
  | { platform: "linux"; pid: number }
  | { platform: "unknown" };

export type ScreenPoint = {
  x: number;
  y: number;
};

export type TriggerEvent = {
  clipboard_text: string;
  origin: WindowToken;
  sequence_ms: number;
  caret: ScreenPoint | null;
};

export type InferenceEvent =
  | { type: "started"; generation_id: number }
  | { type: "token"; generation_id: number; text: string }
  | { type: "done"; generation_id: number; full_text: string }
  | { type: "cancelled"; generation_id: number }
  | { type: "error"; generation_id: number; error: RunaError };

export type ModelDownloadEvent =
  | { type: "started"; total_bytes: number | null }
  | {
      type: "progress";
      downloaded_bytes: number;
      total_bytes: number | null;
    }
  | { type: "done"; path: string }
  | { type: "error"; error: RunaError };
