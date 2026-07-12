import { isPlainObject } from "./model-binding";
import {
  validateVoiceProfileSnapshot,
  type VoiceProfileSnapshot,
} from "./voice";

export const PRESENTER_LINE_VARIABLES = {
  "phase.setup": [],
  "phase.night": [],
  "phase.day": [],
  "phase.speech": [],
  "phase.vote": [],
  "phase.pk": [],
  "phase.last_words": [],
  "phase.ended": [],
  "action.role_assigned": ["player", "role"],
  "action.wolf_kill": ["target"],
  "action.seer_check": ["target"],
  "action.seer_result.wolves": ["target"],
  "action.seer_result.good": ["target"],
  "action.witch_death_info.killed": ["target"],
  "action.witch_death_info.safe": [],
  "action.witch_antidote.used": ["target"],
  "action.witch_antidote.skipped": [],
  "action.witch_poison.used": ["target"],
  "action.witch_poison.skipped": [],
  "action.guard_protect": ["target"],
  "action.hunter_shot": ["target"],
  "resolution.night.deaths": ["players"],
  "resolution.night.safe": [],
  "announcement.deaths": ["players"],
  "announcement.safe": [],
  "prompt.last_words": ["seatNo"],
  "prompt.speech": ["seatNo"],
  "prompt.pk": ["seatNo"],
  "vote.cast": ["voter", "target"],
  "vote.abstain": ["voter"],
  "resolution.exile.exiled": ["player"],
  "resolution.exile.tied": ["players"],
  "resolution.exile.none": [],
  "resolution.pk.exiled": ["player"],
  "resolution.pk.tied": ["players"],
  "resolution.pk.none": [],
  "game_end.all_wolves_dead": [],
  "game_end.all_gods_dead": [],
  "game_end.all_villagers_dead": [],
  "game_end.all_good_dead": [],
  "fallback.announcement": [],
  "fallback.speech": [],
  "fallback.vote": [],
  "fallback.resolution": [],
} as const satisfies Readonly<Record<string, readonly string[]>>;

export type PresenterCopyKey = keyof typeof PRESENTER_LINE_VARIABLES;

export type PresenterLine = {
  readonly template: string;
  readonly variables: readonly string[];
};

export type PresenterStandardLine = PresenterLine;
export type PresenterSeatLine = PresenterLine;

export type PresenterLineCatalog = {
  readonly [Key in PresenterCopyKey]: PresenterLine;
};

export type PresenterDefinition = {
  readonly id: string;
  readonly name: string;
  readonly avatar: string | null;
  readonly voiceProfile: VoiceProfileSnapshot;
  readonly lines: PresenterLineCatalog;
  readonly enabled: boolean;
  readonly createdAt: string;
  readonly updatedAt: string;
};

export type GamePresenterSnapshot = {
  readonly presenterSourceId: string;
  readonly name: string;
  readonly avatar: string | null;
  readonly lines: PresenterLineCatalog;
};

export const PRESENTER_SEAT_COPY_KEYS = [
  "prompt.last_words",
  "prompt.speech",
  "prompt.pk",
] as const satisfies readonly PresenterCopyKey[];

const PLAYER_PROMPT_KEYS = new Set<PresenterCopyKey>(PRESENTER_SEAT_COPY_KEYS);

export function isPresenterSeatCopyKey(
  key: PresenterCopyKey,
): key is (typeof PRESENTER_SEAT_COPY_KEYS)[number] {
  return PLAYER_PROMPT_KEYS.has(key);
}

export function validatePresenterDefinitions(
  definitions: unknown,
): readonly PresenterDefinition[] {
  if (!Array.isArray(definitions)) {
    throw new Error("Presenter definitions must be an array");
  }

  const ids = new Set<string>();
  for (const [index, definition] of definitions.entries()) {
    const path = `Presenter definition[${index}]`;
    if (!isPlainObject(definition)) {
      throw new Error(`${path} must be an object`);
    }

    const id = requireStableId(definition.id, `${path} id`);
    if (ids.has(id)) {
      throw new Error(`Duplicate presenter definition id: ${id}`);
    }
    ids.add(id);

    validateIdentity(definition, `Presenter ${id}`);
    validateVoiceProfileSnapshot(
      definition.voiceProfile,
      `Presenter ${id} voiceProfile`,
    );
    validatePresenterLines(definition.lines, `Presenter ${id} lines`);

    if (typeof definition.enabled !== "boolean") {
      throw new Error(`Presenter ${id} enabled must be a boolean`);
    }
    requireIsoTimestamp(definition.createdAt, `Presenter ${id} createdAt`);
    requireIsoTimestamp(definition.updatedAt, `Presenter ${id} updatedAt`);
  }

  return definitions as readonly PresenterDefinition[];
}

