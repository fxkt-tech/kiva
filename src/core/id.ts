import { randomUUID } from "node:crypto";
import type { DraftId, EventId, GameId } from "./types";

export function createGameId(): GameId {
  return `game_${randomUUID()}` as GameId;
}

export function createDraftId(): DraftId {
  return `draft_${randomUUID()}` as DraftId;
}

export function createEventId(index: number): EventId {
  return `event_${index}_${randomUUID()}` as EventId;
}
