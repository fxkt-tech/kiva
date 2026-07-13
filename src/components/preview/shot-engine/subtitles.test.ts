import { describe, expect, it } from "vitest";
import type { PlaybackItem, PlaybackScenePlayer } from "@/core/playback";
import { subtitleCueForScene, subtitleWindows } from "./subtitles";

describe("preview subtitle cues", () => {
  it("splits long Chinese-heavy text into readable windows", () => {
    const windows = subtitleWindows(
      "各位，我是4号林夏。刚才听了夏宇的发言，我觉得需要先确认预言家的信息。这个线索现在很关键。"
        + "不过夏宇，你说第2夜死了1号和2号，狼人刀了他们，那你觉得这两个死亡和你的预言家身份有没有关联呢？"
        + "比如狼人会不会是因为知道你是预言家所以才刀了被你查过的好人？",
    );

    expect(windows.length).toBeGreaterThan(1);
    expect(windows[0]!.length).toBeLessThanOrEqual(3);
  });

  it("supports a narrower deterministic line width", () => {
    const windows = subtitleWindows("一二三四五六七八九十。十一十二十三十四十五。", 6);

    expect(windows.flat().every((line) => Array.from(line).length <= 6)).toBe(
      true,
    );
  });

  it("clamps an invalid line width instead of stalling", () => {
    expect(subtitleWindows("测试", 0)).toEqual([["测", "试"]]);
  });

  it("selects the active subtitle window from scene progress", () => {
    const scene = playbackScene({
      text: "第一句很长很长很长很长很长很长很长。第二句也很长很长很长很长很长很长很长。第三句继续推进。",
    });

    const early = subtitleCueForScene(scene, 0, player());
    const late = subtitleCueForScene(scene, 0.99, player());

    expect(early?.speaker).toBe("4 号 林夏");
    expect(late?.windowIndex).toBe((late?.windowCount ?? 1) - 1);
  });

  it("lets voiced cue text wrap at the rendered subtitle width", () => {
    const text =
      "1号迟木提出首夜主刀候选为8号苏弦，备选10号任野，明确首夜无行为信息时主备为中立取舍，";
    const scene = playbackScene({
      playerVoice: {
        eventId: "event_voice" as never,
        playerId: "p4" as never,
        file: "event_voice.mp3",
        durationMs: 10_000,
        startsAtOffsetMs: 0,
        cues: [{ text, startMs: 0, endMs: 10_000 }],
      },
    });

    expect(subtitleCueForScene(scene, 0.5, player(), 5_000)?.lines).toEqual([
      text,
    ]);
  });

  it("returns null for empty subtitle text", () => {
    expect(subtitleCueForScene(playbackScene({ title: "", text: "" }), 0, null))
      .toBeNull();
  });
});

function playbackScene(overrides: Partial<PlaybackItem> = {}): PlaybackItem {
  return {
    index: 1,
    phase: "speech",
    kind: "speech",
    title: "4 号 林夏发言",
    text: "Text",
    details: [],
    durationMs: 2000,
    startsAtMs: 0,
    players: [player()],
    presenterName: "守夜人",
    presenterAvatar: null,
    transcriptSpeaker: "player",
    presenterCue: { copyKey: "prompt.speech", text: "请4号玩家发言。", values: {} },
    playerVoice: null,
    presenterVoiceClips: [],
    presenterSourceId: "host",
    stage: null,
    ...overrides,
  };
}

function player(): PlaybackScenePlayer {
  return {
    playerId: "player_4" as PlaybackScenePlayer["playerId"],
    seatNo: 4,
    name: "林夏",
    avatar: null,
    roleName: "狼人",
    status: "alive",
    highlighted: true,
  };
}
