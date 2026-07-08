"use client";

import { FileSearch, X } from "lucide-react";
import { useId, useRef } from "react";
import type { ReactNode } from "react";
import type { GenerationRecord } from "@/core/generation-record";
import { tokenCount } from "@/core/token-usage";

type LlmGenerationDetailsProps = {
  readonly generation: GenerationRecord;
};

export function LlmGenerationDetails({ generation }: LlmGenerationDetailsProps) {
  const dialogRef = useRef<HTMLDialogElement>(null);
  const titleId = useId();
  const reasoning = reasoningText(generation.parsedOutput);

  return (
    <>
      <button
        type="button"
        onClick={() => dialogRef.current?.showModal()}
        aria-label="LLM details"
        title="LLM details"
        className="inline-flex h-7 w-7 items-center justify-center rounded-md border border-interactive-border bg-surface/70 text-muted transition hover:border-interactive-border-hover hover:bg-surface-muted hover:text-foreground"
      >
        <FileSearch aria-hidden="true" className="h-3.5 w-3.5" />
      </button>
      <dialog
        ref={dialogRef}
        aria-labelledby={titleId}
        className="fixed inset-0 m-auto max-h-[82vh] w-[min(920px,calc(100vw-32px))] overflow-hidden rounded-lg border border-interactive-border bg-background p-0 text-foreground backdrop:bg-black/70"
      >
        <div className="flex items-center justify-between gap-3 border-b border-border px-4 py-3">
          <div>
            <h3 id={titleId} className="text-sm font-semibold">
              LLM details
            </h3>
            <p className="mt-1 text-xs text-subtle">
              {generation.status} · {generation.provider}/{generation.model} ·{" "}
              {generation.promptVersion} · {formatTokenTotal(generation)}
            </p>
          </div>
          <form method="dialog">
            <button
              type="submit"
              aria-label="Close"
              title="Close"
              className="inline-flex h-8 w-8 items-center justify-center rounded-md border border-interactive-border bg-surface/70 text-muted transition hover:border-interactive-border-hover hover:bg-surface-muted hover:text-foreground"
            >
              <X aria-hidden="true" className="h-4 w-4" />
            </button>
          </form>
        </div>
        <div className="max-h-[calc(82vh-65px)] space-y-4 overflow-y-auto p-4">
          <GenerationBlock title="Token usage">
            {generation.tokenUsage ? (
              <div className="grid gap-2 text-xs sm:grid-cols-2">
                <KeyValue
                  label="Prompt"
                  value={formatTokenCount(generation.tokenUsage.promptTokens)}
                />
                <KeyValue
                  label="Completion"
                  value={formatTokenCount(generation.tokenUsage.completionTokens)}
                />
                <KeyValue
                  label="Total"
                  value={formatTokenCount(generation.tokenUsage.totalTokens)}
                />
                <KeyValue
                  label="Cached prompt"
                  value={formatTokenCount(generation.tokenUsage.cachedPromptTokens)}
                />
                <KeyValue
                  label="Reasoning"
                  value={formatTokenCount(generation.tokenUsage.reasoningTokens)}
                />
              </div>
            ) : (
              <p className="text-sm text-subtle">
                Token usage was not returned by the provider or this generation
                was created before token recording.
              </p>
            )}
          </GenerationBlock>

          <GenerationBlock title="Request">
            {generation.request ? (
              <div className="space-y-3">
                <KeyValue label="Schema" value={generation.request.schemaName} />
                <TextDump label="System prompt" value={generation.request.systemPrompt} />
                {generation.request.messages.map((message, index) => (
                  <TextDump
                    key={`${message.role}-${index}`}
                    label={`${message.role} message ${index + 1}`}
                    value={message.content}
                  />
                ))}
              </div>
            ) : (
              <p className="text-sm text-subtle">
                This generation was created before request snapshots were recorded.
              </p>
            )}
          </GenerationBlock>

          <GenerationBlock title="Player reasoning">
            <pre className="whitespace-pre-wrap break-words text-xs leading-5 text-muted">
              {reasoning ??
                "No player reasoning field was returned by the model. Hidden model reasoning is not available."}
            </pre>
          </GenerationBlock>

          <GenerationBlock title="Raw output">
            <pre className="whitespace-pre-wrap break-words text-xs leading-5 text-muted">
              {generation.rawOutput ?? "No raw output was recorded."}
            </pre>
          </GenerationBlock>

          <GenerationBlock title="Parsed output">
            <pre className="whitespace-pre-wrap break-words text-xs leading-5 text-muted">
              {generation.parsedOutput
                ? JSON.stringify(generation.parsedOutput, null, 2)
                : "No parsed output was recorded."}
            </pre>
          </GenerationBlock>

          {generation.error ? (
            <GenerationBlock title="Error">
              <pre className="whitespace-pre-wrap break-words text-xs leading-5 text-red-300">
                {generation.error}
              </pre>
            </GenerationBlock>
          ) : null}
        </div>
      </dialog>
    </>
  );
}

function GenerationBlock({
  title,
  children,
}: {
  readonly title: string;
  readonly children: ReactNode;
}) {
  return (
    <section className="rounded-md border border-border bg-surface/55">
      <div className="border-b border-border px-3 py-2 text-xs font-medium uppercase tracking-[0.14em] text-subtle">
        {title}
      </div>
      <div className="p-3">{children}</div>
    </section>
  );
}

function KeyValue({ label, value }: { readonly label: string; readonly value: string }) {
  return (
    <div className="grid gap-1 text-xs sm:grid-cols-[120px_minmax(0,1fr)]">
      <div className="text-subtle">{label}</div>
      <div className="break-words text-muted">{value}</div>
    </div>
  );
}

function TextDump({ label, value }: { readonly label: string; readonly value: string }) {
  return (
    <div>
      <div className="mb-1 text-xs text-subtle">{label}</div>
      <pre className="whitespace-pre-wrap break-words rounded-md bg-background p-3 text-xs leading-5 text-muted">
        {value}
      </pre>
    </div>
  );
}

function reasoningText(parsedOutput: Record<string, unknown> | null): string | null {
  if (!parsedOutput) {
    return null;
  }

  for (const key of ["reasoning", "reason", "thought", "analysis"]) {
    const value = parsedOutput[key];
    if (typeof value === "string" && value.trim().length > 0) {
      return value.trim();
    }
  }

  return null;
}

function formatTokenTotal(generation: GenerationRecord): string {
  if (!generation.tokenUsage) {
    return "tokens not recorded";
  }

  const total =
    tokenCount(generation.tokenUsage.totalTokens) ||
    tokenCount(generation.tokenUsage.promptTokens) +
      tokenCount(generation.tokenUsage.completionTokens);
  return `${formatTokenCount(total)} tokens`;
}

function formatTokenCount(value: number | null | undefined): string {
  if (typeof value !== "number" || !Number.isFinite(value)) {
    return "not returned";
  }

  return new Intl.NumberFormat("en-US").format(value);
}
