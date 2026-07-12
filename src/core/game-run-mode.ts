export const GAME_RUN_MODES = ["game", "scripted"] as const;

export type GameRunMode = (typeof GAME_RUN_MODES)[number];

export function parseGameRunMode(value: unknown): GameRunMode {
  if (value === "game" || value === "scripted") return value;
  throw new Error(`Invalid game run mode: ${String(value)}`);
}

export function normalizeGameRunMode(value: unknown): GameRunMode {
  return value === undefined ? "game" : parseGameRunMode(value);
}
