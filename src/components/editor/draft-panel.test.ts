import React from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import type { DraftEvent } from "@/core/drafts";
import type { GenerationRecord } from "@/core/generation-record";
import { createSeedGame } from "@/core/game";
import { speechBudgetForKey } from "@/core/speech-budget";
import type { DraftId, GameId, PlayerId } from "@/core/types";
import { DraftPanel } from "./draft-panel";

const gameId = "game_1" as GameId;
const draftId = "draft_1" as DraftId;
const createdAt = "2026-06-26T00:00:00.000Z";
const game = createSeedGame({ gameId, createdAt });
const players = game.players;

describe("DraftPanel payload controls", () => {
  it("renders a plain empty state without an automatic next-draft transition", () => {
    const html = renderPanel(null);

    expect(html).toContain("No draft");
    expect(html).not.toContain("Generating next draft");
    expect(html).not.toContain("Generate next draft");
  });

  it("shows initial LLM generation on the generation button without an overlay", () => {
    const html = renderPanel(wolfKillDraft(players[0].playerId));

    expect(html).not.toContain("Generating LLM draft...");
    expect(html).toContain('aria-label="Generating draft"');
    expect(html).toContain("animate-spin");
    expect(html).toContain('aria-label="Enable auto-confirm draft"');
    expect(html).toContain("待确认：狼人密票");
    expect(html).toContain("aria-busy=\"true\"");
  });

  it("renders player options for target payload drafts", () => {
    const html = renderPanel(wolfKillDraft(players[0].playerId));

    expect(html).toContain('name="targetPlayerId"');
    expect(html).toContain(`value="${players[0].playerId}"`);
    expect(html).toContain(`${players[0].seatNo} · ${players[0].actor.identity.name}`);
    expect(html).toContain("Save action");
    expect(html).toContain("待确认：狼人密票");
    expect(html).toContain("投给 1 号");
  });

  it("renders regenerate control for speech drafts", () => {
    const draft = daySpeechDraft();
    const html = renderPanel(draft, [
      generationRecord({ draftId: draft.id, status: "success", error: null }),
    ]);

    expect(html).toContain("Regenerate");
  });

  it("shows the current speech budget and character count", () => {
    const html = renderToStaticMarkup(
      React.createElement(DraftPanel, {
        gameId,
        draft: daySpeechDraft(),
        players,
        alivePlayerIds: players.map((player) => player.playerId),
        generations: [],
        speechBudget: speechBudgetForKey("day_first"),
      }),
    );

    expect(html).toContain("normal · 4/170 字");
  });

  it("renders regenerate control for action drafts", () => {
    const draft = wolfKillDraft(players[0].playerId);
    const html = renderPanel(draft, [
      generationRecord({ draftId: draft.id, status: "success", error: null }),
    ]);

    expect(html).toContain("Regenerate");
  });

  it("renders approved scripted actions as read-only structure", () => {
    const html = renderToStaticMarkup(
      React.createElement(DraftPanel, {
        gameId,
        draft: wolfKillDraft(players[0].playerId),
        players,
        alivePlayerIds: players.map((player) => player.playerId),
        generations: [],
        scriptedStructureLocked: true,
      }),
    );

    expect(html).toContain("此行动由已批准剧本锁定");
    expect(html).not.toContain("Regenerate");
    expect(html).not.toContain('name="targetPlayerId"');
  });

  it("renders latest generation summary for current draft", () => {
    const draft = daySpeechDraft();
    const html = renderPanel(draft, [
      generationRecord({ draftId: draft.id, status: "failed", error: "old error" }),
      generationRecord({ draftId: draft.id, status: "success", error: null }),
    ]);

    expect(html).toContain("Generation");
    expect(html).toContain("success");
    expect(html).toContain("mock/mock-model");
    expect(html).toContain("speech-pipeline:v3");
    expect(html).not.toContain("old error");
  });

  it("renders an LLM details popup for the latest generation", () => {
    const draft = daySpeechDraft();
    const html = renderPanel(draft, [
      generationRecord({ draftId: draft.id, status: "success", error: null }),
    ]);

    expect(html).toContain("LLM details");
    expect(html).toContain('aria-label="Copy LLM details as Markdown"');
    expect(html).toContain("Request");
    expect(html).toContain("system prompt");
    expect(html).toContain("visible context");
    expect(html).toContain("Raw output");
    expect(html).toContain("Parsed output");
    expect(html).toContain("Player reasoning");
    expect(html).toContain("Token usage");
    expect(html).toContain("120 tokens");
  });

  it("renders witch used false field and nullable target select", () => {
    const html = renderPanel(witchAntidoteDraft());

    expect(html).toContain('type="hidden"');
    expect(html).toContain('name="used"');
    expect(html).toContain('value="false"');
    expect(html).toContain('type="checkbox"');
    expect(html).toContain('value=""');
  });

  it("renders abstain option for vote target drafts", () => {
    const html = renderPanel(voteDraft());

    expect(html).toContain("Abstain");
    expect(html).toContain('name="targetPlayerId"');
  });

  it("does not render action payload form for unsupported draft types", () => {
    const html = renderPanel(roleAssignedDraft(players[0].playerId));

    expect(html).not.toContain("Action payload");
    expect(html).not.toContain("Save action");
  });
});

