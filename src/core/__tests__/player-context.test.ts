import { describe, expect, it } from "vitest";
import type { GameEvent } from "../events";
import { createSeedGame } from "../game";
import { buildPlayerLlmContext } from "../player-context";
import type { EventId, GameId } from "../types";

const gameId = "game_1" as GameId;
const createdAt = "2026-06-26T00:00:00.000Z";
const game = createSeedGame({ gameId, createdAt });
const wolf1 = playerByRole("werewolf");
const wolf2 = game.players.find(
  (player) => player.gameRole === "werewolf" && player.playerId !== wolf1.playerId,
)!;
const seer = playerByRole("seer");

describe("player LLM context", () => {
  it("includes only events visible to the viewer", () => {
    const context = buildPlayerLlmContext({
      game,
      events: sampleEvents(),
      viewerPlayerId: seer.playerId,
    });

    expect(context.visibleEvents.map((event) => event.type)).toEqual([
      "role_assigned",
      "phase_started",
      "seer_check_result",
      "death_announced",
    ]);
    expect(context.timeline.map((item) => item.title)).toContain("查验结果");
    expect(context.timeline.map((item) => item.title)).not.toContain("狼人刀人");
    expect(context.timeline.map((item) => item.title)).not.toContain("夜间结算");
    expect(context.knowledge.publicFacts.map((item) => item.title)).toContain(
      "昨夜死讯",
    );
    expect(context.knowledge.privateFacts.map((item) => item.title)).toContain(
      "查验结果",
    );
    expect(context.knowledge.publicClaims).toEqual([]);
  });

  it("exposes viewer role and prompt snapshots", () => {
    const context = buildPlayerLlmContext({
      game,
      events: [],
      viewerPlayerId: seer.playerId,
    });

    expect(context.viewer).toMatchObject({
      roleName: seer.roleName,
      team: seer.team,
      mechanicKey: seer.mechanicKey,
      characterSystemPromptSnapshot: seer.characterSystemPromptSnapshot,
      roleSystemPromptSnapshot: seer.roleSystemPromptSnapshot,
      roleActionPromptSnapshot: seer.roleActionPromptSnapshot,
    });
  });

  it("does not expose hidden roles in the safe roster", () => {
    const context = buildPlayerLlmContext({
      game,
      events: sampleEvents(),
      viewerPlayerId: seer.playerId,
    });

    const self = context.roster.find((player) => player.playerId === seer.playerId);
    const wolf = context.roster.find((player) => player.playerId === wolf1.playerId);

    expect(self).toMatchObject({
      role: "seer",
      faction: "good",
    });
    expect(wolf).not.toHaveProperty("role");
    expect(wolf).not.toHaveProperty("faction");
  });

  it("lets wolves know wolf teammates without exposing non-wolf roles", () => {
    const context = buildPlayerLlmContext({
      game,
      events: sampleEvents(),
      viewerPlayerId: wolf1.playerId,
    });

    expect(context.visibleEvents.map((event) => event.type)).not.toContain(
      "wolf_vote_cast",
    );
    expect(context.visibleEvents.map((event) => event.type)).not.toContain(
      "seer_check_result",
    );
    expect(
      context.roster.find((player) => player.playerId === wolf2.playerId),
    ).toMatchObject({
      role: "werewolf",
      faction: "wolves",
    });
    expect(
      context.roster.find((player) => player.playerId === seer.playerId),
    ).not.toHaveProperty("role");
    expect(
      context.knowledge.factionDiscussion.map((item) => item.text),
    ).toContain("首夜优先统一行动方向。");
  });

  it("does not expose a delayed-reveal individual vote to the next voter", () => {
    const events = [
      {
        ...baseEvent(1),
        type: "phase_started",
        phase: "vote",
        visibility: { kind: "public" },
        payload: { phase: "vote", dayNumber: 1 },
      },
      {
        ...baseEvent(2),
        type: "vote_cast",
        phase: "vote",
        actorPlayerId: wolf1.playerId,
        targetPlayerIds: [seer.playerId],
        visibility: { kind: "host_only" },
        payload: {
          voterPlayerId: wolf1.playerId,
          targetPlayerId: seer.playerId,
          voteType: "exile",
          dayNumber: 1,
          round: 1,
        },
      },
    ] satisfies readonly GameEvent[];

    const context = buildPlayerLlmContext({
      game,
      events,
      viewerPlayerId: seer.playerId,
    });

    expect(context.visibleEvents.map((event) => event.type)).toEqual([
      "phase_started",
    ]);
    expect(context.knowledge.publicFacts).toEqual([]);
  });

  it("keeps the current day, each player's latest earlier stance, and last words", () => {
    const other = game.players.find(
      (player) => player.playerId !== seer.playerId,
    )!;
    const events = [
      publicDaySpeech(1, seer.playerId, 1, "第一天旧观点"),
      publicDaySpeech(2, seer.playerId, 2, "第二天最新观点"),
      publicDaySpeech(3, other.playerId, 1, "另一人的旧观点"),
      {
        ...baseEvent(4),
        type: "last_words_given",
        phase: "day",
        actorPlayerId: other.playerId,
        visibility: { kind: "public" },
        payload: {
          playerId: other.playerId,
          text: "必须保留的遗言",
          dayNumber: 2,
          reason: "exile",
        },
      },
      {
        ...baseEvent(5),
        type: "phase_started",
        phase: "day",
        visibility: { kind: "public" },
        payload: { phase: "day", dayNumber: 3 },
      },
      publicDaySpeech(6, other.playerId, 3, "第三天当前观点"),
    ] satisfies readonly GameEvent[];

    const context = buildPlayerLlmContext({
      game,
      events,
      viewerPlayerId: seer.playerId,
    });

    expect(context.knowledge.publicClaims.map((item) => item.index)).toEqual([
      2, 3, 4, 6,
    ]);
    expect(context.visibleEvents.map((event) => event.index)).toEqual([
      1, 2, 3, 4, 5, 6,
    ]);
  });

  it("rejects unknown viewers", () => {
    expect(() =>
      buildPlayerLlmContext({
        game,
        events: [],
        viewerPlayerId: "missing" as typeof seer.playerId,
      }),
    ).toThrow("Viewer player not found");
  });
});

