import { describe, expect, it } from "vitest";
import { createPlayerSnapshot } from "../player";
import { resolveNightDeaths } from "../rules";
import { deriveGameState } from "../state";
import { projectVisibleEvents } from "../visibility";
import type { GameEvent } from "../events";
import type { EventId, GameId, PlayerId } from "../types";

type EventOf<Type extends GameEvent["type"]> = Extract<
  GameEvent,
  { type: Type }
>;

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

function eventBase(index: number) {
  return {
    id: `e${index}` as EventId,
    gameId,
    index,
    status: "active",
    createdAt: "2026-06-26T00:00:00.000Z",
  } as const;
}

describe("foundation flow", () => {
  it("keeps hidden night truth out of non-owner viewpoints while deriving public death state", () => {
    const dead = resolveNightDeaths({
      wolfKillTargetId: seer,
      antidoteTargetId: null,
      poisonTargetId: null,
    });

    const events = [
      {
        ...eventBase(1),
        type: "phase_started",
        phase: "night",
        visibility: { kind: "public" },
        payload: { phase: "night", dayNumber: 1 },
      } satisfies EventOf<"phase_started">,
      {
        ...eventBase(2),
        type: "wolf_kill_selected",
        phase: "night",
        actorPlayerId: wolf1,
        targetPlayerIds: [seer],
        visibility: { kind: "faction_private", faction: "wolves" },
        payload: { targetPlayerId: seer },
      } satisfies EventOf<"wolf_kill_selected">,
      {
        ...eventBase(3),
        type: "seer_check_result",
        phase: "night",
        actorPlayerId: seer,
        targetPlayerIds: [wolf1],
        visibility: { kind: "player_private", playerIds: [seer] },
        payload: { targetPlayerId: wolf1, result: "wolves" },
      } satisfies EventOf<"seer_check_result">,
      {
        ...eventBase(4),
        type: "night_resolved",
        phase: "night",
        visibility: { kind: "host_only" },
        payload: { deadPlayerIds: dead },
      } satisfies EventOf<"night_resolved">,
      {
        ...eventBase(5),
        type: "death_announced",
        phase: "day",
        visibility: { kind: "public" },
        payload: { deadPlayerIds: dead },
      } satisfies EventOf<"death_announced">,
    ] satisfies readonly GameEvent[];

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

    const villagerEventTypes = villagerView.map(
      (visibleEvent) => visibleEvent.type,
    );
    const wolfEventTypes = wolfView.map((visibleEvent) => visibleEvent.type);
    const seerEventTypes = seerView.map((visibleEvent) => visibleEvent.type);

    expect(villagerEventTypes).toEqual([
      "phase_started",
      "death_announced",
    ]);
    expect(villagerEventTypes).not.toContain("wolf_kill_selected");
    expect(villagerEventTypes).not.toContain("seer_check_result");
    expect(villagerEventTypes).not.toContain("night_resolved");

    expect(wolfEventTypes).toContain("wolf_kill_selected");
    expect(wolfEventTypes).not.toContain("seer_check_result");
    expect(wolfEventTypes).not.toContain("night_resolved");

    expect(seerEventTypes).toContain("seer_check_result");
    expect(seerEventTypes).not.toContain("wolf_kill_selected");
    expect(seerEventTypes).not.toContain("night_resolved");

    expect(state.deadPlayerIds).toEqual([seer]);
  });
});
