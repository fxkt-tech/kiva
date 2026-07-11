import { describe, expect, it } from "vitest";
import type { DraftEvent } from "../drafts";
import type { GameEvent } from "../events";
import {
  formatDraftForHost,
  formatEventForHost,
  formatEventForPublic,
} from "../event-presenter";
import { createSeedGame } from "../game";
import type { DraftId, EventId, GameId } from "../types";

const gameId = "game_1" as GameId;
const createdAt = "2026-06-26T00:00:00.000Z";
const game = createSeedGame({ gameId, createdAt });
const players = game.players;

describe("event presenter", () => {
  it("renders host-facing facts with actors, targets, and roles", () => {
    expect(
      formatEventForHost(
        {
          id: "event_1" as EventId,
          gameId,
          index: 1,
          status: "active",
          type: "role_assigned",
          phase: "setup",
          targetPlayerIds: [players[0].playerId],
          visibility: {
            kind: "player_private",
            playerIds: [players[0].playerId],
          },
          payload: {
            playerId: players[0].playerId,
            role: players[0].gameRole,
            faction: players[0].faction,
          },
          createdAt,
        },
        players,
      ).text,
    ).toContain(`1 号 ${players[0].name} 获得身份`);

    expect(
      formatEventForHost(
        {
          id: "event_2" as EventId,
          gameId,
          index: 2,
          status: "active",
          type: "wolf_vote_cast",
          phase: "night",
          actorPlayerId: players[2].playerId,
          targetPlayerIds: [players[4].playerId],
          visibility: { kind: "host_only" },
          payload: { voterPlayerId: players[2].playerId, targetPlayerId: players[4].playerId, dayNumber: 1 },
          createdAt,
        },
        players,
      ).text,
    ).toContain("投给 5 号");
  });

  it("renders public events but hides non-public events", () => {
    const privateEvent: GameEvent = {
      id: "event_1" as EventId,
      gameId,
      index: 1,
      status: "active",
      type: "wolf_vote_cast",
      phase: "night",
      actorPlayerId: players[2].playerId,
      targetPlayerIds: [players[4].playerId],
      visibility: { kind: "host_only" },
      payload: { voterPlayerId: players[2].playerId, targetPlayerId: players[4].playerId, dayNumber: 1 },
      createdAt,
    };
    const publicEvent: GameEvent = {
      id: "event_2" as EventId,
      gameId,
      index: 2,
      status: "active",
      type: "death_announced",
      phase: "day",
      targetPlayerIds: [players[4].playerId],
      visibility: { kind: "public" },
      payload: { deadPlayerIds: [players[4].playerId] },
      createdAt,
    };

    expect(formatEventForPublic(privateEvent, players)).toBeNull();
    expect(formatEventForPublic(publicEvent, players)).toMatchObject({
      title: "昨夜死讯",
      text: expect.stringContaining("5 号"),
    });
  });

  it("renders draft summaries from editable payloads", () => {
    const draft = {
      id: "draft_1" as DraftId,
      gameId,
      status: "draft",
      type: "witch_poison_decided",
      phase: "night",
      actorPlayerId: players[3].playerId,
      targetPlayerIds: [players[1].playerId],
      visibility: { kind: "player_private", playerIds: [players[3].playerId] },
      payload: { used: true, targetPlayerId: players[1].playerId },
      createdAt,
    } as DraftEvent;

    expect(formatDraftForHost(draft, players)).toMatchObject({
      title: "待确认：女巫毒药",
      text: expect.stringContaining("使用毒药毒 2 号"),
    });
  });
});
