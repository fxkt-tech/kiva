"use client";

import { BarChart3, X } from "lucide-react";
import { useId, useRef } from "react";
import { Button } from "@/components/ui/button";
import { TokenCostCalculator } from "@/components/token-cost-calculator";
import type { TokenUsageSummary } from "@/core/token-usage";

type GameTokenUsageButtonProps = {
  readonly gameTitle: string;
  readonly total: TokenUsageSummary;
  readonly speech: TokenUsageSummary;
  readonly action: TokenUsageSummary;
  readonly script: TokenUsageSummary;
};

export function GameTokenUsageButton({
  gameTitle,
  total,
  speech,
  action,
  script,
}: GameTokenUsageButtonProps) {
  const dialogRef = useRef<HTMLDialogElement>(null);
  const titleId = useId();

  return (
    <>
      <Button
        onClick={() => dialogRef.current?.showModal()}
        buttonStyle="icon"
        iconSize="md"
        aria-label="Show token usage"
        title="Show token usage"
      >
        <BarChart3 aria-hidden="true" className="h-4 w-4" />
      </Button>
      <dialog
        ref={dialogRef}
        aria-labelledby={titleId}
        className="fixed inset-0 m-auto max-h-[82vh] w-[min(560px,calc(100vw-32px))] overflow-hidden rounded-lg border border-interactive-border bg-background p-0 text-foreground backdrop:bg-black/70"
      >
        <div className="flex items-center justify-between gap-3 border-b border-border px-4 py-3">
          <div className="min-w-0">
            <h3 id={titleId} className="truncate text-sm font-semibold">
              Token usage
            </h3>
            <p className="mt-1 truncate text-xs text-subtle">{gameTitle}</p>
          </div>
          <form method="dialog">
            <Button
              type="submit"
              aria-label="Close"
              title="Close"
              buttonStyle="icon"
            >
              <X aria-hidden="true" className="h-4 w-4" />
            </Button>
          </form>
        </div>
        <div className="space-y-3 p-4">
          <TokenCostCalculator
            promptTokens={total.promptTokens}
            reasoningTokens={total.reasoningTokens}
            completionTokens={total.completionTokens}
          />
          <UsageBlock title="Total" summary={total} />
          <div className="grid gap-3 sm:grid-cols-3">
            <UsageBlock title="Speech" summary={speech} compact />
            <UsageBlock title="Action" summary={action} compact />
            <UsageBlock title="Script Author" summary={script} compact />
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
    <section className="rounded-md border border-border bg-surface/55 p-3">
      <div className="mb-2 text-xs font-medium uppercase tracking-[0.14em] text-subtle">
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
      <span className="text-subtle">{label}</span>
      <span className={strong ? "font-semibold text-foreground" : "text-muted"}>
        {formatNumber(value)}
      </span>
    </div>
  );
}

function formatNumber(value: number): string {
  return new Intl.NumberFormat("en-US").format(value);
}
