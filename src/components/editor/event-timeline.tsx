import { rollbackAfterAction } from "@/app/actions";
import { formatEventForHost, formatVisibility } from "@/core/event-presenter";
import type { GameEvent } from "@/core/events";
import type { GenerationRecord } from "@/core/generation-record";
import type { PlayerSnapshot } from "@/core/player";
import type { GameId } from "@/core/types";
import { LlmGenerationDetails } from "./llm-generation-details";

type EventTimelineProps = {
  readonly gameId: GameId;
  readonly events: readonly GameEvent[];
  readonly players: readonly PlayerSnapshot[];
  readonly generations: readonly GenerationRecord[];
};

export function EventTimeline({
  gameId,
  events,
  players,
  generations,
}: EventTimelineProps) {
  const orderedEvents = [...events].sort((left, right) => {
    if (left.index === right.index) {
      return right.createdAt.localeCompare(left.createdAt);
    }

    return right.index - left.index;
  });

  return (
    <section className="flex min-h-0 flex-col rounded-lg border border-zinc-800 bg-zinc-900/45">
      <div className="shrink-0 border-b border-zinc-800 px-3 py-2">
        <h2 className="text-sm font-semibold text-zinc-100">Timeline</h2>
      </div>
      {orderedEvents.length === 0 ? (
        <div className="px-4 py-8 text-sm text-zinc-500">
          No confirmed events yet.
        </div>
      ) : (
        <ol className="min-h-0 flex-1 divide-y divide-zinc-800 overflow-y-auto">
          {orderedEvents.map((event) => {
            const active = event.status === "active";
            const presented = formatEventForHost(event, players);
            const eventGenerations = generationsForEvent(generations, event);
            const latestGeneration = eventGenerations[0] ?? null;

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
                      {presented.title}
                    </div>
                    <p className="mt-1 whitespace-pre-wrap break-words text-sm leading-6 text-zinc-400">
                      {presented.text}
                    </p>
                    {presented.details && presented.details.length > 0 ? (
                      <ul className="mt-2 space-y-1 text-xs text-zinc-500">
                        {presented.details.map((detail) => (
                          <li key={detail}>{detail}</li>
                        ))}
                      </ul>
                    ) : null}
                    {latestGeneration ? (
                      <div className="mt-3 flex flex-wrap items-center gap-2 border-t border-zinc-800 pt-3 text-xs text-zinc-500">
                        <span className="font-medium uppercase tracking-[0.14em]">
                          Generation
                        </span>
                        <span>{latestGeneration.status}</span>
                        <span>
                          {latestGeneration.provider}/{latestGeneration.model}
                        </span>
                        <span>{latestGeneration.promptVersion}</span>
                        {eventGenerations.length > 1 ? (
                          <span>{eventGenerations.length} generations</span>
                        ) : null}
                        <LlmGenerationDetails generation={latestGeneration} />
                        {latestGeneration.error ? (
                          <span className="basis-full break-words text-red-300">
                            {latestGeneration.error}
                          </span>
                        ) : null}
                      </div>
                    ) : null}
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

function generationsForEvent(
  generations: readonly GenerationRecord[],
  event: GameEvent,
): readonly GenerationRecord[] {
  if (!event.createdFromDraftId) {
    return [];
  }

  return [...generations]
    .filter((generation) => generation.draftId === event.createdFromDraftId)
    .sort((left, right) => right.createdAt.localeCompare(left.createdAt));
}
