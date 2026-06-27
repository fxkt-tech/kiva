import type { GameEvent } from "./events";
import type { DraftId, EventId } from "./types";

type DraftFromEvent<Event extends GameEvent> = Omit<
  Event,
  "id" | "index" | "status" | "createdFromDraftId"
> & {
  readonly id: DraftId;
  readonly status: "draft";
  readonly reason?: string;
};

export type DraftEvent = GameEvent extends infer Event
  ? Event extends GameEvent
    ? DraftFromEvent<Event>
    : never
  : never;

export type CreateDraftEventInput = Omit<DraftEvent, "status">;

export type ConfirmDraftEventInput = {
  readonly draft: DraftEvent;
  readonly eventId: EventId;
  readonly index: number;
  readonly createdAt: string;
};

export function createDraftEvent(input: CreateDraftEventInput): DraftEvent {
  return {
    ...input,
    status: "draft",
  } as DraftEvent;
}

export function confirmDraftEvent(input: ConfirmDraftEventInput): GameEvent {
  const draft = input.draft;

  return {
    id: input.eventId,
    gameId: draft.gameId,
    index: input.index,
    status: "active",
    type: draft.type,
    phase: draft.phase,
    actorPlayerId: draft.actorPlayerId,
    targetPlayerIds: draft.targetPlayerIds,
    visibility: draft.visibility,
    payload: draft.payload,
    createdFromDraftId: draft.id,
    createdAt: input.createdAt,
  } as GameEvent;
}
