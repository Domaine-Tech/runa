<script lang="ts">
  import type { DiffSpan } from "../lib/bindings";

  interface Props {
    showSource: boolean;
    sourceText: string;
    resultText: string;
    streaming: boolean;
    diff: DiffSpan[];
  }

  type DisplaySpan =
    | { kind: "equal"; text: string }
    | { kind: "change"; text: string; original: string };

  let { showSource, sourceText, resultText, streaming, diff }: Props = $props();

  let displaySpans: DisplaySpan[] = $derived.by(() => {
    const spans: DisplaySpan[] = [];
    let i = 0;
    while (i < diff.length) {
      const span = diff[i];
      if (!span) break;
      if (span.kind === "equal") {
        spans.push({ kind: "equal", text: span.text });
        i += 1;
        continue;
      }
      if (span.kind === "delete") {
        const deletes: string[] = [];
        while (i < diff.length) {
          const del = diff[i];
          if (!del || del.kind !== "delete") break;
          deletes.push(del.text);
          i += 1;
        }
        const inserts: string[] = [];
        while (i < diff.length) {
          const ins = diff[i];
          if (!ins || ins.kind !== "insert") break;
          inserts.push(ins.text);
          i += 1;
        }
        if (inserts.length > 0) {
          spans.push({
            kind: "change",
            text: inserts.join(""),
            original: deletes.join(""),
          });
        }
        continue;
      }
      spans.push({ kind: "change", text: span.text, original: "" });
      i += 1;
    }
    return spans;
  });

  let showDiff = $derived(!streaming && diff.length > 0);
</script>

<section
  class="min-h-0 flex-1 overflow-auto rounded-lg border border-runa-border bg-runa-panel p-3 text-sm leading-relaxed text-runa-text"
>
  {#if showSource}
    <p class="mb-2 text-xs uppercase tracking-wider text-runa-muted">Source</p>
    <p class="mb-4 whitespace-pre-wrap text-runa-muted">
      {sourceText || "Aucun texte"}
    </p>
  {/if}
  <p class="mb-2 text-xs uppercase tracking-wider text-runa-muted">
    Résultat{streaming ? "…" : ""}
  </p>
  {#if showDiff}
    <p class="mb-3 whitespace-pre-wrap">
      {#each displaySpans as span, i (i)}
        {#if span.kind === "equal"}
          <span>{span.text}</span>
        {:else}
          <span
            class="cursor-help rounded bg-runa-insert-bg/60 text-runa-insert-fg"
            title={span.original || undefined}
            >{span.text}</span
          >
        {/if}
      {/each}
    </p>
  {:else}
    <p class="whitespace-pre-wrap">{resultText || "—"}</p>
  {/if}
</section>
