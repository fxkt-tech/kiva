"use client";

import { useId, useRef } from "react";
import type { ReactNode } from "react";
import type { GenerationRecord } from "@/core/generation-record";

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
        className="rounded-md border border-zinc-700 px-2 py-1 text-xs font-medium text-zinc-300 transition hover:border-zinc-500 hover:text-white"
      >
        LLM details
      </button>
      <dialog
        ref={dialogRef}
        aria-labelledby={titleId}
        className="max-h-[82vh] w-[min(920px,calc(100vw-32px))] overflow-hidden rounded-lg border border-zinc-700 bg-zinc-950 p-0 text-zinc-100 backdrop:bg-black/70"
      >
        <div className="flex items-center justify-between gap-3 border-b border-zinc-800 px-4 py-3">
          <div>
            <h3 id={titleId} className="text-sm font-semibold">
              LLM details
            </h3>
            <p className="mt-1 text-xs text-zinc-500">
              {generation.status} · {generation.provider}/{generation.model} ·{" "}
              {generation.promptVersion}
            </p>
          </div>
          <form method="dialog">
            <button
              type="submit"
              className="rounded-md border border-zinc-700 px-2 py-1 text-xs text-zinc-300 transition hover:border-zinc-500 hover:text-white"
            >
              Close
            </button>
          </form>
        </div>
        <div className="max-h-[calc(82vh-65px)] space-y-4 overflow-y-auto p-4">
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
              <p className="text-sm text-zinc-500">
                This generation was created before request snapshots were recorded.
              </p>
            )}
          </GenerationBlock>

          <GenerationBlock title="Reasoning">
            <pre className="whitespace-pre-wrap break-words text-xs leading-5 text-zinc-300">
              {reasoning ??
                "No explicit reasoning field was returned by the model. Hidden model reasoning is not available."}
            </pre>
          </GenerationBlock>

          <GenerationBlock title="Raw output">
            <pre className="whitespace-pre-wrap break-words text-xs leading-5 text-zinc-300">
              {generation.rawOutput ?? "No raw output was recorded."}
            </pre>
          </GenerationBlock>

          <GenerationBlock title="Parsed output">
            <pre className="whitespace-pre-wrap break-words text-xs leading-5 text-zinc-300">
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
    <section className="rounded-md border border-zinc-800 bg-zinc-900/55">
      <div className="border-b border-zinc-800 px-3 py-2 text-xs font-medium uppercase tracking-[0.14em] text-zinc-500">
        {title}
      </div>
      <div className="p-3">{children}</div>
    </section>
  );
}

function KeyValue({ label, value }: { readonly label: string; readonly value: string }) {
  return (
    <div className="grid gap-1 text-xs sm:grid-cols-[120px_minmax(0,1fr)]">
      <div className="text-zinc-500">{label}</div>
      <div className="break-words text-zinc-300">{value}</div>
    </div>
  );
}

function TextDump({ label, value }: { readonly label: string; readonly value: string }) {
  return (
    <div>
      <div className="mb-1 text-xs text-zinc-500">{label}</div>
      <pre className="whitespace-pre-wrap break-words rounded-md bg-zinc-950 p-3 text-xs leading-5 text-zinc-300">
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
