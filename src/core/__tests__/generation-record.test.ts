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
  temperature: 0.7,
  maxTokens: 500,
  responseFormat: "json",
} as const;

describe("generation records", () => {
  it("creates success records for parsed model output", () => {
    expect(
      createSuccessfulGenerationRecord({
        id: "generation_1",
        gameId,
        draftId,
        playerId,
        purpose: "speech",
        promptVersion: "speech:v1",
        modelBinding,
        inputContextHash: "ctx_hash",
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
      promptVersion: "speech:v1",
      provider: "mock",
      model: "mock-model",
      inputContextHash: "ctx_hash",
      rawOutput: '{"text":"hello"}',
      parsedOutput: { text: "hello" },
      error: null,
      createdAt,
    });
  });

  it("creates failure records without parsed output", () => {
    expect(
      createFailedGenerationRecord({
        id: "generation_2",
        gameId,
        draftId,
        playerId,
        purpose: "speech",
        promptVersion: "speech:v1",
        modelBinding,
        inputContextHash: "ctx_hash",
        rawOutput: "not json",
        error: new Error("bad output"),
        createdAt,
      }),
    ).toMatchObject({
      id: "generation_2",
      status: "failed",
      parsedOutput: null,
      rawOutput: "not json",
      error: "bad output",
    });
  });
});
