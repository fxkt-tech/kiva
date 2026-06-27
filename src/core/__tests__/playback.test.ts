import { describe, expect, it } from "vitest";
import { createSeedGame } from "../game";
import { compilePublicPlayback } from "../playback";
import type { EventVisibility, GameEvent } from "../events";
import type { EventId, GameId, PlayerId } from "../types";

const gameId = "g1" as GameId;
const playerId = "p1" as PlayerId;
const game = createSeedGame({ gameId, createdAt: "2026-06-26T00:00:00.000Z" });
const players = game.players;

function event(
  index: number,
  visibility: EventVisibility,
  overrides: Partial<GameEvent> = {},
): GameEvent {
  return {
    id: `e${index}` as EventId,
    gameId,
    index,
    status: "active",
    type: "phase_started",
    phase: "night",
    visibility,
    payload: { phase: "night", dayNumber: 1 },
    createdAt: "2026-06-26T00:00:00.000Z",
    ...overrides,
  } as GameEvent;
}

describe("playback compiler", () => {
  it("compiles only active public events in playback shape", () => {
    const events = [
      event(7, { kind: "public" }),
      event(2, { kind: "host_only" }),
      event(6, { kind: "custom", playerIds: [playerId] }),
      event(5, { kind: "public" }, { status: "superseded" }),
      event(4, { kind: "faction_private", faction: "wolves" }),
      event(3, { kind: "player_private", playerIds: [playerId] }),
      event(1, { kind: "public" }),
    ];

    expect(compilePublicPlayback(events, players)).toEqual([
      {
        index: 1,
        phase: "night",
        title: "第 1 夜开始",
        text: "夜晚开始。",
        durationMs: 1600,
      },
      {
        index: 7,
        phase: "night",
        title: "第 1 夜开始",
        text: "夜晚开始。",
        durationMs: 1600,
      },
    ]);
  });

  it("assigns stable durations by public event type", () => {
    expect(
      compilePublicPlayback([
        event(1, { kind: "public" }, {
          type: "death_announced",
          phase: "day",
          payload: { deadPlayerIds: [players[0].playerId] },
        }),
        event(2, { kind: "public" }, {
          type: "day_speech_given",
          phase: "speech",
          actorPlayerId: players[0].playerId,
          payload: {
            playerId: players[0].playerId,
            text: "发言",
            dayNumber: 1,
            round: 1,
          },
        }),
        event(3, { kind: "public" }, {
          type: "vote_cast",
          phase: "vote",
          actorPlayerId: players[0].playerId,
          payload: {
            voterPlayerId: players[0].playerId,
            targetPlayerId: null,
            dayNumber: 1,
            round: 1,
            voteType: "exile",
          },
        }),
        event(4, { kind: "public" }, {
          type: "exile_resolved",
          phase: "vote",
          payload: {
            exiledPlayerId: null,
            tiedPlayerIds: [],
            voteTable: [],
            voteType: "exile",
            dayNumber: 1,
            round: 1,
            revealedRoles: [],
          },
        }),
        event(5, { kind: "public" }, {
          type: "game_ended",
          phase: "ended",
          payload: {
            winner: "good",
            reason: "all_wolves_dead",
            dayNumber: 1,
            revealedRoles: [],
          },
        }),
      ], players).map((item) => item.durationMs),
    ).toEqual([2200, 4200, 1200, 2400, 5000]);
  });
});
