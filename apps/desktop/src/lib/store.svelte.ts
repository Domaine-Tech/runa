import type { DiffSpan, Language, Tone, WritingContext } from "./bindings";
import type { InferenceEvent, TriggerEvent } from "./events";

export type TabId = "correct" | "translate";
export type WindowMode = "popup" | "main";

class RunaStore {
  clipboardText = $state("");
  resultText = $state("");
  streaming = $state(false);
  activeTab = $state<TabId>("correct");
  tone = $state<Tone>("fluid");
  context = $state<WritingContext>("note");
  targetLang = $state<Language>("auto");
  errorMessage = $state<string | null>(null);
  diff = $state<DiffSpan[]>([]);
  /** Filtre les événements inference de cette webview. */
  activeGenerationId = $state<number | null>(null);
  canReplace = $state(false);
  windowMode = $state<WindowMode>("popup");

  onTrigger(event: TriggerEvent) {
    this.clipboardText = event.clipboard_text;
    this.resultText = event.clipboard_text;
    this.streaming = false;
    this.errorMessage = null;
    this.diff = [];
    this.activeGenerationId = null;
    this.canReplace = event.origin.platform === "windows";
  }

  beginGeneration(id: number) {
    this.activeGenerationId = id;
    this.diff = [];
  }

  onInference(event: InferenceEvent) {
    if (
      this.activeGenerationId !== null &&
      event.generation_id !== this.activeGenerationId
    ) {
      return;
    }

    switch (event.type) {
      case "started":
        this.streaming = true;
        this.resultText = "";
        this.errorMessage = null;
        this.diff = [];
        break;
      case "token":
        this.resultText += event.text;
        break;
      case "done":
        this.streaming = false;
        this.resultText = event.full_text;
        break;
      case "cancelled":
        this.streaming = false;
        break;
      case "error": {
        this.streaming = false;
        const err = event.error;
        this.errorMessage =
          err.kind === "cancelled" ? "annulé" : String(err.message);
        break;
      }
      default: {
        const _exhaustive: never = event;
        return _exhaustive;
      }
    }
  }

  setDiff(spans: DiffSpan[]) {
    this.diff = spans;
  }
}

export const store = new RunaStore();
