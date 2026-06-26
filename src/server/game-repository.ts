import {
  mkdir,
  readdir,
  readFile,
  rename,
  writeFile,
} from "node:fs/promises";
import { randomUUID } from "node:crypto";
import { join } from "node:path";
import type { DraftEvent } from "@/core/drafts";
import type { GameEvent } from "@/core/events";
import type { Game } from "@/core/game";
import type { GameId } from "@/core/types";

export type GameRecord = {
  readonly game: Game;
  readonly events: readonly GameEvent[];
  readonly draft: DraftEvent | null;
};

export type GameRepository = {
  readonly get: (gameId: GameId) => Promise<GameRecord | null>;
  readonly list: () => Promise<readonly GameRecord[]>;
  readonly save: (record: GameRecord) => Promise<void>;
};

export function createGameRepository(rootDir = ".kiva-data"): GameRepository {
  const gamesDir = join(rootDir, "games");

  async function ensureGamesDir(): Promise<void> {
    await mkdir(gamesDir, { recursive: true });
  }

  function recordPath(gameId: GameId): string {
    return join(gamesDir, `${encodeURIComponent(gameId)}.json`);
  }

  return {
    async get(gameId) {
      try {
        const content = await readFile(recordPath(gameId), "utf8");
        return JSON.parse(content) as GameRecord;
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
            return JSON.parse(content) as GameRecord;
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
  };
}

function isNodeError(error: unknown): error is NodeJS.ErrnoException {
  return error instanceof Error && "code" in error;
}
