import { describe, expect, it } from "vitest";
import { playbackIndexAtMs, type PlaybackItem } from "@/core/playback";
import { buildAudioTimeline } from "../audio/audio-timeline";
import {
  decodeVideoCompositionInput,
  VIDEO_COMPOSITION_SCHEMA_VERSION,
} from "./types";
import {
  compositionDurationInFrames,
  frameToMilliseconds,
  millisecondsToFrame,
  sceneStartFrame,
} from "./timing";
import { legacyGameScriptSnapshot } from "@/core/game-script";

describe("preview v2 composition contracts", () => {
  it("converts time and rounds the duration deterministically", () => {
    expect(millisecondsToFrame(999, 30)).toBe(29);
    expect(frameToMilliseconds(30, 30)).toBe(1000);
    expect(
      compositionDurationInFrames([item({ durationMs: 1001 })], 30),
    ).toBe(31);
    expect(compositionDurationInFrames([], 30)).toBe(1);
  });

  it("seeks inside a scene whose start is not aligned to a video frame", () => {
    const items = [
      item({ index: 0, startsAtMs: 0, durationMs: 23513 }),
      item({ index: 1, startsAtMs: 23513, durationMs: 2761 }),
    ];

    const targetFrame = sceneStartFrame(items[1]!.startsAtMs, 30);
    const landedAtMs = frameToMilliseconds(targetFrame, 30);

    expect(targetFrame).toBe(706);
    expect(playbackIndexAtMs(items, landedAtMs)).toBe(1);
  });

  it("builds sorted presenter clip cues", () => {
    const items = [
      item({
        index: 2,
        startsAtMs: 2000,
        presenterSourceId: "host",
        presenterVoiceClips: [{ clipId: "b", file: "b.mp3", durationMs: 500 }],
      }),
      item({
        index: 1,
        startsAtMs: 0,
        presenterSourceId: "host",
        presenterVoiceClips: [{ clipId: "a", file: "a.mp3", durationMs: 500 }],
      }),
    ];
    const cues = buildAudioTimeline(items);

    expect(cues.map((cue) => cue.startsAtMs)).toEqual([0, 2000]);
    expect(cues[0]?.id).toBe("presenter-voice:1:0:a");
    expect(buildAudioTimeline(items.map((entry) => ({ ...entry, presenterVoiceClips: [] })))).toEqual([]);
  });

  it("rejects unknown persisted schemas", () => {
    expect(() =>
      decodeVideoCompositionInput({ schemaVersion: 99 }),
    ).toThrow("Unsupported");

    expect(
      decodeVideoCompositionInput({
        schemaVersion: VIDEO_COMPOSITION_SCHEMA_VERSION,
        gameId: "game-1",
        gameTitle: "Test",
        script: legacyGameScriptSnapshot(),
        items: [],
        assets: {
          fontUrl: "/font.ttf",
          dayBackgroundUrl: null,
          nightBackgroundUrl: null,
          avatarUrls: {},
        },
        audioCues: [],
      }).gameId,
    ).toBe("game-1");
  });

  it("loads schema v3 compositions with the legacy script presentation", () => {
    const decoded = decodeVideoCompositionInput({
      schemaVersion: 3,
      gameId: "historical-game",
      gameTitle: "Historical",
      items: [],
      assets: {
        fontUrl: "/font.ttf",
        dayBackgroundUrl: null,
        nightBackgroundUrl: null,
        avatarUrls: {},
      },
      audioCues: [],
    });

    expect(decoded.schemaVersion).toBe(VIDEO_COMPOSITION_SCHEMA_VERSION);
    expect(decoded.script.presentation.styleKey).toBe("legacy_v1");
  });

  it("validates optional structured stage presentations", () => {
    const base = {
      schemaVersion: VIDEO_COMPOSITION_SCHEMA_VERSION,
      gameId: "game-1",
      gameTitle: "Test",
      script: legacyGameScriptSnapshot(),
      items: [item({
        stage: {
          kind: "action" as const,
          action: "inspect" as const,
          actor: { kind: "wolves" as const },
          targetPlayerId: null,
          result: "unresolved" as const,
        },
      })],
      assets: {
        fontUrl: "/font.ttf",
        dayBackgroundUrl: null,
        nightBackgroundUrl: null,
        avatarUrls: {},
      },
      audioCues: [],
    };

    expect(decodeVideoCompositionInput(base).items[0]?.stage?.kind).toBe("action");
    expect(() =>
      decodeVideoCompositionInput({
        ...base,
        items: [{ ...base.items[0], stage: { kind: "action", action: "unknown" } }],
      }),
    ).toThrow("Invalid video composition input");
  });
});

function item(overrides: Partial<PlaybackItem> = {}): PlaybackItem {
  return {
    index: 1,
    phase: "day",
    kind: "announcement",
    title: "Record",
    text: "",
    details: [],
    durationMs: 1000,
    startsAtMs: 0,
    players: [],
    presenterName: "守夜人",
    presenterAvatar: null,
    transcriptSpeaker: "presenter",
    presenterCue: { copyKey: "fallback.announcement", text: "" },
    ...overrides,
  };
}
