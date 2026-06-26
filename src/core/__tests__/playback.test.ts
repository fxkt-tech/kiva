import { describe, expect, it } from "vitest";
import { compilePublicPlayback } from "../playback";
import type { EventVisibility, GameEvent } from "../events";
import type { EventId, GameId, PlayerId } from "../types";

const gameId = "g1" as GameId;
const playerId = "p1" as PlayerId;

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
    display: {
      title: `Title ${index}`,
      text: `Text ${index}`,
    },
    ...overrides,
  } as GameEvent;
}

describe("playback compiler", () => {
  it("compiles only active public events in playback shape", () => {
    const events = [
      event(7, { kind: "public" }, { display: undefined }),
      event(2, { kind: "host_only" }),
      event(6, { kind: "custom", playerIds: [playerId] }),
      event(5, { kind: "public" }, { status: "superseded" }),
      event(4, { kind: "faction_private", faction: "wolves" }),
      event(3, { kind: "player_private", playerIds: [playerId] }),
      event(1, { kind: "public" }),
    ];

    expect(compilePublicPlayback(events)).toEqual([
      {
        index: 1,
        phase: "night",
        title: "Title 1",
        text: "Text 1",
      },
      {
        index: 7,
        phase: "night",
        title: "phase_started",
        text: "",
      },
    ]);
  });
});