function renderPanel(
  draft: DraftEvent | null,
  generations: readonly GenerationRecord[] = [],
): string {
  return renderToStaticMarkup(
    React.createElement(DraftPanel, {
      gameId,
      draft,
      players,
      alivePlayerIds: players.map((player) => player.playerId),
      generations,
    }),
  );
}

function wolfKillDraft(targetPlayerId: PlayerId): DraftEvent {
  return {
    id: draftId,
    gameId,
    status: "draft",
    type: "wolf_vote_cast",
    phase: "night",
    actorPlayerId: players[0].playerId,
    targetPlayerIds: [targetPlayerId],
    visibility: { kind: "host_only" },
    payload: { voterPlayerId: players[0].playerId, targetPlayerId, dayNumber: 1 },
    createdAt,
  } as DraftEvent;
}

function witchAntidoteDraft(): DraftEvent {
  return {
    id: draftId,
    gameId,
    status: "draft",
    type: "witch_antidote_decided",
    phase: "night",
    targetPlayerIds: [],
    visibility: { kind: "player_private", playerIds: [players[3].playerId] },
    payload: { used: false, targetPlayerId: null },
    createdAt,
  } as DraftEvent;
}

function voteDraft(): DraftEvent {
  return {
    id: draftId,
    gameId,
    status: "draft",
    type: "vote_cast",
    phase: "vote",
    actorPlayerId: players[0].playerId,
    targetPlayerIds: [],
    visibility: { kind: "public" },
    payload: {
      voterPlayerId: players[0].playerId,
      targetPlayerId: null,
      dayNumber: 1,
      round: 1,
      voteType: "exile",
    },
    createdAt,
  } as DraftEvent;
}

function roleAssignedDraft(playerId: PlayerId): DraftEvent {
  return {
    id: draftId,
    gameId,
    status: "draft",
    type: "role_assigned",
    phase: "setup",
    targetPlayerIds: [playerId],
    visibility: { kind: "player_private", playerIds: [playerId] },
    payload: { playerId, role: "werewolf", faction: "wolves" },
    createdAt,
  } as DraftEvent;
}

function daySpeechDraft(): DraftEvent {
  return {
    id: draftId,
    gameId,
    status: "draft",
    type: "day_speech_given",
    phase: "speech",
    actorPlayerId: players[0].playerId,
    visibility: { kind: "public" },
    payload: {
      playerId: players[0].playerId,
      text: "默认发言",
      dayNumber: 1,
      round: 1,
    },
    createdAt,
  } as DraftEvent;
}

function generationRecord(input: {
  readonly draftId: string;
  readonly status: GenerationRecord["status"];
  readonly error: string | null;
}): GenerationRecord {
  return {
    id: `generation_${input.status}`,
    gameId,
    draftId: input.draftId as GenerationRecord["draftId"],
    playerId: players[0].playerId,
    purpose: "speech",
    status: input.status,
    promptVersion: "speech-pipeline:v3",
    provider: "mock",
    model: "mock-model",
    inputContextHash: "ctx",
    request: {
      schemaName: "werewolf_speech_performance_v1",
      systemPrompt: "system prompt",
      messages: [{ role: "user", content: "visible context" }],
    },
    tokenUsage: {
      promptTokens: 100,
      completionTokens: 20,
      totalTokens: 120,
    },
    rawOutput: "{}",
    parsedOutput:
      input.status === "success" ? { text: "ok", reasoning: "because" } : null,
    error: input.error,
    stages: [
      {
        stage: "decision",
        promptVersion: "speech-intent:v3",
        provider: "mock",
        model: "mock-model",
        request: {
          schemaName: "werewolf_speech_intent_v3",
          systemPrompt: "intent system prompt",
          messages: [{ role: "user", content: "visible intent context" }],
        },
        tokenUsage: null,
        rawOutput: '{"objective":"判断"}',
        parsedOutput: { objective: "判断" },
        error: null,
      },
      {
        stage: "performance",
        promptVersion: "speech-performance:v1",
        provider: "mock",
        model: "mock-model",
        request: {
          schemaName: "werewolf_speech_performance_v1",
          systemPrompt: "system prompt",
          messages: [{ role: "user", content: "visible context" }],
        },
        tokenUsage: null,
        rawOutput: "{}",
        parsedOutput: input.status === "success" ? { text: "ok" } : null,
        error: input.error,
      },
    ],
    createdAt:
      input.status === "success"
        ? "2026-06-26T00:01:00.000Z"
        : "2026-06-26T00:00:00.000Z",
  };
}
