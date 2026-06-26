import { mkdtemp, readFile, rm, stat } from "node:fs/promises";
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

afterEach(async () => {
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
  it("returns a saved game record with game, events, and draft", async () => {
    const rootDir = await createTempDir();
    const repository = createGameRepository(rootDir);
    const gameId = "game-1" as GameId;
    const record: GameRecord = {
      game: game(gameId, "2026-06-26T00:03:00.000Z"),
      events: [event(gameId)],
      draft: draft(gameId),
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
    });
    await repository.save({
      game: game(newerGameId, "2026-06-26T00:04:00.000Z"),
      events: [],
      draft: null,
    });

    await expect(repository.list()).resolves.toEqual([
      {
        game: game(newerGameId, "2026-06-26T00:04:00.000Z"),
        events: [],
        draft: null,
      },
      {
        game: game(olderGameId, "2026-06-26T00:03:00.000Z"),
        events: [],
        draft: null,
      },
    ]);
  });

  it("stores game ids as one safe filename segment under games", async () => {
    const rootDir = await createTempDir();
    const repository = createGameRepository(rootDir);
    const gameId = "../escaped/game" as GameId;

    await repository.save({
      game: game(gameId, "2026-06-26T00:03:00.000Z"),
      events: [],
      draft: null,
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
    });

    const content = await readFile(
      join(rootDir, "games", `${encodeURIComponent(gameId)}.json`),
      "utf8",
    );

    expect(content).toContain('\n  "game":');
    expect(content.endsWith("\n")).toBe(true);
  });
});
