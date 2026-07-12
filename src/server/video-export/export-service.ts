import { cp, mkdir, rename, rm, stat } from "node:fs/promises";
import { join } from "node:path";
import { randomUUID } from "node:crypto";
import { createGameRepository } from "@/server/game-repository";
import type { VideoCompositionInput } from "@/components/preview-v2/composition/types";
import type { GameId } from "@/core/types";
import { ExportRepository } from "./export-repository";
import {
  transitionExportJob,
  updateExportProgress,
} from "./job-state";
import { renderVideo, type VideoRenderHandle } from "./remotion-renderer";
import { buildExportSnapshot } from "./snapshot-builder";
import {
  isTerminalStatus,
  projectExportJob,
  type ExportJob,
  type ExportJobProjection,
} from "./types";
import { verifyVideoOutput } from "./output-verifier";

export class VideoExportService {
  private readonly repository: ExportRepository;
  private readonly gameRepository;
  private readonly queue: string[] = [];
  private activeJobId: string | null = null;
  private activeHandle: VideoRenderHandle | null = null;
  private readonly cancellationRequests = new Set<string>();
  private initialized: Promise<void> | null = null;
  private readonly executeInline: boolean;

  constructor(
    private readonly dataDir = "kivdb",
    options: { readonly executeInline?: boolean } = {},
  ) {
    this.repository = new ExportRepository(dataDir);
    this.gameRepository = createGameRepository(dataDir);
    this.executeInline = options.executeInline ?? false;
  }

  async create(gameId: string): Promise<ExportJobProjection> {
    await this.initialize();
    const record = await this.gameRepository.get(gameId as GameId);
    if (!record) {
      throw new ExportServiceError("game_not_found", "Game not found.", 404);
    }
    const jobId = randomUUID();
    const job: ExportJob = {
      schemaVersion: 1,
      jobId,
      gameId,
      gameTitle: record.game.title,
      status: "queued",
      stage: "queued",
      progress: 0,
      createdAt: new Date().toISOString(),
      startedAt: null,
      completedAt: null,
      retryOfJobId: null,
      warnings: [],
      error: null,
      output: null,
    };
    const jobDir = this.repository.jobDir(gameId, jobId);
    const built = await buildExportSnapshot({
      dataDir: this.dataDir,
      record,
      job,
      jobDir,
    });
    if (built.composition.items.length === 0) {
      await rm(jobDir, { recursive: true, force: true });
      throw new ExportServiceError(
        "empty_playback",
        "There are no confirmed playback records to export.",
        409,
      );
    }
    const snapshotJob = { ...job, warnings: built.warnings };
    await this.repository.create({
      job: snapshotJob,
      composition: built.composition,
    });
    if (this.executeInline) this.enqueue(jobId);
    return projectExportJob(snapshotJob);
  }

  async list(gameId: string): Promise<readonly ExportJobProjection[]> {
    await this.initialize();
    return (await this.repository.list(gameId)).map(projectExportJob);
  }

  async get(jobId: string): Promise<ExportJobProjection | null> {
    await this.initialize();
    const snapshot = await this.repository.find(jobId);
    return snapshot ? projectExportJob(snapshot.job) : null;
  }

  async cancel(jobId: string): Promise<ExportJobProjection> {
    await this.initialize();
    const snapshot = await this.requireJob(jobId);
    if (isTerminalStatus(snapshot.job.status)) {
      return projectExportJob(snapshot.job);
    }
    if (this.activeJobId === jobId) {
      this.cancellationRequests.add(jobId);
      this.activeHandle?.cancel();
    } else {
      const index = this.queue.indexOf(jobId);
      if (index >= 0) {
        this.queue.splice(index, 1);
      }
      const canceled = transitionExportJob(snapshot.job, { status: "canceled" });
      await this.repository.save(canceled);
      return projectExportJob(canceled);
    }
    return projectExportJob(snapshot.job);
  }

  async retry(jobId: string): Promise<ExportJobProjection> {
    await this.initialize();
    const source = await this.requireJob(jobId);
    if (!isTerminalStatus(source.job.status) || source.job.status === "completed") {
      throw new ExportServiceError(
        "job_not_retriable",
        "Only failed, canceled, or interrupted jobs can be retried.",
        409,
      );
    }
    const nextId = randomUUID();
    const now = new Date().toISOString();
    const nextJob: ExportJob = {
      ...source.job,
      jobId: nextId,
      status: "queued",
      stage: "queued",
      progress: 0,
      createdAt: now,
      startedAt: null,
      completedAt: null,
      retryOfJobId: source.job.jobId,
      error: null,
      output: null,
    };
    const sourceDir = this.repository.jobDir(source.job.gameId, source.job.jobId);
    const nextDir = this.repository.jobDir(source.job.gameId, nextId);
    await mkdir(nextDir, { recursive: true });
    await cp(join(sourceDir, "assets"), join(nextDir, "assets"), {
      recursive: true,
    });
    await this.repository.create({
      job: nextJob,
      composition: replaceJobAssetId(
        source.composition,
        source.job.jobId,
        nextId,
      ),
    });
    if (this.executeInline) this.enqueue(nextId);
    return projectExportJob(nextJob);
  }

