import { describe, expect, it } from "vitest";
import type { DraftEvent } from "../drafts";
import type { GameEvent } from "../events";
import { createSeedGame } from "../game";
import { MockLlmClient, type LlmClient } from "../llm";
import { generateSpeechDraft } from "../speech-generation";
import type { DraftId, EventId, GameId } from "../types";

const gameId = "game_1" as GameId;
const createdAt = "2026-06-26T00:00:00.000Z";
const game = createSeedGame({ gameId, createdAt });
const speaker = game.players.find((player) => player.gameRole === "seer")!;
const wolf = game.players.find((player) => player.gameRole === "werewolf")!;
const wolfTeammate = game.players.find(
  (player) =>
    player.gameRole === "werewolf" && player.playerId !== wolf.playerId,
)!;

describe("speech generation", () => {
  it("fills speech draft text from LLM JSON", async () => {
    const result = await generateSpeechDraft({
      game,
      events: [roleAssigned(1)],
      draft: speechDraft("默认发言"),
      llmClient: new MockLlmClient([
        {
          disclosure: "claim",
          text: "我验了 1 号，是查杀。",
          decisionSummary: "公开已有查验结论，推动好人形成明确方向。",
        },
      ]),
      generationId: "generation_1",
      createdAt,
    });

    expect(result.draft.payload).toMatchObject({
      text: "我验了 1 号，是查杀。",
    });
    expect(result.generation).toMatchObject({
      id: "generation_1",
      status: "success",
      playerId: speaker.playerId,
      purpose: "speech",
      request: {
        schemaName: "werewolf_speech_v2",
        systemPrompt: expect.stringContaining("【执行优先级】"),
        messages: expect.arrayContaining([
          expect.objectContaining({
            role: "user",
            content: expect.stringContaining("【当前场景——本轮最高优先级】"),
          }),
        ]),
      },
      parsedOutput: {
        disclosure: "claim",
        text: "我验了 1 号，是查杀。",
        decisionSummary: "公开已有查验结论，推动好人形成明确方向。",
      },
    });
    expect(result.generation?.request?.systemPrompt).toContain(
      speaker.roleSystemPromptSnapshot,
    );
    expect(result.generation?.request?.systemPrompt).toContain(
      "严格区分已确认事实、其他玩家的主张和你自己的推断",
    );
    expect(result.generation?.request?.messages[0]?.content).toContain(
      "decisionSummary",
    );
  });

  it("returns unsupported drafts unchanged without generation record", async () => {
    const draft = roleDraft();
    const result = await generateSpeechDraft({
      game,
      events: [],
      draft,
      llmClient: new MockLlmClient([{ text: "ignored" }]),
      generationId: "generation_1",
      createdAt,
    });

    expect(result).toEqual({ draft, generation: null });
  });

  it("records failure and preserves draft when model output is invalid", async () => {
    const result = await generateSpeechDraft({
      game,
      events: [roleAssigned(1)],
      draft: speechDraft("默认发言"),
      llmClient: invalidTextClient(),
      generationId: "generation_2",
      createdAt,
    });

    expect(result.draft.payload).toMatchObject({ text: "默认发言" });
    expect(result.generation).toMatchObject({
      id: "generation_2",
      status: "failed",
      request: {
        schemaName: "werewolf_speech_v2",
        messages: expect.any(Array),
      },
      parsedOutput: null,
      error: "LLM speech output must include non-empty text",
      attempts: [
        { error: "LLM speech output must include non-empty text" },
        { error: "LLM speech output must include non-empty text" },
      ],
    });
  });

  it("repairs one invalid speech output with a minimal second request", async () => {
    const result = await generateSpeechDraft({
      game,
      events: [roleAssigned(1)],
      draft: speechDraft("默认发言"),
      llmClient: new MockLlmClient([
        { text: "" },
        {
          disclosure: "conceal",
          text: "我会先给出一个后续可以验证的观察方向。",
          decisionSummary: "修复为空的发言文本。",
        },
      ]),
      generationId: "generation_repaired",
      createdAt,
    });

    expect(result.draft.payload).toMatchObject({
      text: "我会先给出一个后续可以验证的观察方向。",
    });
    expect(result.generation).toMatchObject({
      status: "success",
      attempts: [
        { error: "LLM speech output must include non-empty text" },
        {
          request: {
            systemPrompt: expect.stringContaining("只负责修复一个无效的 JSON 输出"),
          },
          error: null,
        },
      ],
    });
  });

  it("repairs an over-limit speech without truncating it", async () => {
    const result = await generateSpeechDraft({
      game,
      events: [roleAssigned(1)],
      draft: speechDraft("默认发言"),
      llmClient: new MockLlmClient([
        {
          disclosure: "claim",
          text: "长".repeat(171),
          decisionSummary: "第一次输出超过长度合同。",
        },
        {
          disclosure: "claim",
          text: "我公开查验结论，并只保留一个关键依据和后续验证方向。",
          decisionSummary: "压缩为本轮必要信息。",
        },
      ]),
      generationId: "generation_length_repaired",
      createdAt,
    });

    expect(result.draft.payload).toMatchObject({
      text: "我公开查验结论，并只保留一个关键依据和后续验证方向。",
    });
    expect(result.generation).toMatchObject({
      status: "success",
      attempts: [
        {
          error:
            "LLM speech text exceeds hard limit: 171 > 170 non-whitespace characters",
        },
        { error: null },
      ],
    });
    expect(
      result.generation?.attempts?.[1]?.request.messages[0]?.content,
    ).toContain("不超过 170 个非空白字符");
  });

  it("preserves the draft when the repaired speech remains over limit", async () => {
    const result = await generateSpeechDraft({
      game,
      events: [roleAssigned(1)],
      draft: speechDraft("默认发言"),
      llmClient: new MockLlmClient([
        {
          disclosure: "claim",
          text: "长".repeat(171),
          decisionSummary: "第一次超长。",
        },
        {
          disclosure: "claim",
          text: "仍".repeat(172),
          decisionSummary: "第二次仍然超长。",
        },
      ]),
      generationId: "generation_length_failed",
      createdAt,
    });

    expect(result.draft.payload).toMatchObject({ text: "默认发言" });
    expect(result.generation).toMatchObject({
      status: "failed",
      error:
        "LLM speech text exceeds hard limit: 172 > 170 non-whitespace characters",
      attempts: [
        {
          error:
            "LLM speech text exceeds hard limit: 171 > 170 non-whitespace characters",
        },
        {
          error:
            "LLM speech text exceeds hard limit: 172 > 170 non-whitespace characters",
        },
      ],
    });
  });

  it("does not reject first-night wolf discussion based on wording", async () => {
    const events = [
      roleAssignedFor(1, wolf),
      roleAssignedFor(2, wolfTeammate),
      {
        ...baseEvent(3),
        type: "phase_started",
        phase: "night",
        visibility: { kind: "public" },
        payload: { phase: "night", dayNumber: 1 },
      },
      {
        ...baseEvent(4),
        type: "wolf_strategy_given",
        phase: "night",
        actorPlayerId: wolfTeammate.playerId,
        visibility: { kind: "faction_private", faction: "wolves" },
        payload: {
          playerId: wolfTeammate.playerId,
          text: "主目标先选 8 号。",
          dayNumber: 1,
        },
      },
    ] satisfies readonly GameEvent[];
    const result = await generateSpeechDraft({
      game,
      events,
      draft: wolfOpinionDraft(),
      llmClient: new MockLlmClient([
        {
          text: "9 号发言风格谨慎，是神职候选，概率更高。",
          decisionSummary: "备选 9 号。",
        },
      ]),
      generationId: "generation_wolf_fact_repair",
      createdAt,
    });

    expect(result.draft.payload).toMatchObject({
      text: expect.stringContaining("概率更高"),
    });
    expect(result.generation).toMatchObject({
      status: "success",
    });
    expect(result.generation).not.toHaveProperty("attempts");
  });

  it("repairs a public special-role speech with an invalid disclosure mode", async () => {
    const result = await generateSpeechDraft({
      game,
      events: [roleAssigned(1)],
      draft: speechDraft("默认发言"),
      llmClient: new MockLlmClient([
        {
          disclosure: "sometimes",
          text: "我先保留身份。",
          decisionSummary: "暂不披露。",
        },
        {
          disclosure: "conceal",
          text: "我先保留身份，关注后续发言。",
          decisionSummary: "暂不披露并留下观察方向。",
        },
      ]),
      generationId: "generation_disclosure_repair",
      createdAt,
    });

    expect(result.generation).toMatchObject({
      status: "success",
      attempts: [
        {
          error:
            "Public special-role speech must include disclosure=conceal or claim",
        },
        { error: null },
      ],
    });
  });

  it("does not compare disclosure mode with natural-language wording", async () => {
    const result = await generateSpeechDraft({
      game,
      events: [roleAssigned(1)],
      draft: speechDraft("默认发言"),
      llmClient: new MockLlmClient([
        {
          disclosure: "conceal",
          text: "我是预言家，昨夜验了 1 号。",
          decisionSummary: "公开查验。",
        },
      ]),
      generationId: "generation_disclosure_consistency_repair",
      createdAt,
    });

    expect(result.generation).toMatchObject({
      status: "success",
    });
    expect(result.generation).not.toHaveProperty("attempts");
  });

  it("does not reject public wolf speech based on semantic wording", async () => {
    const events = [
      roleAssignedFor(1, wolf),
      {
        ...baseEvent(2),
        type: "phase_started",
        phase: "speech",
        visibility: { kind: "public" },
        payload: { phase: "speech", dayNumber: 1 },
      },
    ] satisfies readonly GameEvent[];
    const result = await generateSpeechDraft({
      game,
      events,
      draft: wolfDaySpeechDraft(),
      llmClient: new MockLlmClient([
        {
          text: "昨晚我们狼队刀了 1 号，这是首刀策略。",
          decisionSummary: "泄露真实刀口。",
        },
      ]),
      generationId: "generation_public_wolf_leak_repair",
      createdAt,
    });

    expect(result.draft.payload).toMatchObject({
      text: expect.stringContaining("我们狼队"),
    });
    expect(result.generation).toMatchObject({
      status: "success",
    });
    expect(result.generation).not.toHaveProperty("attempts");
  });

  it("does not reject public claim framing based on semantic wording", async () => {
    const result = await generateSpeechDraft({
      game,
      events: [roleAssigned(1)],
      draft: speechDraft("默认发言"),
      llmClient: new MockLlmClient([
        {
          disclosure: "conceal",
          text: "3 号跳预言家并验 1 号为好人，这是当前最明确的身份信息。",
          decisionSummary: "事实：3 号查验 1 号为好人。",
        },
      ]),
      generationId: "generation_public_claim_repair",
      createdAt,
    });

    expect(result.generation).toMatchObject({
      status: "success",
    });
    expect(result.generation).not.toHaveProperty("attempts");
  });

  it("does not reject quoted attribution based on semantic matching", async () => {
    const source = game.players.find(
      (player) => player.gameRole === "villager",
    )!;
    const events = [
      roleAssigned(1),
      {
        ...baseEvent(2),
        type: "phase_started",
        phase: "speech",
        visibility: { kind: "public" },
        payload: { phase: "speech", dayNumber: 1 },
      },
      {
        ...baseEvent(3),
        type: "day_speech_given",
        phase: "speech",
        actorPlayerId: source.playerId,
        visibility: { kind: "public" },
        payload: {
          playerId: source.playerId,
          text: "暂时没发现矛盾点。",
          dayNumber: 1,
          round: 1,
        },
      },
    ] satisfies readonly GameEvent[];
    const result = await generateSpeechDraft({
      game,
      events,
      draft: speechDraft("默认发言"),
      llmClient: new MockLlmClient([
        {
          disclosure: "conceal",
          text: `${speaker.seatNo} 号仅用“暂时没发现矛盾点”要求 ${source.seatNo} 号回应。`,
          decisionSummary: "回应已有发言。",
        },
      ]),
      generationId: "generation_quote_attribution_repair",
      createdAt,
    });

    expect(result.generation).toMatchObject({
      status: "success",
    });
    expect(result.generation).not.toHaveProperty("attempts");
  });
});

