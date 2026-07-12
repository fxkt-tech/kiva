import type { VoiceProfileSnapshot } from "@/core/voice";

export type WordBoundary = {
  readonly part: string;
  readonly start: number;
  readonly end: number;
};

export type VoiceSynthesisRequest = {
  readonly sourceText: string;
  readonly profile: VoiceProfileSnapshot;
  readonly outputPath: string;
};

export type VoiceSynthesisResult = {
  readonly synthesizedText: string;
  readonly audioPath: string;
  readonly durationMs: number;
  readonly wordBoundaries: readonly WordBoundary[];
  readonly generator: {
    readonly adapter: "node-edge-tts";
    readonly packageVersion: string;
  };
};

export type VoiceSynthesisAdapter = {
  readonly synthesize: (
    request: VoiceSynthesisRequest,
  ) => Promise<VoiceSynthesisResult>;
};