export function validateGamePresenterSnapshot(
  snapshot: unknown,
): GamePresenterSnapshot {
  if (!isPlainObject(snapshot)) {
    throw new Error("Game presenter snapshot must be an object");
  }

  const sourceId = requireStableId(
    snapshot.presenterSourceId,
    "Game presenter snapshot presenterSourceId",
  );
  validateIdentity(snapshot, `Game presenter ${sourceId}`);
  validatePresenterLines(snapshot.lines, `Game presenter ${sourceId} lines`);
  return snapshot as GamePresenterSnapshot;
}

export function createGamePresenterSnapshot(
  definition: PresenterDefinition,
): GamePresenterSnapshot {
  return {
    presenterSourceId: definition.id,
    name: definition.name,
    avatar: definition.avatar,
    lines: structuredClone(definition.lines),
  };
}

function validateIdentity(
  value: Record<string, unknown>,
  path: string,
): void {
  if (!isNonBlankString(value.name)) {
    throw new Error(`${path} must include a name`);
  }
  if (
    value.avatar !== null &&
    (typeof value.avatar !== "string" || value.avatar.trim().length === 0)
  ) {
    throw new Error(`${path} avatar must be null or a non-blank string`);
  }
}

function validatePresenterLines(value: unknown, path: string): void {
  if (!isPlainObject(value)) {
    throw new Error(`${path} must be an object`);
  }

  const expectedKeys = Object.keys(PRESENTER_LINE_VARIABLES).sort();
  const actualKeys = Object.keys(value).sort();
  if (actualKeys.join("\0") !== expectedKeys.join("\0")) {
    const missing = expectedKeys.filter((key) => !(key in value));
    const unknown = actualKeys.filter(
      (key) => !(key in PRESENTER_LINE_VARIABLES),
    );
    throw new Error(
      `${path} keys are invalid (missing: ${missing.join(", ") || "none"}; unknown: ${unknown.join(", ") || "none"})`,
    );
  }

  for (const key of expectedKeys as PresenterCopyKey[]) {
    validatePresenterLine(value[key], key, `${path}.${key}`);
  }
}

function validatePresenterLine(
  value: unknown,
  key: PresenterCopyKey,
  path: string,
): void {
  if (!isPlainObject(value)) {
    throw new Error(`${path} must be an object`);
  }
  if (!isNonBlankString(value.template)) {
    throw new Error(`${path}.template must be a non-blank string`);
  }

  const expectedVariables = PRESENTER_LINE_VARIABLES[key];
  if (
    !Array.isArray(value.variables) ||
    value.variables.some((variable) => typeof variable !== "string") ||
    value.variables.join("\0") !== expectedVariables.join("\0")
  ) {
    throw new Error(
      `${path}.variables must equal [${expectedVariables.join(", ")}]`,
    );
  }
  validateTemplatePlaceholders(value.template, expectedVariables, path);

}

function validateTemplatePlaceholders(
  template: string,
  expectedVariables: readonly string[],
  path: string,
): void {
  const placeholders = [...template.matchAll(/\{([a-zA-Z][a-zA-Z0-9]*)\}/g)]
    .map((match) => match[1]!)
    .sort();
  const expected = [...expectedVariables].sort();
  const unsupportedBraces = template.replace(
    /\{[a-zA-Z][a-zA-Z0-9]*\}/g,
    "",
  );

  if (
    placeholders.join("\0") !== expected.join("\0") ||
    unsupportedBraces.includes("{") ||
    unsupportedBraces.includes("}")
  ) {
    throw new Error(
      `${path}.template placeholders must equal [${expected.join(", ")}]`,
    );
  }
}

function requireStableId(value: unknown, path: string): string {
  if (!isNonBlankString(value) || value !== value.trim()) {
    throw new Error(`${path} must be a trimmed non-blank string`);
  }
  return value;
}

function requireIsoTimestamp(value: unknown, path: string): void {
  if (
    typeof value !== "string" ||
    Number.isNaN(Date.parse(value)) ||
    new Date(value).toISOString() !== value
  ) {
    throw new Error(`${path} must be an ISO timestamp`);
  }
}

function isNonBlankString(value: unknown): value is string {
  return typeof value === "string" && value.trim().length > 0;
}
