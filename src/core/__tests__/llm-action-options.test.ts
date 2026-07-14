import { describe, expect, it } from "vitest";
import type { DraftEvent } from "../drafts";
import type { GameEvent } from "../events";
import { createSeedGame } from "../game";
import { legalActionOptions } from "../llm-action-options";
import type { DraftId, EventId, GameId, PlayerId } from "../types";

const gameId = "game_1" as GameId;
const createdAt = "2026-06-26T00:00:00.000Z";
const game = createSeedGame({ gameId, createdAt });
const seer = playerByRole("seer");
const witch = playerByRole("witch");
const guard = playerByRole("guard");
const villager = playerByRole("villager");

describe("LLM action options", () => {
  it("limits PK votes to the tied candidates", () => {
    const tiedPlayerIds = [game.players[1]!.playerId, game.players[3]!.playerId];
    const events = [
      phaseStarted(1, "speech"),
      {
        ...baseEvent(2),
        type: "exile_resolved",
        phase: "vote",
        visibility: { kind: "public" },
        payload: {
          exiledPlayerId: null,
          tiedPlayerIds,
          voteType: "exile",
          voteTable: [],
          dayNumber: 1,
          round: 1,
          revealedRoles: [],
        },
      },
      phaseStarted(3, "pk"),
    ] satisfies readonly GameEvent[];

    expect(
      legalActionOptions({
        game,
        events,
        draft: voteDraft("pk", villager.playerId),
      }).targetPlayerIds,
    ).toEqual(tiedPlayerIds);
  });

  it("disallows abstaining when the ruleset forbids it", () => {
    const strictGame = {
      ...game,
      ruleset: { ...game.ruleset, allowAbstainVote: false },
    };

    expect(
      legalActionOptions({
        game: strictGame,
        events: [phaseStarted(1, "vote")],
        draft: voteDraft("exile", villager.playerId),
      }).allowNoTarget,
    ).toBe(false);
  });

  it("excludes targets the seer has already checked", () => {
    const checkedPlayerId = game.players[0]!.playerId;
    const events = [
      phaseStarted(1, "night"),
      {
        ...baseEvent(2),
        type: "seer_check_result",
        phase: "night",
        actorPlayerId: seer.playerId,
        targetPlayerIds: [checkedPlayerId],
        visibility: { kind: "player_private", playerIds: [seer.playerId] },
        payload: { targetPlayerId: checkedPlayerId, result: "good" },
      },
    ] satisfies readonly GameEvent[];

    const options = legalActionOptions({
      game,
      events,
      draft: seerDraft(checkedPlayerId),
    });

    expect(options.targetPlayerIds).not.toContain(checkedPlayerId);
    expect(options.targetPlayerIds).not.toContain(seer.playerId);
  });

  it("prevents poison after using antidote when same-night dual use is forbidden", () => {
    const rescuedPlayerId = game.players[0]!.playerId;
    const events = [
      phaseStarted(1, "night"),
      {
        ...baseEvent(2),
        type: "witch_antidote_decided",
        phase: "night",
        actorPlayerId: witch.playerId,
        targetPlayerIds: [rescuedPlayerId],
        visibility: { kind: "player_private", playerIds: [witch.playerId] },
        payload: { used: true, targetPlayerId: rescuedPlayerId },
      },
    ] satisfies readonly GameEvent[];

    expect(
      legalActionOptions({
        game,
        events,
        draft: witchPoisonDraft(),
      }),
    ).toMatchObject({ targetPlayerIds: [], canUse: false });
  });

  it("excludes the previous guard target when consecutive protection is forbidden", () => {
    const previousTarget = villager.playerId;
    const events = [
      phaseStarted(1, "night", 1),
      {
        ...baseEvent(2),
        type: "guard_protect_selected",
        phase: "night",
        actorPlayerId: guard.playerId,
        targetPlayerIds: [previousTarget],
        visibility: { kind: "player_private", playerIds: [guard.playerId] },
        payload: { targetPlayerId: previousTarget },
      },
      phaseStarted(3, "day", 1),
      phaseStarted(4, "night", 2),
    ] satisfies readonly GameEvent[];

    const options = legalActionOptions({
      game,
      events,
      draft: guardDraft(previousTarget),
    });

    expect(options.targetPlayerIds).not.toContain(previousTarget);
  });

  it("forces no antidote use for forbidden first-night self-save", () => {
    const strictGame = {
      ...game,
      ruleset: { ...game.ruleset, witchFirstNightSelfSave: false },
    };
    const events = [
      phaseStarted(1, "night", 1),
      {
        ...baseEvent(2),
        type: "wolf_vote_resolved",
        phase: "night",
        targetPlayerIds: [witch.playerId],
        visibility: { kind: "faction_private", faction: "wolves" },
        payload: {
          votes: [],
          tallies: [],
          tiedTargetPlayerIds: [witch.playerId],
          targetPlayerId: witch.playerId,
          resolution: "majority",
          dayNumber: 1,
        },
      },
    ] satisfies readonly GameEvent[];

    expect(
      legalActionOptions({
        game: strictGame,
        events,
        draft: witchAntidoteDraft(),
      }),
    ).toMatchObject({ targetPlayerIds: [], allowNoTarget: true, canUse: false });
  });
});

