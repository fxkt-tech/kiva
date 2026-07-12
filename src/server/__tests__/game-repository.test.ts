import { mkdir, mkdtemp, readFile, rm, stat, writeFile } from "node:fs/promises";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { afterEach, describe, expect, it } from "vitest";
import { createDraftEvent, type DraftEvent } from "@/core/drafts";
import type { GameEvent } from "@/core/events";
import { createSeedGame, type Game } from "@/core/game";
import type { DraftId, EventId, GameId } from "@/core/types";
import {
  createGameRepository,
  type GameRecord,
} from "../game-repository";

const tempDirs: string[] = [];
const originalCwd = process.cwd();

afterEach(async () => {
  process.chdir(originalCwd);
  await Promise.all(
    tempDirs.splice(0).map((dir) => rm(dir, { recursive: true, force: true })),
  );
});

async function createTempDir(): Promise<string> {
  const dir = await mkdtemp(join(tmpdir(), "kiva-game-repository-"));
  tempDirs.push(dir);
  return dir;
}

function game(gameId: GameId, updatedAt: string): Game {
  return {
    ...createSeedGame({ gameId, createdAt: "2026-06-26T00:00:00.000Z" }),
    updatedAt,
  };
}

function legacyGameWithoutPlayerLibrarySnapshots(
  gameId: GameId,
  updatedAt: string,
): unknown {
  const currentGame = game(gameId, updatedAt);

  return {
    ...currentGame,
    players: currentGame.players.map((player) => ({
      playerId: player.playerId,
      seatNo: player.seatNo,
      profileSourceId: player.profileSourceId,
      name: player.name,
      persona: player.persona,
      speakingStyle: player.speakingStyle,
      reasoningStyle: player.reasoningStyle,
      systemPrompt: player.systemPrompt,
      modelBindingSnapshot: player.modelBindingSnapshot,
      gameRole: player.gameRole,
      faction: player.faction,
      initialPrivateKnowledge: player.initialPrivateKnowledge,
    })),
  };
}

function event(gameId: GameId): GameEvent {
  return {
    id: "event-1" as EventId,
    gameId,
    index: 1,
    status: "active",
    type: "phase_started",
    phase: "night",
    visibility: { kind: "public" },
    payload: { phase: "night", dayNumber: 1 },
    createdAt: "2026-06-26T00:01:00.000Z",
  };
}

function draft(gameId: GameId): DraftEvent {
  return createDraftEvent({
    id: "draft-1" as DraftId,
    gameId,
    type: "phase_started",
    phase: "day",
    visibility: { kind: "public" },
    payload: { phase: "day", dayNumber: 1 },
    createdAt: "2026-06-26T00:02:00.000Z",
  });
}