function sampleEvents(): readonly GameEvent[] {
  return [
    roleAssigned(1, wolf1),
    roleAssigned(2, seer),
    {
      ...baseEvent(3),
      type: "phase_started",
      phase: "night",
      visibility: { kind: "public" },
      payload: { phase: "night", dayNumber: 1 },
    },
    {
      ...baseEvent(4),
      type: "wolf_strategy_given",
      phase: "night",
      actorPlayerId: wolf1.playerId,
      visibility: { kind: "faction_private", faction: "wolves" },
      payload: {
        playerId: wolf1.playerId,
        text: "首夜优先统一行动方向。",
        dayNumber: 1,
      },
    },
    {
      ...baseEvent(5),
      type: "wolf_vote_cast",
      phase: "night",
      actorPlayerId: wolf1.playerId,
      targetPlayerIds: [seer.playerId],
      visibility: { kind: "host_only" },
      payload: { voterPlayerId: wolf1.playerId, targetPlayerId: seer.playerId, dayNumber: 1 },
    },
    {
      ...baseEvent(6),
      type: "seer_check_result",
      phase: "night",
      actorPlayerId: seer.playerId,
      targetPlayerIds: [wolf1.playerId],
      visibility: { kind: "player_private", playerIds: [seer.playerId] },
      payload: { targetPlayerId: wolf1.playerId, result: "wolves" },
    },
    {
      ...baseEvent(7),
      type: "night_resolved",
      phase: "night",
      targetPlayerIds: [seer.playerId],
      visibility: { kind: "host_only" },
      payload: {
        deadPlayerIds: [seer.playerId],
        deaths: [{ playerId: seer.playerId, reason: "wolf_kill" }],
      },
    },
    {
      ...baseEvent(8),
      type: "death_announced",
      phase: "day",
      targetPlayerIds: [seer.playerId],
      visibility: { kind: "public" },
      payload: { deadPlayerIds: [seer.playerId] },
    },
  ] satisfies readonly GameEvent[];
}

function roleAssigned(index: number, player: typeof game.players[number]) {
  return {
    ...baseEvent(index),
    type: "role_assigned",
    phase: "setup",
    targetPlayerIds: [player.playerId],
    visibility: { kind: "player_private", playerIds: [player.playerId] },
    payload: {
      playerId: player.playerId,
      role: player.gameRole,
      faction: player.faction,
    },
  } satisfies Extract<GameEvent, { type: "role_assigned" }>;
}

function publicDaySpeech(
  index: number,
  playerId: typeof seer.playerId,
  dayNumber: number,
  text: string,
) {
  return {
    ...baseEvent(index),
    type: "day_speech_given",
    phase: "speech",
    actorPlayerId: playerId,
    visibility: { kind: "public" },
    payload: { playerId, text, dayNumber, round: 1 },
  } satisfies Extract<GameEvent, { type: "day_speech_given" }>;
}

function baseEvent(index: number) {
  return {
    id: `event_${index}` as EventId,
    gameId,
    index,
    status: "active",
    createdAt: `2026-06-26T00:0${index}:00.000Z`,
  } as const;
}

function playerByRole(role: typeof game.players[number]["gameRole"]) {
  const player = game.players.find((candidate) => candidate.gameRole === role);
  if (player === undefined) {
    throw new Error(`Missing seeded player for role: ${role}`);
  }

  return player;
}
