import { getActiveEvents } from "./event-log";
import type { GameEvent } from "./events";
import type { Phase } from "./types";

export type PlaybackItem = {
  readonly index: number;
  readonly phase: Phase;
  readonly title: string;
  readonly text: string;
};

export function compilePublicPlayback(
  events: readonly GameEvent[],
): readonly PlaybackItem[] {
  return getActiveEvents(events)
    .filter((event) => event.visibility.kind === "public")
    .map((event) => ({
      index: event.index,
      phase: event.phase,
      title: event.display?.title ?? event.type,
      text: event.display?.text ?? "",
    }));
}
