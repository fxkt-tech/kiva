import type { PlaybackItem } from "@/core/playback";
import {
  legacyGameScriptSnapshot,
  validateGameScriptSnapshot,
  type GameScriptSnapshot,
} from "@/core/game-script";

export const VIDEO_COMPOSITION_SCHEMA_VERSION = 4 as const;

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
  readonly script: GameScriptSnapshot;
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
  const normalized =
    value.schemaVersion === 3
      ? {
          ...value,
          schemaVersion: VIDEO_COMPOSITION_SCHEMA_VERSION,
          script: legacyGameScriptSnapshot(),
        }
      : value;
  if (normalized.schemaVersion !== VIDEO_COMPOSITION_SCHEMA_VERSION) {
    throw new Error("Unsupported video composition schema version.");
  }
  if (
    typeof normalized.gameId !== "string" ||
    typeof normalized.gameTitle !== "string" ||
    !isGameScriptSnapshot(normalized.script) ||
    !Array.isArray(normalized.items) ||
    !normalized.items.every(isPlaybackItem) ||
    !isCompositionAssets(normalized.assets) ||
    !Array.isArray(normalized.audioCues) ||
    !normalized.audioCues.every(isAudioCue)
  ) {
    throw new Error("Invalid video composition input.");
  }

  return normalized as VideoCompositionInput;
}

function isGameScriptSnapshot(value: unknown): value is GameScriptSnapshot {
  try {
    validateGameScriptSnapshot(value);
    return true;
  } catch {
    return false;
  }
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
    value.players.every(isPlaybackPlayer) &&
    typeof value.presenterName === "string" &&
    isNullableString(value.presenterAvatar) &&
    (value.transcriptSpeaker === "presenter" ||
      value.transcriptSpeaker === "player") &&
    isPresenterCue(value.presenterCue) &&
    (value.playerVoice === undefined || isPlayerVoice(value.playerVoice)) &&
    (value.stage === undefined || value.stage === null || isStagePresentation(value.stage))
  );
}

function isStagePresentation(value: unknown): boolean {
  if (!isRecord(value)) {
    return false;
  }
  switch (value.kind) {
    case "action":
      return (
        isStageAction(value.action) &&
        isStageActor(value.actor) &&
        isNullableString(value.targetPlayerId) &&
        isStageActionResult(value.result)
      );
    case "night_result":
      return (
        Array.isArray(value.deaths) &&
        value.deaths.every(
          (death) =>
            isRecord(death) &&
            typeof death.playerId === "string" &&
            (death.reason === null ||
              death.reason === "wolf_kill" ||
              death.reason === "witch_poison"),
        )
      );
    case "vote_result":
      return (
        (value.voteType === "exile" || value.voteType === "pk") &&
        (value.outcome === "exiled" ||
          value.outcome === "tied" ||
          value.outcome === "no_exile") &&
        isNullableString(value.exiledPlayerId) &&
        Array.isArray(value.candidates) &&
        value.candidates.every(
          (candidate) =>
            isRecord(candidate) &&
            typeof candidate.playerId === "string" &&
            Number.isInteger(candidate.votes) &&
            (candidate.votes as number) >= 0,
        ) &&
        Number.isInteger(value.abstentions) &&
        (value.abstentions as number) >= 0
      );
    case "game_result":
      return (
        (value.winner === "wolves" || value.winner === "good") &&
        (value.reason === "all_wolves_dead" ||
          value.reason === "all_gods_dead" ||
          value.reason === "all_villagers_dead" ||
          value.reason === "all_good_dead")
      );
    default:
      return false;
  }
}

function isStageAction(value: unknown): boolean {
  return (
    value === "protect" ||
    value === "attack" ||
    value === "inspect" ||
    value === "antidote" ||
    value === "poison"
  );
}

function isStageActor(value: unknown): boolean {
  return (
    isRecord(value) &&
    (value.kind === "wolves" ||
      (value.kind === "player" && typeof value.playerId === "string"))
  );
}

function isStageActionResult(value: unknown): boolean {
  return (
    value === "selected" ||
    value === "good" ||
    value === "wolves" ||
    value === "used" ||
    value === "skipped" ||
    value === "unresolved"
  );
}

function isPlayerVoice(value: unknown): boolean {
  return (
    value === null ||
    (isRecord(value) &&
      typeof value.eventId === "string" &&
      typeof value.playerId === "string" &&
      typeof value.file === "string" &&
      isFiniteNonNegative(value.durationMs) &&
      isFiniteNonNegative(value.startsAtOffsetMs) &&
      Array.isArray(value.cues) &&
      value.cues.every(
        (cue) =>
          isRecord(cue) &&
          typeof cue.text === "string" &&
          isFiniteNonNegative(cue.startMs) &&
          isFiniteNonNegative(cue.endMs) &&
          cue.endMs > cue.startMs,
      ))
  );
}

function isPresenterCue(value: unknown): boolean {
  return (
    isRecord(value) &&
    typeof value.copyKey === "string" &&
    typeof value.text === "string"
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
