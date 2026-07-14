import { describe, expect, it } from "vitest";
import { createSeedGame } from "../game";
import { MockLlmClient } from "../llm";
import { generateSpeechDraft } from "../speech-generation";
import type { DraftEvent } from "../drafts";
import type { DraftId, GameId } from "../types";

const game = createSeedGame({
  gameId: "speech_pipeline" as GameId,
  createdAt: "2026-07-14T00:00:00.000Z",
});
const villager = game.players.find((player) => player.ruleRole.id === "villager")!;

function draft(): Extract<DraftEvent, { type: "day_speech_given" }> {
  return {
    id: "speech_draft" as DraftId,
    gameId: game.id,
    status: "draft",
    type: "day_speech_given",
    phase: "speech",
    visibility: { kind: "public" },
    actorPlayerId: villager.playerId,
    targetPlayerIds: [],
    payload: { playerId: villager.playerId, dayNumber: 1, round: 1, text: "" },
    createdAt: game.createdAt,
  };
}

describe("speech PlayerIntent pipeline", () => {
  it("separates decision and performance into two recorded LLM stages", async () => {
    const result = await generateSpeechDraft({
      game,
      events: [],
      draft: draft(),
      llmClient: new MockLlmClient([
        {
          objective: "要求补充可核验信息",
          conclusion: "当前不下定论",
          evidenceEventIndexes: [],
          uncertainty: "证据不足",
          disclosure: "not_applicable",
          intendedEffect: "推动下一位补充",
        },
        { text: "现在证据还不够，我先不下定论，后面的人请把能核对的信息说清楚。" },
      ]),
      generationId: "generation_1",
      createdAt: game.createdAt,
    });
    expect(result.draft.type).toBe("day_speech_given");
    if (result.draft.type !== "day_speech_given") return;
    expect(result.draft.payload.text).toContain("证据还不够");
    expect(result.generation?.status).toBe("success");
    expect(result.generation?.stages.map((stage) => stage.stage)).toEqual([
      "decision",
      "performance",
    ]);
    expect(result.generation?.stages[0]?.request.schemaName).toBe("werewolf_speech_intent_v3");
    expect(result.generation?.stages[1]?.request.schemaName).toBe("werewolf_speech_performance_v1");
  });

  it("rejects intent evidence that was not visible before performance", async () => {
    const result = await generateSpeechDraft({
      game,
      events: [],
      draft: draft(),
      llmClient: new MockLlmClient([
        {
          objective: "判断",
          conclusion: "错误引用",
          evidenceEventIndexes: [999],
          uncertainty: null,
          disclosure: "not_applicable",
          intendedEffect: "说服",
        },
        {
          objective: "判断",
          conclusion: "无证据则保留",
          evidenceEventIndexes: [],
          uncertainty: "没有可见证据",
          disclosure: "not_applicable",
          intendedEffect: "继续观察",
        },
        { text: "我手里没有可核验的证据，这轮先保留。" },
      ]),
      generationId: "generation_2",
      createdAt: game.createdAt,
    });
    expect(result.generation?.status).toBe("success");
    expect(result.generation?.stages[0]?.attempts).toHaveLength(2);
  });
});
