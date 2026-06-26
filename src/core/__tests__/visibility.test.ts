import { describe, expect, it } from "vitest";
import { projectVisibleEvents } from "../visibility";
import type { GameEvent } from "../events";
import type { EventId, GameId, PlayerId } from "../types";

const gameId = "g1" as GameId;
const wolf = "p1" as PlayerId;
const seer = "p3" as PlayerId;
const witch = "p4" as PlayerId;

function baseEvent(overrides: Partial<GameEvent>): GameEvent {
  return {
    id: "e1" as EventId,
    gameId,
    index: 1,
    status: "active",
    type: "phase_started",
    phase: "night",
    visibility: { kind: "public" },
    payload: { phase: "night", dayNumber: 1 },
    createdAt: "2026-06-26T00:00:00.000Z",
    ...overrides,
  } as GameEvent;
}

describe("visibility projector", () => {
  it("shows public events to every player", () => {
    const events = [baseEvent({ index: 1, visibility: { kind: "public" } })];
    expect(
      projectVisibleEvents(events, seer, { wolfPlayerIds: [wolf] }),
    ).toHaveLength(1);
  });

  it("hides host-only events from players", () => {
    const events = [baseEvent({ index: 1, visibility: { kind: "host_only" } })];
    expect(
      projectVisibleEvents(events, seer, { wolfPlayerIds: [wolf] }),
    ).toHaveLength(0);
  });

  it("shows private player events only to listed players", () => {
    const events = [
      baseEvent({
        index: 1,
        visibility: { kind: "player_private", playerIds: [seer] },
      }),
    ];

    expect(
      projectVisibleEvents(events, seer, { wolfPlayerIds: [wolf] }),
    ).toHaveLength(1);
    expect(
      projectVisibleEvents(events, witch, { wolfPlayerIds: [wolf] }),
    ).toHaveLength(0);
  });

  it("shows custom visibility events only to listed players", () => {
    const events = [
      baseEvent({
        index: 1,
        visibility: { kind: "custom", playerIds: [seer] },
      }),
    ];

    expect(
      projectVisibleEvents(events, seer, { wolfPlayerIds: [wolf] }),
    ).toHaveLength(1);
    expect(
      projectVisibleEvents(events, witch, { wolfPlayerIds: [wolf] }),
    ).toHaveLength(0);
  });

  it("shows wolf faction events only to wolves", () => {
    const events = [
      baseEvent({
        index: 1,
        visibility: { kind: "faction_private", faction: "wolves" },
      }),
    ];

    expect(
      projectVisibleEvents(events, wolf, { wolfPlayerIds: [wolf] }),
    ).toHaveLength(1);
    expect(
      projectVisibleEvents(events, seer, { wolfPlayerIds: [wolf] }),
    ).toHaveLength(0);
  });

  it("does not show superseded events", () => {
    const events = [
      baseEvent({
        index: 1,
        status: "superseded",
        visibility: { kind: "public" },
      }),
    ];

    expect(
      projectVisibleEvents(events, seer, { wolfPlayerIds: [wolf] }),
    ).toHaveLength(0);
  });

  it("returns visible active events in sorted index order after filtering", () => {
    const events = [
      baseEvent({
        id: "e4" as EventId,
        index: 4,
        visibility: { kind: "player_private", playerIds: [witch] },
      }),
      baseEvent({
        id: "e2" as EventId,
        index: 2,
        visibility: { kind: "host_only" },
      }),
      baseEvent({
        id: "e3" as EventId,
        index: 3,
        visibility: { kind: "public" },
      }),
      baseEvent({
        id: "e1" as EventId,
        index: 1,
        visibility: { kind: "player_private", playerIds: [seer] },
      }),
      baseEvent({
        id: "e5" as EventId,
        index: 5,
        status: "superseded",
        visibility: { kind: "public" },
      }),
    ];

    expect(
      projectVisibleEvents(events, seer, { wolfPlayerIds: [wolf] }).map(
        (event) => event.index,
      ),
    ).toEqual([1, 3]);
  });
});
