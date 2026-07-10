import type { PlaybackItem } from "@/core/playback";

export const VIDEO_COMPOSITION_SCHEMA_VERSION = 1 as const;

export type AudioCueKind =
  | "system-voice"
  | "player-voice"
  | "music"
  | "effect";

export type AudioCue = {
  readonly id: string;
  readonly kind: AudioCueKind;
  readonly src: string;
  readonly startsAtMs: number;
  readonly durationMs: number | null;
  readonly trimStartMs: number;
  readonly volume: number;
};

export type CompositionAssets = {
  readonly fontUrl: string;
  readonly dayBackgroundUrl: string | null;
  readonly nightBackgroundUrl: string | null;
  readonly avatarUrls: Readonly<Record<string, string>>;
};

export type VideoCompositionInput = {
  readonly schemaVersion: typeof VIDEO_COMPOSITION_SCHEMA_VERSION;
  readonly gameId: string;
  readonly gameTitle: string;
  readonly items: readonly PlaybackItem[];
  readonly assets: CompositionAssets;
  readonly audioCues: readonly AudioCue[];
};

export function decodeVideoCompositionInput(
  value: unknown,
): VideoCompositionInput {
  if (!isRecord(value)) {
    throw new Error("Video composition input must be an object.");
  }
  if (value.schemaVersion !== VIDEO_COMPOSITION_SCHEMA_VERSION) {
    throw new Error("Unsupported video composition schema version.");
  }
  if (
    typeof value.gameId !== "string" ||
    typeof value.gameTitle !== "string" ||
    !Array.isArray(value.items) ||
    !value.items.every(isPlaybackItem) ||
    !isCompositionAssets(value.assets) ||
    !Array.isArray(value.audioCues) ||
    !value.audioCues.every(isAudioCue)
  ) {
    throw new Error("Invalid video composition input.");
  }

  return value as VideoCompositionInput;
}

function isPlaybackItem(value: unknown): value is PlaybackItem {
  return (
    isRecord(value) &&
    Number.isInteger(value.index) &&
    typeof value.phase === "string" &&
    isPlaybackKind(value.kind) &&
    typeof value.title === "string" &&
    typeof value.text === "string" &&
    Array.isArray(value.details) &&
    value.details.every((detail) => typeof detail === "string") &&
    isFiniteNonNegative(value.durationMs) &&
    isFiniteNonNegative(value.startsAtMs) &&
    Array.isArray(value.players) &&
    value.players.every(isPlaybackPlayer)
  );
}

function isPlaybackKind(value: unknown): boolean {
  return (
    value === "phase" ||
    value === "announcement" ||
    value === "speech" ||
    value === "vote" ||
    value === "resolution"
  );
}

function isPlaybackPlayer(value: unknown): boolean {
  return (
    isRecord(value) &&
    typeof value.playerId === "string" &&
    Number.isInteger(value.seatNo) &&
    typeof value.name === "string" &&
    isNullableString(value.avatar) &&
    typeof value.roleName === "string" &&
    (value.status === "alive" || value.status === "dead") &&
    typeof value.highlighted === "boolean"
  );
}

function isCompositionAssets(value: unknown): value is CompositionAssets {
  return (
    isRecord(value) &&
    typeof value.fontUrl === "string" &&
    isNullableString(value.dayBackgroundUrl) &&
    isNullableString(value.nightBackgroundUrl) &&
    isStringRecord(value.avatarUrls)
  );
}

function isAudioCue(value: unknown): value is AudioCue {
  return (
    isRecord(value) &&
    typeof value.id === "string" &&
    isAudioCueKind(value.kind) &&
    typeof value.src === "string" &&
    isFiniteNonNegative(value.startsAtMs) &&
    (value.durationMs === null || isFiniteNonNegative(value.durationMs)) &&
    isFiniteNonNegative(value.trimStartMs) &&
    typeof value.volume === "number" &&
    Number.isFinite(value.volume) &&
    value.volume >= 0 &&
    value.volume <= 1
  );
}

function isAudioCueKind(value: unknown): value is AudioCueKind {
  return (
    value === "system-voice" ||
    value === "player-voice" ||
    value === "music" ||
    value === "effect"
  );
}

function isNullableString(value: unknown): value is string | null {
  return value === null || typeof value === "string";
}

function isStringRecord(value: unknown): value is Readonly<Record<string, string>> {
  return (
    isRecord(value) &&
    Object.values(value).every((entry) => typeof entry === "string")
  );
}

function isFiniteNonNegative(value: unknown): value is number {
  return typeof value === "number" && Number.isFinite(value) && value >= 0;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}
