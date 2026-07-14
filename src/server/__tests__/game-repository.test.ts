import { mkdir, mkdtemp, readFile, rm, stat, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { afterEach, describe, expect, it } from "vitest";
import { createDraftEvent, type DraftEvent } from "@/core/drafts";
import type { GameEvent } from "@/core/events";
import { createSeedGame, type Game } from "@/core/game";
import type { DraftId, EventId, GameId } from "@/core/types";
import {
  createGameRepository,
  GAME_RECORD_SCHEMA_VERSION,
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

function record(
  gameId: GameId,
  updatedAt = "2026-06-26T00:03:00.000Z",
): GameRecord {
  return {
    schemaVersion: GAME_RECORD_SCHEMA_VERSION,
    game: game(gameId, updatedAt),
    events: [],
    draft: null,
    generations: [],
    voiceArtifactsByEventId: {},
    episodeScript: null,
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

async function writePersistedRecord(
  rootDir: string,
  gameId: GameId,
  value: unknown,
): Promise<void> {
  const directory = join(rootDir, "games", encodeURIComponent(gameId));
  await mkdir(directory, { recursive: true });
  await writeFile(join(directory, "record.json"), JSON.stringify(value), "utf8");
}

describe("game repository", () => {
  it("uses kivdb as the default data directory", async () => {
    const rootDir = await createTempDir();
    process.chdir(rootDir);
    const repository = createGameRepository();
    const gameId = "game-1" as GameId;

    await repository.save(record(gameId));

    const savedFile = await stat(
      join(rootDir, "kivdb", "games", encodeURIComponent(gameId), "record.json"),
    );
    expect(savedFile.isFile()).toBe(true);
  });

  it("round-trips a complete current game record", async () => {
    const repository = createGameRepository(await createTempDir());
    const gameId = "game-round-trip" as GameId;
    const current: GameRecord = {
      ...record(gameId),
      events: [event(gameId)],
      draft: draft(gameId),
    };

    await repository.save(current);

    await expect(repository.get(gameId)).resolves.toEqual(current);
  });

  it("rejects role notifications that diverge from the immutable Game snapshot", async () => {
    const rootDir = await createTempDir();
    const repository = createGameRepository(rootDir);
    const gameId = "role-truth" as GameId;
    const current = record(gameId);
    const player = current.game.players[0]!;
    const mismatchedEvent = {
      id: "event-role" as EventId,
      gameId,
      index: 1,
      status: "active" as const,
      type: "role_assigned" as const,
      phase: "setup" as const,
      visibility: {
        kind: "player_private" as const,
        playerIds: [player.playerId],
      },
      targetPlayerIds: [player.playerId],
      payload: {
        playerId: player.playerId,
        role: "werewolf" as const,
        faction: "wolves" as const,
      },
      createdAt: "2026-06-26T00:01:00.000Z",
    };
    await writePersistedRecord(rootDir, gameId, {
      ...current,
      events: [mismatchedEvent],
    });

    await expect(repository.get(gameId)).rejects.toThrow(
      `Role assignment does not match immutable Game snapshot: ${player.playerId}`,
    );
  });

  it("deletes records and treats a missing record as a no-op", async () => {
    const rootDir = await createTempDir();
    const repository = createGameRepository(rootDir);
    const gameId = "game-delete" as GameId;
    await repository.save(record(gameId));

    await repository.delete(gameId);

    await expect(repository.get(gameId)).resolves.toBeNull();
    await expect(repository.delete(gameId)).resolves.toBeUndefined();
    await expect(
      stat(join(rootDir, "games", encodeURIComponent(gameId), "record.json")),
    ).rejects.toMatchObject({ code: "ENOENT" });
  });

  it("lists saved records newest first", async () => {
    const repository = createGameRepository(await createTempDir());
    const olderGameId = "older-game" as GameId;
    const newerGameId = "newer-game" as GameId;
    await repository.save(record(olderGameId, "2026-06-26T00:03:00.000Z"));
    await repository.save(record(newerGameId, "2026-06-26T00:04:00.000Z"));

    const records = await repository.list();

    expect(records.map((item) => item.game.id)).toEqual([
      newerGameId,
      olderGameId,
    ]);
  });

  it("rejects records without the current record schema", async () => {
    const rootDir = await createTempDir();
    const repository = createGameRepository(rootDir);
    const gameId = "missing-schema" as GameId;
    const { schemaVersion: _schemaVersion, ...oldRecord } = record(gameId);
    await writePersistedRecord(rootDir, gameId, oldRecord);

    await expect(repository.get(gameId)).rejects.toThrow(
      "Unsupported game record schema: undefined",
    );
  });

  it("rejects records with missing run mode or episode state", async () => {
    const rootDir = await createTempDir();
    const repository = createGameRepository(rootDir);
    const missingModeId = "missing-run-mode" as GameId;
    const current = record(missingModeId);
    const { runMode: _runMode, ...gameWithoutMode } = current.game;
    await writePersistedRecord(rootDir, missingModeId, {
      ...current,
      game: gameWithoutMode,
    });
    await expect(repository.get(missingModeId)).rejects.toThrow(
      "Game keys are invalid",
    );

    const scriptedId = "missing-episode-state" as GameId;
    const scripted = record(scriptedId);
    await writePersistedRecord(rootDir, scriptedId, {
      ...scripted,
      game: { ...scripted.game, runMode: "scripted" },
      episodeScript: null,
    });
    await expect(repository.get(scriptedId)).rejects.toThrow(
      "Episode script state must be an object",
    );
  });

  it("rejects removed player aliases and incomplete presenter snapshots", async () => {
    const rootDir = await createTempDir();
    const repository = createGameRepository(rootDir);
    const aliasId = "removed-player-alias" as GameId;
    const aliased = record(aliasId);
    await writePersistedRecord(rootDir, aliasId, {
      ...aliased,
      game: {
        ...aliased.game,
        players: aliased.game.players.map((player, index) =>
          index === 0 ? { ...player, systemPrompt: "removed" } : player,
        ),
      },
    });
    await expect(repository.get(aliasId)).rejects.toThrow("keys are invalid");

    const presenterId = "missing-presenter" as GameId;
    const noPresenter = record(presenterId);
    await writePersistedRecord(rootDir, presenterId, {
      ...noPresenter,
      game: { ...noPresenter.game, presenter: undefined },
    });
    await expect(repository.get(presenterId)).rejects.toThrow(
      "Game keys are invalid",
    );
  });

  it("rejects old prompt generations instead of normalizing them", async () => {
    const rootDir = await createTempDir();
    const repository = createGameRepository(rootDir);
    const gameId = "old-prompt" as GameId;
    const current = record(gameId);
    await writePersistedRecord(rootDir, gameId, {
      ...current,
      generations: [
        {
          id: "generation-1",
          gameId,
          draftId: "draft-1",
          playerId: current.game.players[0]!.playerId,
          purpose: "speech",
          status: "success",
          promptVersion: "speech:v1",
          provider: "mock",
          model: "mock-model",
          inputContextHash: "ctx",
          request: {
            schemaName: "werewolf_speech_v1",
            systemPrompt: "old",
            messages: [],
          },
          tokenUsage: null,
          rawOutput: "{}",
          parsedOutput: {},
          error: null,
          createdAt: "2026-06-26T00:02:00.000Z",
          stages: [
            {
              stage: "decision",
              promptVersion: "speech:v1",
              provider: "mock",
              model: "mock-model",
              request: {
                schemaName: "werewolf_speech_v1",
                systemPrompt: "old",
                messages: [],
              },
              tokenUsage: null,
              rawOutput: "{}",
              parsedOutput: {},
              error: null,
            },
          ],
        },
      ],
    });

    await expect(repository.get(gameId)).rejects.toThrow(
      "Unsupported generation prompt version: speech:v1",
    );
  });

  it("stores game ids as one safe path segment", async () => {
    const rootDir = await createTempDir();
    const repository = createGameRepository(rootDir);
    const gameId = "../escaped/game" as GameId;
    await repository.save(record(gameId));

    await expect(
      stat(join(rootDir, "games", encodeURIComponent(gameId), "record.json")),
    ).resolves.toMatchObject({ isFile: expect.any(Function) });
    await expect(stat(join(rootDir, "escaped"))).rejects.toMatchObject({
      code: "ENOENT",
    });
  });

  it("writes pretty JSON with a trailing newline", async () => {
    const rootDir = await createTempDir();
    const repository = createGameRepository(rootDir);
    const gameId = "game-format" as GameId;
    await repository.save(record(gameId));

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
      void firstRepository.withGameLock(gameId, async () => {
        order.push("first-start");
        resolve();
        await new Promise<void>((release) => {
          releaseFirstLock = release;
        });
        order.push("first-end");
      });
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
