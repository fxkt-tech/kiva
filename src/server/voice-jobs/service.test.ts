import { mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { afterEach, describe, expect, it } from "vitest";
import { createSeedGame } from "@/core/game";
import type { GameEvent } from "@/core/events";
import type { EventId, GameId, PlayerId } from "@/core/types";
import {
  createGameRepository,
  GAME_RECORD_SCHEMA_VERSION,
} from "@/server/game-repository";
import type { VoiceSynthesisAdapter } from "@/server/voice-synthesis/types";
import { VoiceJobService } from "./service";

const roots: string[] = [];

afterEach(async () => {
  await Promise.all(roots.splice(0).map((root) => rm(root, { recursive: true, force: true })));
});

describe("voice jobs", () => {
  it("blocks scripted voice generation before episode approval", async () => {
    const root = await mkdtemp(join(tmpdir(), "kiva-voice-episode-lock-"));
    roots.push(root);
    const gameId = "locked_scripted_game" as GameId;
    const game = {
      ...createSeedGame({ gameId, createdAt: "2026-07-12T00:00:00.000Z" }),
      runMode: "scripted" as const,
    };
    await createGameRepository(root).save({
      schemaVersion: GAME_RECORD_SCHEMA_VERSION,
      game,
      events: [],
      draft: null,
      generations: [],
      voiceArtifactsByEventId: {},
      episodeScript: { status: "idle" },
    });

    await expect(new VoiceJobService(root).create(gameId)).rejects.toMatchObject({
      code: "episode_not_approved",
    });
  });

  it("keeps API-created jobs queued until a worker claims them", async () => {
    const root = await mkdtemp(join(tmpdir(), "kiva-voice-worker-"));
    roots.push(root);
    const gameId = "worker_game" as GameId;
    const game = createSeedGame({ gameId, createdAt: "2026-07-12T00:00:00.000Z" });
    const speech: GameEvent = {
      id: "event_worker_1" as EventId,
      gameId,
      index: 1,
      status: "active",
      type: "day_speech_given",
      phase: "speech",
      actorPlayerId: "p1" as PlayerId,
      visibility: { kind: "public" },
      payload: { playerId: "p1" as PlayerId, text: "交给工作进程。", dayNumber: 1, round: 1 },
      createdAt: "2026-07-12T00:01:00.000Z",
    };
    await createGameRepository(root).save({
      schemaVersion: GAME_RECORD_SCHEMA_VERSION,
      game,
      events: [speech],
      draft: null,
      generations: [],
      voiceArtifactsByEventId: {},
      episodeScript: null,
    });
    const adapter: VoiceSynthesisAdapter = {
      async synthesize(request) {
        await createGameRepository(root).withGameLock(gameId, async () => {});
        await writeFile(request.outputPath, "fake mp3");
        return {
          synthesizedText: request.sourceText,
          audioPath: request.outputPath,
          durationMs: 900,
          wordBoundaries: [{ part: request.sourceText, start: 0, end: 850 }],
          generator: { adapter: "node-edge-tts", packageVersion: "test" },
        };
      },
    };
    const service = new VoiceJobService(root, adapter, { executeInline: false });
    const created = await service.create(gameId);
    expect(created.status).toBe("queued");
    expect((await service.list(gameId))[0]?.status).toBe("queued");

    expect(await service.processQueuedOnce()).toBe(true);
    expect((await service.list(gameId))[0]?.status).toBe("completed");
    expect(await service.processQueuedOnce()).toBe(false);
  });

  it("persists one immutable player voice artifact and skips it on retry", async () => {
    const root = await mkdtemp(join(tmpdir(), "kiva-voice-job-"));
    roots.push(root);
    const gameId = "voice_game" as GameId;
    const game = createSeedGame({ gameId, createdAt: "2026-07-12T00:00:00.000Z" });
    const speech: GameEvent = {
      id: "event_voice_1" as EventId,
      gameId,
      index: 1,
      status: "active",
      type: "day_speech_given",
      phase: "speech",
      actorPlayerId: "p1" as PlayerId,
      visibility: { kind: "public" },
      payload: {
        playerId: "p1" as PlayerId,
        text: "第一句。第二句！",
        dayNumber: 1,
        round: 1,
      },
      createdAt: "2026-07-12T00:01:00.000Z",
    };
    const repository = createGameRepository(root);
    await repository.save({
      schemaVersion: GAME_RECORD_SCHEMA_VERSION,
      game,
      events: [speech],
      draft: null,
      generations: [],
      voiceArtifactsByEventId: {},
      episodeScript: null,
    });

    let calls = 0;
    const adapter: VoiceSynthesisAdapter = {
      async synthesize(request) {
        calls += 1;
        await writeFile(request.outputPath, "fake mp3");
        return {
          synthesizedText: request.sourceText,
          audioPath: request.outputPath,
          durationMs: 1600,
          wordBoundaries: [
            { part: "第一句。", start: 0, end: 700 },
            { part: "第二句！", start: 800, end: 1500 },
          ],
          generator: { adapter: "node-edge-tts", packageVersion: "test" },
        };
      },
    };
    const service = new VoiceJobService(root, adapter, { executeInline: true });
    await service.create(gameId);
    const completed = await waitForTerminal(service, gameId);

    expect(completed.status).toBe("completed");
    expect(calls).toBe(1);
    const saved = await repository.get(gameId);
    expect(saved?.voiceArtifactsByEventId.event_voice_1).toMatchObject({
      synthesizedText: "第一句。第二句！",
      audio: { file: "event_voice_1.mp3", durationMs: 1600 },
      cues: [
        { text: "第一句。", startMs: 0, endMs: 700 },
        { text: "第二句！", startMs: 800, endMs: 1600 },
      ],
    });
    expect(await readFile(repository.voicePath(gameId, "event_voice_1"), "utf8")).toBe("fake mp3");
    await expect(service.create(gameId)).rejects.toMatchObject({ code: "voice_complete" });
    expect(calls).toBe(1);
  });
});

async function waitForTerminal(service: VoiceJobService, gameId: string) {
  for (let attempt = 0; attempt < 100; attempt += 1) {
    const [job] = await service.list(gameId);
    if (job && ["completed", "failed", "canceled", "interrupted"].includes(job.status)) return job;
    await new Promise((resolve) => setTimeout(resolve, 5));
  }
  throw new Error("Voice job did not finish");
}
