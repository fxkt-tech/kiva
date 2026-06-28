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
    });

    const savedFile = await stat(
      join(rootDir, "kivdb", "games", `${encodeURIComponent(gameId)}.json`),
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
    };

    await repository.save(record);

    await expect(repository.get(gameId)).resolves.toEqual(record);
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
    });
    await repository.save({
      game: game(newerGameId, "2026-06-26T00:04:00.000Z"),
      events: [],
      draft: null,
      generations: [],
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
    await mkdir(join(rootDir, "games"), { recursive: true });
    await writeFile(
      join(rootDir, "games", `${encodeURIComponent(gameId)}.json`),
      JSON.stringify(oldRecord),
      "utf8",
    );

    await expect(repository.get(gameId)).resolves.toMatchObject({
      game: { id: gameId },
      events: [],
      draft: null,
      generations: [],
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
    await mkdir(join(rootDir, "games"), { recursive: true });
    await writeFile(
      join(rootDir, "games", `${encodeURIComponent(gameId)}.json`),
      JSON.stringify(oldRecord),
      "utf8",
    );

    const loaded = await repository.get(gameId);

    expect(loaded?.game.players[0]).toMatchObject({
      characterSourceId: null,
      roleSourceId: "werewolf",
      roleName: "狼人",
      team: "wolf",
      mechanicKey: "wolf_kill",
      characterSystemPromptSnapshot:
        "你是狼人杀对局中的一名玩家，只能依据你可见的信息行动。",
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
    await mkdir(join(rootDir, "games"), { recursive: true });
    await writeFile(
      join(rootDir, "games", `${encodeURIComponent(olderGameId)}.json`),
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
    await writeFile(
      join(rootDir, "games", `${encodeURIComponent(newerGameId)}.json`),
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
      roleSourceId: "werewolf",
      roleName: "狼人",
      team: "wolf",
      mechanicKey: "wolf_kill",
      characterSystemPromptSnapshot:
        "你是狼人杀对局中的一名玩家，只能依据你可见的信息行动。",
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
    });

    await expect(
      stat(join(rootDir, "games", `${encodeURIComponent(gameId)}.json`)),
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
    });

    const content = await readFile(
      join(rootDir, "games", `${encodeURIComponent(gameId)}.json`),
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
});
