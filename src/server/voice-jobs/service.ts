import { createHash, randomUUID } from "node:crypto";
import { rm } from "node:fs/promises";
import { compilePublicPlayback } from "@/core/playback";
import type { EventId, GameId, PlayerId } from "@/core/types";
import type { PlayerVoiceArtifact } from "@/core/voice";
import { createGameRepository } from "@/server/game-repository";
import { createEdgeVoiceAdapter } from "@/server/voice-synthesis/edge-adapter";
import { aggregateSpeechCues } from "@/server/voice-synthesis/speech-cues";
import type { VoiceSynthesisAdapter } from "@/server/voice-synthesis/types";
import { VoiceJobRepository } from "./repository";
import {
  isActiveVoiceJob,
  type VoiceJob,
  type VoiceJobItem,
  type VoiceJobProjection,
  type VoiceJobStatus,
} from "./types";

export class VoiceJobService {
  private readonly jobs: VoiceJobRepository;
  private readonly games;
  private readonly queue: Array<{ readonly gameId: string; readonly jobId: string }> = [];
  private active: { gameId: string; jobId: string } | null = null;
  private initialized = new Set<string>();
  private readonly canceled = new Set<string>();
  private readonly executeInline: boolean;

  constructor(
    private readonly dataDir = "kivdb",
    private readonly adapter: VoiceSynthesisAdapter = createEdgeVoiceAdapter({
      proxy: process.env.EDGE_TTS_PROXY,
    }),
    options: { readonly executeInline?: boolean } = {},
  ) {
    this.jobs = new VoiceJobRepository(dataDir);
    this.games = createGameRepository(dataDir);
    this.executeInline = options.executeInline ?? false;
  }

  async create(gameId: string): Promise<VoiceJobProjection> {
    await this.initialize(gameId);
    const record = await this.games.get(gameId as GameId);
    if (!record) throw new VoiceJobError("game_not_found", "Game not found", 404);
    const targets = eligibleTargets(record);
    const missing = targets.filter(
      (target) => !(target.eventId in record.voiceArtifactsByEventId),
    );
    if (missing.length === 0) {
      throw new VoiceJobError("voice_complete", "All player voices are already generated", 409);
    }
    const job: VoiceJob = {
      schemaVersion: 1,
      jobId: randomUUID(),
      gameId,
      status: "queued",
      progress: 0,
      createdAt: new Date().toISOString(),
      startedAt: null,
      completedAt: null,
      retryOfJobId: null,
      items: missing.map((target) => ({ ...target, status: "queued", error: null })),
      error: null,
    };
    await this.jobs.create(job);
    if (this.executeInline) this.enqueue(gameId, job.jobId);
    return job;
  }

  async list(gameId: string): Promise<readonly VoiceJobProjection[]> {
    await this.initialize(gameId);
    return this.jobs.list(gameId);
  }

  async cancel(gameId: string, jobId: string): Promise<VoiceJobProjection> {
    const job = await this.require(gameId, jobId);
    if (!isActiveVoiceJob(job.status)) return job;
    this.canceled.add(jobId);
    if (job.status === "queued") {
      const index = this.queue.findIndex((entry) => entry.jobId === jobId);
      if (index >= 0) this.queue.splice(index, 1);
    }
    const canceled = finish(job, "canceled");
    await this.jobs.save(canceled);
    return canceled;
  }

  async retry(gameId: string, jobId: string): Promise<VoiceJobProjection> {
    const source = await this.require(gameId, jobId);
    if (!["failed", "canceled", "interrupted"].includes(source.status)) {
      throw new VoiceJobError("job_not_retriable", "Voice job is not retriable", 409);
    }
    const record = await this.games.get(gameId as GameId);
    if (!record) throw new VoiceJobError("game_not_found", "Game not found", 404);
    const remaining = source.items.filter(
      (item) => !(item.eventId in record.voiceArtifactsByEventId),
    );
    if (remaining.length === 0) {
      throw new VoiceJobError("voice_complete", "All player voices are already generated", 409);
    }
    const next: VoiceJob = {
      ...source,
      jobId: randomUUID(),
      status: "queued",
      progress: 0,
      createdAt: new Date().toISOString(),
      startedAt: null,
      completedAt: null,
      retryOfJobId: source.jobId,
      items: remaining.map((item) => ({ ...item, status: "queued", error: null })),
      error: null,
    };
    await this.jobs.create(next);
    if (this.executeInline) this.enqueue(gameId, next.jobId);
    return next;
  }

