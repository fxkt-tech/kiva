import { describe, expect, it } from "vitest";
import type { GameEvent } from "../events";
import { createSeedGame } from "../game";
import { buildPlayerLlmContext } from "../player-context";
import type { EventId, GameId } from "../types";

const gameId = "game_1" as GameId;
const createdAt = "2026-06-26T00:00:00.000Z";
const game = createSeedGame({ gameId, createdAt });
const [wolf1, wolf2, seer, witch, villager] = game.players;

describe("player LLM context", () => {
  it("includes only events visible to the viewer", () => {
    const context = buildPlayerLlmContext({
      game,
      events: sampleEvents(),
      viewerPlayerId: seer.playerId,
    });

    expect(context.visibleEvents.map((event) => event.type)).toEqual([
      "role_assigned",
      "phase_started",
      "seer_check_result",
      "death_announced",
    ]);
    expect(context.timeline.map((item) => item.title)).toContain("查验结果");
    expect(context.timeline.map((item) => item.title)).not.toContain("狼人刀人");
    expect(context.timeline.map((item) => item.title)).not.toContain("夜间结算");
  });

  it("exposes viewer role and prompt snapshots", () => {
    const context = buildPlayerLlmContext({
      game,
      events: [],
      viewerPlayerId: seer.playerId,
    });

    expect(context.viewer).toMatchObject({
      roleName: seer.roleName,
      team: seer.team,
      mechanicKey: seer.mechanicKey,
      characterSystemPromptSnapshot: seer.characterSystemPromptSnapshot,
      roleSystemPromptSnapshot: seer.roleSystemPromptSnapshot,
      roleActionPromptSnapshot: seer.roleActionPromptSnapshot,
      systemPrompt: seer.systemPrompt,
    });
  });

  it("does not expose hidden roles in the safe roster", () => {
    const context = buildPlayerLlmContext({
      game,
      events: sampleEvents(),
      viewerPlayerId: seer.playerId,
    });

    const self = context.roster.find((player) => player.playerId === seer.playerId);
    const wolf = context.roster.find((player) => player.playerId === wolf1.playerId);

    expect(self).toMatchObject({
      role: "seer",
      faction: "good",
    });
    expect(wolf).not.toHaveProperty("role");
    expect(wolf).not.toHaveProperty("faction");
  });

  it("lets wolves know wolf teammates without exposing non-wolf roles", () => {
    const context = buildPlayerLlmContext({
      game,
      events: sampleEvents(),
      viewerPlayerId: wolf1.playerId,
    });

    expect(context.visibleEvents.map((event) => event.type)).toContain(
      "wolf_kill_selected",
    );
    expect(context.visibleEvents.map((event) => event.type)).not.toContain(
      "seer_check_result",
    );
    expect(
      context.roster.find((player) => player.playerId === wolf2.playerId),
    ).toMatchObject({
      role: "werewolf",
      faction: "wolves",
    });
    expect(
      context.roster.find((player) => player.playerId === seer.playerId),
    ).not.toHaveProperty("role");
  });

  it("rejects unknown viewers", () => {
    expect(() =>
      buildPlayerLlmContext({
        game,
        events: [],
        viewerPlayerId: "missing" as typeof seer.playerId,
      }),
    ).toThrow("Viewer player not found");
  });
});

function sampleEvents(): readonly GameEvent[] {
  return [
    roleAssigned(1, wolf1),
    roleAssigned(2, seer),
    {
      ...baseEvent(3),
      type: "phase_started",
      phase: "night",
      visibility: { kind: "public" },
      payload: { phase: "night", dayNumber: 1 },
    },
    {
      ...baseEvent(4),
      type: "wolf_kill_selected",
      phase: "night",
      actorPlayerId: wolf1.playerId,
      targetPlayerIds: [seer.playerId],
      visibility: { kind: "faction_private", faction: "wolves" },
      payload: { targetPlayerId: seer.playerId },
    },
    {
      ...baseEvent(5),
      type: "seer_check_result",
      phase: "night",
      actorPlayerId: seer.playerId,
      targetPlayerIds: [wolf1.playerId],
      visibility: { kind: "player_private", playerIds: [seer.playerId] },
      payload: { targetPlayerId: wolf1.playerId, result: "wolves" },
    },
    {
      ...baseEvent(6),
      type: "night_resolved",
      phase: "night",
      targetPlayerIds: [seer.playerId],
      visibility: { kind: "host_only" },
      payload: { deadPlayerIds: [seer.playerId] },
    },
    {
      ...baseEvent(7),
      type: "death_announced",
      phase: "day",
      targetPlayerIds: [seer.playerId],
      visibility: { kind: "public" },
      payload: { deadPlayerIds: [seer.playerId] },
    },
  ] satisfies readonly GameEvent[];
}

function roleAssigned(index: number, player: typeof game.players[number]) {
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
  } satisfies Extract<GameEvent, { type: "role_assigned" }>;
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
