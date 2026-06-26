import type { GameEvent } from "./events";

export function getActiveEvents(
  events: readonly GameEvent[],
): readonly GameEvent[] {
  return events
    .filter((event) => event.status === "active")
    .sort((a, b) => a.index - b.index);
}

export function appendEvent(
  events: readonly GameEvent[],
  nextEvent: GameEvent,
): readonly GameEvent[] {
  const activeEvents = getActiveEvents(events);

  if (nextEvent.status !== "active") {
    throw new Error("Only active events can be appended");
  }

  const existingGameIds = new Set(events.map((event) => event.gameId));
  if (existingGameIds.size > 1) {
    throw new Error("Event log contains multiple game ids");
  }

  const logGameId = events.at(0)?.gameId;
  if (logGameId !== undefined && nextEvent.gameId !== logGameId) {
    throw new Error(
      `Cannot append event for game ${nextEvent.gameId} to log for game ${logGameId}`,
    );
  }

  if (events.some((event) => event.id === nextEvent.id)) {
    throw new Error(`Duplicate event id ${nextEvent.id}`);
  }

  activeEvents.forEach((event, index) => {
    const expectedIndex = index + 1;
    if (event.index !== expectedIndex) {
      throw new Error(
        `Active event log is not contiguous at index ${expectedIndex}`,
      );
    }
  });

  const lastIndex = activeEvents.at(-1)?.index ?? 0;
  const expectedIndex = lastIndex + 1;

  if (nextEvent.index !== expectedIndex) {
    throw new Error(
      `Expected next event index ${expectedIndex} but received ${nextEvent.index}`,
    );
  }

  return [...events, nextEvent];
}

export function rollbackAfterIndex(
  events: readonly GameEvent[],
  index: number,
): readonly GameEvent[] {
  return events.map((event) => {
    if (event.status === "active" && event.index > index) {
      return { ...event, status: "superseded" };
    }

    return event;
  });
}