  async processQueuedOnce(): Promise<boolean> {
    const candidates: Array<{ gameId: string; job: VoiceJob }> = [];
    for (const record of await this.games.list()) {
      const gameId = record.game.id;
      await this.initialize(gameId);
      for (const job of await this.jobs.list(gameId)) {
        if (job.status === "queued") candidates.push({ gameId, job });
      }
    }
    candidates.sort((left, right) =>
      left.job.createdAt.localeCompare(right.job.createdAt),
    );
    const next = candidates[0];
    if (!next) return false;
    try {
      await this.run(next.gameId, next.job.jobId);
    } catch (error) {
      await this.jobs.save({
        ...finish(next.job, "failed"),
        error: sanitizeVoiceError(error),
      });
    }
    return true;
  }

  private async initialize(gameId: string): Promise<void> {
    if (this.initialized.has(gameId)) return;
    this.initialized.add(gameId);
    for (const job of await this.jobs.list(gameId)) {
      if (job.status === "preparing" || job.status === "generating") {
        await this.jobs.save(finish(job, "interrupted"));
      }
    }
  }

  private enqueue(gameId: string, jobId: string): void {
    this.queue.push({ gameId, jobId });
    void this.drain();
  }

  private async drain(): Promise<void> {
    if (this.active) return;
    const next = this.queue.shift();
    if (!next) return;
    const { gameId, jobId } = next;
    this.active = next;
    try {
      await this.run(gameId, jobId);
    } finally {
      this.active = null;
      this.canceled.delete(jobId);
      void this.drain();
    }
  }

  private async run(gameId: string, jobId: string): Promise<void> {
    let job = await this.require(gameId, jobId);
    job = { ...job, status: "preparing", startedAt: new Date().toISOString() };
    await this.jobs.save(job);
    job = { ...job, status: "generating" };
    await this.jobs.save(job);

    for (let index = 0; index < job.items.length; index += 1) {
      job = await this.require(gameId, jobId);
      if (job.status === "canceled") return;
      if (this.canceled.has(jobId)) {
        job = finish(job, "canceled");
        await this.jobs.save(job);
        return;
      }
      let item: VoiceJobItem = { ...job.items[index]!, status: "generating", error: null };
      job = replaceItem(job, index, item);
      await this.jobs.save(job);
      try {
        const outcome = await this.generateItem(gameId as GameId, item);
        item = { ...item, status: outcome, error: null };
      } catch (error) {
        item = {
          ...item,
          status: "failed",
          error: sanitizeVoiceError(error),
        };
      }
      const persisted = await this.require(gameId, jobId);
      if (persisted.status === "canceled") return;
      job = replaceItem(job, index, item);
      await this.jobs.save(job);
      if (this.canceled.has(jobId)) {
        job = finish(job, "canceled");
        await this.jobs.save(job);
        return;
      }
    }

    const failed = job.items.some((item) => item.status === "failed");
    job = finish(job, failed ? "failed" : "completed");
    await this.jobs.save(job);
  }

