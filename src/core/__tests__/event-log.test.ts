import { describe, expect, it } from "vitest";
import { appendEvent, getActiveEvents, rollbackAfterIndex } from "../event-log";
import type { GameEvent } from "../events";
import type { EventId, GameId } from "../types";

type EventOverrides = {
  readonly id?: EventId;
  readonly gameId?: GameId;
};

function event(
  index: number,
  status: GameEvent["status"] = "active",
  overrides: EventOverrides = {},
): GameEvent {
  return {
    id: overrides.id ?? (`e${index}` as EventId),
    gameId: overrides.gameId ?? ("g1" as GameId),
    index,
    status,
    type: "phase_started",
    phase: "night",
    visibility: { kind: "public" },
    payload: { phase: "night", dayNumber: 1 },
    createdAt: new Date("2026-06-26T00:00:00.000Z").toISOString(),
  };
}

describe("event log", () => {
  it("filters only active events in index order", () => {
    expect(
      getActiveEvents([event(3), event(1), event(2, "superseded")]),
    ).toEqual([event(1), event(3)]);
  });

  it("appends the next sequential active event", () => {
    const appended = appendEvent([event(1)], event(2));
    expect(appended.map((e) => e.index)).toEqual([1, 2]);
  });

  it("rejects non-sequential append", () => {
    expect(() => appendEvent([event(1)], event(3))).toThrow(
      "Expected next event index 2 but received 3",
    );
  });

  it("rejects non-active append", () => {
    expect(() => appendEvent([event(1)], event(2, "superseded"))).toThrow(
      "Only active events can be appended",
    );
  });

  it("rejects duplicate event ids", () => {
    expect(() =>
      appendEvent([event(1), event(2, "superseded")], event(2, "active")),
    ).toThrow("Duplicate event id e2");
  });

  it("rejects cross-game append", () => {
    expect(() =>
      appendEvent([event(1)], event(2, "active", { gameId: "g2" as GameId })),
    ).toThrow("Cannot append event for game g2 to log for game g1");
  });

  it("rejects corrupt active event sequences", () => {
    expect(() =>
      appendEvent([event(1), event(3)], event(4, "active")),
    ).toThrow("Active event log is not contiguous at index 2");
  });

  it("marks events after rollback index as superseded", () => {
    const rolledBack = rollbackAfterIndex([event(1), event(2), event(3)], 1);
    expect(rolledBack.map((e) => [e.index, e.status])).toEqual([
      [1, "active"],
      [2, "superseded"],
      [3, "superseded"],
    ]);
  });
});
