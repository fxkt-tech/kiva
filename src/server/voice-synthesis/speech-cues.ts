import type { SpeechCue } from "@/core/voice";
import type { WordBoundary } from "./types";

const DEFAULT_MAX_CUE_CHARACTERS = 42;

export function aggregateSpeechCues(
  boundaries: readonly WordBoundary[],
  audioDurationMs: number,
  maxCharacters = DEFAULT_MAX_CUE_CHARACTERS,
): readonly SpeechCue[] {
  if (boundaries.length === 0) {
    throw new Error("Cannot create speech cues without word boundaries");
  }

  const cues: SpeechCue[] = [];
  let parts: string[] = [];
  let startMs = boundaries[0]!.start;
  let endMs = startMs;

  const flush = () => {
    const text = parts.join("").trim();
    if (text) {
      cues.push({ text, startMs, endMs });
    }
    parts = [];
  };

  for (const boundary of boundaries) {
    if (parts.length === 0) {
      startMs = boundary.start;
    }
    parts.push(boundary.part);
    endMs = boundary.end;
    const text = parts.join("").trim();
    if (/[。！？；.!?;]$/u.test(text) || Array.from(text).length >= maxCharacters) {
      flush();
    }
  }
  flush();

  const last = cues.at(-1);
  if (last && audioDurationMs > last.endMs) {
    cues[cues.length - 1] = { ...last, endMs: audioDurationMs };
  }
  return cues;
}
