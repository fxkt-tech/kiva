"use client";

import {
  Download,
  Loader2,
  RefreshCw,
  RotateCcw,
  Video,
  X,
} from "lucide-react";
import { useCallback, useEffect, useState } from "react";
import { Button } from "@/components/ui/button";
import { iconButtonClassName } from "@/components/ui/button-styles";
import type { ExportJobProjection } from "@/server/video-export/types";

export function ExportPanel({
  gameId,
  canExport,
}: {
  readonly gameId: string;
  readonly canExport: boolean;
}) {
  const [jobs, setJobs] = useState<readonly ExportJobProjection[]>([]);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const hasActive = jobs.some((job) =>
    ["queued", "preparing", "rendering"].includes(job.status),
  );

  const refresh = useCallback(async () => {
    const response = await fetch(
      "/api/games/" + encodeURIComponent(gameId) + "/preview-v2/exports",
      { cache: "no-store" },
    );
    const body = (await response.json()) as {
      jobs?: readonly ExportJobProjection[];
      error?: { message?: string };
    };
    if (!response.ok) {
      throw new Error(body.error?.message ?? "Unable to load export jobs.");
    }
    setJobs(body.jobs ?? []);
  }, [gameId]);

  useEffect(() => {
    refresh().catch((caught: unknown) => setError(messageOf(caught)));
  }, [refresh]);

  useEffect(() => {
    if (!hasActive) {
      return;
    }
    const timer = window.setInterval(() => {
      refresh().catch((caught: unknown) => setError(messageOf(caught)));
    }, 1000);
    return () => window.clearInterval(timer);
  }, [hasActive, refresh]);

  async function createExport() {
    setBusy(true);
    setError(null);
    try {
      await request(
        "/api/games/" + encodeURIComponent(gameId) + "/preview-v2/exports",
      );
      await refresh();
    } catch (caught) {
      setError(messageOf(caught));
    } finally {
      setBusy(false);
    }
  }

  async function act(job: ExportJobProjection, action: "cancel" | "retry") {
    setBusy(true);
    setError(null);
    try {
      await request(
        "/api/preview-v2/exports/" +
          encodeURIComponent(job.jobId) +
          "/" +
          action,
      );
      await refresh();
    } catch (caught) {
      setError(messageOf(caught));
    } finally {
      setBusy(false);
    }
  }

  return (
    <section className="grid min-h-0 gap-3 rounded-lg border border-border bg-surface/45 p-4 lg:h-full lg:grid-rows-[auto_auto_minmax(0,1fr)]">
      <div className="flex items-center justify-between gap-4">
        <div>
          <div className="text-xs font-semibold uppercase tracking-[0.16em] text-foreground">
            Render archive
          </div>
          <div className="mt-1 text-sm text-muted">
            已确认事件 · MP4 / H.264 · 所有产物永久保留
          </div>
        </div>
        <div className="flex gap-2">
          <Button
            aria-label="Refresh exports"
            buttonStyle="icon"
            disabled={busy}
            iconSize="md"
            onClick={() =>
              refresh().catch((caught: unknown) => setError(messageOf(caught)))
            }
            title="Refresh exports"
          >
            <RefreshCw size={17} />
          </Button>
          <Button
            aria-label="Generate MP4"
            buttonStyle="icon"
            disabled={!canExport || busy}
            iconSize="md"
            onClick={createExport}
            title="Generate MP4"
          >
            {busy ? (
              <Loader2 className="animate-spin" size={17} />
            ) : (
              <Video size={18} />
            )}
          </Button>
        </div>
      </div>
      {error ? (
        <div className="rounded-md border border-danger-badge-foreground/30 bg-danger-badge px-3 py-2 text-xs text-danger-badge-foreground">
          {error}
        </div>
      ) : null}
      {!canExport ? (
        <div className="text-sm text-muted">
          没有已确认的播放记录，暂时无法生成视频。
        </div>
      ) : null}
      {jobs.length > 0 ? (
        <ol className="grid max-h-72 content-start gap-2 overflow-y-auto lg:max-h-none lg:min-h-0">
          {jobs.map((job) => (
            <li
              className="grid grid-cols-[minmax(0,1fr)_auto] items-center gap-x-3 gap-y-2 rounded-md border border-border bg-surface px-3 py-2"
              key={job.jobId}
            >
              <div className="min-w-0">
                <div className="flex items-center gap-2 text-xs font-semibold uppercase tracking-[0.12em] text-foreground">
                  {activeStatus(job.status) ? (
                    <Loader2 className="animate-spin" size={13} />
                  ) : null}
                  {job.status}
                </div>
                <div className="mt-1 truncate font-mono text-[11px] text-subtle">
                  {job.jobId}
                </div>
                {job.error ? (
                  <div className="mt-1 text-xs text-danger-badge-foreground">
                    {job.error.message}
                  </div>
                ) : null}
              </div>
              <div className="col-span-2 row-start-2">
                <div className="h-1.5 overflow-hidden rounded-full bg-surface-strong">
                  <div
                    className="h-full rounded-full bg-accent"
                    style={{ width: Math.round(job.progress * 100) + "%" }}
                  />
                </div>
                <div className="mt-1 flex justify-between font-mono text-[10px] text-subtle">
                  <span>{job.stage}</span>
                  <span>{Math.round(job.progress * 100)}%</span>
                </div>
              </div>
              <div className="flex gap-2">
                {activeStatus(job.status) ? (
                  <Button
                    aria-label="Cancel export"
                    buttonStyle="icon"
                    disabled={busy}
                    iconSize="sm"
                    onClick={() => act(job, "cancel")}
                    title="Cancel export"
                  >
                    <X size={14} />
                  </Button>
                ) : null}
                {["failed", "canceled", "interrupted"].includes(job.status) ? (
                  <Button
                    aria-label="Retry export"
                    buttonStyle="icon"
                    disabled={busy}
                    iconSize="sm"
                    onClick={() => act(job, "retry")}
                    title="Retry export"
                  >
                    <RotateCcw size={14} />
                  </Button>
                ) : null}
                {job.downloadUrl ? (
                  <a
                    aria-label="Download MP4"
                    className={iconButtonClassName({ size: "sm", variant: "success" })}
                    href={job.downloadUrl}
                    title="Download MP4"
                  >
                    <Download size={14} />
                  </a>
                ) : null}
              </div>
            </li>
          ))}
        </ol>
      ) : (
        <div className="rounded-md border border-dashed border-border px-4 py-5 text-center text-sm text-subtle">
          尚未生成视频
        </div>
      )}
    </section>
  );
}

async function request(url: string): Promise<void> {
  const response = await fetch(url, { method: "POST" });
  const body = (await response.json()) as { error?: { message?: string } };
  if (!response.ok) {
    throw new Error(body.error?.message ?? "Export request failed.");
  }
}

function activeStatus(status: ExportJobProjection["status"]): boolean {
  return status === "queued" || status === "preparing" || status === "rendering";
}

function messageOf(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
