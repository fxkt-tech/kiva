import { describe, expect, it } from "vitest";
import { createPlayerSnapshot } from "../player";
import { deriveGameState } from "../state";
import type { GameEvent } from "../events";
import type { EventId, GameId, PlayerId } from "../types";

const gameId = "g1" as GameId;
const p1 = "p1" as PlayerId;
const p2 = "p2" as PlayerId;
const p3 = "p3" as PlayerId;
const unknownPlayerId = "unknown" as PlayerId;

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

describe("deriveGameState", () => {
  const players = [
    createPlayerSnapshot({
      playerId: p1,
      seatNo: 1,
      name: "P1",
      gameRole: "werewolf",
    }),
    createPlayerSnapshot({
      playerId: p2,
      seatNo: 2,
      name: "P2",
      gameRole: "werewolf",
    }),
    createPlayerSnapshot({
      playerId: p3,
      seatNo: 3,
      name: "P3",
      gameRole: "seer",
    }),
  ];

  it("starts all players alive and tracks current phase", () => {
    const state = deriveGameState(players, [
      event(1, {
        type: "phase_started",
        phase: "night",
        payload: { phase: "night", dayNumber: 1 },
      }),
    ]);

    expect(state.currentPhase).toBe("night");
    expect(state.dayNumber).toBe(1);
    expect(state.alivePlayerIds).toEqual([p1, p2, p3]);
  });

  it("marks night-resolved deaths as dead", () => {
    const state = deriveGameState(players, [
      event(1, {
        type: "night_resolved",
        payload: { deadPlayerIds: [p3] },
      }),
    ]);

    expect(state.alivePlayerIds).toEqual([p1, p2]);
    expect(state.deadPlayerIds).toEqual([p3]);
  });

  it("tracks witch medicine usage", () => {
    const state = deriveGameState(players, [
      event(1, {
        type: "witch_antidote_decided",
        actorPlayerId: p3,
        payload: { used: true, targetPlayerId: p2 },
      }),
      event(2, {
        type: "witch_poison_decided",
        actorPlayerId: p3,
        payload: { used: false, targetPlayerId: null },
      }),
    ]);

    expect(state.witch.antidoteAvailable).toBe(false);
    expect(state.witch.poisonAvailable).toBe(true);
  });

  it("derives latest phase by active event index", () => {
    const state = deriveGameState(players, [
      event(2, {
        type: "phase_started",
        phase: "day",
        payload: { phase: "day", dayNumber: 2 },
      }),
      event(1, {
        type: "phase_started",
        phase: "night",
        payload: { phase: "night", dayNumber: 1 },
      }),
    ]);

    expect(state.currentPhase).toBe("day");
    expect(state.dayNumber).toBe(2);
  });

  it("ignores superseded deaths and medicine uses", () => {
    const state = deriveGameState(players, [
      event(1, {
        status: "superseded",
        type: "night_resolved",
        payload: { deadPlayerIds: [p3] },
      }),
      event(2, {
        status: "superseded",
        type: "witch_antidote_decided",
        actorPlayerId: p3,
        payload: { used: true, targetPlayerId: p2 },
      }),
    ]);

    expect(state.alivePlayerIds).toEqual([p1, p2, p3]);
    expect(state.deadPlayerIds).toEqual([]);
    expect(state.witch.antidoteAvailable).toBe(true);
  });

  it("throws when a night-resolved death references an unknown player", () => {
    expect(() =>
      deriveGameState(players, [
        event(1, {
          type: "night_resolved",
          payload: { deadPlayerIds: [unknownPlayerId] },
        }),
      ]),
    ).toThrow("Unknown dead player id unknown");
  });

  it("keeps duplicate night-resolved death ids idempotent", () => {
    const state = deriveGameState(players, [
      event(1, {
        type: "night_resolved",
        payload: { deadPlayerIds: [p3, p3] },
      }),
    ]);

    expect(state.alivePlayerIds).toEqual([p1, p2]);
    expect(state.deadPlayerIds).toEqual([p3]);
  });
});
