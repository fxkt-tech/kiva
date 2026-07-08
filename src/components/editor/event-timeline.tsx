import { rollbackAfterAction } from "@/app/actions";
import { formatEventForHost, formatVisibility } from "@/core/event-presenter";
import type { GameEvent } from "@/core/events";
import type { GenerationRecord } from "@/core/generation-record";
import type { PlayerSnapshot } from "@/core/player";
import type { GameId } from "@/core/types";
import { Maximize2, Minimize2, Undo2 } from "lucide-react";
import Link from "next/link";
import { LlmGenerationDetails } from "./llm-generation-details";

type EventTimelineProps = {
  readonly gameId: GameId;
  readonly events: readonly GameEvent[];
  readonly players: readonly PlayerSnapshot[];
  readonly generations: readonly GenerationRecord[];
  readonly previewHidden?: boolean;
  readonly togglePreviewHref?: string;
};

export function EventTimeline({
  gameId,
  events,
  players,
  generations,
  previewHidden = false,
  togglePreviewHref,
}: EventTimelineProps) {
  const orderedEvents = [...events].sort((left, right) => {
    if (left.index === right.index) {
      return right.createdAt.localeCompare(left.createdAt);
    }

    return right.index - left.index;
  });

  return (
    <section className="flex min-h-0 flex-col rounded-lg border border-border bg-surface/45">
      <div className="flex shrink-0 items-center justify-between gap-3 border-b border-border px-3 py-2">
        <h2 className="text-sm font-semibold text-foreground">Timeline</h2>
        {togglePreviewHref ? (
          <Link
            href={togglePreviewHref}
            aria-label={previewHidden ? "Show preview" : "Hide preview"}
            title={previewHidden ? "Show preview" : "Hide preview"}
            className="inline-flex h-8 w-8 shrink-0 items-center justify-center rounded-md border border-interactive-border bg-surface/70 text-muted transition hover:border-interactive-border-hover hover:bg-surface-muted hover:text-foreground"
          >
            {previewHidden ? (
              <Minimize2 aria-hidden="true" className="h-4 w-4" />
            ) : (
              <Maximize2 aria-hidden="true" className="h-4 w-4" />
            )}
          </Link>
        ) : null}
      </div>
      {orderedEvents.length === 0 ? (
        <div className="px-4 py-8 text-sm text-subtle">
          No confirmed events yet.
        </div>
      ) : (
        <ol className="min-h-0 flex-1 divide-y divide-border overflow-y-auto">
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
                    : "bg-background/35 px-4 py-4 opacity-45"
                }
              >
                <div className="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
                  <div className="min-w-0">
                    <div className="flex flex-wrap items-center gap-2 text-xs text-subtle">
                      <span>#{event.index}</span>
                      <span>{event.phase}</span>
                      <span>{formatVisibility(event.visibility)}</span>
                      <span>{event.status}</span>
                    </div>
                    <div className="mt-2 break-words text-sm font-medium text-foreground">
                      {presented.title}
                    </div>
                    <p className="mt-1 whitespace-pre-wrap break-words text-sm leading-6 text-muted">
                      {presented.text}
                    </p>
                    {presented.details && presented.details.length > 0 ? (
                      <ul className="mt-2 space-y-1 text-xs text-subtle">
                        {presented.details.map((detail) => (
                          <li key={detail}>{detail}</li>
                        ))}
                      </ul>
                    ) : null}
                    {latestGeneration ? (
                      <div className="mt-3 flex flex-wrap items-center gap-2 border-t border-border pt-3 text-xs text-subtle">
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
                        aria-label={`Roll back after event ${event.index}`}
                        title={`Roll back after event ${event.index}`}
                        className="inline-flex h-8 w-8 items-center justify-center rounded-md border border-transparent bg-warning-badge text-warning-badge-foreground transition hover:bg-surface-strong"
                      >
                        <Undo2 aria-hidden="true" className="h-4 w-4" />
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
