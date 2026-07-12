export type VoiceJobStatus =
  | "queued"
  | "preparing"
  | "generating"
  | "completed"
  | "failed"
  | "canceled"
  | "interrupted";

export type VoiceJobItem = {
  readonly eventId: string;
  readonly playerId: string;
  readonly label: string;
  readonly status: "queued" | "generating" | "completed" | "failed" | "skipped";
  readonly error: string | null;
};

export type VoiceJob = {
  readonly schemaVersion: 1;
  readonly jobId: string;
  readonly gameId: string;
  readonly status: VoiceJobStatus;
  readonly progress: number;
  readonly createdAt: string;
  readonly startedAt: string | null;
  readonly completedAt: string | null;
  readonly retryOfJobId: string | null;
  readonly items: readonly VoiceJobItem[];
  readonly error: string | null;
};

export type VoiceJobProjection = VoiceJob;

export function isActiveVoiceJob(status: VoiceJobStatus): boolean {
  return status === "queued" || status === "preparing" || status === "generating";
}
