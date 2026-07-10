import { describe, expect, it } from "vitest";
import type { PlaybackItem } from "@/core/playback";
import {
  buildAudioTimeline,
  systemVoiceCueForItem,
} from "../audio/audio-timeline";
import {
  decodeVideoCompositionInput,
  VIDEO_COMPOSITION_SCHEMA_VERSION,
} from "./types";
import {
  compositionDurationInFrames,
  frameToMilliseconds,
  millisecondsToFrame,
} from "./timing";

describe("preview v2 composition contracts", () => {
  it("converts time and rounds the duration deterministically", () => {
    expect(millisecondsToFrame(999, 30)).toBe(29);
    expect(frameToMilliseconds(30, 30)).toBe(1000);
    expect(
      compositionDurationInFrames([item({ durationMs: 1001 })], 30),
    ).toBe(31);
    expect(compositionDurationInFrames([], 30)).toBe(1);
  });

  it("builds sorted optional audio cues", () => {
    const items = [
      item({ index: 2, startsAtMs: 2000 }),
      item({ index: 1, startsAtMs: 0 }),
    ];
    const cues = buildAudioTimeline(items, (scene) => ({
      kind: "effect",
      src: "/effect.mp3",
      startsAtMs: scene.startsAtMs,
      durationMs: 500,
      trimStartMs: 0,
      volume: 0.5,
    }));

    expect(cues.map((cue) => cue.startsAtMs)).toEqual([0, 2000]);
    expect(cues[0]?.id).toBe("effect:1:0");
    expect(buildAudioTimeline(items, () => null)).toEqual([]);
  });

  it("bounds system voice cues to their playback scene", () => {
    const scene = item({
      title: "第 1 夜开始",
      durationMs: 2160,
    });

    expect(systemVoiceCueForItem(scene)?.durationMs).toBe(2160);
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
    ...overrides,
  };
}
