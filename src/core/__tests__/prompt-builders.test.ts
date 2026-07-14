import { describe, expect, it } from "vitest";
import { createSeedGame } from "../game";
import { buildPlayerLlmContext } from "../player-context";
import {
  buildActionPrompt,
  buildSpeechIntentPrompt,
  buildSpeechPerformancePrompt,
} from "../prompt-builders";
import type { DraftEvent } from "../drafts";
import type { DraftId, GameId } from "../types";

const game = createSeedGame({
  gameId: "prompt_game" as GameId,
  createdAt: "2026-07-14T00:00:00.000Z",
});
const villager = game.players.find((player) => player.ruleRole.id === "villager")!;

function speechDraft(): Extract<DraftEvent, { type: "day_speech_given" }> {
  return {
    id: "speech" as DraftId,
    gameId: game.id,
    status: "draft",
    type: "day_speech_given",
    phase: "speech",
    visibility: { kind: "public" },
    actorPlayerId: villager.playerId,
    targetPlayerIds: [],
    payload: {
      playerId: villager.playerId,
      dayNumber: 1,
      round: 1,
      text: "",
    },
    createdAt: game.createdAt,
  };
}

describe("v3 prompt builders", () => {
  it("builds a decision prompt from Actor runtime fields and code Rule Role", () => {
    const context = buildPlayerLlmContext({ game, events: [], viewerPlayerId: villager.playerId });
    const prompt = buildSpeechIntentPrompt({ context, draft: speechDraft() });
    expect(prompt.schemaName).toBe("werewolf_speech_intent_v3");
    expect(prompt.systemPrompt).toContain(villager.actor.core.stableCore);
    expect(prompt.systemPrompt).toContain(villager.ruleRole.name);
    expect(prompt.systemPrompt).not.toContain("characterSystemPromptSnapshot");
    expect(prompt.messages[0]!.content).toContain("evidenceEventIndexes");
    expect(prompt.messages[0]!.content).not.toContain('"text"');
  });

  it("builds a performance request from intent, selected evidence, and expression only", () => {
    const context = buildPlayerLlmContext({ game, events: [], viewerPlayerId: villager.playerId });
    const prompt = buildSpeechPerformancePrompt({
      context,
      draft: speechDraft(),
      intent: {
        objective: "要求补充信息",
        conclusion: "当前不下定论",
        evidenceEventIndexes: [],
        uncertainty: "证据不足",
        disclosure: "not_applicable",
        intendedEffect: "推动发言",
      },
      evidence: [],
      speechBudget: {
        key: "day_first",
        tier: "normal",
        targetMinCharacters: 20,
        targetMaxCharacters: 80,
        hardMaxCharacters: 100,
      },
    });
    expect(prompt.schemaName).toBe("werewolf_speech_performance_v1");
    expect(prompt.systemPrompt).toContain(villager.actor.expression.cadence);
    expect(prompt.messages[0]!.content).toContain("无；不得自行补证据");
    expect(prompt.messages[0]!.content).not.toContain(villager.actor.cognition.evidencePolicy);
  });

  it("keeps action generation as a single decision stage", () => {
    const context = buildPlayerLlmContext({ game, events: [], viewerPlayerId: villager.playerId });
    const draft = {
      ...speechDraft(),
      id: "vote" as DraftId,
      type: "vote_cast" as const,
      phase: "vote" as const,
      payload: { voterPlayerId: villager.playerId, dayNumber: 1, round: 1, voteType: "exile" as const, targetPlayerId: null },
    };
    const prompt = buildActionPrompt({
      context,
      draft,
      options: { targetPlayerIds: [], allowNoTarget: true },
    });
    expect(prompt.schemaName).toBe("werewolf_target_action_v3");
  });
});
