import {
  mkdir,
  readFile,
  readdir,
  rename,
  stat,
  writeFile,
} from "node:fs/promises";
import { dirname, join } from "node:path";
import {
  decodeVideoCompositionInput,
  type VideoCompositionInput,
} from "@/components/preview-v2/composition/types";
import { assertExactObjectKeys } from "@/core/model-binding";
import type { ExportJob, ExportJobSnapshot } from "./types";

export class ExportRepository {
  readonly exportsRoot: string;

  constructor(dataDir = "kivdb") {
    this.exportsRoot = join(dataDir, "exports");
  }

  jobDir(gameId: string, jobId: string): string {
    assertIdentifier(gameId);
    assertIdentifier(jobId);
    return join(this.exportsRoot, gameId, jobId);
  }

  async create(snapshot: ExportJobSnapshot): Promise<void> {
    const job = decodeJob(snapshot.job);
    const composition = decodeVideoCompositionInput(snapshot.composition);
    const directory = this.jobDir(job.gameId, job.jobId);
    await mkdir(join(directory, "assets"), { recursive: true });
    await atomicJson(join(directory, "input.json"), composition);
    await atomicJson(join(directory, "job.json"), job);
  }

  async save(job: ExportJob): Promise<void> {
    const validated = decodeJob(job);
    await atomicJson(
      join(this.jobDir(validated.gameId, validated.jobId), "job.json"),
      validated,
    );
  }

  async get(gameId: string, jobId: string): Promise<ExportJobSnapshot | null> {
    const directory = this.jobDir(gameId, jobId);
    try {
      const [jobRaw, inputRaw] = await Promise.all([
        readFile(join(directory, "job.json"), "utf8"),
        readFile(join(directory, "input.json"), "utf8"),
      ]);
      return {
        job: decodeJob(JSON.parse(jobRaw)),
        composition: decodeVideoCompositionInput(JSON.parse(inputRaw)),
      };
    } catch (error) {
      if (isNotFound(error)) {
        return null;
      }
      throw error;
    }
  }

  private async getJob(gameId: string, jobId: string): Promise<ExportJob | null> {
    const directory = this.jobDir(gameId, jobId);
    try {
      return decodeJob(
        JSON.parse(await readFile(join(directory, "job.json"), "utf8")),
      );
    } catch (error) {
      if (isNotFound(error)) {
        return null;
      }
      throw error;
    }
  }

  async find(jobId: string): Promise<ExportJobSnapshot | null> {
    assertIdentifier(jobId);
    let gameDirs: string[];
    try {
      gameDirs = await readdir(this.exportsRoot);
    } catch (error) {
      if (isNotFound(error)) {
        return null;
      }
      throw error;
    }
    for (const gameId of gameDirs) {
      if (!isIdentifier(gameId)) {
        continue;
      }
      const snapshot = await this.get(gameId, jobId);
      if (snapshot) {
        return snapshot;
      }
    }
    return null;
  }

  async list(gameId: string): Promise<readonly ExportJob[]> {
    assertIdentifier(gameId);
    const gameRoot = join(this.exportsRoot, gameId);
    let entries: string[];
    try {
      entries = await readdir(gameRoot);
    } catch (error) {
      if (isNotFound(error)) {
        return [];
      }
      throw error;
    }
    const jobs = await Promise.all(
      entries
        .filter(isIdentifier)
        .map((jobId) => this.getJob(gameId, jobId)),
    );
    return jobs
      .filter((job): job is ExportJob => job !== null)
      .sort((left, right) => right.createdAt.localeCompare(left.createdAt));
  }

  async outputSize(job: ExportJob): Promise<number> {
    const info = await stat(
      join(this.jobDir(job.gameId, job.jobId), "output.mp4"),
    );
    return info.size;
  }
}

export function assertIdentifier(value: string): void {
  if (!isIdentifier(value)) {
    throw new Error("Invalid export identifier.");
  }
}

function isIdentifier(value: string): boolean {
  return /^[a-zA-Z0-9_-]{1,128}$/.test(value);
}

function decodeJob(value: unknown): ExportJob {
  if (!isRecord(value)) {
    throw new Error("Invalid export job record.");
  }
  if (value.schemaVersion !== 1) {
    throw new Error("Unsupported export job schema version.");
  }
  assertExactObjectKeys(value, "Export job", [
    "schemaVersion",
    "jobId",
    "gameId",
    "gameTitle",
    "status",
    "stage",
    "progress",
    "createdAt",
    "startedAt",
    "completedAt",
    "retryOfJobId",
    "warnings",
    "error",
    "output",
  ]);
  if (
    typeof value.jobId !== "string" ||
    !isIdentifier(value.jobId) ||
    typeof value.gameId !== "string" ||
    !isIdentifier(value.gameId) ||
    typeof value.gameTitle !== "string" ||
    !isJobStatus(value.status) ||
    !isJobStage(value.stage) ||
    typeof value.progress !== "number" ||
    !Number.isFinite(value.progress) ||
    value.progress < 0 ||
    value.progress > 1 ||
    typeof value.createdAt !== "string" ||
    !isNullableString(value.startedAt) ||
    !isNullableString(value.completedAt) ||
    !isNullableString(value.retryOfJobId) ||
    !Array.isArray(value.warnings) ||
    !value.warnings.every((warning) => typeof warning === "string") ||
    !isJobError(value.error) ||
    !isJobOutput(value.output)
  ) {
    throw new Error("Invalid export job record.");
  }
  return value as ExportJob;
}

function isJobStage(value: unknown): boolean {
  return (
    value === "queued" ||
    value === "preparing" ||
    value === "rendering" ||
    value === "encoding" ||
    value === "muxing" ||
    value === "completed" ||
    value === "failed" ||
    value === "canceled" ||
    value === "interrupted"
  );
}

function isJobStatus(value: unknown): boolean {
  return (
    value === "queued" ||
    value === "preparing" ||
    value === "rendering" ||
    value === "completed" ||
    value === "failed" ||
    value === "canceled" ||
    value === "interrupted"
  );
}

function isJobError(value: unknown): boolean {
  return (
    value === null ||
    (isRecord(value) &&
      hasExactKeys(value, ["code", "message"]) &&
      typeof value.code === "string" &&
      typeof value.message === "string")
  );
}

function isJobOutput(value: unknown): boolean {
  return (
    value === null ||
    (isRecord(value) &&
      hasExactKeys(value, ["fileName", "sizeBytes", "durationMs"]) &&
      typeof value.fileName === "string" &&
      typeof value.sizeBytes === "number" &&
      Number.isFinite(value.sizeBytes) &&
      value.sizeBytes >= 0 &&
      typeof value.durationMs === "number" &&
      Number.isFinite(value.durationMs) &&
      value.durationMs > 0)
  );
}

function isNullableString(value: unknown): value is string | null {
  return value === null || typeof value === "string";
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function hasExactKeys(
  value: Record<string, unknown>,
  keys: readonly string[],
): boolean {
  return Object.keys(value).length === keys.length && keys.every((key) => key in value);
}

async function atomicJson(path: string, value: unknown): Promise<void> {
  await mkdir(dirname(path), { recursive: true });
  const temp = path + ".tmp-" + process.pid + "-" + Date.now();
  await writeFile(temp, JSON.stringify(value, null, 2) + "\n", "utf8");
  await rename(temp, path);
}

function isNotFound(error: unknown): boolean {
  return (
    typeof error === "object" &&
    error !== null &&
    "code" in error &&
    error.code === "ENOENT"
  );
}
