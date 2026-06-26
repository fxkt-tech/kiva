import { rollbackAfterAction } from "@/app/actions";
import type { GameEvent, EventVisibility } from "@/core/events";
import type { GameId } from "@/core/types";

type EventTimelineProps = {
  readonly gameId: GameId;
  readonly events: readonly GameEvent[];
};

export function EventTimeline({ gameId, events }: EventTimelineProps) {
  const orderedEvents = [...events].sort((left, right) => {
    if (left.index === right.index) {
      return left.createdAt.localeCompare(right.createdAt);
    }

    return left.index - right.index;
  });

  return (
    <section className="rounded-lg border border-zinc-800 bg-zinc-900/45">
      <div className="border-b border-zinc-800 px-4 py-3">
        <h2 className="text-sm font-semibold text-zinc-100">Timeline</h2>
      </div>
      {orderedEvents.length === 0 ? (
        <div className="px-4 py-8 text-sm text-zinc-500">
          No confirmed events yet.
        </div>
      ) : (
        <ol className="divide-y divide-zinc-800">
          {orderedEvents.map((event) => {
            const active = event.status === "active";

            return (
              <li
                key={`${event.id}-${event.status}`}
                className={
                  active
                    ? "px-4 py-4"
                    : "bg-zinc-950/35 px-4 py-4 opacity-45"
                }
              >
                <div className="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
                  <div className="min-w-0">
                    <div className="flex flex-wrap items-center gap-2 text-xs text-zinc-500">
                      <span>#{event.index}</span>
                      <span>{event.phase}</span>
                      <span>{formatVisibility(event.visibility)}</span>
                      <span>{event.status}</span>
                    </div>
                    <div className="mt-2 break-words text-sm font-medium text-zinc-100">
                      {event.display?.title ?? event.type}
                    </div>
                    <p className="mt-1 whitespace-pre-wrap break-words text-sm leading-6 text-zinc-400">
                      {event.display?.text || "No display text."}
                    </p>
                  </div>
                  {active ? (
                    <form
                      action={rollbackAfterAction.bind(
                        null,
                        gameId,
                        event.index,
                      )}
                      className="shrink-0"
                    >
                      <button
                        type="submit"
                        className="rounded-md border border-zinc-700 px-3 py-2 text-xs font-medium text-zinc-300 transition hover:border-amber-500 hover:text-amber-200"
                      >
                        Roll back after
                      </button>
                    </form>
                  ) : null}
                </div>
              </li>
            );
          })}
        </ol>
      )}
    </section>
  );
}

function formatVisibility(visibility: EventVisibility): string {
  switch (visibility.kind) {
    case "public":
      return "public";
    case "host_only":
      return "host only";
    case "player_private":
      return `private ${visibility.playerIds.length}`;
    case "faction_private":
      return `${visibility.faction} private`;
    case "custom":
      return `custom ${visibility.playerIds.length}`;
  }
}
