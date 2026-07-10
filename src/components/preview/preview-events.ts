import { confirmDraftEvent, type DraftEvent } from "@/core/drafts";
import { getActiveEvents } from "@/core/event-log";
import type { GameEvent } from "@/core/events";
import type { EventId } from "@/core/types";

export function eventsWithDraftPreview(
  events: readonly GameEvent[],
  draft: DraftEvent | null,
): readonly GameEvent[] {
  if (!draft) {
    return events;
  }

  const nextIndex = getActiveEvents(events).length + 1;
  return [
    ...events,
    confirmDraftEvent({
      draft,
      eventId: ("preview_" + draft.id) as EventId,
      index: nextIndex,
      createdAt: draft.createdAt,
    }),
  ];
}