  async processQueuedOnce(): Promise<boolean> {
    await this.initialize();
    const candidates: ExportJob[] = [];
    let gameIds: string[];
    try {
      gameIds = await (await import("node:fs/promises")).readdir(
        this.repository.exportsRoot,
      );
    } catch {
      return false;
    }
    for (const gameId of gameIds) {
      if (!/^[a-zA-Z0-9_-]{1,128}$/.test(gameId)) continue;
      candidates.push(
        ...(await this.repository.list(gameId)).filter(
          (job) => job.status === "queued",
        ),
      );
    }
    candidates.sort((left, right) => left.createdAt.localeCompare(right.createdAt));
    const next = candidates[0];
    if (!next) return false;
    try {
      await this.run(next.jobId);
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      await this.repository.save(
        transitionExportJob(next, {
          status: "failed",
          error: { code: "snapshot_invalid", message: sanitizeError(message) },
        }),
      );
    }
    return true;
  }

  async outputPath(jobId: string): Promise<{
    readonly job: ExportJob;
    readonly path: string;
  }> {
    const snapshot = await this.requireJob(jobId);
    if (snapshot.job.status !== "completed") {
      throw new ExportServiceError(
        "output_not_ready",
        "Export output is not ready.",
        409,
      );
    }
    return {
      job: snapshot.job,
      path: join(
        this.repository.jobDir(snapshot.job.gameId, snapshot.job.jobId),
        "output.mp4",
      ),
    };
  }

  async assetPath(
    jobId: string,
    segments: readonly string[],
  ): Promise<string> {
    const snapshot = await this.requireJob(jobId);
    if (
      segments.length === 0 ||
      segments.some((segment) => !/^[a-zA-Z0-9_.-]+$/.test(segment))
    ) {
      throw new ExportServiceError("invalid_asset", "Invalid asset path.", 400);
    }
    const path = join(
      this.repository.jobDir(snapshot.job.gameId, snapshot.job.jobId),
      "assets",
      ...segments,
    );
    await stat(path);
    return path;
  }

  private async initialize(): Promise<void> {
    if (!this.initialized) {
      this.initialized = this.reconcileInterrupted();
    }
    await this.initialized;
  }

  private async reconcileInterrupted(): Promise<void> {
    let gameIds: string[];
    try {
      gameIds = await (await import("node:fs/promises")).readdir(
        this.repository.exportsRoot,
      );
    } catch {
      return;
    }
    for (const gameId of gameIds) {
      if (!/^[a-zA-Z0-9_-]{1,128}$/.test(gameId)) {
        continue;
      }
      for (const job of await this.repository.list(gameId)) {
        if (
          job.status === "preparing" ||
          job.status === "rendering"
        ) {
          await this.repository.save(
            transitionExportJob(job, { status: "interrupted" }),
          );
        }
      }
    }
  }

  private enqueue(jobId: string): void {
    if (!this.queue.includes(jobId) && this.activeJobId !== jobId) {
      this.queue.push(jobId);
    }
    void this.drain();
  }

  private async drain(): Promise<void> {
    if (this.activeJobId) {
      return;
    }
    const jobId = this.queue.shift();
    if (!jobId) {
      return;
    }
    this.activeJobId = jobId;
    try {
      await this.run(jobId);
    } finally {
      this.activeHandle = null;
      this.activeJobId = null;
      this.cancellationRequests.delete(jobId);
      void this.drain();
    }
  }

