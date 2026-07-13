import { assertExactObjectKeys, isPlainObject } from "./model-binding";
import type { EventId, PlayerId } from "./types";

export const PRESENTER_EDGE_VOICE = "zh-CN-YunjianNeural";
export const MALE_PLAYER_EDGE_VOICE = "zh-CN-YunxiNeural";
export const FEMALE_PLAYER_EDGE_VOICE = "zh-CN-XiaoxiaoNeural";
export const EDGE_VOICE_RATE = "+20%";

export type EdgeVoiceProfileSnapshot = {
  readonly provider: "edge";
  readonly adapterVersion: 1;
  readonly voice: string;
  readonly lang: string;
  readonly pitch: string;
  readonly rate: string;
  readonly volume: string;
};

export type VoiceProfileSnapshot = EdgeVoiceProfileSnapshot;

export type SpeechCue = {
  readonly text: string;
  readonly startMs: number;
  readonly endMs: number;
};

export type PlayerVoiceArtifact = {
  readonly eventId: EventId;
  readonly playerId: PlayerId;
  readonly sourceTextHash: string;
  readonly synthesizedText: string;
  readonly voiceProfile: VoiceProfileSnapshot;
  readonly generator: {
    readonly adapter: "node-edge-tts";
    readonly packageVersion: string;
  };
  readonly audio: {
    readonly file: string;
    readonly durationMs: number;
  };
  readonly cues: readonly SpeechCue[];
  readonly createdAt: string;
};

export function edgeVoiceProfile(
  voice: string,
  options: Partial<Pick<EdgeVoiceProfileSnapshot, "pitch" | "rate" | "volume">> = {},
): EdgeVoiceProfileSnapshot {
  return {
    provider: "edge",
    adapterVersion: 1,
    voice,
    lang: "zh-CN",
    pitch: options.pitch ?? "+0Hz",
    rate: options.rate ?? "+0%",
    volume: options.volume ?? "+0%",
  };
}

export function validateVoiceProfileSnapshot(
  value: unknown,
  path: string,
): VoiceProfileSnapshot {
  if (!isPlainObject(value)) {
    throw new Error(`${path} must be a valid Edge voice profile`);
  }
  assertExactObjectKeys(value, path, [
    "provider",
    "adapterVersion",
    "voice",
    "lang",
    "pitch",
    "rate",
    "volume",
  ]);
  if (
    value.provider !== "edge" ||
    value.adapterVersion !== 1 ||
    !isNonBlank(value.voice) ||
    !isNonBlank(value.lang) ||
    !isNonBlank(value.pitch) ||
    !isNonBlank(value.rate) ||
    !isNonBlank(value.volume)
  ) {
    throw new Error(`${path} must be a valid Edge voice profile`);
  }
  return value as VoiceProfileSnapshot;
}

export function validatePlayerVoiceArtifact(
  value: unknown,
  eventId: string,
): PlayerVoiceArtifact {
  const path = `Voice artifact ${eventId}`;
  if (!isPlainObject(value)) {
    throw new Error(`${path} is invalid`);
  }
  assertExactObjectKeys(value, path, [
    "eventId",
    "playerId",
    "sourceTextHash",
    "synthesizedText",
    "voiceProfile",
    "generator",
    "audio",
    "cues",
    "createdAt",
  ]);
  if (
    value.eventId !== eventId ||
    !isNonBlank(value.playerId) ||
    !isNonBlank(value.sourceTextHash) ||
    typeof value.synthesizedText !== "string" ||
    !isPlainObject(value.generator) ||
    value.generator.adapter !== "node-edge-tts" ||
    !isNonBlank(value.generator.packageVersion) ||
    !isPlainObject(value.audio) ||
    !isSafeVoiceFile(value.audio.file) ||
    !isPositiveFinite(value.audio.durationMs) ||
    !Array.isArray(value.cues) ||
    !value.cues.every(isSpeechCue) ||
    !isNonBlank(value.createdAt)
  ) {
    throw new Error(`${path} is invalid`);
  }
  assertExactObjectKeys(value.generator, `${path} generator`, [
    "adapter",
    "packageVersion",
  ]);
  assertExactObjectKeys(value.audio, `${path} audio`, ["file", "durationMs"]);
  validateVoiceProfileSnapshot(value.voiceProfile, `${path} voiceProfile`);
  return value as PlayerVoiceArtifact;
}

function isSpeechCue(value: unknown): value is SpeechCue {
  if (!isPlainObject(value)) return false;
  try {
    assertExactObjectKeys(value, "Speech cue", ["text", "startMs", "endMs"]);
  } catch {
    return false;
  }
  return (
    isNonBlank(value.text) &&
    isNonNegativeFinite(value.startMs) &&
    isPositiveFinite(value.endMs) &&
    value.endMs > value.startMs
  );
}

function isSafeVoiceFile(value: unknown): value is string {
  return typeof value === "string" && /^[a-zA-Z0-9_-]+\.mp3$/.test(value);
}

function isNonBlank(value: unknown): value is string {
  return typeof value === "string" && value.trim().length > 0;
}

function isNonNegativeFinite(value: unknown): value is number {
  return typeof value === "number" && Number.isFinite(value) && value >= 0;
}

function isPositiveFinite(value: unknown): value is number {
  return typeof value === "number" && Number.isFinite(value) && value > 0;
}
