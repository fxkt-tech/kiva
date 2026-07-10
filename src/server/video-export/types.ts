import type { VideoCompositionInput } from "@/components/preview-v2/composition/types";

export type ExportJobStatus =
  | "queued"
  | "preparing"
  | "rendering"
  | "completed"
  | "failed"
  | "canceled"
  | "interrupted";

export type ExportJobStage =
  | "queued"
  | "preparing"
  | "rendering"
  | "encoding"
  | "muxing"
  | "completed"
  | "failed"
  | "canceled"
  | "interrupted";

export type ExportJob = {
  readonly schemaVersion: 1;
  readonly jobId: string;
  readonly gameId: string;
  readonly gameTitle: string;
  readonly status: ExportJobStatus;
  readonly stage: ExportJobStage;
  readonly progress: number;
  readonly createdAt: string;
  readonly startedAt: string | null;
  readonly completedAt: string | null;
  readonly retryOfJobId: string | null;
  readonly warnings: readonly string[];
  readonly error: { readonly code: string; readonly message: string } | null;
  readonly output: {
    readonly fileName: string;
    readonly sizeBytes: number;
    readonly durationMs: number;
  } | null;
};

export type ExportJobSnapshot = {
  readonly job: ExportJob;
  readonly composition: VideoCompositionInput;
};

export type ExportJobProjection = ExportJob & {
  readonly downloadUrl: string | null;
};

export function projectExportJob(job: ExportJob): ExportJobProjection {
  return {
    ...job,
    downloadUrl:
      job.status === "completed"
        ? "/api/preview-v2/exports/" + job.jobId + "/download"
        : null,
  };
}

export function isTerminalStatus(status: ExportJobStatus): boolean {
  return (
    status === "completed" ||
    status === "failed" ||
    status === "canceled" ||
    status === "interrupted"
  );
}