function speechDraft(text: string): Extract<DraftEvent, { type: "day_speech_given" }> {
  return {
    id: "draft_1" as DraftId,
    gameId,
    status: "draft",
    type: "day_speech_given",
    phase: "speech",
    actorPlayerId: speaker.playerId,
    visibility: { kind: "public" },
    payload: {
      playerId: speaker.playerId,
      text,
      dayNumber: 1,
      round: 1,
    },
    createdAt,
  };
}

function roleDraft(): DraftEvent {
  return {
    id: "draft_2" as DraftId,
    gameId,
    status: "draft",
    type: "role_assigned",
    phase: "setup",
    targetPlayerIds: [speaker.playerId],
    visibility: { kind: "player_private", playerIds: [speaker.playerId] },
    payload: {
      playerId: speaker.playerId,
      role: speaker.gameRole,
      faction: speaker.faction,
    },
    createdAt,
  } as DraftEvent;
}

function roleAssigned(index: number): Extract<GameEvent, { type: "role_assigned" }> {
  return roleAssignedFor(index, speaker);
}

function roleAssignedFor(
  index: number,
  player: typeof game.players[number],
): Extract<GameEvent, { type: "role_assigned" }> {
  return {
    ...baseEvent(index),
    type: "role_assigned",
    phase: "setup",
    targetPlayerIds: [player.playerId],
    visibility: { kind: "player_private", playerIds: [player.playerId] },
    payload: {
      playerId: player.playerId,
      role: player.gameRole,
      faction: player.faction,
    },
  };
}

