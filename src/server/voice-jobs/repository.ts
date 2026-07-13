import { mkdir, readFile, readdir, rename, writeFile } from "node:fs/promises";
import { dirname, join } from "node:path";
import { assertExactObjectKeys, isPlainObject } from "@/core/model-binding";
import type { VoiceJob, VoiceJobStatus } from "./types";

export class VoiceJobRepository {
  constructor(private readonly dataDir = "kivdb") {}

  private directory(gameId: string): string {
    assertIdentifier(gameId);
    return join(this.dataDir, "games", gameId, "voice-jobs");
  }

  private path(gameId: string, jobId: string): string {
    assertIdentifier(jobId);
    return join(this.directory(gameId), `${jobId}.json`);
  }

  async create(job: VoiceJob): Promise<void> {
    const validated = decodeVoiceJob(job);
    const path = this.path(validated.gameId, validated.jobId);
    try {
      await readFile(path);
      throw new Error(`Voice job already exists: ${job.jobId}`);
    } catch (error) {
      if (!isNotFound(error)) throw error;
    }
    await atomicJson(path, validated);
  }

  async save(job: VoiceJob): Promise<void> {
    const validated = decodeVoiceJob(job);
    await atomicJson(
      this.path(validated.gameId, validated.jobId),
      validated,
    );
  }

  async get(gameId: string, jobId: string): Promise<VoiceJob | null> {
    try {
      return decodeVoiceJob(
        JSON.parse(await readFile(this.path(gameId, jobId), "utf8")),
      );
    } catch (error) {
      if (isNotFound(error)) return null;
      throw error;
    }
  }

  async list(gameId: string): Promise<readonly VoiceJob[]> {
    let files: string[];
    try {
      files = await readdir(this.directory(gameId));
    } catch (error) {
      if (isNotFound(error)) return [];
      throw error;
    }
    const jobs = await Promise.all(
      files
        .filter((file) => file.endsWith(".json"))
        .map((file) => this.get(gameId, file.slice(0, -5))),
    );
    return jobs
      .filter((job): job is VoiceJob => job !== null)
      .sort((a, b) => b.createdAt.localeCompare(a.createdAt));
  }
}

function decodeVoiceJob(value: unknown): VoiceJob {
  if (!isPlainObject(value)) {
    throw new Error("Invalid voice job record");
  }
  if (value.schemaVersion !== 1) {
    throw new Error("Unsupported voice job schema version");
  }
  assertExactObjectKeys(value, "Voice job", [
    "schemaVersion",
    "jobId",
    "gameId",
    "status",
    "progress",
    "createdAt",
    "startedAt",
    "completedAt",
    "retryOfJobId",
    "items",
    "error",
  ]);
  const job = value as VoiceJob;
  if (
    !isIdentifier(job.jobId) ||
    !isIdentifier(job.gameId) ||
    !isStatus(job.status) ||
    !isProgress(job.progress) ||
    typeof job.createdAt !== "string" ||
    !isNullableString(job.startedAt) ||
    !isNullableString(job.completedAt) ||
    !isNullableString(job.retryOfJobId) ||
    !Array.isArray(job.items) ||
    !job.items.every(isVoiceJobItem) ||
    !isNullableString(job.error)
  ) {
    throw new Error("Invalid voice job record");
  }
  return job;
}

function isVoiceJobItem(value: unknown): boolean {
  return (
    isPlainObject(value) &&
    hasExactKeys(value, ["eventId", "playerId", "label", "status", "error"]) &&
    isIdentifier(value.eventId) &&
    isIdentifier(value.playerId) &&
    typeof value.label === "string" &&
    ["queued", "generating", "completed", "failed", "skipped"].includes(
      String(value.status),
    ) &&
    isNullableString(value.error)
  );
}

function isProgress(value: unknown): value is number {
  return (
    typeof value === "number" &&
    Number.isFinite(value) &&
    value >= 0 &&
    value <= 1
  );
}

function isNullableString(value: unknown): value is string | null {
  return value === null || typeof value === "string";
}

function hasExactKeys(
  value: Record<string, unknown>,
  keys: readonly string[],
): boolean {
  return Object.keys(value).length === keys.length && keys.every((key) => key in value);
}

function isStatus(value: unknown): value is VoiceJobStatus {
  return ["queued", "preparing", "generating", "completed", "failed", "canceled", "interrupted"].includes(String(value));
}

function assertIdentifier(value: string): void {
  if (!isIdentifier(value)) throw new Error("Invalid voice job identifier");
}

function isIdentifier(value: unknown): value is string {
  return typeof value === "string" && /^[a-zA-Z0-9_-]{1,160}$/.test(value);
}

async function atomicJson(path: string, value: unknown): Promise<void> {
  await mkdir(dirname(path), { recursive: true });
  const temp = `${path}.tmp-${process.pid}-${Date.now()}`;
  await writeFile(temp, `${JSON.stringify(value, null, 2)}\n`, "utf8");
  await rename(temp, path);
}

function isNotFound(error: unknown): boolean {
  return typeof error === "object" && error !== null && "code" in error && error.code === "ENOENT";
}
