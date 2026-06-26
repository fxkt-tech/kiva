import { describe, expect, it } from "vitest";
import { createPlayerSnapshot } from "../player";
import { resolveNightDeaths } from "../rules";
import { deriveGameState } from "../state";
import { projectVisibleEvents } from "../visibility";
import type { GameEvent } from "../events";
import type { EventId, GameId, PlayerId } from "../types";

const gameId = "g1" as GameId;
const wolf1 = "p1" as PlayerId;
const wolf2 = "p2" as PlayerId;
const seer = "p3" as PlayerId;
const witch = "p4" as PlayerId;
const villager1 = "p5" as PlayerId;
const villager2 = "p6" as PlayerId;

const players = [
  createPlayerSnapshot({
    playerId: wolf1,
    seatNo: 1,
    name: "Wolf 1",
    gameRole: "werewolf",
  }),
  createPlayerSnapshot({
    playerId: wolf2,
    seatNo: 2,
    name: "Wolf 2",
    gameRole: "werewolf",
  }),
  createPlayerSnapshot({
    playerId: seer,
    seatNo: 3,
    name: "Seer",
    gameRole: "seer",
  }),
  createPlayerSnapshot({
    playerId: witch,
    seatNo: 4,
    name: "Witch",
    gameRole: "witch",
  }),
  createPlayerSnapshot({
    playerId: villager1,
    seatNo: 5,
    name: "Villager 1",
    gameRole: "villager",
  }),
  createPlayerSnapshot({
    playerId: villager2,
    seatNo: 6,
    name: "Villager 2",
    gameRole: "villager",
  }),
];

function event(index: number, event: Partial<GameEvent>): GameEvent {
  return {
    id: `e${index}` as EventId,
    gameId,
    index,
    status: "active",
    type: "phase_started",
    phase: "night",
    visibility: { kind: "public" },
    payload: { phase: "night", dayNumber: 1 },
    createdAt: "2026-06-26T00:00:00.000Z",
    ...event,
  } as GameEvent;
}

describe("foundation flow", () => {
  it("keeps hidden night truth out of non-owner viewpoints while deriving public death state", () => {
    const dead = resolveNightDeaths({
      wolfKillTargetId: seer,
      antidoteTargetId: null,
      poisonTargetId: null,
    });

    const events = [
      event(1, {
        type: "phase_started",
        phase: "night",
        payload: { phase: "night", dayNumber: 1 },
      }),
      event(2, {
        type: "wolf_kill_selected",
        actorPlayerId: wolf1,
        targetPlayerIds: [seer],
        visibility: { kind: "faction_private", faction: "wolves" },
        payload: { targetPlayerId: seer },
      }),
      event(3, {
        type: "seer_check_result",
        actorPlayerId: seer,
        targetPlayerIds: [wolf1],
        visibility: { kind: "player_private", playerIds: [seer] },
        payload: { targetPlayerId: wolf1, result: "wolves" },
      }),
      event(4, {
        type: "night_resolved",
        visibility: { kind: "host_only" },
        payload: { deadPlayerIds: dead },
      }),
      event(5, {
        type: "death_announced",
        phase: "day",
        visibility: { kind: "public" },
        payload: { deadPlayerIds: dead },
      }),
    ];

    const villagerView = projectVisibleEvents(events, villager1, {
      wolfPlayerIds: [wolf1, wolf2],
    });
    const wolfView = projectVisibleEvents(events, wolf1, {
      wolfPlayerIds: [wolf1, wolf2],
    });
    const seerView = projectVisibleEvents(events, seer, {
      wolfPlayerIds: [wolf1, wolf2],
    });
    const state = deriveGameState(players, events);

    expect(villagerView.map((visibleEvent) => visibleEvent.type)).toEqual([
      "phase_started",
      "death_announced",
    ]);
    expect(wolfView.map((visibleEvent) => visibleEvent.type)).toContain(
      "wolf_kill_selected",
    );
    expect(seerView.map((visibleEvent) => visibleEvent.type)).toContain(
      "seer_check_result",
    );
    expect(state.deadPlayerIds).toEqual([seer]);
  });
});
