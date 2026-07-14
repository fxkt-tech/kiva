import {
  mkdir,
  readdir,
  readFile,
  rename,
  rm,
  writeFile,
} from "node:fs/promises";
import { randomUUID } from "node:crypto";
import { join } from "node:path";
import type { DraftEvent } from "@/core/drafts";
import type { GameEvent } from "@/core/events";
import type { Game } from "@/core/game";
import { parseGameRunMode } from "@/core/game-run-mode";
import {
  validateEpisodeScriptState,
  type EpisodeScriptState,
} from "@/core/episode-script";
import {
  validateGenerationRecord,
  type GenerationRecord,
} from "@/core/generation-record";
import {
  validatePlayerVoiceArtifact,
  type PlayerVoiceArtifact,
} from "@/core/voice";
import { validateGamePresenterSnapshot } from "@/core/presenter-definition";
import { validateGameScriptSnapshot } from "@/core/game-script";
import { validateBoard, validatePlayerSnapshot } from "@/core/player";
import { factionForRuleRole, isRuleRoleId } from "@/core/rule-role";
import {
  validateRuleset,
  type GameId,
} from "@/core/types";
import {
  assertExactObjectKeys,
  isPlainObject,
} from "@/core/model-binding";
import { acquireDirectoryLock } from "./directory-lock";

export const GAME_RECORD_SCHEMA_VERSION = 2 as const;

export type GameRecord = {
  readonly schemaVersion: typeof GAME_RECORD_SCHEMA_VERSION;
  readonly game: Game;
  readonly events: readonly GameEvent[];
  readonly draft: DraftEvent | null;
  readonly generations: readonly GenerationRecord[];
  readonly voiceArtifactsByEventId: Readonly<Record<string, PlayerVoiceArtifact>>;
  readonly episodeScript: EpisodeScriptState | null;
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
        return validateGameRecord(JSON.parse(content));
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
            return validateGameRecord(JSON.parse(content));
          }),
      );

      return [...records].sort((left, right) =>
        right.game.updatedAt.localeCompare(left.game.updatedAt),
      );
    },

    async save(record) {
      await ensureGamesDir();
      const validated = validateGameRecord(record);
      const targetPath = recordPath(validated.game.id);
      await mkdir(gameDir(validated.game.id), { recursive: true });
      const tempPath = `${targetPath}.${randomUUID()}.tmp`;
      await writeFile(
        tempPath,
        `${JSON.stringify(validated, null, 2)}\n`,
        "utf8",
      );
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
      const release = await acquireDirectoryLock(lockPath(gameId));
      try {
        return await operation();
      } finally {
        await release();
      }
    },
  };
}

export function validateGameRecord(rawRecord: unknown): GameRecord {
  if (!isPlainObject(rawRecord)) {
    throw new Error("Game record must be an object");
  }
  if (rawRecord.schemaVersion !== GAME_RECORD_SCHEMA_VERSION) {
    throw new Error(
      `Unsupported game record schema: ${String(rawRecord.schemaVersion)}`,
    );
  }
  assertExactObjectKeys(rawRecord, "Game record", [
    "schemaVersion",
    "game",
    "events",
    "draft",
    "generations",
    "voiceArtifactsByEventId",
    "episodeScript",
  ]);
  if (!isPlainObject(rawRecord.game)) {
    throw new Error("Game record game must be an object");
  }
  const game = validateGame(rawRecord.game);
  if (!Array.isArray(rawRecord.events)) {
    throw new Error("Game record events must be an array");
  }
  const events = rawRecord.events.map((event) =>
    validateEventEnvelope(event, game.id),
  );
  validateRoleAssignmentTruth(events, game);
  const draft = rawRecord.draft === null
    ? null
    : validateDraftEnvelope(rawRecord.draft, game.id);
  validateRoleAssignmentTruth(draft ? [draft] : [], game);
  if (!Array.isArray(rawRecord.generations)) {
    throw new Error("Game record generations must be an array");
  }
  const generations = rawRecord.generations.map((generation) =>
    validateGenerationRecord(generation, game.id),
  );
  if (!isPlainObject(rawRecord.voiceArtifactsByEventId)) {
    throw new Error("Game record voiceArtifactsByEventId must be an object");
  }
  const voiceArtifactsByEventId = Object.fromEntries(
    Object.entries(rawRecord.voiceArtifactsByEventId).map(([eventId, artifact]) => [
      eventId,
      validatePlayerVoiceArtifact(artifact, eventId),
    ]),
  );

  return {
    schemaVersion: GAME_RECORD_SCHEMA_VERSION,
    game,
    events,
    draft,
    generations,
    voiceArtifactsByEventId,
    episodeScript: validateEpisodeScriptState(
      rawRecord.episodeScript,
      game.runMode,
    ),
  };
}

