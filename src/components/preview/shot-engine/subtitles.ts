import type { PlaybackItem, PlaybackScenePlayer } from "@/core/playback";
import { clamp01 } from "./animation";

export type SubtitleCue = {
  readonly speaker: string;
  readonly lines: readonly string[];
  readonly windowIndex: number;
  readonly windowCount: number;
};

const MAX_LINE_CHARS = 31;
const MAX_LINES = 3;

export function subtitleCueForScene(
  scene: PlaybackItem,
  progress: number,
  speaker: PlaybackScenePlayer | null,
  sceneMs?: number,
): SubtitleCue | null {
  if (scene.playerVoice && sceneMs !== undefined) {
    const voiceMs = sceneMs - scene.playerVoice.startsAtOffsetMs;
    const cueIndex = scene.playerVoice.cues.findIndex(
      (cue) => voiceMs >= cue.startMs && voiceMs < cue.endMs,
    );
    if (cueIndex < 0) return null;
    const cue = scene.playerVoice.cues[cueIndex]!;
    return {
      speaker: speaker ? `${speaker.seatNo} 号 ${speaker.name}` : scene.title,
      lines: [cue.text],
      windowIndex: cueIndex,
      windowCount: scene.playerVoice.cues.length,
    };
  }
  const sourceText = scene.text || scene.title;
  if (!sourceText.trim()) {
    return null;
  }

  const windows = subtitleWindows(sourceText);
  if (windows.length === 0) {
    return null;
  }

  const windowIndex = Math.min(
    windows.length - 1,
    Math.floor(clamp01(progress) * windows.length),
  );

  return {
    speaker: speaker ? `${speaker.seatNo} 号 ${speaker.name}` : scene.title,
    lines: windows[windowIndex]!,
    windowIndex,
    windowCount: windows.length,
  };
}

export function subtitleWindows(
  text: string,
  maxLineChars = MAX_LINE_CHARS,
): readonly (readonly string[])[] {
  const normalized = text.replace(/\s+/g, " ").trim();
  if (!normalized) {
    return [];
  }

  const lineWidth = Number.isFinite(maxLineChars)
    ? Math.max(1, Math.floor(maxLineChars))
    : MAX_LINE_CHARS;
  const phrases = splitByPunctuation(normalized);
  const lines = phrases.flatMap((phrase) => splitByLength(phrase, lineWidth));
  const windows: string[][] = [];

  for (let index = 0; index < lines.length; index += MAX_LINES) {
    windows.push(lines.slice(index, index + MAX_LINES));
  }

  return windows;
}

function splitByPunctuation(text: string): readonly string[] {
  const segments: string[] = [];
  let segment = "";

  for (const character of Array.from(text)) {
    segment += character;
    if (/[。！？；.!?;]/u.test(character)) {
      segments.push(segment.trim());
      segment = "";
    }
  }

  if (segment.trim()) {
    segments.push(segment.trim());
  }

  return segments;
}

function splitByLength(text: string, maxChars: number): readonly string[] {
  const characters = Array.from(text);
  const lines: string[] = [];

  for (let index = 0; index < characters.length; index += maxChars) {
    lines.push(characters.slice(index, index + maxChars).join(""));
  }

  return lines;
}
