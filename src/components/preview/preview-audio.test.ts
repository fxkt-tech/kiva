import { describe, expect, it } from "vitest";
import type { PlaybackItem } from "@/core/playback";
import { systemVoiceSourceForScene } from "./preview-audio";

describe("preview audio", () => {
  it("maps fixed playback scenes to system voice assets", () => {
    expect(
      systemVoiceSourceForScene(scene({ title: "第 1 夜开始", text: "夜晚开始。" })),
    ).toBe("/kivdb-assets/voice/system/phase_night_start.mp3");
    expect(systemVoiceSourceForScene(scene({ title: "狼人刀人" }))).toBe(
      "/kivdb-assets/voice/system/action_wolf_kill.mp3",
    );
    expect(
      systemVoiceSourceForScene(
        scene({ title: "昨夜死讯", text: "昨夜平安夜，没有玩家死亡。" }),
      ),
    ).toBe("/kivdb-assets/voice/system/announcement_safe_night.mp3");
    expect(
      systemVoiceSourceForScene(
        scene({
          title: "1 号 周知发言",
          kind: "speech",
          players: [player({ seatNo: 1, highlighted: true })],
        }),
      ),
    ).toBe("/kivdb-assets/voice/system/player_1_speech_prompt.mp3");
    expect(
      systemVoiceSourceForScene(
        scene({
          title: "2 号 顾清妍遗言",
          kind: "speech",
          players: [player({ seatNo: 2, highlighted: true })],
        }),
      ),
    ).toBe("/kivdb-assets/voice/system/player_2_last_words_prompt.mp3");
    expect(
      systemVoiceSourceForScene(
        scene({
          title: "3 号 夏宇PK 发言",
          kind: "speech",
          players: [player({ seatNo: 3, highlighted: true })],
        }),
      ),
    ).toBe("/kivdb-assets/voice/system/player_3_pk_speech_prompt.mp3");
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
    ...overrides,
  };
}

function player(
  overrides: Partial<PlaybackItem["players"][number]> &
    Pick<PlaybackItem["players"][number], "seatNo">,
): PlaybackItem["players"][number] {
  return {
    playerId: `player_${overrides.seatNo}` as PlaybackItem["players"][number]["playerId"],
    name: "玩家",
    avatar: null,
    roleName: "平民",
    status: "alive",
    highlighted: false,
    ...overrides,
  };
}
