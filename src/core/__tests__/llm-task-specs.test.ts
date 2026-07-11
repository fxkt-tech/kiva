import { describe, expect, it } from "vitest";
import type { DraftEvent } from "../drafts";
import {
  isLlmActionDraft,
  isLlmSpeechDraft,
  LLM_ACTION_DRAFT_TYPES,
  LLM_SPEECH_DRAFT_TYPES,
  taskSpecForDraft,
} from "../llm-task-specs";
import type { DraftId, GameId, PlayerId } from "../types";

const gameId = "game_1" as GameId;
const playerId = "p1" as PlayerId;

describe("LLM task specifications", () => {
  it("defines all five speech and seven action draft types", () => {
    expect(LLM_SPEECH_DRAFT_TYPES).toHaveLength(5);
    expect(LLM_ACTION_DRAFT_TYPES).toHaveLength(7);

    for (const draft of supportedDrafts()) {
      const spec = taskSpecForDraft(draft, { hasPriorDaySpeech: false });
      expect(spec.scene.length).toBeGreaterThan(0);
      expect(spec.audience.length).toBeGreaterThan(0);
      expect(spec.objective.length).toBeGreaterThan(0);
      expect(spec.mustCover.length).toBeGreaterThan(0);
      expect(spec.mustNot.length).toBeGreaterThan(0);
    }
  });

  it("distinguishes public wolf speech from faction-private wolf discussion", () => {
    const privateSpec = taskSpecForDraft(draft("wolf_opinion_given"));
    const publicSpec = taskSpecForDraft(draft("day_speech_given"));

    expect(privateSpec.channel).toBe("狼人私聊");
    expect(privateSpec.disclosure).toContain("不需要隐藏");
    expect(publicSpec.channel).toBe("公开发言");
    expect(publicSpec.disclosure).toContain("狼队秘密永远不得直接披露");
  });

  it("gives first and later day speakers different completion goals", () => {
    const first = taskSpecForDraft(draft("day_speech_given"), {
      hasPriorDaySpeech: false,
    });
    const later = taskSpecForDraft(draft("day_speech_given"), {
      hasPriorDaySpeech: true,
    });

    expect(first.progress).toContain("第一位");
    expect(first.objective).toContain("观察框架");
    expect(later.objective).toContain("回应本日已有发言");
  });

  it("splits exile and PK vote semantics and rejects unsupported sheriff voting", () => {
    const exile = taskSpecForDraft(voteDraft("exile"));
    const pk = taskSpecForDraft(voteDraft("pk"));

    expect(exile.scene).toContain("放逐投票");
    expect(exile.channel).toBe("投票选择");
    expect(pk.scene).toContain("PK 投票");
    expect(pk.mustNot).toContain("不得投给非 PK 候选");
    expect(() => taskSpecForDraft(voteDraft("sheriff"))).toThrow(
      "LLM sheriff voting is not supported",
    );
  });

  it("does not classify non-LLM drafts as supported", () => {
    const roleDraft = draft("role_assigned");
    expect(isLlmSpeechDraft(roleDraft)).toBe(false);
    expect(isLlmActionDraft(roleDraft)).toBe(false);
  });

  it("gives every LLM task variant its own semantic contract", () => {
    const cases = [
      [draft("wolf_strategy_given"), "首夜狼队制定整体战术", "speech"],
      [draft("wolf_opinion_given"), "狼队内部讨论", "speech"],
      [draft("day_speech_given"), "依次公开发言", "speech"],
      [draft("last_words_given"), "公开遗言", "speech"],
      [draft("pk_speech_given"), "PK 公开发言", "speech"],
      [draft("guard_protect_selected"), "守卫选择", "target"],
      [draft("wolf_vote_cast"), "狼人密封刀票", "target"],
      [draft("seer_check_selected"), "预言家选择", "target"],
      [draft("witch_antidote_decided"), "女巫决定是否使用解药", "optional_action"],
      [draft("witch_poison_decided"), "女巫决定是否使用毒药", "optional_action"],
      [draft("hunter_shot_decided"), "猎人出局后选择", "target"],
      [voteDraft("exile"), "放逐投票", "target"],
      [voteDraft("pk"), "PK 投票", "target"],
    ] as const;

    for (const [taskDraft, scene, outputKind] of cases) {
      expect(taskSpecForDraft(taskDraft)).toMatchObject({ scene: expect.stringContaining(scene), outputKind });
    }
  });
});

