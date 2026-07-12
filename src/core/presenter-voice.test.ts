import { describe, expect, it } from "vitest";
import { presenterPlanForTemplate, resolvePresenterVoiceClips } from "./presenter-voice";
import { edgeVoiceProfile } from "./voice";

describe("presenter voice planning", () => {
  it("prebuilds complete utterances for all 12 seat prompts", () => {
    const plan = presenterPlanForTemplate(
      "prompt.speech",
      "请{seatNo}号玩家开始发言。",
    );
    expect(plan.tokens).toEqual([
      {
        kind: "seat-variant",
        clipIdPrefix: "line.prompt_speech.seat",
        variableName: "seatNo",
      },
    ]);
    expect(Object.keys(plan.literalClips)).toHaveLength(12);
    expect(plan.literalClips["line.prompt_speech.seat.3"]).toBe(
      "请3号玩家开始发言。",
    );
  });

  it("resolves a seat prompt to one continuous presenter clip", () => {
    const manifest = {
      schemaVersion: 1 as const,
      presenterId: "host",
      profile: edgeVoiceProfile("zh-CN-YunxiNeural"),
      clips: {
        "line.prompt_speech.seat.3": {
          file: "prompt-seat-3.mp3",
          text: "请3号玩家开始发言。",
          durationMs: 900,
        },
      },
      plans: {
        "prompt.speech": [
          {
            kind: "seat-variant" as const,
            clipIdPrefix: "line.prompt_speech.seat",
            variableName: "seatNo",
          },
        ],
      },
    } as never;
    expect(
      resolvePresenterVoiceClips(manifest, {
        copyKey: "prompt.speech",
        text: "请3号玩家开始发言。",
        values: { seatNo: "3" },
      }),
    ).toEqual([
      {
        clipId: "line.prompt_speech.seat.3",
        file: "prompt-seat-3.mp3",
        durationMs: 900,
      },
    ]);
  });

  it("prebuilds a complete utterance for a single target seat", () => {
    const plan = presenterPlanForTemplate(
      "action.seer_check",
      "预言家查验{target}。",
    );
    expect(plan.literalClips["line.action_seer_check.seat.12"]).toBe(
      "预言家查验12号。",
    );
    expect(plan.tokens).toEqual([
      {
        kind: "seat-variant",
        clipIdPrefix: "line.action_seer_check.seat",
        variableName: "target",
      },
    ]);
  });
});