  private async generateItem(
    gameId: GameId,
    item: VoiceJobItem,
  ): Promise<"completed" | "skipped"> {
    const record = await this.games.get(gameId);
    if (!record) throw new Error("Game not found");
    if (item.eventId in record.voiceArtifactsByEventId) return "skipped";
    const target = eligibleTargets(record).find((candidate) => candidate.eventId === item.eventId);
    if (!target) throw new Error("Player speech event is no longer eligible");
    const player = record.game.players.find((candidate) => candidate.playerId === target.playerId);
    if (!player) throw new Error("Player for speech event was not found");
    const tempPath = await this.games.voiceTempPath(gameId, item.eventId);
    try {
      const result = await this.adapter.synthesize({
        sourceText: target.text,
        profile: player.voiceProfileSnapshot,
        outputPath: tempPath,
      });
      const cues = aggregateSpeechCues(
        result.wordBoundaries,
        result.durationMs,
      );
      return await this.games.withGameLock(gameId, async () => {
        const latest = await this.games.get(gameId);
        if (!latest) throw new Error("Game not found");
        if (item.eventId in latest.voiceArtifactsByEventId) {
          await rm(tempPath, { force: true });
          return "skipped";
        }
        const file = await this.games.publishVoice(gameId, item.eventId, tempPath);
        const artifact: PlayerVoiceArtifact = {
          eventId: item.eventId as EventId,
          playerId: item.playerId as PlayerId,
          sourceTextHash: createHash("sha256").update(target.text).digest("hex"),
          synthesizedText: result.synthesizedText,
          voiceProfile: player.voiceProfileSnapshot,
          generator: result.generator,
          audio: { file, durationMs: result.durationMs },
          cues,
          createdAt: new Date().toISOString(),
        };
        await this.games.save({
          ...latest,
          voiceArtifactsByEventId: {
            ...latest.voiceArtifactsByEventId,
            [item.eventId]: artifact,
          },
        });
        return "completed";
      });
    } catch (error) {
      await rm(tempPath, { force: true });
      throw error;
    }
  }

  private async require(gameId: string, jobId: string): Promise<VoiceJob> {
    const job = await this.jobs.get(gameId, jobId);
    if (!job) throw new VoiceJobError("job_not_found", "Voice job not found", 404);
    return job;
  }
}

function sanitizeVoiceError(error: unknown): string {
  const message = error instanceof Error ? error.message : String(error);
  return message
    .replaceAll(process.cwd(), "<workspace>")
    .replace(/(?:\/[^\s:]+){2,}/g, "<path>")
    .slice(0, 500);
}

function eligibleTargets(record: Awaited<ReturnType<ReturnType<typeof createGameRepository>["get"]>>) {
  if (!record) return [];
  const items = compilePublicPlayback(record.events, record.game.players, {
    presenter: record.game.presenter,
    audience: "director",
  });
  return items.flatMap((item) => {
    if (item.transcriptSpeaker !== "player") return [];
    const event = record.events.find((candidate) => candidate.index === item.index);
    const player = item.players.find((candidate) => candidate.highlighted);
    if (!event || !player) return [];
    return [{
      eventId: event.id,
      playerId: player.playerId,
      label: `${player.seatNo}号 ${player.name}`,
      text: item.text,
    }];
  });
}

function replaceItem(job: VoiceJob, index: number, item: VoiceJobItem): VoiceJob {
  const items = job.items.map((candidate, itemIndex) => itemIndex === index ? item : candidate);
  const done = items.filter((candidate) => ["completed", "failed", "skipped"].includes(candidate.status)).length;
  return { ...job, items, progress: items.length === 0 ? 1 : done / items.length };
}

function finish(job: VoiceJob, status: VoiceJobStatus): VoiceJob {
  return { ...job, status, completedAt: new Date().toISOString(), progress: status === "completed" ? 1 : job.progress };
}

export class VoiceJobError extends Error {
  constructor(readonly code: string, message: string, readonly httpStatus: number) {
    super(message);
  }
}

const globals = globalThis as typeof globalThis & { __kivaVoiceJobServices?: Map<string, VoiceJobService> };
const services = globals.__kivaVoiceJobServices ?? (globals.__kivaVoiceJobServices = new Map());

export function getVoiceJobService(dataDir = process.env.KIVA_DATA_DIR ?? "kivdb"): VoiceJobService {
  let service = services.get(dataDir);
  if (!service) {
    service = new VoiceJobService(dataDir, undefined, { executeInline: false });
    services.set(dataDir, service);
  }
  return service;
}
