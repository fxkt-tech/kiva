import { describe, expect, it } from "vitest";
import {
  createFailedGenerationRecord,
  createSuccessfulGenerationRecord,
} from "../generation-record";
import type { DraftId, GameId, PlayerId } from "../types";

const gameId = "game_1" as GameId;
const draftId = "draft_1" as DraftId;
const playerId = "player_1" as PlayerId;
const createdAt = "2026-06-27T00:00:00.000Z";
const modelBinding = {
  provider: "mock",
  model: "mock-model",
  responseFormat: "json",
} as const;
const request = {
  schemaName: "werewolf_speech_v2",
  systemPrompt: "system prompt",
  messages: [{ role: "user", content: "visible context" }],
} as const;
const attempts = [
  {
    request,
    tokenUsage: null,
    rawOutput: '{"text":""}',
    parsedOutput: { text: "" },
    error: "text must not be empty",
  },
  {
    request: {
      ...request,
      systemPrompt: "repair prompt",
    },
    tokenUsage: null,
    rawOutput: '{"text":"hello"}',
    parsedOutput: { text: "hello" },
    error: null,
  },
] as const;

describe("generation records", () => {
  it("creates success records for parsed model output", () => {
    expect(
      createSuccessfulGenerationRecord({
        id: "generation_1",
        gameId,
        draftId,
        playerId,
        purpose: "speech",
        promptVersion: "speech:v2",
        modelBinding,
        inputContextHash: "ctx_hash",
        request,
        rawOutput: '{"text":"hello"}',
        parsedOutput: { text: "hello" },
        createdAt,
      }),
    ).toEqual({
      id: "generation_1",
      gameId,
      draftId,
      playerId,
      purpose: "speech",
      status: "success",
      promptVersion: "speech:v2",
      provider: "mock",
      model: "mock-model",
      inputContextHash: "ctx_hash",
      request,
      tokenUsage: null,
      rawOutput: '{"text":"hello"}',
      parsedOutput: { text: "hello" },
      error: null,
      createdAt,
    });
  });

  it("stores successful token usage", () => {
    expect(
      createSuccessfulGenerationRecord({
        id: "generation_1",
        gameId,
        draftId,
        playerId,
        purpose: "speech",
        promptVersion: "speech:v2",
        modelBinding,
        inputContextHash: "ctx_hash",
        request,
        tokenUsage: {
          promptTokens: 100,
          completionTokens: 20,
          totalTokens: 120,
          cachedPromptTokens: 10,
          reasoningTokens: 3,
        },
        rawOutput: '{"text":"hello"}',
        parsedOutput: { text: "hello" },
        createdAt,
      }).tokenUsage,
    ).toEqual({
      promptTokens: 100,
      completionTokens: 20,
      totalTokens: 120,
      cachedPromptTokens: 10,
      reasoningTokens: 3,
    });
  });

  it("stores all validation attempts when repair was needed", () => {
    const record = createSuccessfulGenerationRecord({
      id: "generation_repaired",
      gameId,
      draftId,
      playerId,
      purpose: "speech",
      promptVersion: "speech:v2",
      modelBinding,
      inputContextHash: "ctx_hash",
      request,
      rawOutput: '{"text":"hello"}',
      parsedOutput: { text: "hello" },
      createdAt,
      attempts,
    });

    expect(record.attempts).toEqual(attempts);
  });

  it("creates failure records without parsed output", () => {
    expect(
      createFailedGenerationRecord({
        id: "generation_2",
        gameId,
        draftId,
        playerId,
        purpose: "speech",
        promptVersion: "speech:v2",
        modelBinding,
        inputContextHash: "ctx_hash",
        request,
        rawOutput: "not json",
        error: new Error("bad output"),
        createdAt,
      }),
    ).toMatchObject({
      id: "generation_2",
      status: "failed",
      parsedOutput: null,
      request,
      tokenUsage: null,
      rawOutput: "not json",
      error: "bad output",
    });
  });

  it("stores failed repair attempts", () => {
    const record = createFailedGenerationRecord({
      id: "generation_failed_repair",
      gameId,
      draftId,
      playerId,
      purpose: "speech",
      promptVersion: "speech:v2",
      modelBinding,
      inputContextHash: "ctx_hash",
      request,
      rawOutput: '{"text":""}',
      error: new Error("text must not be empty"),
      createdAt,
      attempts: attempts.slice(0, 1),
    });

    expect(record.attempts).toEqual(attempts.slice(0, 1));
  });
});