function validateGame(value: Record<string, unknown>): Game {
  assertExactObjectKeys(value, "Game", [
    "id",
    "title",
    "status",
    "runMode",
    "ruleset",
    "presenter",
    "script",
    "players",
    "createdAt",
    "updatedAt",
  ]);
  for (const field of ["id", "title", "createdAt", "updatedAt"] as const) {
    if (!isNonBlankString(value[field])) {
      throw new Error(`Game ${field} must be set`);
    }
  }
  if (value.status !== "drafting" && value.status !== "ended") {
    throw new Error("Game status is invalid");
  }
  const runMode = parseGameRunMode(value.runMode);
  const ruleset = validateRuleset(value.ruleset);
  const presenter = validateGamePresenterSnapshot(value.presenter);
  const script = validateGameScriptSnapshot(value.script);
  if (!Array.isArray(value.players)) {
    throw new Error("Game players must be an array");
  }
  const players = value.players.map(validatePlayerSnapshot);
  const board = validateBoard(players, ruleset);
  if (!board.ok) {
    throw new Error(`Game board is invalid: ${board.reason}`);
  }

  return {
    id: value.id as Game["id"],
    title: value.title as string,
    status: value.status,
    runMode,
    ruleset,
    presenter,
    script,
    players,
    createdAt: value.createdAt as string,
    updatedAt: value.updatedAt as string,
  };
}

function validateEventEnvelope(value: unknown, gameId: GameId): GameEvent {
  if (!isPlainObject(value)) {
    throw new Error("Game event envelope is invalid");
  }
  assertExactObjectKeys(
    value,
    "Game event",
    [
      "id",
      "gameId",
      "index",
      "status",
      "type",
      "phase",
      "visibility",
      "payload",
      "createdAt",
    ],
    ["actorPlayerId", "targetPlayerIds", "createdFromDraftId"],
  );
  if (
    !isNonBlankString(value.id) ||
    value.gameId !== gameId ||
    !Number.isInteger(value.index) ||
    (value.index as number) < 1 ||
    (value.status !== "active" && value.status !== "superseded") ||
    !isGameEventType(value.type) ||
    !isPhase(value.phase) ||
    !isVisibility(value.visibility) ||
    !isPlainObject(value.payload) ||
    !isNonBlankString(value.createdAt)
  ) {
    throw new Error("Game event envelope is invalid");
  }
  validateOptionalEventFields(value);
  validateCurrentEventPayload(value.type, value.payload);
  return structuredClone(value) as GameEvent;
}

function validateDraftEnvelope(value: unknown, gameId: GameId): DraftEvent {
  if (!isPlainObject(value)) {
    throw new Error("Game draft envelope is invalid");
  }
  assertExactObjectKeys(
    value,
    "Game draft",
    [
      "id",
      "gameId",
      "status",
      "type",
      "phase",
      "visibility",
      "payload",
      "createdAt",
    ],
    ["actorPlayerId", "targetPlayerIds", "reason"],
  );
  if (
    !isNonBlankString(value.id) ||
    value.gameId !== gameId ||
    value.status !== "draft" ||
    !isGameEventType(value.type) ||
    !isPhase(value.phase) ||
    !isVisibility(value.visibility) ||
    !isPlainObject(value.payload) ||
    !isNonBlankString(value.createdAt)
  ) {
    throw new Error("Game draft envelope is invalid");
  }
  validateOptionalEventFields(value);
  if (value.reason !== undefined && typeof value.reason !== "string") {
    throw new Error("Game draft reason is invalid");
  }
  validateCurrentEventPayload(value.type, value.payload);
  return structuredClone(value) as DraftEvent;
}

