import React from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import type { GameEvent } from "@/core/events";
import { createSeedGame } from "@/core/game";
import type { GenerationRecord } from "@/core/generation-record";
import type { DraftId, EventId, GameId, PlayerId } from "@/core/types";
import { EventTimeline } from "./event-timeline";

const gameId = "game_1" as GameId;
const createdAt = "2026-06-26T00:00:00.000Z";
const game = createSeedGame({ gameId, createdAt });
const draftId = "draft_1" as DraftId;

describe("EventTimeline", () => {
  it("renders newest events first", () => {
    const html = renderToStaticMarkup(
      React.createElement(EventTimeline, {
        gameId,
        events: [
          phaseStartedEvent(1),
          phaseStartedEvent(3),
          phaseStartedEvent(2),
        ],
        players: game.players,
        generations: [],
      }),
    );

    expect(html.indexOf("#3")).toBeLessThan(html.indexOf("#2"));
    expect(html.indexOf("#2")).toBeLessThan(html.indexOf("#1"));
  });

  it("shows LLM details for events created from a generated draft", () => {
    const html = renderToStaticMarkup(
      React.createElement(EventTimeline, {
        gameId,
        events: [phaseStartedEvent(1, draftId)],
        players: game.players,
        generations: [generationRecord(draftId, "2026-06-26T00:02:00.000Z")],
      }),
    );

    expect(html).toContain("LLM details");
    expect(html).toContain("success");
    expect(html).toContain("openai-compatible/test-model");
    expect(html).toContain("speech:v1");
  });
});

function phaseStartedEvent(index: number, createdFromDraftId?: DraftId): GameEvent {
  return {
    id: `event_${index}` as EventId,
    gameId,
    index,
    status: "active",
    type: "phase_started",
    phase: "night",
    visibility: { kind: "public" },
    payload: { phase: "night", dayNumber: index },
    createdFromDraftId,
    createdAt: `2026-06-26T00:0${index}:00.000Z`,
  };
}

function generationRecord(draftId: DraftId, createdAt: string): GenerationRecord {
  return {
    id: `gen_${createdAt}`,
    gameId,
    draftId,
    playerId: game.players[0]!.playerId as PlayerId,
    purpose: "speech",
    status: "success",
    promptVersion: "speech:v1",
    provider: "openai-compatible",
    model: "test-model",
    inputContextHash: "hash_1",
    request: {
      schemaName: "speech",
      systemPrompt: "system",
      messages: [{ role: "user", content: "visible context" }],
    },
    tokenUsage: null,
    rawOutput: '{"text":"ok"}',
    parsedOutput: { text: "ok" },
    error: null,
    createdAt,
  };
}
