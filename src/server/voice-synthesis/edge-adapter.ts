import { readFile, rm, stat } from "node:fs/promises";
import { EdgeTTS } from "node-edge-tts";
import { mp3DurationMs } from "./mp3-duration";
import type {
  VoiceSynthesisAdapter,
  VoiceSynthesisResult,
  WordBoundary,
} from "./types";

const PACKAGE_VERSION = "1.2.10";

export function createEdgeVoiceAdapter(input: {
  readonly proxy?: string;
  readonly timeoutMs?: number;
} = {}): VoiceSynthesisAdapter {
  return {
    async synthesize(request): Promise<VoiceSynthesisResult> {
      const synthesizedText = prepareEdgeSpeechText(request.sourceText);
      if (!synthesizedText) {
        throw new Error("Speech text is empty after Edge adaptation");
      }

      const subtitlePath = `${request.outputPath}.json`;
      await Promise.all([
        rm(request.outputPath, { force: true }),
        rm(subtitlePath, { force: true }),
      ]);

      const tts = new EdgeTTS({
        voice: request.profile.voice,
        lang: request.profile.lang,
        outputFormat: "audio-24khz-48kbitrate-mono-mp3",
        pitch: request.profile.pitch,
        rate: request.profile.rate,
        volume: request.profile.volume,
        saveSubtitles: true,
        proxy: input.proxy,
        timeout: input.timeoutMs ?? 60_000,
      });

      try {
        await tts.ttsPromise(synthesizedText, request.outputPath);
        const [audio, subtitleJson, fileInfo] = await Promise.all([
          readFile(request.outputPath),
          readFile(subtitlePath, "utf8"),
          stat(request.outputPath),
        ]);
        if (fileInfo.size === 0) {
          throw new Error("Edge TTS returned an empty audio file");
        }
        const wordBoundaries = decodeWordBoundaries(JSON.parse(subtitleJson));
        const durationMs = Math.ceil(mp3DurationMs(audio));
        if (durationMs <= 0) {
          throw new Error("Edge TTS returned audio with no measurable duration");
        }
        validateBoundaryCoverage(synthesizedText, wordBoundaries);

        return {
          synthesizedText,
          audioPath: request.outputPath,
          durationMs,
          wordBoundaries,
          generator: {
            adapter: "node-edge-tts",
            packageVersion: PACKAGE_VERSION,
          },
        };
      } catch (error) {
        await Promise.all([
          rm(request.outputPath, { force: true }),
          rm(subtitlePath, { force: true }),
        ]);
        throw error;
      } finally {
        await rm(subtitlePath, { force: true });
      }
    },
  };
}

export function prepareEdgeSpeechText(sourceText: string): string {
  return sourceText
    .replace(/[（(]\s*停顿\s*[）)]/gu, "，")
    .replace(/[（(][^（）()]*[）)]/gu, "")
    .replace(/\s+/gu, " ")
    .replace(/\s+([，。！？；：,.!?;:])/gu, "$1")
    .replace(/([。！？；])，/gu, "$1")
    .trim();
}

export function decodeWordBoundaries(value: unknown): readonly WordBoundary[] {
  if (!Array.isArray(value) || !value.every(isWordBoundary)) {
    throw new Error("Edge TTS subtitle metadata is invalid");
  }
  let previousEnd = 0;
  for (const boundary of value) {
    if (boundary.start < previousEnd || boundary.end <= boundary.start) {
      throw new Error("Edge TTS subtitle boundaries are not ordered");
    }
    previousEnd = boundary.end;
  }
  return value;
}

function validateBoundaryCoverage(
  text: string,
  boundaries: readonly WordBoundary[],
): void {
  const expected = normalizeCoverage(text);
  const actual = normalizeCoverage(boundaries.map((item) => item.part).join(""));
  if (!expected || expected !== actual) {
    throw new Error("Edge TTS subtitle boundaries do not cover synthesized text");
  }
}

function isWordBoundary(value: unknown): value is WordBoundary {
  return (
    typeof value === "object" &&
    value !== null &&
    typeof (value as WordBoundary).part === "string" &&
    Number.isFinite((value as WordBoundary).start) &&
    Number.isFinite((value as WordBoundary).end) &&
    (value as WordBoundary).start >= 0
  );
}

function normalizeCoverage(value: string): string {
  return value.replace(/\s+/gu, "");
}