  private async run(jobId: string): Promise<void> {
    let snapshot = await this.requireJob(jobId);
    let current = transitionExportJob(snapshot.job, { status: "preparing" });
    await this.repository.save(current);
    const jobDir = this.repository.jobDir(current.gameId, current.jobId);
    const partial = join(jobDir, "output.partial.mp4");
    const output = join(jobDir, "output.mp4");
    await rm(partial, { force: true });
    let progressWrite = Promise.resolve();

    try {
      current = transitionExportJob(current, {
        status: "rendering",
        progress: 0.01,
      });
      await this.repository.save(current);
      const composition = absolutizeComposition(
        snapshot.composition,
        renderOrigin(),
      );
      let lastSavedAt = 0;
      this.activeHandle = renderVideo({
        composition,
        outputPath: partial,
        onProgress: ({ progress, stage }) => {
          const now = Date.now();
          current = updateExportProgress(current, {
            progress,
            stage,
          });
          if (now - lastSavedAt > 500 || progress >= 1) {
            lastSavedAt = now;
            const progressSnapshot = current;
            progressWrite = progressWrite.then(() =>
              this.repository.save(progressSnapshot),
            );
          }
        },
      });
      if (this.cancellationRequests.has(jobId)) {
        this.activeHandle.cancel();
      }
      const cancellationPoll = setInterval(() => {
        void this.repository.find(jobId).then((latest) => {
          if (latest?.job.status === "canceled") this.activeHandle?.cancel();
        });
      }, 500);
      try {
        await this.activeHandle.promise;
      } finally {
        clearInterval(cancellationPoll);
      }
      await progressWrite;
      const persisted = await this.repository.find(jobId);
      if (persisted?.job.status === "canceled") {
        await rm(partial, { force: true });
        return;
      }
      const verified = await verifyVideoOutput(partial, {
        expectAudio: composition.audioCues.length > 0,
      });
      await rename(partial, output);
      const size = (await stat(output)).size;
      current = transitionExportJob(current, {
        status: "completed",
        output: {
          fileName: exportFileName(current),
          sizeBytes: size,
          durationMs: verified.durationMs,
        },
      });
      await this.repository.save(current);
    } catch (error) {
      await rm(partial, { force: true });
      await progressWrite;
      snapshot = await this.requireJob(jobId);
      if (snapshot.job.status === "canceled") {
        return;
      }
      const message = error instanceof Error ? error.message : String(error);
      const canceled =
        this.cancellationRequests.delete(jobId) || /cancel/i.test(message);
      current = transitionExportJob(current, {
        status: canceled ? "canceled" : "failed",
        error: canceled
          ? null
          : { code: "render_failed", message: sanitizeError(message) },
      });
      await this.repository.save(current);
    }
  }

  private async requireJob(jobId: string) {
    const snapshot = await this.repository.find(jobId);
    if (!snapshot) {
      throw new ExportServiceError("job_not_found", "Export job not found.", 404);
    }
    return snapshot;
  }
}

export class ExportServiceError extends Error {
  constructor(
    readonly code: string,
    message: string,
    readonly status: number,
  ) {
    super(message);
  }
}

const globalServices = globalThis as typeof globalThis & {
  __kivaVideoExportServices?: Map<string, VideoExportService>;
};
const services =
  globalServices.__kivaVideoExportServices ??
  (globalServices.__kivaVideoExportServices = new Map());

export function getVideoExportService(
  dataDir = process.env.KIVA_DATA_DIR ?? "kivdb",
): VideoExportService {
  let service = services.get(dataDir);
  if (!service) {
    service = new VideoExportService(dataDir, { executeInline: false });
    services.set(dataDir, service);
  }
  return service;
}

function renderOrigin(): string {
  return (process.env.KIVA_RENDER_ORIGIN ?? "http://127.0.0.1:9090").replace(
    /\/+$/,
    "",
  );
}

function absolutizeComposition(
  composition: VideoCompositionInput,
  origin: string,
): VideoCompositionInput {
  const absolute = (source: string) =>
    source.startsWith("/") ? new URL(source, origin).toString() : source;
  return {
    ...composition,
    assets: {
      fontUrl: absolute(composition.assets.fontUrl),
      dayBackgroundUrl: composition.assets.dayBackgroundUrl
        ? absolute(composition.assets.dayBackgroundUrl)
        : null,
      nightBackgroundUrl: composition.assets.nightBackgroundUrl
        ? absolute(composition.assets.nightBackgroundUrl)
        : null,
      avatarUrls: Object.fromEntries(
        Object.entries(composition.assets.avatarUrls).map(([key, value]) => [
          key,
          absolute(value),
        ]),
      ),
    },
    audioCues: composition.audioCues.map((cue) => ({
      ...cue,
      src: absolute(cue.src),
    })),
  };
}

function replaceJobAssetId(
  composition: VideoCompositionInput,
  oldId: string,
  newId: string,
): VideoCompositionInput {
  const replace = (source: string) => source.replace(oldId, newId);
  return {
    ...composition,
    assets: {
      fontUrl: replace(composition.assets.fontUrl),
      dayBackgroundUrl: composition.assets.dayBackgroundUrl
        ? replace(composition.assets.dayBackgroundUrl)
        : null,
      nightBackgroundUrl: composition.assets.nightBackgroundUrl
        ? replace(composition.assets.nightBackgroundUrl)
        : null,
      avatarUrls: Object.fromEntries(
        Object.entries(composition.assets.avatarUrls).map(([key, value]) => [
          key,
          replace(value),
        ]),
      ),
    },
    audioCues: composition.audioCues.map((cue) => ({
      ...cue,
      src: replace(cue.src),
    })),
  };
}

function exportFileName(job: ExportJob): string {
  return (
    "kiva-" +
    job.gameId +
    "-" +
    job.createdAt.replace(/[:.]/g, "-") +
    ".mp4"
  );
}

function sanitizeError(message: string): string {
  return message.replaceAll(process.cwd(), "<project>").slice(0, 1000);
}
