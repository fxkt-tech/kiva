import {
  mkdir,
  readdir,
  readFile,
  rename,
  rm,
  stat,
  writeFile,
} from "node:fs/promises";
import { randomUUID } from "node:crypto";
import { setTimeout } from "node:timers/promises";
import { join } from "node:path";
import type { DraftEvent } from "@/core/drafts";
import type { GameEvent } from "@/core/events";
import type { Game } from "@/core/game";
import type { GenerationRecord } from "@/core/generation-record";
import {
  validatePlayerVoiceArtifact,
  type PlayerVoiceArtifact,
} from "@/core/voice";
import { validateGamePresenterSnapshot } from "@/core/presenter-definition";
import {
  legacyGameScriptSnapshot,
  validateGameScriptSnapshot,
} from "@/core/game-script";
import { createPlayerSnapshot } from "@/core/player";
import {
  createDefaultRuleset,
  createSixPlayerRuleset,
  type GameId,
  type Ruleset,
} from "@/core/types";

export type GameRecord = {
  readonly game: Game;
  readonly events: readonly GameEvent[];
  readonly draft: DraftEvent | null;
  readonly generations: readonly GenerationRecord[];
  readonly voiceArtifactsByEventId: Readonly<Record<string, PlayerVoiceArtifact>>;
};

export type GameRepository = {
  readonly get: (gameId: GameId) => Promise<GameRecord | null>;
  readonly list: () => Promise<readonly GameRecord[]>;
  readonly save: (record: GameRecord) => Promise<void>;
  readonly delete: (gameId: GameId) => Promise<void>;
  readonly voicePath: (gameId: GameId, eventId: string) => string;
  readonly voiceTempPath: (gameId: GameId, eventId: string) => Promise<string>;
  readonly publishVoice: (
    gameId: GameId,
    eventId: string,
    tempPath: string,
  ) => Promise<string>;
  readonly withGameLock: <T>(
    gameId: GameId,
    operation: () => Promise<T>,
  ) => Promise<T>;
};

export function createGameRepository(rootDir = "kivdb"): GameRepository {
  const gamesDir = join(rootDir, "games");
  const locksDir = join(rootDir, "locks");

  async function ensureGamesDir(): Promise<void> {
    await mkdir(gamesDir, { recursive: true });
  }

  async function ensureLocksDir(): Promise<void> {
    await mkdir(locksDir, { recursive: true });
  }

  function gameDir(gameId: GameId): string {
    return join(gamesDir, encodeURIComponent(gameId));
  }

  function recordPath(gameId: GameId): string {
    return join(gameDir(gameId), "record.json");
  }

  function lockPath(gameId: GameId): string {
    return join(locksDir, `${encodeURIComponent(gameId)}.lock`);
  }

  function voiceFileName(eventId: string): string {
    if (!/^[a-zA-Z0-9_-]{1,160}$/.test(eventId)) {
      throw new Error("Invalid voice event identifier");
    }
    return `${eventId}.mp3`;
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
      const entries = await readdir(gamesDir, { withFileTypes: true });
      const records = await Promise.all(
        entries
          .filter((entry) => entry.isDirectory())
          .map(async (entry) => {
            const content = await readFile(
              join(gamesDir, entry.name, "record.json"),
              "utf8",
            );
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
      await mkdir(gameDir(record.game.id), { recursive: true });
      const tempPath = `${targetPath}.${randomUUID()}.tmp`;
      await writeFile(tempPath, `${JSON.stringify(record, null, 2)}\n`, "utf8");
      await rename(tempPath, targetPath);
    },

    async delete(gameId) {
      await rm(gameDir(gameId), { recursive: true, force: true });
    },

    voicePath(gameId, eventId) {
      return join(gameDir(gameId), "voice", voiceFileName(eventId));
    },

    async voiceTempPath(gameId, eventId) {
      const directory = join(gameDir(gameId), "voice", ".tmp");
      await mkdir(directory, { recursive: true });
      return join(directory, `${voiceFileName(eventId)}.${randomUUID()}.tmp`);
    },

    async publishVoice(gameId, eventId, tempPath) {
      const target = join(gameDir(gameId), "voice", voiceFileName(eventId));
      await mkdir(join(gameDir(gameId), "voice"), { recursive: true });
      await rename(tempPath, target);
      return voiceFileName(eventId);
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
  const record = rawRecord as Omit<
    GameRecord,
    "generations" | "voiceArtifactsByEventId"
  > & {
    readonly generations?: readonly GenerationRecord[];
    readonly voiceArtifactsByEventId?: Readonly<Record<string, PlayerVoiceArtifact>>;
  };

  const voiceArtifactsByEventId = Object.fromEntries(
    Object.entries(record.voiceArtifactsByEventId ?? {}).map(([eventId, artifact]) => [
      eventId,
      validatePlayerVoiceArtifact(artifact, eventId),
    ]),
  );

  return {
    ...record,
    game: {
      ...record.game,
      presenter: validateGamePresenterSnapshot(record.game.presenter),
      script: record.game.script
        ? validateGameScriptSnapshot(record.game.script)
        : legacyGameScriptSnapshot(),
      ruleset: normalizeRuleset(record.game.ruleset),
      players: record.game.players.map((player) =>
        createPlayerSnapshot(player),
      ),
    },
    generations: (record.generations ?? []).map(normalizeGenerationRecord),
    voiceArtifactsByEventId,
  };
}

function normalizeGenerationRecord(generation: GenerationRecord): GenerationRecord {
  return {
    ...generation,
    tokenUsage: generation.tokenUsage ?? null,
  };
}

function normalizeRuleset(ruleset: Ruleset): Ruleset {
  const fallback =
    ruleset.playerCount === 6 ? createSixPlayerRuleset() : createDefaultRuleset();

  return {
    ...fallback,
    ...ruleset,
    roleCounts: {
      ...fallback.roleCounts,
      ...ruleset.roleCounts,
    },
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
  const token = randomUUID();
  const ownerPath = join(path, "owner.json");

  while (true) {
    try {
      await mkdir(path);
      await writeFile(
        ownerPath,
        JSON.stringify({ pid: process.pid, token, createdAt: Date.now() }),
        "utf8",
      );
      return async () => {
        const owner = await readLockOwner(ownerPath);
        if (owner?.token === token) await rm(path, { recursive: true, force: true });
      };
    } catch (error) {
      if (isNodeError(error) && error.code === "EEXIST") {
        if (await lockIsStale(path, ownerPath)) {
          await rm(path, { recursive: true, force: true });
          continue;
        }
        await setTimeout(10);
        continue;
      }

      throw error;
    }
  }
}

type LockOwner = { readonly pid: number; readonly token: string };

async function readLockOwner(path: string): Promise<LockOwner | null> {
  try {
    const value = JSON.parse(await readFile(path, "utf8")) as Partial<LockOwner>;
    return Number.isInteger(value.pid) && typeof value.token === "string"
      ? value as LockOwner
      : null;
  } catch {
    return null;
  }
}

async function lockIsStale(path: string, ownerPath: string): Promise<boolean> {
  const owner = await readLockOwner(ownerPath);
  if (owner) return !processIsAlive(owner.pid);
  try {
    return Date.now() - (await stat(path)).mtimeMs > 5_000;
  } catch (error) {
    return isNodeError(error) && error.code === "ENOENT";
  }
}

function processIsAlive(pid: number): boolean {
  try {
    process.kill(pid, 0);
    return true;
  } catch {
    return false;
  }
}
