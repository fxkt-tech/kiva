import { getActiveEvents } from "./event-log";
import type { EventVisibility, GameEvent } from "./events";
import type { PlayerId } from "./types";

export type VisibilityContext = {
  readonly wolfPlayerIds: readonly PlayerId[];
};

export function projectVisibleEvents(
  events: readonly GameEvent[],
  viewerPlayerId: PlayerId,
  context: VisibilityContext,
): readonly GameEvent[] {
  return getActiveEvents(events).filter((event) =>
    canPlayerSee(event.visibility, viewerPlayerId, context),
  );
}

export function canPlayerSee(
  visibility: EventVisibility,
  viewerPlayerId: PlayerId,
  context: VisibilityContext,
): boolean {
  if (visibility.kind === "public") return true;
  if (visibility.kind === "host_only") return false;
  if (visibility.kind === "player_private") {
    return visibility.playerIds.includes(viewerPlayerId);
  }
  if (visibility.kind === "custom") {
    return visibility.playerIds.includes(viewerPlayerId);
  }
  if (visibility.kind === "faction_private") {
    return (
      visibility.faction === "wolves" &&
      context.wolfPlayerIds.includes(viewerPlayerId)
    );
  }
  return false;
}