describe("game repository", () => {
  it("uses kivdb as the default data directory", async () => {
    const rootDir = await createTempDir();
    process.chdir(rootDir);
    const repository = createGameRepository();
    const gameId = "game-1" as GameId;

    await repository.save({
      game: game(gameId, "2026-06-26T00:03:00.000Z"),
      events: [],
      draft: null,
      generations: [],
    voiceArtifactsByEventId: {},
    });

    const savedFile = await stat(
      join(rootDir, "kivdb", "games", encodeURIComponent(gameId), "record.json"),
    );
    expect(savedFile.isFile()).toBe(true);
  });

  it("returns a saved game record with game, events, and draft", async () => {
    const rootDir = await createTempDir();
    const repository = createGameRepository(rootDir);
    const gameId = "game-1" as GameId;
    const record: GameRecord = {
      game: game(gameId, "2026-06-26T00:03:00.000Z"),
      events: [event(gameId)],
      draft: draft(gameId),
      generations: [],
    voiceArtifactsByEventId: {},
      episodeScript: null,
    };

    await repository.save(record);

    await expect(repository.get(gameId)).resolves.toEqual(record);
  });

  it("deletes a saved game record", async () => {
    const rootDir = await createTempDir();
    const repository = createGameRepository(rootDir);
    const gameId = "game-delete" as GameId;

    await repository.save({
      game: game(gameId, "2026-06-26T00:03:00.000Z"),
      events: [],
      draft: null,
      generations: [],
    voiceArtifactsByEventId: {},
    });

    await repository.delete(gameId);

    await expect(repository.get(gameId)).resolves.toBeNull();
    await expect(
      stat(join(rootDir, "games", encodeURIComponent(gameId), "record.json")),
    ).rejects.toMatchObject({ code: "ENOENT" });
  });

  it("treats deleting a missing game as a no-op", async () => {
    const repository = createGameRepository(await createTempDir());

    await expect(repository.delete("missing-game" as GameId)).resolves.toBeUndefined();
  });

  it("lists saved records newest first by game updated time", async () => {
    const rootDir = await createTempDir();
    const repository = createGameRepository(rootDir);
    const olderGameId = "older-game" as GameId;
    const newerGameId = "newer-game" as GameId;

    await repository.save({
      game: game(olderGameId, "2026-06-26T00:03:00.000Z"),
      events: [],
      draft: null,
      generations: [],
    voiceArtifactsByEventId: {},
    });
    await repository.save({
      game: game(newerGameId, "2026-06-26T00:04:00.000Z"),
      events: [],
      draft: null,
      generations: [],
    voiceArtifactsByEventId: {},
    });

    const records = await repository.list();
    expect(records.map((record) => record.game.id)).toEqual([
      newerGameId,
      olderGameId,
    ]);
    expect(records.map((record) => record.generations)).toEqual([[], []]);
  });

  it("normalizes old records without generations", async () => {
    const rootDir = await createTempDir();
    const repository = createGameRepository(rootDir);
    const gameId = "old-game" as GameId;
    const oldRecord = {
      game: game(gameId, "2026-06-26T00:03:00.000Z"),
      events: [],
      draft: null,
    };
    await mkdir(join(rootDir, "games", encodeURIComponent(gameId)), { recursive: true });
    await writeFile(
      join(rootDir, "games", encodeURIComponent(gameId), "record.json"),
      JSON.stringify(oldRecord),
      "utf8",
    );

    await expect(repository.get(gameId)).resolves.toMatchObject({
      game: { id: gameId, runMode: "game" },
      events: [],
      draft: null,
      generations: [],
    voiceArtifactsByEventId: {},
    });
  });

  it("normalizes a scripted record without episode state to idle", async () => {
    const rootDir = await createTempDir();
    const repository = createGameRepository(rootDir);
    const gameId = "old-scripted-game" as GameId;
    await mkdir(join(rootDir, "games", encodeURIComponent(gameId)), { recursive: true });
    await writeFile(
      join(rootDir, "games", encodeURIComponent(gameId), "record.json"),
      JSON.stringify({
        game: { ...game(gameId, "2026-06-26T00:03:00.000Z"), runMode: "scripted" },
        events: [],
        draft: null,
      }),
      "utf8",
    );

    await expect(repository.get(gameId)).resolves.toMatchObject({
      game: { runMode: "scripted" },
      episodeScript: { status: "idle" },
    });
  });

  it("rejects game records without a presenter snapshot", async () => {
    const rootDir = await createTempDir();
    const repository = createGameRepository(rootDir);
    const gameId = "missing-presenter" as GameId;
    const gameWithoutPresenter = {
      ...game(gameId, "2026-06-26T00:03:00.000Z"),
      presenter: undefined,
    };
    await mkdir(join(rootDir, "games", encodeURIComponent(gameId)), { recursive: true });
    await writeFile(
      join(rootDir, "games", encodeURIComponent(gameId), "record.json"),
      JSON.stringify({
        game: gameWithoutPresenter,
        events: [],
        draft: null,
        generations: [],
      voiceArtifactsByEventId: {},
      }),
      "utf8",
    );

    await expect(repository.get(gameId)).rejects.toThrow(
      "Game presenter snapshot must be an object",
    );
  });

  it("normalizes old generation records without token usage", async () => {
    const rootDir = await createTempDir();
    const repository = createGameRepository(rootDir);
    const gameId = "old-generation-token-usage" as GameId;
    const oldRecord = {
      game: game(gameId, "2026-06-26T00:03:00.000Z"),
      events: [],
      draft: null,
      generations: [
        {
          id: "generation_1",
          gameId,
          draftId: "draft_1",
          playerId: "player_1",
          purpose: "speech",
          status: "success",
          promptVersion: "speech:v1",
          provider: "mock",
          model: "mock-model",
          inputContextHash: "ctx",
          request: null,
          rawOutput: "{}",
          parsedOutput: {},
          error: null,
          createdAt: "2026-06-26T00:02:00.000Z",
        },
      ],
    };
    await mkdir(join(rootDir, "games", encodeURIComponent(gameId)), { recursive: true });
    await writeFile(
      join(rootDir, "games", encodeURIComponent(gameId), "record.json"),
      JSON.stringify(oldRecord),
      "utf8",
    );

    await expect(repository.get(gameId)).resolves.toMatchObject({
      generations: [{ tokenUsage: null }],
    });
  });

  it("normalizes old player snapshots when loading a game", async () => {
    const rootDir = await createTempDir();
    const repository = createGameRepository(rootDir);
    const gameId = "old-player-snapshots" as GameId;
    const oldRecord = {
      game: legacyGameWithoutPlayerLibrarySnapshots(
        gameId,
        "2026-06-26T00:03:00.000Z",
      ),
      events: [],
      draft: null,
    };
    await mkdir(join(rootDir, "games", encodeURIComponent(gameId)), { recursive: true });
    await writeFile(
      join(rootDir, "games", encodeURIComponent(gameId), "record.json"),
      JSON.stringify(oldRecord),
      "utf8",
    );

    const loaded = await repository.get(gameId);

    expect(loaded?.game.players[0]).toMatchObject({
      characterSourceId: null,
      roleSourceId: "villager",
      roleName: "平民",
      team: "villager",
      mechanicKey: "none",
      characterSystemPromptSnapshot:
        "你是狼人杀玩家周序。你负责维护事实账本，优先票型、顺序和可验证记录。不要把他人主张记成事实，也不要为数字编造含义。",
      roleSystemPromptSnapshot: "",
      roleActionPromptSnapshot: null,
      avatar: null,
    });
  });

  it("normalizes old player snapshots when listing games", async () => {
    const rootDir = await createTempDir();
    const repository = createGameRepository(rootDir);
    const olderGameId = "older-old-player-snapshots" as GameId;
    const newerGameId = "newer-old-player-snapshots" as GameId;
    await mkdir(join(rootDir, "games", encodeURIComponent(olderGameId)), { recursive: true });
    await writeFile(
      join(rootDir, "games", encodeURIComponent(olderGameId), "record.json"),
      JSON.stringify({
        game: legacyGameWithoutPlayerLibrarySnapshots(
          olderGameId,
          "2026-06-26T00:03:00.000Z",
        ),
        events: [],
        draft: null,
      }),
      "utf8",
    );
    await mkdir(join(rootDir, "games", encodeURIComponent(newerGameId)), { recursive: true });
    await writeFile(
      join(rootDir, "games", encodeURIComponent(newerGameId), "record.json"),
      JSON.stringify({
        game: legacyGameWithoutPlayerLibrarySnapshots(
          newerGameId,
          "2026-06-26T00:04:00.000Z",
        ),
        events: [],
        draft: null,
      }),
      "utf8",
    );

    const records = await repository.list();

    expect(records.map((record) => record.game.id)).toEqual([
      newerGameId,
      olderGameId,
    ]);
    expect(records[0].game.players[0]).toMatchObject({
      characterSourceId: null,
      roleSourceId: "villager",
      roleName: "平民",
      team: "villager",
      mechanicKey: "none",
      characterSystemPromptSnapshot:
        "你是狼人杀玩家周序。你负责维护事实账本，优先票型、顺序和可验证记录。不要把他人主张记成事实，也不要为数字编造含义。",
      roleSystemPromptSnapshot: "",
      roleActionPromptSnapshot: null,
      avatar: null,
    });
  });

  it("stores game ids as one safe filename segment under games", async () => {
    const rootDir = await createTempDir();
    const repository = createGameRepository(rootDir);
    const gameId = "../escaped/game" as GameId;

    await repository.save({
      game: game(gameId, "2026-06-26T00:03:00.000Z"),
      events: [],
      draft: null,
      generations: [],
    voiceArtifactsByEventId: {},
    });

    await expect(
      stat(join(rootDir, "games", encodeURIComponent(gameId), "record.json")),
    ).resolves.toMatchObject({ isFile: expect.any(Function) });
    await expect(stat(join(rootDir, "escaped"))).rejects.toMatchObject({
      code: "ENOENT",
    });
    await expect(repository.get(gameId)).resolves.toMatchObject({
      game: { id: gameId },
    });
  });

  it("writes pretty JSON with a trailing newline", async () => {
    const rootDir = await createTempDir();
    const repository = createGameRepository(rootDir);
    const gameId = "game-format" as GameId;

    await repository.save({
      game: game(gameId, "2026-06-26T00:03:00.000Z"),
      events: [],
      draft: null,
      generations: [],
    voiceArtifactsByEventId: {},
    });

    const content = await readFile(
      join(rootDir, "games", encodeURIComponent(gameId), "record.json"),
      "utf8",
    );

    expect(content).toContain('\n  "game":');
    expect(content.endsWith("\n")).toBe(true);
  });

  it("serializes game locks across repository instances", async () => {
    const rootDir = await createTempDir();
    const firstRepository = createGameRepository(rootDir);
    const secondRepository = createGameRepository(rootDir);
    const gameId = "game-lock" as GameId;
    const order: string[] = [];
    let releaseFirstLock: () => void = () => {};
    const firstLockStarted = new Promise<void>((resolve) => {
      const firstLock = firstRepository.withGameLock(gameId, async () => {
        order.push("first-start");
        resolve();
        await new Promise<void>((release) => {
          releaseFirstLock = release;
        });
        order.push("first-end");
      });
      void firstLock;
    });

    await firstLockStarted;
    const secondLock = secondRepository.withGameLock(gameId, async () => {
      order.push("second");
    });

    await new Promise((resolve) => setTimeout(resolve, 25));
    expect(order).toEqual(["first-start"]);

    releaseFirstLock();
    await secondLock;

    expect(order).toEqual(["first-start", "first-end", "second"]);
  });

  it("recovers a lock owned by a dead process", async () => {
    const rootDir = await createTempDir();
    const gameId = "stale-lock" as GameId;
    const lockDir = join(rootDir, "locks", `${gameId}.lock`);
    await mkdir(lockDir, { recursive: true });
    await writeFile(
      join(lockDir, "owner.json"),
      JSON.stringify({ pid: 999_999_999, token: "dead", createdAt: 0 }),
      "utf8",
    );

    let entered = false;
    await createGameRepository(rootDir).withGameLock(gameId, async () => {
      entered = true;
    });

    expect(entered).toBe(true);
    await expect(stat(lockDir)).rejects.toMatchObject({ code: "ENOENT" });
  });
});
