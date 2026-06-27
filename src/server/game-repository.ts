import {
  mkdir,
  readdir,
  readFile,
  rename,
  rmdir,
  writeFile,
} from "node:fs/promises";
import { randomUUID } from "node:crypto";
import { setTimeout } from "node:timers/promises";
import { join } from "node:path";
import type { DraftEvent } from "@/core/drafts";
import type { GameEvent } from "@/core/events";
import type { Game } from "@/core/game";
import type { GenerationRecord } from "@/core/generation-record";
import { createPlayerSnapshot } from "@/core/player";
import type { GameId } from "@/core/types";

export type GameRecord = {
  readonly game: Game;
  readonly events: readonly GameEvent[];
  readonly draft: DraftEvent | null;
  readonly generations: readonly GenerationRecord[];
};

export type GameRepository = {
  readonly get: (gameId: GameId) => Promise<GameRecord | null>;
  readonly list: () => Promise<readonly GameRecord[]>;
  readonly save: (record: GameRecord) => Promise<void>;
  readonly withGameLock: <T>(
    gameId: GameId,
    operation: () => Promise<T>,
  ) => Promise<T>;
};

export function createGameRepository(rootDir = ".kiva-data"): GameRepository {
  const gamesDir = join(rootDir, "games");
  const locksDir = join(rootDir, "locks");

  async function ensureGamesDir(): Promise<void> {
    await mkdir(gamesDir, { recursive: true });
  }

  async function ensureLocksDir(): Promise<void> {
    await mkdir(locksDir, { recursive: true });
  }

  function recordPath(gameId: GameId): string {
    return join(gamesDir, `${encodeURIComponent(gameId)}.json`);
  }

  function lockPath(gameId: GameId): string {
    return join(locksDir, `${encodeURIComponent(gameId)}.lock`);
  }

  return {
    async get(gameId) {
      try {
        const content = await readFile(recordPath(gameId), "utf8");
        return normalizeRecord(JSON.parse(content));
      } catch (error) {
        if (isNodeError(error) && error.code === "ENOENT") {
          return null;
        }
        throw error;
      }
    },

    async list() {
      await ensureGamesDir();
      const filenames = await readdir(gamesDir);
      const records = await Promise.all(
        filenames
          .filter((filename) => filename.endsWith(".json"))
          .map(async (filename) => {
            const content = await readFile(join(gamesDir, filename), "utf8");
            return normalizeRecord(JSON.parse(content));
          }),
      );

      return [...records].sort((left, right) =>
        right.game.updatedAt.localeCompare(left.game.updatedAt),
      );
    },

    async save(record) {
      await ensureGamesDir();
      const targetPath = recordPath(record.game.id);
      const tempPath = `${targetPath}.${randomUUID()}.tmp`;
      await writeFile(tempPath, `${JSON.stringify(record, null, 2)}\n`, "utf8");
      await rename(tempPath, targetPath);
    },

    async withGameLock(gameId, operation) {
      const release = await acquireLock(lockPath(gameId), ensureLocksDir);
      try {
        return await operation();
      } finally {
        await release();
      }
    },
  };
}

function normalizeRecord(rawRecord: unknown): GameRecord {
  const record = rawRecord as Omit<GameRecord, "generations"> & {
    readonly generations?: readonly GenerationRecord[];
  };

  return {
    ...record,
    game: {
      ...record.game,
      players: record.game.players.map((player) =>
        createPlayerSnapshot(player),
      ),
    },
    generations: record.generations ?? [],
  };
}

function isNodeError(error: unknown): error is NodeJS.ErrnoException {
  return error instanceof Error && "code" in error;
}

async function acquireLock(
  path: string,
  ensureParentDir: () => Promise<void>,
): Promise<() => Promise<void>> {
  await ensureParentDir();

  while (true) {
    try {
      await mkdir(path);
      return () => rmdir(path);
    } catch (error) {
      if (isNodeError(error) && error.code === "EEXIST") {
        await setTimeout(10);
        continue;
      }

      throw error;
    }
  }
}
