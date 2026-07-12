"use client";

import { Loader2, Mic2, RefreshCw, RotateCcw, X } from "lucide-react";
import { useRouter } from "next/navigation";
import { useCallback, useEffect, useRef, useState } from "react";
import { Button } from "@/components/ui/button";
import {
  isActiveVoiceJob,
  type VoiceJobProjection,
} from "@/server/voice-jobs/types";

export function VoicePanel({
  gameId,
  missingCount,
}: {
  readonly gameId: string;
  readonly missingCount: number;
}) {
  const router = useRouter();
  const [jobs, setJobs] = useState<readonly VoiceJobProjection[]>([]);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const artifactProgress = useRef<VoiceArtifactProgress | null>(null);
  const active = jobs.some((job) => isActiveVoiceJob(job.status));
  const pollIntervalMs = active ? 30_000 : null;

  const refresh = useCallback(async () => {
    const response = await fetch(`/api/games/${encodeURIComponent(gameId)}/voice-jobs`, {
      cache: "no-store",
    });
    const body = (await response.json()) as {
      jobs?: readonly VoiceJobProjection[];
      error?: { message?: string };
    };
    if (!response.ok) throw new Error(body.error?.message ?? "无法读取配音任务");
    const nextJobs = body.jobs ?? [];
    const nextProgress = voiceArtifactProgress(nextJobs);
    if (shouldRefreshComposition(artifactProgress.current, nextProgress)) {
      router.refresh();
    }
    artifactProgress.current = nextProgress;
    setJobs(nextJobs);
  }, [gameId, router]);

  useEffect(() => {
    refresh().catch((caught) => setError(messageOf(caught)));
  }, [refresh]);

  useEffect(() => {
    if (pollIntervalMs === null) return;
    const timer = window.setInterval(() => {
      refresh().catch((caught) => setError(messageOf(caught)));
    }, pollIntervalMs);
    return () => window.clearInterval(timer);
  }, [pollIntervalMs, refresh]);

  async function request(path = "") {
    setBusy(true);
    setError(null);
    try {
      const response = await fetch(
        `/api/games/${encodeURIComponent(gameId)}/voice-jobs${path}`,
        { method: "POST" },
      );
      const body = (await response.json()) as { error?: { message?: string } };
      if (!response.ok) throw new Error(body.error?.message ?? "配音任务请求失败");
      await refresh();
    } catch (caught) {
      setError(messageOf(caught));
    } finally {
      setBusy(false);
    }
  }

  return (
    <section className="grid gap-3 rounded-lg border border-border bg-surface/45 p-4">
      <div className="flex items-center justify-between gap-3">
        <div>
          <div className="text-xs font-semibold uppercase tracking-[0.16em] text-foreground">
            Player voices
          </div>
          <div className="mt-1 text-xs text-subtle">
            {missingCount === 0 ? "玩家配音已就绪" : `还有 ${missingCount} 条待生成`}
          </div>
        </div>
        <div className="flex gap-2">
          <Button
            aria-label="刷新配音任务"
            buttonStyle="icon"
            disabled={busy}
            iconSize="md"
            onClick={() => refresh().catch((caught) => setError(messageOf(caught)))}
            title="刷新"
          >
            <RefreshCw size={16} />
          </Button>
          <Button
            aria-label="生成全部玩家配音"
            buttonStyle="icon"
            disabled={busy || active || missingCount === 0}
            iconSize="md"
            onClick={() => request()}
            title="生成全部玩家配音"
          >
            {busy || active ? <Loader2 className="animate-spin" size={16} /> : <Mic2 size={17} />}
          </Button>
        </div>
      </div>
      {error ? <div className="rounded-md bg-danger-badge px-3 py-2 text-xs text-danger-badge-foreground">{error}</div> : null}
      {jobs[0] ? (
        <div className="rounded-md border border-border bg-surface px-3 py-2 text-xs">
          <div className="flex items-center justify-between gap-3">
            <span className="font-semibold uppercase tracking-[0.1em]">{jobs[0].status}</span>
            <span className="font-mono text-subtle">{Math.round(jobs[0].progress * 100)}%</span>
          </div>
          <div className="mt-2 h-1.5 overflow-hidden rounded-full bg-surface-strong">
            <div className="h-full bg-accent" style={{ width: `${Math.round(jobs[0].progress * 100)}%` }} />
          </div>
          <div className="mt-2 flex justify-end gap-2">
            {isActiveVoiceJob(jobs[0].status) ? (
              <Button aria-label="取消配音任务" buttonStyle="icon" iconSize="sm" onClick={() => request(`/${jobs[0]!.jobId}/cancel`)} title="取消">
                <X size={14} />
              </Button>
            ) : null}
            {["failed", "canceled", "interrupted"].includes(jobs[0].status) ? (
              <Button aria-label="重试配音任务" buttonStyle="icon" iconSize="sm" onClick={() => request(`/${jobs[0]!.jobId}/retry`)} title="重试">
                <RotateCcw size={14} />
              </Button>
            ) : null}
          </div>
          {jobs[0].items.filter((item) => item.error).map((item) => (
            <div className="mt-2 text-danger-badge-foreground" key={item.eventId}>
              {item.label}: {item.error}
            </div>
          ))}
        </div>
      ) : null}
    </section>
  );
}

function messageOf(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

type VoiceArtifactProgress = {
  readonly jobId: string;
  readonly completedItems: number;
};

export function voiceArtifactProgress(
  jobs: readonly VoiceJobProjection[],
): VoiceArtifactProgress | null {
  const latest = jobs[0];
  if (!latest) return null;
  return {
    jobId: latest.jobId,
    completedItems: latest.items.filter((item) =>
      item.status === "completed" || item.status === "skipped").length,
  };
}

export function shouldRefreshComposition(
  previous: VoiceArtifactProgress | null,
  next: VoiceArtifactProgress | null,
): boolean {
  return previous !== null &&
    next !== null &&
    previous.jobId === next.jobId &&
    next.completedItems > previous.completedItems;
}
