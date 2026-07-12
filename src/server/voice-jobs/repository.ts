import { mkdir, readFile, readdir, rename, writeFile } from "node:fs/promises";
import { dirname, join } from "node:path";
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
    const path = this.path(job.gameId, job.jobId);
    try {
      await readFile(path);
      throw new Error(`Voice job already exists: ${job.jobId}`);
    } catch (error) {
      if (!isNotFound(error)) throw error;
    }
    await atomicJson(path, job);
  }

  async save(job: VoiceJob): Promise<void> {
    await atomicJson(this.path(job.gameId, job.jobId), job);
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
  const job = value as VoiceJob;
  if (
    !value ||
    typeof value !== "object" ||
    job.schemaVersion !== 1 ||
    !isIdentifier(job.jobId) ||
    !isIdentifier(job.gameId) ||
    !isStatus(job.status) ||
    !Array.isArray(job.items)
  ) {
    throw new Error("Invalid voice job record");
  }
  return job;
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