function supportedDrafts(): readonly (Parameters<typeof taskSpecForDraft>[0])[] {
  return [
    ...LLM_SPEECH_DRAFT_TYPES.map((type) => draft(type)),
    ...LLM_ACTION_DRAFT_TYPES.map((type) =>
      type === "vote_cast" ? voteDraft("exile") : draft(type),
    ),
  ];
}

function voteDraft(
  voteType: "sheriff" | "exile" | "pk",
): Extract<DraftEvent, { type: "vote_cast" }> {
  return {
    ...baseDraft("vote_cast"),
    phase: "vote",
    actorPlayerId: playerId,
    visibility: { kind: "host_only" },
    payload: {
      voterPlayerId: playerId,
      targetPlayerId: null,
      dayNumber: 1,
      round: voteType === "pk" ? 2 : 1,
      voteType,
    },
  };
}

function draft<Type extends DraftEvent["type"]>(type: Type): Extract<
  DraftEvent,
  { type: Type }
> {
  const base = baseDraft(type);
  switch (type) {
    case "wolf_strategy_given":
    case "wolf_opinion_given":
    case "day_speech_given":
    case "pk_speech_given":
      return {
        ...base,
        phase: type.startsWith("wolf_") ? "night" : type === "pk_speech_given" ? "pk" : "speech",
        actorPlayerId: playerId,
        visibility: type.startsWith("wolf_")
          ? { kind: "faction_private", faction: "wolves" }
          : { kind: "public" },
        payload: { playerId, text: "", dayNumber: 1, round: 1 },
      } as unknown as Extract<DraftEvent, { type: Type }>;
    case "last_words_given":
      return {
        ...base,
        phase: "last_words",
        actorPlayerId: playerId,
        visibility: { kind: "public" },
        payload: { playerId, text: "", dayNumber: 1, reason: "exile" },
      } as unknown as Extract<DraftEvent, { type: Type }>;
    case "vote_cast":
      return voteDraft("exile") as Extract<DraftEvent, { type: Type }>;
    case "witch_antidote_decided":
    case "witch_poison_decided":
      return {
        ...base,
        phase: "night",
        actorPlayerId: playerId,
        visibility: { kind: "player_private", playerIds: [playerId] },
        payload: { used: false, targetPlayerId: null },
      } as unknown as Extract<DraftEvent, { type: Type }>;
    case "wolf_vote_cast":
      return {
        ...base,
        phase: "night",
        actorPlayerId: playerId,
        visibility: { kind: "host_only" },
        payload: { voterPlayerId: playerId, targetPlayerId: playerId, dayNumber: 1 },
      } as unknown as Extract<DraftEvent, { type: Type }>;
    case "seer_check_selected":
    case "guard_protect_selected":
    case "hunter_shot_decided":
      return {
        ...base,
        phase: type === "hunter_shot_decided" ? "last_words" : "night",
        actorPlayerId: playerId,
        visibility: { kind: "player_private", playerIds: [playerId] },
        payload: { targetPlayerId: playerId },
      } as unknown as Extract<DraftEvent, { type: Type }>;
    default:
      return {
        ...base,
        phase: "setup",
        visibility: { kind: "player_private", playerIds: [playerId] },
        payload: { playerId, role: "villager", faction: "good" },
      } as unknown as Extract<DraftEvent, { type: Type }>;
  }
}

function baseDraft<Type extends DraftEvent["type"]>(type: Type) {
  return {
    id: `draft_${type}` as DraftId,
    gameId,
    status: "draft" as const,
    type,
    createdAt: "2026-06-26T00:00:00.000Z",
  };
}
