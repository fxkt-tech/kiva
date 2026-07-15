"use client";

import {
  Check,
  ChevronRight,
  ChevronsDown,
  ChevronsUp,
  Copy,
  FileSearch,
  Film,
} from "lucide-react";
import { useState } from "react";
import type { ReactNode } from "react";
import {
  generationMarkdown,
  LlmGenerationDetailsContent,
} from "@/components/editor/llm-generation-details";
import { Button } from "@/components/ui/button";
import type { EpisodeAuthorRequestRecord } from "@/core/episode-script";
import { tokenCount } from "@/core/token-usage";

export function EpisodeAuthorRequestWorkspace({
  requests,
}: {
  readonly requests: readonly EpisodeAuthorRequestRecord[];
}) {
  const labels = episodeAuthorRequestLabels(requests);
  const entries = requests
    .map((request, index) => ({
      request,
      label: labels[index]!,
      sequence: index + 1,
    }))
    .reverse();
  const [selectedId, setSelectedId] = useState<string | null>(
    () => entries[0]?.request.id ?? null,
  );
  const [copyStatus, setCopyStatus] = useState<"idle" | "copied" | "error">(
    "idle",
  );
  const [allExpanded, setAllExpanded] = useState(true);
  const [expansionRevision, setExpansionRevision] = useState(0);
  const selected =
    entries.find(({ request }) => request.id === selectedId) ?? entries[0] ?? null;

  async function copySelectedRequest() {
    if (!selected) return;
    try {
      await navigator.clipboard.writeText(generationMarkdown(selected.request));
      setCopyStatus("copied");
    } catch {
      setCopyStatus("error");
    }
  }

  return (
    <>
      <section className="flex min-h-0 flex-col overflow-hidden rounded-lg border border-border bg-surface/45">
        <header className="flex h-12 shrink-0 items-center justify-between gap-3 border-b border-border px-3">
          <h2 className="text-sm font-semibold">LLM requests</h2>
          <span className="font-mono text-[10px] uppercase tracking-[0.08em] text-subtle">
            {requests.length} · {formatRequestTokens(requests)} tokens
          </span>
        </header>
        {entries.length === 0 ? (
          <EmptyPanel
            icon={<Film aria-hidden="true" className="h-5 w-5" />}
            copy="开始生成后，请求记录会按执行顺序出现在这里。"
          />
        ) : (
          <div className="min-h-0 flex-1 overflow-y-auto overscroll-contain">
            <div className="divide-y divide-border">
              {entries.map(({ request, label, sequence }) => {
                const selectedRequest = request.id === selected?.request.id;
                return (
                  <button
                    key={request.id}
                    type="button"
                    aria-label={`${label} request`}
                    aria-pressed={selectedRequest}
                    onClick={() => {
                      setSelectedId(request.id);
                      setCopyStatus("idle");
                      setAllExpanded(true);
                      setExpansionRevision((revision) => revision + 1);
                    }}
                    className={`flex w-full min-w-0 items-center gap-2.5 px-3 py-2.5 text-left text-xs transition-colors ${
                      selectedRequest
                        ? "bg-accent/10 shadow-[inset_2px_0_0_var(--color-accent)]"
                        : "hover:bg-surface-muted/35"
                    }`}
                  >
                    <span className="shrink-0 font-mono text-[10px] text-subtle">
                      {String(sequence).padStart(2, "0")}
                    </span>
                    <span
                      aria-hidden="true"
                      className={
                        request.status === "success"
                          ? "h-1.5 w-1.5 shrink-0 rounded-full bg-good-badge-foreground"
                          : "h-1.5 w-1.5 shrink-0 rounded-full bg-danger-badge-foreground"
                      }
                    />
                    <span className="min-w-0 flex-1 truncate font-medium text-foreground">
                      {label}
                      <span className="sr-only"> · {request.status}</span>
                    </span>
                    <ChevronRight
                      aria-hidden="true"
                      className={`h-3.5 w-3.5 shrink-0 ${selectedRequest ? "text-accent" : "text-subtle"}`}
                    />
                  </button>
                );
              })}
            </div>
          </div>
        )}
      </section>

      <section className="flex min-h-0 flex-col overflow-hidden rounded-lg border border-border bg-surface/45">
        <header className="flex h-12 shrink-0 items-center justify-between gap-3 border-b border-border px-3">
          <h2 className="text-sm font-semibold">LLM Details</h2>
          <span className="font-mono text-[10px] uppercase tracking-[0.08em] text-subtle">
            {selected ? `Request ${String(selected.sequence).padStart(2, "0")}` : "No selection"}
          </span>
        </header>
        {selected ? (
          <>
            <div className="shrink-0 border-b border-border px-3 py-2.5">
              <div className="flex items-start justify-between gap-3">
                <div className="min-w-0">
                  <h3 className="truncate text-xs font-semibold text-foreground">
                    {selected.label}
                  </h3>
                  <p className="mt-1 truncate font-mono text-[10px] text-subtle">
                    {selected.request.status} · {selected.request.provider}/
                    {selected.request.model} · {selected.request.promptVersion}
                  </p>
                </div>
                <div className="flex shrink-0 items-center gap-2">
                  <Button
                    type="button"
                    onClick={() => {
                      setAllExpanded((expanded) => !expanded);
                      setExpansionRevision((revision) => revision + 1);
                    }}
                    aria-label={
                      allExpanded
                        ? "Collapse all LLM detail sections"
                        : "Expand all LLM detail sections"
                    }
                    title={allExpanded ? "全部折叠" : "全部展开"}
                    className="inline-flex h-7 items-center justify-center gap-1.5 px-2 py-0 text-[11px]"
                  >
                    {allExpanded ? (
                      <ChevronsUp aria-hidden="true" className="h-3.5 w-3.5" />
                    ) : (
                      <ChevronsDown aria-hidden="true" className="h-3.5 w-3.5" />
                    )}
                    {allExpanded ? "全部折叠" : "全部展开"}
                  </Button>
                  <Button
                    type="button"
                    onClick={copySelectedRequest}
                    aria-label="Copy selected LLM details as Markdown"
                    title={
                      copyStatus === "copied"
                        ? "Copied"
                        : copyStatus === "error"
                          ? "Copy failed"
                          : "Copy as Markdown"
                    }
                    buttonStyle="icon"
                    iconSize="xs"
                  >
                    {copyStatus === "copied" ? (
                      <Check aria-hidden="true" className="h-3.5 w-3.5" />
                    ) : (
                      <Copy aria-hidden="true" className="h-3.5 w-3.5" />
                    )}
                  </Button>
                </div>
              </div>
            </div>
            <div className="min-h-0 flex-1 overflow-y-auto overscroll-contain p-3">
              <LlmGenerationDetailsContent
                key={`${selected.request.id}:${expansionRevision}`}
                generation={selected.request}
                showDecisionSummary={false}
                defaultExpanded={allExpanded}
              />
            </div>
          </>
        ) : (
          <EmptyPanel
            icon={<FileSearch aria-hidden="true" className="h-5 w-5" />}
            copy="选择一条请求后，这里会显示 Prompt、Token 和模型输出。"
          />
        )}
      </section>
    </>
  );
}

