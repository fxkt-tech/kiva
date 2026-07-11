import { describe, expect, it } from "vitest";
import type { PlaybackItem } from "@/core/playback";
import { systemVoiceSourceForScene } from "./preview-audio";

describe("preview audio", () => {
  it("uses the resolved presenter cue without matching scene titles", () => {
    expect(
      systemVoiceSourceForScene(
        scene({
          title: "任意标题",
          presenterCue: {
            copyKey: "phase.night",
            text: "天黑请闭眼。",
            voiceFile: "phase_night_start.mp3",
          },
        }),
      ),
    ).toBe("/kivdb-assets/voice/system/phase_night_start.mp3");
    expect(systemVoiceSourceForScene(scene({ title: "第 1 夜开始" }))).toBeNull();
  });
});

function scene(overrides: Partial<PlaybackItem>): PlaybackItem {
  return {
    index: 1,
    phase: "day",
    kind: "announcement",
    title: "发言阶段",
    text: "",
    details: [],
    durationMs: 2200,
    startsAtMs: 0,
    players: [],
    presenterName: "守夜人",
    presenterAvatar: null,
    transcriptSpeaker: "presenter",
    presenterCue: { copyKey: "fallback.announcement", text: "", voiceFile: null },
    ...overrides,
  };
}
