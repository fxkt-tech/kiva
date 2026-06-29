import { describe, expect, it } from "vitest";
import { createSeedGame } from "../game";
import {
  compilePublicPlayback,
  playbackTotalDurationMs,
  playbackIndexAtMs,
} from "../playback";
import type { EventVisibility, GameEvent } from "../events";
import type { EventId, GameId, PlayerId } from "../types";

const gameId = "g1" as GameId;
const playerId = "p1" as PlayerId;
const game = createSeedGame({ gameId, createdAt: "2026-06-26T00:00:00.000Z" });
const players = game.players;

function event(
  index: number,
  visibility: EventVisibility,
  overrides: Partial<GameEvent> = {},
): GameEvent {
  return {
    id: `e${index}` as EventId,
    gameId,
    index,
    status: "active",
    type: "phase_started",
    phase: "night",
    visibility,
    payload: { phase: "night", dayNumber: 1 },
    createdAt: "2026-06-26T00:00:00.000Z",
    ...overrides,
  } as GameEvent;
}

describe("playback compiler", () => {
  it("compiles only active public events in playback shape", () => {
    const events = [
      event(7, { kind: "public" }),
      event(2, { kind: "host_only" }),
      event(6, { kind: "custom", playerIds: [playerId] }),
      event(5, { kind: "public" }, { status: "superseded" }),
      event(4, { kind: "faction_private", faction: "wolves" }),
      event(3, { kind: "player_private", playerIds: [playerId] }),
      event(1, { kind: "public" }),
    ];

    expect(compilePublicPlayback(events, players)).toEqual([
      {
        index: 1,
        phase: "night",
        kind: "phase",
        title: "第 1 夜开始",
        text: "夜晚开始。",
        details: [],
        durationMs: 1600,
        startsAtMs: 0,
        players: expect.arrayContaining([
          expect.objectContaining({
            playerId: players[0].playerId,
            avatar: players[0].avatar,
            roleName: players[0].roleName,
            status: "alive",
          }),
        ]),
      },
      {
        index: 7,
        phase: "night",
        kind: "phase",
        title: "第 1 夜开始",
        text: "夜晚开始。",
        details: [],
        durationMs: 1600,
        startsAtMs: 1600,
        players: expect.arrayContaining([
          expect.objectContaining({
            playerId: players[0].playerId,
            roleName: players[0].roleName,
            status: "alive",
          }),
        ]),
      },
    ]);
  });

  it("carries player avatar paths into playback scenes", () => {
    const avatar = "/kivdb-assets/characters/avatar_qinchuan.png";
    const playback = compilePublicPlayback(
      [event(1, { kind: "public" })],
      players.map((player, index) =>
        index === 0 ? { ...player, avatar } : player,
      ),
    );

    expect(playback[0]?.players[0]).toMatchObject({ avatar });
  });

  it("assigns stable durations by public event type", () => {
    expect(
      compilePublicPlayback([
        event(1, { kind: "public" }, {
          type: "death_announced",
          phase: "day",
          payload: { deadPlayerIds: [players[0].playerId] },
        }),
        event(2, { kind: "public" }, {
          type: "day_speech_given",
          phase: "speech",
          actorPlayerId: players[0].playerId,
          payload: {
            playerId: players[0].playerId,
            text: "发言",
            dayNumber: 1,
            round: 1,
          },
        }),
        event(3, { kind: "public" }, {
          type: "vote_cast",
          phase: "vote",
          actorPlayerId: players[0].playerId,
          payload: {
            voterPlayerId: players[0].playerId,
            targetPlayerId: null,
            dayNumber: 1,
            round: 1,
            voteType: "exile",
          },
        }),
        event(4, { kind: "public" }, {
          type: "exile_resolved",
          phase: "vote",
          payload: {
            exiledPlayerId: null,
            tiedPlayerIds: [],
            voteTable: [],
            voteType: "exile",
            dayNumber: 1,
            round: 1,
            revealedRoles: [],
          },
        }),
        event(5, { kind: "public" }, {
          type: "game_ended",
          phase: "ended",
          payload: {
            winner: "good",
            reason: "all_wolves_dead",
            dayNumber: 1,
            revealedRoles: [],
          },
        }),
      ], players).map((item) => item.durationMs),
    ).toEqual([2200, 4200, 2400, 5000]);
  });

  it("allows playback rhythm overrides and estimates longer speech duration from text length", () => {
    const longSpeech = "我认为今天的信息已经足够多，前置位的逻辑有明显断点，后置位需要重点解释投票理由。";

    const playback = compilePublicPlayback([
      event(1, { kind: "public" }, {
        type: "phase_started",
        phase: "day",
        payload: { phase: "day", dayNumber: 1 },
      }),
      event(2, { kind: "public" }, {
        type: "day_speech_given",
        phase: "speech",
        actorPlayerId: players[0].playerId,
        payload: {
          playerId: players[0].playerId,
          text: longSpeech,
          dayNumber: 1,
          round: 1,
        },
      }),
    ], players, {
      rhythm: {
        phaseMs: 900,
        speechBaseMs: 1000,
        speechMsPerCharacter: 100,
        speechMinMs: 2000,
        speechMaxMs: 6000,
      },
    });

    expect(playback.map((item) => item.durationMs)).toEqual([
      900,
      Math.min(6000, Math.max(2000, 1000 + longSpeech.length * 100)),
    ]);
    expect(playback[1]?.startsAtMs).toBe(900);
  });

  it("uses scene duration overrides before falling back to rhythm estimates", () => {
    const playback = compilePublicPlayback([
      event(1, { kind: "public" }, {
        type: "phase_started",
        phase: "night",
        payload: { phase: "night", dayNumber: 1 },
      }),
      event(2, { kind: "public" }, {
        type: "day_speech_given",
        phase: "speech",
        actorPlayerId: players[0].playerId,
        payload: {
          playerId: players[0].playerId,
          text: "短发言",
          dayNumber: 1,
          round: 1,
        },
      }),
    ], players, {
      durationForScene: (scene) => scene.title === "第 1 夜开始" ? 2160 : null,
    });

    expect(playback.map((item) => item.durationMs)).toEqual([2160, 4200]);
    expect(playback[1]?.startsAtMs).toBe(2160);
  });


  it("keeps public player status from revealing private night deaths before announcement", () => {
    const killedPlayerId = players[0].playerId;

    const playback = compilePublicPlayback([
      event(1, { kind: "public" }),
      event(2, { kind: "host_only" }, {
        type: "night_resolved",
        phase: "night",
        payload: { deadPlayerIds: [killedPlayerId] },
      }),
      event(3, { kind: "public" }, {
        type: "death_announced",
        phase: "day",
        payload: { deadPlayerIds: [killedPlayerId] },
      }),
    ], players);

    expect(playback[0]?.players.find((player) => player.playerId === killedPlayerId))
      .toMatchObject({ status: "alive" });
    expect(playback[1]?.players.find((player) => player.playerId === killedPlayerId))
      .toMatchObject({ status: "dead", highlighted: true });
  });

  it("carries presenter details into playback scenes", () => {
    const playback = compilePublicPlayback([
      event(1, { kind: "public" }, {
        type: "exile_resolved",
        phase: "vote",
        payload: {
          exiledPlayerId: players[1].playerId,
          tiedPlayerIds: [],
          voteTable: [
            {
              voterPlayerId: players[0].playerId,
              targetPlayerId: players[1].playerId,
            },
          ],
          voteType: "exile",
          dayNumber: 1,
          round: 1,
          revealedRoles: [],
        },
      }),
    ], players);

    expect(playback[0]).toMatchObject({
      kind: "resolution",
      details: [expect.stringContaining("->")],
    });
    expect(playback[0]?.players.find((player) => player.playerId === players[1].playerId))
      .toMatchObject({ status: "dead", highlighted: true });
  });

  it("suppresses individual public vote casts and plays the resolved vote as one result scene", () => {
    const playback = compilePublicPlayback([
      event(1, { kind: "public" }, {
        type: "vote_cast",
        phase: "vote",
        actorPlayerId: players[0].playerId,
        targetPlayerIds: [players[1].playerId],
        payload: {
          voterPlayerId: players[0].playerId,
          targetPlayerId: players[1].playerId,
          dayNumber: 1,
          round: 1,
          voteType: "exile",
        },
      }),
      event(2, { kind: "public" }, {
        type: "vote_cast",
        phase: "vote",
        actorPlayerId: players[1].playerId,
        targetPlayerIds: [],
        payload: {
          voterPlayerId: players[1].playerId,
          targetPlayerId: null,
          dayNumber: 1,
          round: 1,
          voteType: "exile",
        },
      }),
      event(3, { kind: "public" }, {
        type: "exile_resolved",
        phase: "vote",
        targetPlayerIds: [players[1].playerId],
        payload: {
          exiledPlayerId: players[1].playerId,
          tiedPlayerIds: [],
          voteTable: [
            {
              voterPlayerId: players[0].playerId,
              targetPlayerId: players[1].playerId,
            },
            {
              voterPlayerId: players[1].playerId,
              targetPlayerId: null,
            },
          ],
          voteType: "exile",
          dayNumber: 1,
          round: 1,
          revealedRoles: [],
        },
      }),
    ], players);

    expect(playback).toHaveLength(1);
    expect(playback[0]).toMatchObject({
      index: 3,
      kind: "resolution",
      title: "投票结算",
      text: expect.stringContaining("出局"),
      details: [
        expect.stringContaining("->"),
        expect.stringContaining("弃票"),
      ],
    });
  });

  it("can compile director playback with private night actions", () => {
    const playback = compilePublicPlayback([
      event(1, { kind: "public" }),
      event(2, { kind: "faction_private", faction: "wolves" }, {
        type: "wolf_kill_selected",
        phase: "night",
        actorPlayerId: players[0].playerId,
        targetPlayerIds: [players[2].playerId],
        payload: { targetPlayerId: players[2].playerId },
      }),
      event(3, { kind: "player_private", playerIds: [players[2].playerId] }, {
        type: "seer_check_selected",
        phase: "night",
        actorPlayerId: players[2].playerId,
        targetPlayerIds: [players[0].playerId],
        payload: { targetPlayerId: players[0].playerId },
      }),
      event(4, { kind: "player_private", playerIds: [players[3].playerId] }, {
        type: "witch_antidote_decided",
        phase: "night",
        actorPlayerId: players[3].playerId,
        targetPlayerIds: [players[2].playerId],
        payload: { used: true, targetPlayerId: players[2].playerId },
      }),
    ], players, { audience: "director" });

    expect(playback.map((item) => item.title)).toEqual([
      "第 1 夜开始",
      "狼人刀人",
      "预言家查验",
      "女巫解药",
    ]);
  });

  it("resolves total duration and scene index from timeline milliseconds", () => {
    const playback = [
      playbackItem(1, 0, 1000),
      playbackItem(2, 1000, 2500),
      playbackItem(3, 3500, 500),
    ];

    expect(playbackTotalDurationMs(playback)).toBe(4000);
    expect(playbackIndexAtMs(playback, -1)).toBe(0);
    expect(playbackIndexAtMs(playback, 0)).toBe(0);
    expect(playbackIndexAtMs(playback, 999)).toBe(0);
    expect(playbackIndexAtMs(playback, 1000)).toBe(1);
    expect(playbackIndexAtMs(playback, 3999)).toBe(2);
    expect(playbackIndexAtMs(playback, 9999)).toBe(2);
    expect(playbackIndexAtMs([], 0)).toBe(0);
  });
});

function playbackItem(
  index: number,
  startsAtMs: number,
  durationMs: number,
): ReturnType<typeof compilePublicPlayback>[number] {
  return {
    index,
    phase: "day",
    kind: "announcement",
    title: `Scene ${index}`,
    text: "",
    details: [],
    durationMs,
    startsAtMs,
    players: [],
  };
}
