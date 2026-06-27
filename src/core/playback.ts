import { getActiveEvents } from "./event-log";
import { formatEventForPublic } from "./event-presenter";
import type { GameEvent } from "./events";
import type { PlayerSnapshot } from "./player";
import type { Phase } from "./types";

export type PlaybackItem = {
  readonly index: number;
  readonly phase: Phase;
  readonly title: string;
  readonly text: string;
  readonly durationMs: number;
};

export function compilePublicPlayback(
  events: readonly GameEvent[],
  players: readonly PlayerSnapshot[],
): readonly PlaybackItem[] {
  return getActiveEvents(events)
    .filter((event) => event.visibility.kind === "public")
    .flatMap((event) => {
      const presented = formatEventForPublic(event, players);
      return presented
        ? [
            {
              index: event.index,
              phase: event.phase,
              title: presented.title,
              text: presented.text,
              durationMs: durationForEvent(event),
            },
          ]
        : [];
    });
}

function durationForEvent(event: GameEvent): number {
  switch (event.type) {
    case "phase_started":
      return 1600;
    case "death_announced":
      return 2200;
    case "last_words_given":
    case "day_speech_given":
    case "pk_speech_given":
      return 4200;
    case "vote_cast":
      return 1200;
    case "exile_resolved":
      return 2400;
    case "game_ended":
      return 5000;
    default:
      return 2200;
  }
}