function voteDraft(
  voteType: "exile" | "pk",
  voterPlayerId: PlayerId,
): Extract<DraftEvent, { type: "vote_cast" }> {
  return {
    ...draftBase("vote_cast"),
    phase: "vote",
    actorPlayerId: voterPlayerId,
    visibility: { kind: "host_only" },
    payload: {
      voterPlayerId,
      targetPlayerId: null,
      dayNumber: 1,
      round: voteType === "pk" ? 2 : 1,
      voteType,
    },
  };
}

function seerDraft(
  targetPlayerId: PlayerId,
): Extract<DraftEvent, { type: "seer_check_selected" }> {
  return {
    ...draftBase("seer_check_selected"),
    phase: "night",
    actorPlayerId: seer.playerId,
    targetPlayerIds: [targetPlayerId],
    visibility: { kind: "player_private", playerIds: [seer.playerId] },
    payload: { targetPlayerId },
  };
}

function witchPoisonDraft(): Extract<
  DraftEvent,
  { type: "witch_poison_decided" }
> {
  return {
    ...draftBase("witch_poison_decided"),
    phase: "night",
    actorPlayerId: witch.playerId,
    visibility: { kind: "player_private", playerIds: [witch.playerId] },
    payload: { used: false, targetPlayerId: null },
  };
}

function witchAntidoteDraft(): Extract<
  DraftEvent,
  { type: "witch_antidote_decided" }
> {
  return {
    ...draftBase("witch_antidote_decided"),
    phase: "night",
    actorPlayerId: witch.playerId,
    visibility: { kind: "player_private", playerIds: [witch.playerId] },
    payload: { used: false, targetPlayerId: null },
  };
}

function guardDraft(
  targetPlayerId: PlayerId,
): Extract<DraftEvent, { type: "guard_protect_selected" }> {
  return {
    ...draftBase("guard_protect_selected"),
    phase: "night",
    actorPlayerId: guard.playerId,
    targetPlayerIds: [targetPlayerId],
    visibility: { kind: "player_private", playerIds: [guard.playerId] },
    payload: { targetPlayerId },
  };
}

function phaseStarted(
  index: number,
  phase: Extract<GameEvent, { type: "phase_started" }>["payload"]["phase"],
  dayNumber = 1,
): Extract<GameEvent, { type: "phase_started" }> {
  return {
    ...baseEvent(index),
    type: "phase_started",
    phase,
    visibility: { kind: "public" },
    payload: { phase, dayNumber },
  };
}

function draftBase<Type extends DraftEvent["type"]>(type: Type) {
  return {
    id: `draft_${type}` as DraftId,
    gameId,
    status: "draft" as const,
    type,
    createdAt,
  };
}

function baseEvent(index: number) {
  return {
    id: `event_${index}` as EventId,
    gameId,
    index,
    status: "active" as const,
    createdAt,
  };
}

function playerByRole(role: typeof game.players[number]["ruleRole"]["id"]) {
  const player = game.players.find((candidate) => candidate.ruleRole.id === role);
  if (!player) throw new Error(`Missing player for role ${role}`);
  return player;
}
