import { mkdtemp, rm, writeFile } from "node:fs/promises";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { afterEach, describe, expect, it } from "vitest";
import {
  VIDEO_COMPOSITION_SCHEMA_VERSION,
  type VideoCompositionInput,
} from "@/components/preview-v2/composition/types";
import { legacyGameScriptSnapshot } from "@/core/game-script";
import { ExportRepository } from "./export-repository";
import { getVideoExportService, VideoExportService } from "./export-service";
import {
  transitionExportJob,
  updateExportProgress,
} from "./job-state";
import type { ExportJob } from "./types";

const directories: string[] = [];

afterEach(async () => {
  await Promise.all(
    directories.splice(0).map((directory) =>
      rm(directory, { recursive: true, force: true }),
    ),
  );
});

describe("video export persistence", () => {
  it("enforces the job state machine", () => {
    const preparing = transitionExportJob(job(), {
      status: "preparing",
      now: "2026-07-10T00:00:01.000Z",
    });
    const rendering = transitionExportJob(preparing, { status: "rendering" });
    const progressed = updateExportProgress(rendering, {
      stage: "encoding",
      progress: 0.7,
    });
    const completed = transitionExportJob(progressed, {
      status: "completed",
      output: {
        fileName: "video.mp4",
        sizeBytes: 100,
        durationMs: 1000,
      },
    });

    expect(completed.progress).toBe(1);
    expect(completed.output?.fileName).toBe("video.mp4");
    const canceled = transitionExportJob(job(), { status: "canceled" });
    expect(canceled.stage).toBe("canceled");
    expect(() =>
      transitionExportJob(completed, { status: "rendering" }),
    ).toThrow("Illegal");
  });

  it("round-trips a versioned snapshot atomically", async () => {
    const root = await mkdtemp(join(tmpdir(), "kiva-export-"));
    directories.push(root);
    const repository = new ExportRepository(root);
    const initialJob = job();
    const composition = fixtureComposition();
    await repository.create({ job: initialJob, composition });

    const loaded = await repository.get(initialJob.gameId, initialJob.jobId);
    expect(loaded?.job.status).toBe("queued");
    expect(loaded?.composition.gameTitle).toBe("Test Game");

    const preparing = transitionExportJob(initialJob, { status: "preparing" });
    await repository.save(preparing);
    expect((await repository.list(initialJob.gameId))[0]?.status).toBe(
      "preparing",
    );
  });

  it("lists job metadata without decoding a stale composition snapshot", async () => {
    const root = await mkdtemp(join(tmpdir(), "kiva-export-"));
    directories.push(root);
    const repository = new ExportRepository(root);
    const initialJob = job();
    await repository.create({
      job: initialJob,
      composition: fixtureComposition(),
    });
    await writeFile(
      join(repository.jobDir(initialJob.gameId, initialJob.jobId), "input.json"),
      JSON.stringify({ ...fixtureComposition(), schemaVersion: 2 }),
      "utf8",
    );

    await expect(repository.list(initialJob.gameId)).resolves.toEqual([
      initialJob,
    ]);
    await expect(
      repository.get(initialJob.gameId, initialJob.jobId),
    ).rejects.toThrow("Unsupported video composition schema version");
  });

  it("keeps queued exports queued when the API service initializes", async () => {
    const root = await mkdtemp(join(tmpdir(), "kiva-export-worker-"));
    directories.push(root);
    const repository = new ExportRepository(root);
    const initialJob = job();
    await repository.create({ job: initialJob, composition: fixtureComposition() });

    const service = new VideoExportService(root, { executeInline: false });
    expect((await service.list(initialJob.gameId))[0]?.status).toBe("queued");
    expect((await repository.list(initialJob.gameId))[0]?.status).toBe("queued");
  });

  it("fails a stale queued snapshot without crashing the video worker", async () => {
    const root = await mkdtemp(join(tmpdir(), "kiva-export-stale-worker-"));
    directories.push(root);
    const repository = new ExportRepository(root);
    const initialJob = job();
    await repository.create({ job: initialJob, composition: fixtureComposition() });
    await writeFile(
      join(repository.jobDir(initialJob.gameId, initialJob.jobId), "input.json"),
      JSON.stringify({ ...fixtureComposition(), schemaVersion: 2 }),
      "utf8",
    );

    const service = new VideoExportService(root, { executeInline: false });
    expect(await service.processQueuedOnce()).toBe(true);
    expect((await repository.list(initialJob.gameId))[0]).toMatchObject({
      status: "failed",
      error: { code: "snapshot_invalid" },
    });
  });

  it("rejects path traversal identifiers", async () => {
    const root = await mkdtemp(join(tmpdir(), "kiva-export-"));
    directories.push(root);
    const repository = new ExportRepository(root);
    expect(() => repository.jobDir("../game", "job")).toThrow("Invalid");
  });

  it("rejects malformed persisted job records", async () => {
    const root = await mkdtemp(join(tmpdir(), "kiva-export-"));
    directories.push(root);
    const repository = new ExportRepository(root);
    const initialJob = job();
    await repository.create({
      job: initialJob,
      composition: fixtureComposition(),
    });
    await writeFile(
      join(repository.jobDir(initialJob.gameId, initialJob.jobId), "job.json"),
      JSON.stringify({ ...initialJob, progress: 2 }),
      "utf8",
    );

    await expect(
      repository.get(initialJob.gameId, initialJob.jobId),
    ).rejects.toThrow("Invalid export job record");
  });

  it("shares one export service per data directory", () => {
    expect(getVideoExportService("same-root")).toBe(
      getVideoExportService("same-root"),
    );
    expect(getVideoExportService("same-root")).not.toBe(
      getVideoExportService("other-root"),
    );
  });
});

function job(): ExportJob {
  return {
    schemaVersion: 1,
    jobId: "job-1",
    gameId: "game-1",
    gameTitle: "Test Game",
    status: "queued",
    stage: "queued",
    progress: 0,
    createdAt: "2026-07-10T00:00:00.000Z",
    startedAt: null,
    completedAt: null,
    retryOfJobId: null,
    warnings: [],
    error: null,
    output: null,
  };
}

function fixtureComposition(): VideoCompositionInput {
  return {
    schemaVersion: VIDEO_COMPOSITION_SCHEMA_VERSION,
    gameId: "game-1",
    gameTitle: "Test Game",
    script: legacyGameScriptSnapshot(),
    items: [],
    assets: {
      fontUrl: "/font.ttf",
      dayBackgroundUrl: null,
      nightBackgroundUrl: null,
      avatarUrls: {},
    },
    audioCues: [],
  };
}
