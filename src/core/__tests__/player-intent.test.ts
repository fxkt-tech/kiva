import { describe, expect, it } from "vitest";
import type { DraftEvent } from "../drafts";
import type { GameEvent } from "../events";
import { createSeedGame } from "../game";
import { buildPlayerLlmContext } from "../player-context";
import {
  selectedIntentEvidence,
  validatePlayerSpeechIntent,
} from "../player-intent";
import {
  buildSpeechIntentPrompt,
  buildSpeechPerformancePrompt,
} from "../prompt-builders";
import type { DraftId, EventId, GameId } from "../types";

const game = createSeedGame({
  gameId: "intent_game" as GameId,
  createdAt: "2026-07-14T00:00:00.000Z",
});
const seer = game.players.find((player) => player.ruleRole.id === "seer")!;
const targets = game.players.filter((player) => player.playerId !== seer.playerId);
const events: readonly GameEvent[] = [
  seerResult(2, targets[0]!.playerId, targets[0]!.ruleRole.faction),
  seerResult(3, targets[1]!.playerId, targets[1]!.ruleRole.faction),
];
const context = buildPlayerLlmContext({
  game,
  events,
  viewerPlayerId: seer.playerId,
});
const intentPrompt = buildSpeechIntentPrompt({
  context,
  draft: speechDraft(),
});

describe("PlayerIntent evidence boundary", () => {
  it("keeps private evidence out of a concealed public intent", () => {
    expect(() =>
      validatePlayerSpeechIntent({
        value: intent("conceal", [2]),
        evidenceScope: intentPrompt.evidenceScope,
        publicSpeech: true,
        requireDisclosure: true,
      }),
    ).toThrow("selected evidence outside visibility");
  });

  it("allows an explicit claim to select private evidence", () => {
    const value = validatePlayerSpeechIntent({
      value: intent("claim", [2]),
      evidenceScope: intentPrompt.evidenceScope,
      publicSpeech: true,
      requireDisclosure: true,
    });

    expect(
      selectedIntentEvidence({
        intent: value,
        evidenceScope: intentPrompt.evidenceScope,
        publicSpeech: true,
      }),
    ).toMatchObject([{ index: 2, type: "seer_check_result" }]);
  });

  it("projects only selected evidence into the performance request", () => {
    const value = validatePlayerSpeechIntent({
      value: intent("claim", [2]),
      evidenceScope: intentPrompt.evidenceScope,
      publicSpeech: true,
      requireDisclosure: true,
    });
    const evidence = selectedIntentEvidence({
      intent: value,
      evidenceScope: intentPrompt.evidenceScope,
      publicSpeech: true,
    });
    const prompt = buildSpeechPerformancePrompt({
      context,
      draft: speechDraft(),
      intent: value,
      evidence,
      speechBudget: {
        key: "day_first",
        tier: "normal",
        targetMinCharacters: 20,
        targetMaxCharacters: 80,
        hardMaxCharacters: 100,
      },
    });
    const content = prompt.messages[0]!.content;

    expect(content).toContain("[事件 2]");
    expect(content).not.toContain("[事件 3]");
    expect(content).not.toContain("你的私有事实");
  });

  it("rejects evidence omitted from the actual prompt budget", () => {
    const source = context.knowledge.privateFacts[0]!;
    const boundedContext = {
      ...context,
      knowledge: {
        ...context.knowledge,
        publicFacts: Array.from({ length: 41 }, (_, index) => ({
          ...source,
          index: index + 1,
        })),
        privateFacts: [],
      },
    };
    const prompt = buildSpeechIntentPrompt({
      context: boundedContext,
      draft: speechDraft(),
    });

    expect(prompt.evidenceScope.publicFacts).toHaveLength(40);
    expect(prompt.evidenceScope.publicFacts[0]?.index).toBe(2);
    expect(prompt.messages[0]!.content).not.toContain("[事件 1]");
    expect(() =>
      validatePlayerSpeechIntent({
        value: intent("conceal", [1]),
        evidenceScope: prompt.evidenceScope,
        publicSpeech: true,
        requireDisclosure: true,
      }),
    ).toThrow("selected evidence outside visibility");
  });
});

function intent(
  disclosure: "conceal" | "claim",
  evidenceEventIndexes: readonly number[],
) {
  return {
    objective: "公开当前查验判断",
    conclusion: "给出一项可核验结论",
    evidenceEventIndexes,
    uncertainty: "其他玩家身份仍未知",
    disclosure,
    intendedEffect: "让全场据此更新判断",
  };
}

function speechDraft(): Extract<DraftEvent, { type: "day_speech_given" }> {
  return {
    id: "intent_speech" as DraftId,
    gameId: game.id,
    status: "draft",
    type: "day_speech_given",
    phase: "speech",
    visibility: { kind: "public" },
    actorPlayerId: seer.playerId,
    targetPlayerIds: [],
    payload: {
      playerId: seer.playerId,
      dayNumber: 1,
      round: 1,
      text: "",
    },
    createdAt: game.createdAt,
  };
}

function seerResult(
  index: number,
  targetPlayerId: (typeof game.players)[number]["playerId"],
  result: "wolves" | "good",
): Extract<GameEvent, { type: "seer_check_result" }> {
  return {
    id: `event_${index}` as EventId,
    gameId: game.id,
    index,
    status: "active",
    type: "seer_check_result",
    phase: "night",
    actorPlayerId: seer.playerId,
    targetPlayerIds: [targetPlayerId],
    visibility: { kind: "player_private", playerIds: [seer.playerId] },
    payload: { targetPlayerId, result },
    createdAt: game.createdAt,
  };
}
