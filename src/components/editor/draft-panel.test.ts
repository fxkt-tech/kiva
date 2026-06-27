import React from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import type { DraftEvent } from "@/core/drafts";
import type { GenerationRecord } from "@/core/generation-record";
import { createSeedGame } from "@/core/game";
import type { DraftId, GameId, PlayerId } from "@/core/types";
import { DraftPanel } from "./draft-panel";

const gameId = "game_1" as GameId;
const draftId = "draft_1" as DraftId;
const createdAt = "2026-06-26T00:00:00.000Z";
const game = createSeedGame({ gameId, createdAt });
const players = game.players;

describe("DraftPanel payload controls", () => {
  it("renders player options for target payload drafts", () => {
    const html = renderPanel(wolfKillDraft(players[0].playerId));

    expect(html).toContain('name="targetPlayerId"');
    expect(html).toContain(`value="${players[0].playerId}"`);
    expect(html).toContain("1 · 秦川");
    expect(html).toContain("Save action");
    expect(html).toContain("待确认：狼人刀人");
    expect(html).toContain("选择击杀 1 号");
  });

  it("renders regenerate control for speech drafts", () => {
    const html = renderPanel(daySpeechDraft());

    expect(html).toContain("Regenerate");
  });

  it("does not render regenerate control for non-speech drafts", () => {
    const html = renderPanel(wolfKillDraft(players[0].playerId));

    expect(html).not.toContain("Regenerate");
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
    expect(html).toContain("speech:v1");
    expect(html).not.toContain("old error");
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
  draft: DraftEvent,
  generations: readonly GenerationRecord[] = [],
): string {
  return renderToStaticMarkup(
    React.createElement(DraftPanel, { gameId, draft, players, generations }),
  );
}

function wolfKillDraft(targetPlayerId: PlayerId): DraftEvent {
  return {
    id: draftId,
    gameId,
    status: "draft",
    type: "wolf_kill_selected",
    phase: "night",
    targetPlayerIds: [targetPlayerId],
    visibility: { kind: "faction_private", faction: "wolves" },
    payload: { targetPlayerId },
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
    promptVersion: "speech:v1",
    provider: "mock",
    model: "mock-model",
    inputContextHash: "ctx",
    rawOutput: "{}",
    parsedOutput: input.status === "success" ? { text: "ok" } : null,
    error: input.error,
    createdAt:
      input.status === "success"
        ? "2026-06-26T00:01:00.000Z"
        : "2026-06-26T00:00:00.000Z",
  };
}
