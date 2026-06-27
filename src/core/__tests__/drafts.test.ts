import { describe, expect, it } from "vitest";
import { confirmDraftEvent, createDraftEvent } from "../drafts";
import {
  createDefaultRuleset,
  type DraftId,
  type EventId,
  type GameId,
  type PlayerId,
} from "../types";

const gameId = "game_1" as GameId;
const draftId = "draft_1" as DraftId;
const eventId = "event_1" as EventId;
const playerId = "player_1" as PlayerId;

describe("drafts", () => {
  it("creates an editable draft that is not an official fact", () => {
    const draft = createDraftEvent({
      id: draftId,
      gameId,
      type: "death_announced",
      phase: "day",
      actorPlayerId: undefined,
      targetPlayerIds: [playerId],
      visibility: { kind: "public" },
      payload: { deadPlayerIds: [playerId] },
      createdAt: "2026-06-26T00:00:00.000Z",
    });

    expect(draft.status).toBe("draft");
    expect(draft.payload).toEqual({ deadPlayerIds: [playerId] });
  });

  it("confirms a draft into the next active official event", () => {
    const draft = createDraftEvent({
      id: draftId,
      gameId,
      type: "death_announced",
      phase: "day",
      actorPlayerId: undefined,
      targetPlayerIds: [playerId],
      visibility: { kind: "public" },
      payload: { deadPlayerIds: [playerId] },
      createdAt: "2026-06-26T00:00:00.000Z",
    });

    const event = confirmDraftEvent({
      draft,
      eventId,
      index: 3,
      createdAt: "2026-06-26T00:01:00.000Z",
    });

    expect(event).toMatchObject({
      id: eventId,
      gameId,
      index: 3,
      status: "active",
      createdFromDraftId: draftId,
      type: "death_announced",
      phase: "day",
    });
    expect(event).not.toHaveProperty("reason");
  });

  it("creates a fixed six-player game aggregate", async () => {
    const { createSeedGame } = await import("../game");
    const record = createSeedGame({
      gameId,
      createdAt: "2026-06-26T00:00:00.000Z",
      ruleset: createDefaultRuleset(),
    });

    expect(record.players).toHaveLength(6);
    expect(record.players.map((player) => player.seatNo)).toEqual([
      1, 2, 3, 4, 5, 6,
    ]);
  });
});