function EmptyPanel({
  icon,
  copy,
}: {
  readonly icon: ReactNode;
  readonly copy: string;
}) {
  return (
    <div className="grid min-h-0 flex-1 place-items-center p-6 text-center text-subtle">
      <div>
        <div className="flex justify-center">{icon}</div>
        <p className="mt-2 text-xs">{copy}</p>
      </div>
    </div>
  );
}

function episodeAuthorRequestLabels(
  requests: readonly EpisodeAuthorRequestRecord[],
): readonly string[] {
  let run = 0;
  let beatBatch = 0;
  return requests.map((request) => {
    if (request.task.kind === "story") {
      run += 1;
      beatBatch = 0;
      return run === 1 ? "Story spine" : `Story spine · run ${run}`;
    }
    switch (request.task.kind) {
      case "ensemble":
        return "Ensemble map";
      case "actor_arc":
        return `Actor arc · ${request.task.playerId}`;
      case "relationship":
        return `Relationship · ${request.task.playerIds.join("/")}`;
      case "beats":
        beatBatch += 1;
        return run <= 1
          ? `Scene beats ${beatBatch}`
          : `Scene beats ${beatBatch} · run ${run}`;
    }
  });
}

function formatRequestTokens(
  requests: readonly EpisodeAuthorRequestRecord[],
): string {
  const total = requests.reduce((sum, request) => {
    if (!request.tokenUsage) return sum;
    return (
      sum +
      (tokenCount(request.tokenUsage.totalTokens) ||
        tokenCount(request.tokenUsage.promptTokens) +
          tokenCount(request.tokenUsage.completionTokens))
    );
  }, 0);
  return new Intl.NumberFormat("en-US").format(total);
}