const GAME_EVENT_TYPES = new Set([
  "phase_started",
  "role_assigned",
  "wolf_leader_selected",
  "wolf_strategy_given",
  "wolf_opinion_given",
  "wolf_vote_cast",
  "wolf_vote_resolved",
  "seer_check_selected",
  "seer_check_result",
  "witch_death_info_shown",
  "witch_antidote_decided",
  "witch_poison_decided",
  "guard_protect_selected",
  "hunter_shot_decided",
  "night_resolved",
  "death_announced",
  "last_words_given",
  "day_speech_given",
  "vote_cast",
  "pk_speech_given",
  "exile_resolved",
  "game_ended",
]);

function isGameEventType(value: unknown): value is GameEvent["type"] {
  return typeof value === "string" && GAME_EVENT_TYPES.has(value);
}

function validateOptionalEventFields(value: Record<string, unknown>): void {
  if (
    value.actorPlayerId !== undefined &&
    !isNonBlankString(value.actorPlayerId)
  ) {
    throw new Error("Game event actorPlayerId is invalid");
  }
  if (
    value.targetPlayerIds !== undefined &&
    (!Array.isArray(value.targetPlayerIds) ||
      value.targetPlayerIds.some((playerId) => !isNonBlankString(playerId)))
  ) {
    throw new Error("Game event targetPlayerIds are invalid");
  }
  if (
    value.createdFromDraftId !== undefined &&
    !isNonBlankString(value.createdFromDraftId)
  ) {
    throw new Error("Game event createdFromDraftId is invalid");
  }
}

function validateCurrentEventPayload(
  type: GameEvent["type"],
  payload: Record<string, unknown>,
): void {
  if (type === "role_assigned") {
    assertExactObjectKeys(payload, "Role assigned payload", [
      "playerId",
      "role",
      "faction",
    ]);
    if (
      !isNonBlankString(payload.playerId) ||
      !isRuleRoleId(payload.role) ||
      payload.faction !== factionForRuleRole(payload.role)
    ) {
      throw new Error("Role assigned payload is invalid");
    }
    return;
  }
  if (type !== "night_resolved") return;
  assertExactObjectKeys(payload, "Night resolved payload", [
    "deadPlayerIds",
    "deaths",
  ]);
  const deadPlayerIds = payload.deadPlayerIds;
  const deaths = payload.deaths;
  if (
    !Array.isArray(deadPlayerIds) ||
    deadPlayerIds.some((playerId) => !isNonBlankString(playerId)) ||
    !Array.isArray(deaths) ||
    deaths.some(
      (death) =>
        !isPlainObject(death) ||
        Object.keys(death).length !== 2 ||
        !isNonBlankString(death.playerId) ||
        (death.reason !== "wolf_kill" && death.reason !== "witch_poison"),
    ) ||
    deadPlayerIds.length !== deaths.length ||
    deadPlayerIds.some(
      (playerId, index) =>
        playerId !== (deaths[index] as Record<string, unknown>).playerId,
    )
  ) {
    throw new Error("Night resolved payload is invalid");
  }
}

function validateRoleAssignmentTruth(
  events: readonly (GameEvent | DraftEvent)[],
  game: Game,
): void {
  for (const event of events) {
    if (event.type !== "role_assigned") continue;
    const player = game.players.find(
      (candidate) => candidate.playerId === event.payload.playerId,
    );
    if (
      !player ||
      event.payload.role !== player.ruleRole.id ||
      event.payload.faction !== player.ruleRole.faction
    ) {
      throw new Error(
        `Role assignment does not match immutable Game snapshot: ${event.payload.playerId}`,
      );
    }
  }
}

function isVisibility(value: unknown): boolean {
  if (!isPlainObject(value)) return false;
  switch (value.kind) {
    case "public":
    case "host_only":
      return Object.keys(value).length === 1;
    case "faction_private":
      return (
        Object.keys(value).length === 2 && value.faction === "wolves"
      );
    case "player_private":
    case "custom":
      return (
        Object.keys(value).length === 2 &&
        Array.isArray(value.playerIds) &&
        value.playerIds.every(isNonBlankString)
      );
    default:
      return false;
  }
}

function isPhase(value: unknown): boolean {
  return (
    value === "setup" ||
    value === "night" ||
    value === "day" ||
    value === "last_words" ||
    value === "speech" ||
    value === "vote" ||
    value === "pk" ||
    value === "ended"
  );
}

function isNonBlankString(value: unknown): value is string {
  return typeof value === "string" && value.trim().length > 0;
}

function isNodeError(error: unknown): error is NodeJS.ErrnoException {
  return error instanceof Error && "code" in error;
}
