"use client";

import { BarChart3, X } from "lucide-react";
import { useId, useRef } from "react";
import type { TokenUsageSummary } from "@/core/token-usage";

type GameTokenUsageButtonProps = {
  readonly gameTitle: string;
  readonly total: TokenUsageSummary;
  readonly speech: TokenUsageSummary;
  readonly action: TokenUsageSummary;
};

export function GameTokenUsageButton({
  gameTitle,
  total,
  speech,
  action,
}: GameTokenUsageButtonProps) {
  const dialogRef = useRef<HTMLDialogElement>(null);
  const titleId = useId();

  return (
    <>
      <button
        type="button"
        onClick={() => dialogRef.current?.showModal()}
        className="inline-flex h-9 w-9 items-center justify-center rounded-md border border-amber-900/80 text-amber-300 transition hover:border-amber-600 hover:text-amber-200"
        aria-label="Show token usage"
        title="Show token usage"
      >
        <BarChart3 aria-hidden="true" className="h-4 w-4" />
      </button>
      <dialog
        ref={dialogRef}
        aria-labelledby={titleId}
        className="fixed inset-0 m-auto max-h-[82vh] w-[min(560px,calc(100vw-32px))] overflow-hidden rounded-lg border border-zinc-700 bg-zinc-950 p-0 text-zinc-100 backdrop:bg-black/70"
      >
        <div className="flex items-center justify-between gap-3 border-b border-zinc-800 px-4 py-3">
          <div className="min-w-0">
            <h3 id={titleId} className="truncate text-sm font-semibold">
              Token usage
            </h3>
            <p className="mt-1 truncate text-xs text-zinc-500">{gameTitle}</p>
          </div>
          <form method="dialog">
            <button
              type="submit"
              aria-label="Close"
              title="Close"
              className="inline-flex h-8 w-8 items-center justify-center rounded-md border border-zinc-700 text-zinc-300 transition hover:border-zinc-500 hover:text-white"
            >
              <X aria-hidden="true" className="h-4 w-4" />
            </button>
          </form>
        </div>
        <div className="space-y-3 p-4">
          <UsageBlock title="Total" summary={total} />
          <div className="grid gap-3 sm:grid-cols-2">
            <UsageBlock title="Speech" summary={speech} compact />
            <UsageBlock title="Action" summary={action} compact />
          </div>
        </div>
      </dialog>
    </>
  );
}

function UsageBlock({
  title,
  summary,
  compact = false,
}: {
  readonly title: string;
  readonly summary: TokenUsageSummary;
  readonly compact?: boolean;
}) {
  return (
    <section className="rounded-md border border-zinc-800 bg-zinc-900/55 p-3">
      <div className="mb-2 text-xs font-medium uppercase tracking-[0.14em] text-zinc-500">
        {title}
      </div>
      <div className={compact ? "space-y-1.5" : "grid gap-2 sm:grid-cols-2"}>
        <Metric label="Total" value={summary.totalTokens} strong />
        <Metric label="Prompt" value={summary.promptTokens} />
        <Metric label="Completion" value={summary.completionTokens} />
        <Metric label="Cached prompt" value={summary.cachedPromptTokens} />
        <Metric label="Reasoning" value={summary.reasoningTokens} />
        <Metric label="Recorded requests" value={summary.recordedGenerations} />
        {summary.unrecordedGenerations > 0 ? (
          <Metric label="Unrecorded requests" value={summary.unrecordedGenerations} />
        ) : null}
      </div>
    </section>
  );
}

function Metric({
  label,
  value,
  strong = false,
}: {
  readonly label: string;
  readonly value: number;
  readonly strong?: boolean;
}) {
  return (
    <div className="flex items-baseline justify-between gap-4 text-xs">
      <span className="text-zinc-500">{label}</span>
      <span className={strong ? "font-semibold text-zinc-100" : "text-zinc-300"}>
        {formatNumber(value)}
      </span>
    </div>
  );
}

function formatNumber(value: number): string {
  return new Intl.NumberFormat("en-US").format(value);
}
