import { describe, expect, it } from "vitest";
import { deriveGameState } from "../state";
import type { GameEvent } from "../events";
import type { EventId, GameId, PlayerId } from "../types";
import { testPlayer } from "./test-player";

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
    testPlayer(p1, 1, "werewolf"),
    testPlayer(p2, 2, "werewolf"),
    testPlayer(p3, 3, "seer"),
  ];

  it("derives default state when there are no events", () => {
    const state = deriveGameState(players, []);

    expect(state.currentPhase).toBe("setup");
    expect(state.dayNumber).toBe(0);
    expect(state.alivePlayerIds).toEqual([p1, p2, p3]);
    expect(state.deadPlayerIds).toEqual([]);
    expect(state.pendingLastWords).toEqual([]);
    expect(state.daySpeech).toEqual({
      dayNumber: 0,
      round: 1,
      spokenPlayerIds: [],
      pkSpokenPlayerIds: [],
    });
    expect(state.votes).toEqual([]);
    expect(state.pk).toEqual({ status: "none" });
    expect(state.ended).toBeNull();
    expect(state.witch.antidoteAvailable).toBe(true);
    expect(state.witch.poisonAvailable).toBe(true);
  });

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
        payload: {
          deadPlayerIds: [p3],
          deaths: [{ playerId: p3, reason: "wolf_kill" }],
        },
      }),
    ]);

    expect(state.alivePlayerIds).toEqual([p1, p2]);
    expect(state.deadPlayerIds).toEqual([p3]);
  });

  it("tracks night deaths as pending last words until last words are given", () => {
    const state = deriveGameState(players, [
      event(1, {
        type: "night_resolved",
        payload: {
          deadPlayerIds: [p3],
          deaths: [{ playerId: p3, reason: "wolf_kill" }],
        },
      }),
      event(2, {
        type: "death_announced",
        phase: "day",
        payload: { deadPlayerIds: [p3] },
      }),
      event(3, {
        type: "last_words_given",
        phase: "last_words",
        actorPlayerId: p3,
        payload: {
          playerId: p3,
          text: "夜里倒牌，我留个警徽流。",
          dayNumber: 1,
          reason: "night_death",
        },
      }),
    ]);

    expect(state.pendingLastWords).toEqual([]);
  });

  it("waits for public death announcement before night deaths need last words", () => {
    const state = deriveGameState(players, [
      event(1, {
        type: "night_resolved",
        payload: {
          deadPlayerIds: [p3],
          deaths: [{ playerId: p3, reason: "wolf_kill" }],
        },
      }),
    ]);

    expect(state.deadPlayerIds).toEqual([p3]);
    expect(state.pendingLastWords).toEqual([]);
  });

  it("keeps announced night deaths pending before last words are given", () => {
    const state = deriveGameState(players, [
      event(1, {
        type: "night_resolved",
        payload: {
          deadPlayerIds: [p3],
          deaths: [{ playerId: p3, reason: "wolf_kill" }],
        },
      }),
      event(2, {
        type: "death_announced",
        phase: "day",
        payload: { deadPlayerIds: [p3] },
      }),
    ]);

    expect(state.pendingLastWords).toEqual([
      { playerId: p3, reason: "night_death" },
    ]);
  });

  it("marks exiled players dead and pending last words with exile reason", () => {
    const state = deriveGameState(players, [
      event(1, {
        type: "exile_resolved",
        phase: "vote",
        targetPlayerIds: [p2],
        payload: {
          exiledPlayerId: p2,
          tiedPlayerIds: [],
          voteType: "exile",
          voteTable: [{ voterPlayerId: p1, targetPlayerId: p2 }],
          dayNumber: 1,
          round: 1,
          revealedRoles: [],
        },
      }),
    ]);

    expect(state.alivePlayerIds).toEqual([p1, p3]);
    expect(state.deadPlayerIds).toEqual([p2]);
    expect(state.pendingLastWords).toEqual([
      { playerId: p2, reason: "exile" },
    ]);
  });

  it("aggregates day speeches, pk speeches, and votes by day round and vote type", () => {
    const state = deriveGameState(players, [
      event(1, {
        type: "phase_started",
        phase: "speech",
        payload: { phase: "speech", dayNumber: 1 },
      }),
      event(2, {
        type: "day_speech_given",
        phase: "speech",
        actorPlayerId: p1,
        payload: {
          playerId: p1,
          text: "我先聊。",
          dayNumber: 1,
          round: 1,
        },
      }),
      event(3, {
        type: "pk_speech_given",
        phase: "pk",
        actorPlayerId: p2,
        payload: {
          playerId: p2,
          text: "我进 PK。",
          dayNumber: 1,
          round: 2,
        },
      }),
      event(4, {
        type: "vote_cast",
        phase: "vote",
        actorPlayerId: p1,
        targetPlayerIds: [p2],
        payload: {
          voterPlayerId: p1,
          targetPlayerId: p2,
          dayNumber: 1,
          round: 1,
          voteType: "exile",
        },
      }),
      event(5, {
        type: "vote_cast",
        phase: "vote",
        actorPlayerId: p2,
        payload: {
          voterPlayerId: p2,
          targetPlayerId: null,
          dayNumber: 1,
          round: 1,
          voteType: "exile",
        },
      }),
      event(6, {
        type: "vote_cast",
        phase: "vote",
        actorPlayerId: p3,
        targetPlayerIds: [p1],
        payload: {
          voterPlayerId: p3,
          targetPlayerId: p1,
          dayNumber: 1,
          round: 2,
          voteType: "pk",
        },
      }),
    ]);

    expect(state.daySpeech).toEqual({
      dayNumber: 1,
      round: 2,
      spokenPlayerIds: [],
      pkSpokenPlayerIds: [p2],
    });
    expect(state.votes).toEqual([
      {
        dayNumber: 1,
        round: 1,
        voteType: "exile",
        votes: [
          { voterPlayerId: p1, targetPlayerId: p2 },
          { voterPlayerId: p2, targetPlayerId: null },
        ],
      },
      {
        dayNumber: 1,
        round: 2,
        voteType: "pk",
        votes: [{ voterPlayerId: p3, targetPlayerId: p1 }],
      },
    ]);
  });

  it("derives pending pk from daily exile ties", () => {
    const state = deriveGameState(players, [
      event(1, {
        type: "exile_resolved",
        phase: "vote",
        payload: {
          exiledPlayerId: null,
          tiedPlayerIds: [p1, p2],
          voteType: "exile",
          voteTable: [
            { voterPlayerId: p1, targetPlayerId: p2 },
            { voterPlayerId: p2, targetPlayerId: p1 },
          ],
          dayNumber: 1,
          round: 1,
          revealedRoles: [],
        },
      }),
    ]);

    expect(state.pk).toEqual({
      status: "pending",
      dayNumber: 1,
      round: 1,
      tiedPlayerIds: [p1, p2],
    });
  });

  it("represents no exile for no-exile results and second ties", () => {
    const noExile = deriveGameState(players, [
      event(1, {
        type: "exile_resolved",
        phase: "vote",
        payload: {
          exiledPlayerId: null,
          tiedPlayerIds: [],
          voteType: "exile",
          voteTable: [],
          dayNumber: 1,
          round: 1,
          revealedRoles: [],
        },
      }),
    ]);
    const secondTie = deriveGameState(players, [
      event(1, {
        type: "exile_resolved",
        phase: "vote",
        payload: {
          exiledPlayerId: null,
          tiedPlayerIds: [p1, p2],
          voteType: "pk",
          voteTable: [
            { voterPlayerId: p1, targetPlayerId: p2 },
            { voterPlayerId: p2, targetPlayerId: p1 },
          ],
          dayNumber: 1,
          round: 2,
          revealedRoles: [],
        },
      }),
    ]);

    expect(noExile.pk).toEqual({
      status: "no_exile",
      dayNumber: 1,
      round: 1,
      tiedPlayerIds: [],
      reason: "no_exile",
    });
    expect(secondTie.pk).toEqual({
      status: "no_exile",
      dayNumber: 1,
      round: 2,
      tiedPlayerIds: [p1, p2],
      reason: "second_tie",
    });
  });

  it("derives ended state with revealed roles", () => {
    const revealedRoles = [
      { playerId: p1, roleId: "werewolf", roleName: "狼人", faction: "wolves" },
      { playerId: p3, roleId: "seer", roleName: "预言家", faction: "good" },
    ] as const;
    const state = deriveGameState(players, [
      event(1, {
        type: "game_ended",
        phase: "ended",
        payload: {
          winner: "good",
          reason: "all_wolves_dead",
          dayNumber: 2,
          revealedRoles,
        },
      }),
    ]);

    expect(state.currentPhase).toBe("ended");
    expect(state.ended).toEqual({
      winner: "good",
      reason: "all_wolves_dead",
      revealedRoles,
    });
  });

  it("does not mix previous-day speeches into the current day", () => {
    const state = deriveGameState(players, [
      event(1, {
        type: "phase_started",
        phase: "speech",
        payload: { phase: "speech", dayNumber: 1 },
      }),
      event(2, {
        type: "day_speech_given",
        phase: "speech",
        actorPlayerId: p1,
        payload: {
          playerId: p1,
          text: "第 1 天发言。",
          dayNumber: 1,
          round: 1,
        },
      }),
      event(3, {
        type: "pk_speech_given",
        phase: "pk",
        actorPlayerId: p2,
        payload: {
          playerId: p2,
          text: "第 1 天 PK 发言。",
          dayNumber: 1,
          round: 2,
        },
      }),
      event(4, {
        type: "phase_started",
        phase: "speech",
        payload: { phase: "speech", dayNumber: 2 },
      }),
    ]);

    expect(state.daySpeech).toEqual({
      dayNumber: 2,
      round: 1,
      spokenPlayerIds: [],
      pkSpokenPlayerIds: [],
    });
  });

  it("clears previous-day pk result when a later day starts", () => {
    const state = deriveGameState(players, [
      event(1, {
        type: "exile_resolved",
        phase: "vote",
        payload: {
          exiledPlayerId: null,
          tiedPlayerIds: [p1, p2],
          voteType: "pk",
          voteTable: [],
          dayNumber: 1,
          round: 2,
          revealedRoles: [],
        },
      }),
      event(2, {
        type: "phase_started",
        phase: "night",
        payload: { phase: "night", dayNumber: 2 },
      }),
    ]);

    expect(state.pk).toEqual({ status: "none" });
  });

  it("derives extended state from active events sorted by index", () => {
    const state = deriveGameState(players, [
      event(3, {
        type: "last_words_given",
        phase: "last_words",
        actorPlayerId: p3,
        payload: {
          playerId: p3,
          text: "这条按 index 在死亡后发生。",
          dayNumber: 1,
          reason: "night_death",
        },
      }),
      event(1, {
        type: "night_resolved",
        payload: {
          deadPlayerIds: [p3],
          deaths: [{ playerId: p3, reason: "wolf_kill" }],
        },
      }),
      event(2, {
        status: "superseded",
        type: "exile_resolved",
        phase: "vote",
        payload: {
          exiledPlayerId: p2,
          tiedPlayerIds: [],
          voteType: "exile",
          voteTable: [],
          dayNumber: 1,
          round: 1,
          revealedRoles: [],
        },
      }),
    ]);

    expect(state.deadPlayerIds).toEqual([p3]);
    expect(state.pendingLastWords).toEqual([]);
    expect(state.pk).toEqual({ status: "none" });
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
        payload: {
          deadPlayerIds: [p3],
          deaths: [{ playerId: p3, reason: "wolf_kill" }],
        },
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
          payload: {
            deadPlayerIds: [unknownPlayerId],
            deaths: [{ playerId: unknownPlayerId, reason: "wolf_kill" }],
          },
        }),
      ]),
    ).toThrow("Unknown dead player id unknown");
  });

  it("throws when an exile references an unknown player", () => {
    expect(() =>
      deriveGameState(players, [
        event(1, {
          type: "exile_resolved",
          phase: "vote",
          payload: {
            exiledPlayerId: unknownPlayerId,
            tiedPlayerIds: [],
            voteType: "exile",
            voteTable: [],
            dayNumber: 1,
            round: 1,
            revealedRoles: [],
          },
        }),
      ]),
    ).toThrow("Unknown dead player id unknown");
  });

  it("keeps duplicate night-resolved death ids idempotent", () => {
    const state = deriveGameState(players, [
      event(1, {
        type: "night_resolved",
        payload: {
          deadPlayerIds: [p3, p3],
          deaths: [
            { playerId: p3, reason: "wolf_kill" },
            { playerId: p3, reason: "wolf_kill" },
          ],
        },
      }),
    ]);

    expect(state.alivePlayerIds).toEqual([p1, p2]);
    expect(state.deadPlayerIds).toEqual([p3]);
  });
});
