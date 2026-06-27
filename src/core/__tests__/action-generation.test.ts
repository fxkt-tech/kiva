import { describe, expect, it } from "vitest";
import type { DraftEvent } from "../drafts";
import type { GameEvent } from "../events";
import { createSeedGame } from "../game";
import { MockLlmClient } from "../llm";
import { generateActionDraft } from "../action-generation";
import type { DraftId, EventId, GameId, PlayerId } from "../types";

const gameId = "game_1" as GameId;
const createdAt = "2026-06-26T00:00:00.000Z";
const game = createSeedGame({ gameId, createdAt });
const [wolf, , seer, witch, villager] = game.players;

describe("action generation", () => {
  it("applies legal seer target suggestions", async () => {
    const result = await generateActionDraft({
      game,
      events: setupEvents(),
      draft: seerDraft(villager.playerId),
      llmClient: new MockLlmClient([{ targetPlayerId: wolf.playerId }]),
      generationId: "generation_1",
      createdAt,
    });

    expect(result.draft).toMatchObject({
      type: "seer_check_selected",
      targetPlayerIds: [wolf.playerId],
      payload: { targetPlayerId: wolf.playerId },
    });
    expect(result.generation).toMatchObject({
      status: "success",
      purpose: "action",
      request: {
        schemaName: "werewolf_action_v1",
        systemPrompt: expect.stringContaining("行动建议"),
        messages: expect.arrayContaining([
          expect.objectContaining({
            role: "user",
            content: expect.stringContaining("可选目标"),
          }),
        ]),
      },
      parsedOutput: { targetPlayerId: wolf.playerId },
    });
  });

  it("rejects illegal wolf kill targets and keeps draft unchanged", async () => {
    const draft = wolfDraft(villager.playerId);
    const result = await generateActionDraft({
      game,
      events: setupEvents(),
      draft,
      llmClient: new MockLlmClient([{ targetPlayerId: "missing" }]),
      generationId: "generation_2",
      createdAt,
    });

    expect(result.draft).toEqual(draft);
    expect(result.generation).toMatchObject({
      status: "failed",
      purpose: "action",
      request: {
        schemaName: "werewolf_action_v1",
        messages: expect.any(Array),
      },
      error: "Illegal targetPlayerId for wolf_kill_selected",
    });
  });

  it("applies vote abstain suggestions", async () => {
    const result = await generateActionDraft({
      game,
      events: setupEvents(),
      draft: voteDraft(villager.playerId),
      llmClient: new MockLlmClient([{ targetPlayerId: null }]),
      generationId: "generation_3",
      createdAt,
    });

    expect(result.draft).toMatchObject({
      type: "vote_cast",
      targetPlayerIds: [],
      payload: { targetPlayerId: null },
    });
  });

  it("applies witch medicine suggestions", async () => {
    const result = await generateActionDraft({
      game,
      events: setupEvents(),
      draft: witchPoisonDraft(),
      llmClient: new MockLlmClient([
        { used: true, targetPlayerId: wolf.playerId },
      ]),
      generationId: "generation_4",
      createdAt,
    });

    expect(result.draft).toMatchObject({
      type: "witch_poison_decided",
      targetPlayerIds: [wolf.playerId],
      payload: { used: true, targetPlayerId: wolf.playerId },
    });
  });
});

function setupEvents(): readonly GameEvent[] {
  return [
    roleAssigned(1, wolf.playerId, "werewolf", "wolves"),
    roleAssigned(2, seer.playerId, "seer", "good"),
    roleAssigned(3, witch.playerId, "witch", "good"),
    roleAssigned(4, villager.playerId, "villager", "good"),
    {
      ...baseEvent(5),
      type: "phase_started",
      phase: "night",
      visibility: { kind: "public" },
      payload: { phase: "night", dayNumber: 1 },
    },
  ] satisfies readonly GameEvent[];
}

function seerDraft(targetPlayerId: PlayerId): DraftEvent {
  return {
    ...draftBase("seer_check_selected"),
    phase: "night",
    actorPlayerId: seer.playerId,
    targetPlayerIds: [targetPlayerId],
    visibility: { kind: "player_private", playerIds: [seer.playerId] },
    payload: { targetPlayerId },
  } as DraftEvent;
}

function wolfDraft(targetPlayerId: PlayerId): DraftEvent {
  return {
    ...draftBase("wolf_kill_selected"),
    phase: "night",
    actorPlayerId: wolf.playerId,
    targetPlayerIds: [targetPlayerId],
    visibility: { kind: "faction_private", faction: "wolves" },
    payload: { targetPlayerId },
  } as DraftEvent;
}

function voteDraft(targetPlayerId: PlayerId): DraftEvent {
  return {
    ...draftBase("vote_cast"),
    phase: "vote",
    actorPlayerId: villager.playerId,
    targetPlayerIds: [targetPlayerId],
    visibility: { kind: "public" },
    payload: {
      voterPlayerId: villager.playerId,
      targetPlayerId,
      dayNumber: 1,
      round: 1,
      voteType: "exile",
    },
  } as DraftEvent;
}

function witchPoisonDraft(): DraftEvent {
  return {
    ...draftBase("witch_poison_decided"),
    phase: "night",
    actorPlayerId: witch.playerId,
    targetPlayerIds: [],
    visibility: { kind: "player_private", playerIds: [witch.playerId] },
    payload: { used: false, targetPlayerId: null },
  } as DraftEvent;
}

function draftBase(type: DraftEvent["type"]) {
  return {
    id: "draft_1" as DraftId,
    gameId,
    status: "draft",
    type,
    createdAt,
  } as const;
}

function roleAssigned(
  index: number,
  playerId: PlayerId,
  role: "werewolf" | "seer" | "witch" | "villager",
  faction: "wolves" | "good",
): Extract<GameEvent, { type: "role_assigned" }> {
  return {
    ...baseEvent(index),
    type: "role_assigned",
    phase: "setup",
    targetPlayerIds: [playerId],
    visibility: { kind: "player_private", playerIds: [playerId] },
    payload: { playerId, role, faction },
  };
}

function baseEvent(index: number) {
  return {
    id: `event_${index}` as EventId,
    gameId,
    index,
    status: "active",
    createdAt: `2026-06-26T00:0${index}:00.000Z`,
  } as const;
}
