import { describe, expect, it } from "vitest";
import { confirmDraftEvent, createDraftEvent } from "../drafts";
import { createSeedGame } from "../game";
import { compilePublicPlayback } from "../playback";
import type { GameEvent, RevealedRole, VoteTableEntry } from "../events";
import type { DraftId, EventId, GameId, PlayerId } from "../types";

const gameId = "game_1" as GameId;
const player1 = "player_1" as PlayerId;
const player2 = "player_2" as PlayerId;
const game = createSeedGame({
  gameId,
  createdAt: "2026-06-26T00:00:00.000Z",
});

describe("full game event model", () => {
  it("supports last words, speeches, voting, exile, and game end payloads", () => {
    const voteTable = [
      { voterPlayerId: player1, targetPlayerId: player2 },
      { voterPlayerId: player2, targetPlayerId: null },
    ] satisfies readonly VoteTableEntry[];

    const revealedRoles = [
      {
        playerId: player1,
        roleId: "seer",
        roleName: "预言家",
        faction: "good",
      },
    ] satisfies readonly RevealedRole[];

    const events = [
      {
        id: "event_1" as EventId,
        gameId,
        index: 1,
        status: "active",
        type: "last_words_given",
        phase: "last_words",
        actorPlayerId: player1,
        visibility: { kind: "public" },
        payload: {
          playerId: player1,
          text: "我觉得 2 号很可疑。",
          dayNumber: 1,
          reason: "night_death",
        },
        createdAt: "2026-06-26T00:00:00.000Z",
      },
      {
        id: "event_2" as EventId,
        gameId,
        index: 2,
        status: "active",
        type: "day_speech_given",
        phase: "speech",
        actorPlayerId: player2,
        visibility: { kind: "public" },
        payload: {
          playerId: player2,
          text: "我会听后置位。",
          dayNumber: 1,
          round: 1,
        },
        createdAt: "2026-06-26T00:01:00.000Z",
      },
      {
        id: "event_3" as EventId,
        gameId,
        index: 3,
        status: "active",
        type: "pk_speech_given",
        phase: "pk",
        actorPlayerId: player1,
        visibility: { kind: "public" },
        payload: {
          playerId: player1,
          text: "PK 发言。",
          dayNumber: 1,
          round: 1,
        },
        createdAt: "2026-06-26T00:02:00.000Z",
      },
      {
        id: "event_4" as EventId,
        gameId,
        index: 4,
        status: "active",
        type: "vote_cast",
        phase: "vote",
        actorPlayerId: player1,
        targetPlayerIds: [player2],
        visibility: { kind: "public" },
        payload: {
          voterPlayerId: player1,
          targetPlayerId: player2,
          dayNumber: 1,
          round: 1,
          voteType: "exile",
        },
        createdAt: "2026-06-26T00:03:00.000Z",
      },
      {
        id: "event_5" as EventId,
        gameId,
        index: 5,
        status: "active",
        type: "exile_resolved",
        phase: "vote",
        targetPlayerIds: [player2],
        visibility: { kind: "public" },
        payload: {
          exiledPlayerId: player2,
          tiedPlayerIds: [],
          voteType: "exile",
          voteTable,
          dayNumber: 1,
          round: 1,
          revealedRoles,
        },
        createdAt: "2026-06-26T00:04:00.000Z",
      },
      {
        id: "event_6" as EventId,
        gameId,
        index: 6,
        status: "active",
        type: "game_ended",
        phase: "ended",
        visibility: { kind: "public" },
        payload: {
          winner: "good",
          reason: "all_wolves_dead",
          dayNumber: 1,
          revealedRoles,
        },
        createdAt: "2026-06-26T00:05:00.000Z",
      },
    ] satisfies readonly GameEvent[];

    expect(events.map((event) => event.type)).toEqual([
      "last_words_given",
      "day_speech_given",
      "pk_speech_given",
      "vote_cast",
      "exile_resolved",
      "game_ended",
    ]);
    expect(events[0].payload).toMatchObject({
      dayNumber: 1,
      reason: "night_death",
    });
    expect(events[3].payload).toMatchObject({
      dayNumber: 1,
      round: 1,
      voteType: "exile",
    });
    expect(events[4].payload).toMatchObject({
      exiledPlayerId: player2,
      voteType: "exile",
      voteTable: [
        { voterPlayerId: player1, targetPlayerId: player2 },
        { voterPlayerId: player2, targetPlayerId: null },
      ],
    });
    expect(events[5].payload).toMatchObject({
      winner: "good",
      reason: "all_wolves_dead",
      revealedRoles: [
        { playerId: player1, roleId: "seer", roleName: "预言家" },
      ],
    });
  });

  it("keeps draft confirmation and public playback compatible with new events", () => {
    const draft = createDraftEvent({
      id: "draft_1" as DraftId,
      gameId,
      type: "vote_cast",
      phase: "vote",
      actorPlayerId: player1,
      targetPlayerIds: [],
      visibility: { kind: "public" },
      payload: {
        voterPlayerId: player1,
        targetPlayerId: null,
        dayNumber: 1,
        round: 1,
        voteType: "pk",
      },
      createdAt: "2026-06-26T00:00:00.000Z",
    });

    const event = confirmDraftEvent({
      draft,
      eventId: "event_1" as EventId,
      index: 1,
      createdAt: "2026-06-26T00:01:00.000Z",
    });

    expect(event).toMatchObject({
      status: "active",
      type: "vote_cast",
      payload: { targetPlayerId: null, voteType: "pk" },
    });
    expect(compilePublicPlayback([event], game.players)).toMatchObject([
      {
        index: 1,
        phase: "vote",
        kind: "vote",
        title: "PK 投票",
        text: expect.stringContaining("弃票"),
        durationMs: 1200,
      },
    ]);
  });
});
