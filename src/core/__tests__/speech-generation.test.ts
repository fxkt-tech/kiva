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
const speaker = game.players[2];

describe("speech generation", () => {
  it("fills speech draft text from LLM JSON", async () => {
    const result = await generateSpeechDraft({
      game,
      events: [roleAssigned(1)],
      draft: speechDraft("默认发言"),
      llmClient: new MockLlmClient([
        {
          text: "我验了 1 号，是查杀。",
          reasoning: "我是预言家，需要公开推进自己的查验结论。",
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
        schemaName: "werewolf_speech_v1",
        systemPrompt: expect.stringContaining(
          speaker.characterSystemPromptSnapshot,
        ),
        messages: expect.arrayContaining([
          expect.objectContaining({
            role: "user",
            content: expect.stringContaining("当前要生成的发言"),
          }),
        ]),
      },
      parsedOutput: {
        text: "我验了 1 号，是查杀。",
        reasoning: "我是预言家，需要公开推进自己的查验结论。",
      },
    });
    expect(result.generation?.request?.systemPrompt).toContain(
      speaker.roleSystemPromptSnapshot,
    );
    expect(result.generation?.request?.systemPrompt).toContain(
      "你只能依据用户消息中列出的可见信息发言",
    );
    expect(result.generation?.request?.messages[0]?.content).toContain(
      "reasoning",
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
        schemaName: "werewolf_speech_v1",
        messages: expect.any(Array),
      },
      parsedOutput: null,
      error: "LLM speech output must include non-empty text",
    });
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
  return {
    id: `event_${index}` as EventId,
    gameId,
    index,
    status: "active",
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
      };
    },
  };
}
