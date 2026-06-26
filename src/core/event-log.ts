import type { GameEvent } from "./events";

export function getActiveEvents(
  events: readonly GameEvent[],
): readonly GameEvent[] {
  return events
    .filter((event) => event.status === "active")
    .slice()
    .sort((a, b) => a.index - b.index);
}

export function appendEvent(
  events: readonly GameEvent[],
  nextEvent: GameEvent,
): readonly GameEvent[] {
  const activeEvents = getActiveEvents(events);
  const lastIndex = activeEvents.at(-1)?.index ?? 0;
  const expectedIndex = lastIndex + 1;

  if (nextEvent.index !== expectedIndex) {
    throw new Error(
      `Expected next event index ${expectedIndex} but received ${nextEvent.index}`,
    );
  }

  if (nextEvent.status !== "active") {
    throw new Error("Only active events can be appended");
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