function wolfOpinionDraft(): Extract<
  DraftEvent,
  { type: "wolf_opinion_given" }
> {
  return {
    id: "draft_wolf_opinion" as DraftId,
    gameId,
    status: "draft",
    type: "wolf_opinion_given",
    phase: "night",
    actorPlayerId: wolf.playerId,
    visibility: { kind: "faction_private", faction: "wolves" },
    payload: { playerId: wolf.playerId, text: "", dayNumber: 1 },
    createdAt,
  };
}

function wolfDaySpeechDraft(): Extract<
  DraftEvent,
  { type: "day_speech_given" }
> {
  return {
    id: "draft_wolf_day_speech" as DraftId,
    gameId,
    status: "draft",
    type: "day_speech_given",
    phase: "speech",
    actorPlayerId: wolf.playerId,
    visibility: { kind: "public" },
    payload: {
      playerId: wolf.playerId,
      text: "",
      dayNumber: 1,
      round: 1,
    },
    createdAt,
  };
}

function baseEvent(index: number) {
  return {
    id: `event_${index}` as EventId,
    gameId,
    index,
    status: "active" as const,
    createdAt,
  };
}

function invalidTextClient(): LlmClient {
  return {
    async generateJson(request) {
      return {
        provider: request.modelBinding.provider,
        model: request.modelBinding.model,
        rawText: '{"text":""}',
        parsed: { text: "" },
        usage: null,
      };
    },
  };
}
