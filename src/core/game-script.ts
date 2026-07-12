import { isPlainObject } from "./model-binding";

export const GAME_SCRIPT_VISUAL_STYLE_KEYS = [
  "legacy_v1",
  "midnight_archive_v1",
] as const;

export type GameScriptVisualStyleKey =
  (typeof GAME_SCRIPT_VISUAL_STYLE_KEYS)[number];

export type GameScriptPresentation = {
  readonly styleKey: GameScriptVisualStyleKey;
  readonly coverImage: string;
  readonly dayBackground: string;
  readonly nightBackground: string;
  readonly colors: {
    readonly ink: string;
    readonly paper: string;
    readonly accent: string;
    readonly signal: string;
    readonly night: string;
  };
};

export type GameScriptDefinition = {
  readonly id: string;
  readonly name: string;
  readonly theme: string;
  readonly background: string;
  readonly atmosphere: readonly string[];
  readonly presentation: GameScriptPresentation;
  readonly enabled: boolean;
  readonly createdAt: string;
  readonly updatedAt: string;
};

export type GameScriptSnapshot = {
  readonly scriptSourceId: string;
  readonly name: string;
  readonly theme: string;
  readonly background: string;
  readonly atmosphere: readonly string[];
  readonly presentation: GameScriptPresentation;
};

const COLOR_KEYS = ["ink", "paper", "accent", "signal", "night"] as const;

export function validateGameScriptDefinitions(
  input: unknown,
): readonly GameScriptDefinition[] {
  if (!Array.isArray(input)) {
    throw new Error("Game script definitions must be an array");
  }

  const ids = new Set<string>();
  for (const [index, definition] of input.entries()) {
    const path = `Game script definition[${index}]`;
    if (!isPlainObject(definition)) throw new Error(`${path} must be an object`);
    const id = requireStableId(definition.id, `${path} id`);
    if (ids.has(id)) throw new Error(`Duplicate game script definition id: ${id}`);
    ids.add(id);
    validateIdentityAndNarrative(definition, `Game script ${id}`);
    validatePresentation(definition.presentation, `Game script ${id} presentation`, false);
    if (typeof definition.enabled !== "boolean") {
      throw new Error(`Game script ${id} enabled must be a boolean`);
    }
    requireIsoTimestamp(definition.createdAt, `Game script ${id} createdAt`);
    requireIsoTimestamp(definition.updatedAt, `Game script ${id} updatedAt`);
  }

  return input as readonly GameScriptDefinition[];
}

export function createGameScriptSnapshot(
  definition: GameScriptDefinition,
): GameScriptSnapshot {
  validateGameScriptDefinitions([definition]);
  return {
    scriptSourceId: definition.id,
    name: definition.name,
    theme: definition.theme,
    background: definition.background,
    atmosphere: [...definition.atmosphere],
    presentation: structuredClone(definition.presentation),
  };
}

export function validateGameScriptSnapshot(input: unknown): GameScriptSnapshot {
  if (!isPlainObject(input)) throw new Error("Game script snapshot must be an object");
  const sourceId = requireStableId(input.scriptSourceId, "Game script snapshot source id");
  validateIdentityAndNarrative(input, `Game script snapshot ${sourceId}`);
  validatePresentation(
    input.presentation,
    `Game script snapshot ${sourceId} presentation`,
    true,
  );
  return input as GameScriptSnapshot;
}

export function legacyGameScriptSnapshot(): GameScriptSnapshot {
  return {
    scriptSourceId: "legacy",
    name: "经典回放",
    theme: "传统狼人杀记录",
    background: "一场按标准流程进行的狼人杀对局。",
    atmosphere: ["月夜", "庄园", "案卷记录"],
    presentation: {
      styleKey: "legacy_v1",
      coverImage: "/kivdb-assets/preview/night-background.png",
      dayBackground: "/kivdb-assets/preview/day-background.png",
      nightBackground: "/kivdb-assets/preview/night-background.png",
      colors: {
        ink: "#080A09",
        paper: "#FFF7E7",
        accent: "#F0D084",
        signal: "#FF767B",
        night: "#071015",
      },
    },
  };
}

function validateIdentityAndNarrative(
  value: Record<string, unknown>,
  path: string,
): void {
  requireText(value.name, `${path} name`);
  requireText(value.theme, `${path} theme`);
  requireText(value.background, `${path} background`);
  if (
    !Array.isArray(value.atmosphere) ||
    value.atmosphere.length === 0 ||
    value.atmosphere.some((item) => typeof item !== "string" || item.trim() !== item || !item)
  ) {
    throw new Error(`${path} atmosphere must contain trimmed non-blank strings`);
  }
}

function validatePresentation(
  input: unknown,
  path: string,
  allowLegacyAssets: boolean,
): void {
  if (!isPlainObject(input)) throw new Error(`${path} must be an object`);
  if (!(GAME_SCRIPT_VISUAL_STYLE_KEYS as readonly unknown[]).includes(input.styleKey)) {
    throw new Error(`${path} styleKey is invalid`);
  }
  for (const field of ["coverImage", "dayBackground", "nightBackground"] as const) {
    const value = input[field];
    if (typeof value !== "string" || !isScriptAsset(value, allowLegacyAssets)) {
      throw new Error(`${path} ${field} must be a safe internal PNG asset`);
    }
  }
  if (!isPlainObject(input.colors)) throw new Error(`${path} colors must be an object`);
  for (const key of COLOR_KEYS) {
    if (typeof input.colors[key] !== "string" || !/^#[0-9A-F]{6}$/i.test(input.colors[key])) {
      throw new Error(`${path} colors.${key} must be #RRGGBB`);
    }
  }
}

function isScriptAsset(value: string, allowLegacy: boolean): boolean {
  return (
    /^\/kivdb-assets\/scripts\/[a-zA-Z0-9_-]+\.png$/.test(value) ||
    (allowLegacy && /^\/kivdb-assets\/preview\/[a-zA-Z0-9_-]+\.png$/.test(value))
  );
}

function requireStableId(value: unknown, path: string): string {
  if (typeof value !== "string" || !/^[a-zA-Z0-9_-]+$/.test(value)) {
    throw new Error(`${path} must be a stable identifier`);
  }
  return value;
}

function requireText(value: unknown, path: string): void {
  if (typeof value !== "string" || value.trim() !== value || value.length === 0) {
    throw new Error(`${path} must be a trimmed non-blank string`);
  }
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
