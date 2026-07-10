import type {
  ExportJob,
  ExportJobStage,
  ExportJobStatus,
} from "./types";

const allowedTransitions: Readonly<
  Record<ExportJobStatus, readonly ExportJobStatus[]>
> = {
  queued: ["preparing", "canceled", "interrupted", "failed"],
  preparing: ["rendering", "canceled", "interrupted", "failed"],
  rendering: ["completed", "canceled", "interrupted", "failed"],
  completed: [],
  failed: [],
  canceled: [],
  interrupted: [],
};

export function transitionExportJob(
  job: ExportJob,
  update: {
    readonly status: ExportJobStatus;
    readonly stage?: ExportJobStage;
    readonly progress?: number;
    readonly error?: ExportJob["error"];
    readonly output?: ExportJob["output"];
    readonly now?: string;
  },
): ExportJob {
  if (!allowedTransitions[job.status].includes(update.status)) {
    throw new Error(
      "Illegal export job transition: " + job.status + " -> " + update.status,
    );
  }
  const now = update.now ?? new Date().toISOString();
  return {
    ...job,
    status: update.status,
    stage: update.stage ?? stageForStatus(update.status),
    progress: clampProgress(
      update.progress ?? (update.status === "completed" ? 1 : job.progress),
    ),
    startedAt:
      job.startedAt ?? (update.status === "preparing" ? now : job.startedAt),
    completedAt: isFinal(update.status) ? now : job.completedAt,
    error: update.error ?? job.error,
    output: update.output ?? job.output,
  };
}

export function updateExportProgress(
  job: ExportJob,
  input: {
    readonly stage: ExportJobStage;
    readonly progress: number;
  },
): ExportJob {
  if (job.status !== "rendering") {
    return job;
  }
  return {
    ...job,
    stage: input.stage,
    progress: Math.max(job.progress, clampProgress(input.progress)),
  };
}

function stageForStatus(status: ExportJobStatus): ExportJobStage {
  switch (status) {
    case "queued":
      return "queued";
    case "preparing":
      return "preparing";
    case "rendering":
      return "rendering";
    case "completed":
      return "completed";
    case "failed":
      return "failed";
    case "canceled":
      return "canceled";
    case "interrupted":
      return "interrupted";
  }
}

function isFinal(status: ExportJobStatus): boolean {
  return (
    status === "completed" ||
    status === "failed" ||
    status === "canceled" ||
    status === "interrupted"
  );
}

function clampProgress(progress: number): number {
  return Math.min(1, Math.max(0, progress));
}
