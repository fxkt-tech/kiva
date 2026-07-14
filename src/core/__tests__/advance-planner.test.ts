import { describe, expect, it } from "vitest";
import { planNextDraft } from "../advance-planner";
import { confirmDraftEvent, createDraftEvent, type DraftEvent } from "../drafts";
import type { GameEvent } from "../events";
import type { Game } from "../game";
import { createSeedGame } from "../game";
import { compilePublicPlayback } from "../playback";
import {
  factionForRuleRole,
  type DraftId,
  type EventId,
  type GameId,
  type RuleRoleId,
  type Phase,
  type PlayerId,
} from "../types";

const gameId = "game_1" as GameId;
const createdAt = "2026-06-26T00:00:00.000Z";

function createGame() {
  return createSeedGame({ gameId, createdAt });
}

function draftId(index: number) {
  return `draft_${index}` as DraftId;
}

function eventId(index: number) {
  return `event_${index}` as EventId;
}

function confirmNext(
  events: readonly GameEvent[],
  draft: DraftEvent,
): readonly GameEvent[] {
  return [
    ...events,
    confirmDraftEvent({
      draft,
      eventId: eventId(events.length + 1),
      index: events.filter((event) => event.status === "active").length + 1,
      createdAt,
    }),
  ];
}

function requireDraft(draft: DraftEvent | null): DraftEvent {
  if (!draft) {
    throw new Error("Expected planner to return a draft");
  }

  return draft;
}

describe("advance planner", () => {
  it("drafts the first role assignment as player-private", () => {
    const game = createGame();
    const draft = planNextDraft({
      game,
      events: [],
      draftId: draftId(1),
      createdAt,
    });

    expect(draft).toMatchObject({
      id: draftId(1),
      status: "draft",
      gameId,
      type: "role_assigned",
      phase: "setup",
      visibility: {
        kind: "player_private",
        playerIds: [game.players[0].playerId],
      },
      payload: {
        playerId: game.players[0].playerId,
        role: game.players[0].ruleRole.id,
        faction: game.players[0].ruleRole.faction,
      },
    });
  });

  it("drafts night 1 after all roles are assigned", () => {
    const game = createGame();
    let events: readonly GameEvent[] = [];

    for (let index = 1; index <= game.players.length; index += 1) {
      const draft = planNextDraft({
        game,
        events,
        draftId: draftId(index),
        createdAt,
      });

      const confirmedDraft = requireDraft(draft);
      expect(confirmedDraft.type).toBe("role_assigned");
      events = confirmNext(events, confirmedDraft);
    }

    const draft = planNextDraft({
      game,
      events,
      draftId: draftId(7),
      createdAt,
    });

    expect(draft).toMatchObject({
      type: "phase_started",
      phase: "night",
      visibility: { kind: "public" },
      payload: { phase: "night", dayNumber: 1 },
    });
  });

  it("plans the deterministic first night chain", () => {
    const game = createGame();
    let events: readonly GameEvent[] = [];

    for (let index = 1; index <= game.players.length + 1; index += 1) {
      const draft = planNextDraft({
        game,
        events,
        draftId: draftId(index),
        createdAt,
      });

      events = confirmNext(events, requireDraft(draft));
    }

    const plannedTypes: GameEvent["type"][] = [];

    for (
      let index = game.players.length + 2;
      index <= game.players.length + 19;
      index += 1
    ) {
      const draft = planNextDraft({
        game,
        events,
        draftId: draftId(index),
        createdAt,
      });

      const confirmedDraft = requireDraft(draft);
      plannedTypes.push(confirmedDraft.type);
      events = confirmNext(events, confirmedDraft);
    }

    expect(plannedTypes).toEqual([
      "guard_protect_selected",
      "wolf_leader_selected",
      "wolf_strategy_given",
      "wolf_opinion_given",
      "wolf_opinion_given",
      "wolf_opinion_given",
      "wolf_vote_cast",
      "wolf_vote_cast",
      "wolf_vote_cast",
      "wolf_vote_cast",
      "wolf_vote_resolved",
      "seer_check_selected",
      "seer_check_result",
      "witch_death_info_shown",
      "witch_antidote_decided",
      "witch_poison_decided",
      "night_resolved",
      "death_announced",
    ]);
    expect(
      planNextDraft({ game, events, draftId: draftId(26), createdAt }),
    ).toMatchObject({
      type: "phase_started",
      payload: { phase: "speech", dayNumber: 1 },
    });
  });

  it("ignores superseded events when choosing the next draft", () => {
    const game = createGame();
    const staleRoleAssignment = confirmDraftEvent({
      draft: requireDraft(
        planNextDraft({
          game,
          events: [],
          draftId: draftId(1),
          createdAt,
        }),
      ),
      eventId: eventId(1),
      index: 1,
      createdAt,
    });

    const draft = planNextDraft({
      game,
      events: [{ ...staleRoleAssignment, status: "superseded" }],
      draftId: draftId(2),
      createdAt,
    });

    expect(draft).toMatchObject({
      type: "role_assigned",
      payload: { playerId: game.players[0].playerId },
    });
  });

  it("keeps the immutable Game snapshot as role truth after setup", () => {
    const game = createGame();
    const events = [
      ...assignedRoleEvents(game.players.map((player) => player.ruleRole.id)),
      confirmedPhaseStarted("night", 1, game.players.length + 1),
    ];
    const swappedEvents = events.map((event) => {
      if (
        event.type === "role_assigned" &&
        event.payload.playerId === game.players[1].playerId
      ) {
        return {
          ...event,
          payload: {
            ...event.payload,
            role: "villager" as RuleRoleId,
            faction: factionForRuleRole("villager"),
          },
        };
      }

      if (
        event.type === "role_assigned" &&
        event.payload.playerId === game.players[4].playerId
      ) {
        return {
          ...event,
          payload: {
            ...event.payload,
            role: "seer" as RuleRoleId,
            faction: factionForRuleRole("seer"),
          },
        };
      }

      return event;
    });

    const guardProtect = requireDraft(
      planNextDraft({
        game,
        events: swappedEvents,
        draftId: draftId(20),
        createdAt,
      }),
    );
    let actionEvents = confirmNext(swappedEvents, guardProtect);
    let seerCheck: DraftEvent | null = null;
    for (let step = 0; step < 20; step += 1) {
      const draft = planNextDraft({
        game,
        events: actionEvents,
        draftId: draftId(21 + step),
        createdAt,
      });
      if (draft?.type === "seer_check_selected") {
        seerCheck = draft;
        break;
      }
      actionEvents = confirmNext(actionEvents, requireDraft(draft));
    }

    expect(seerCheck).toMatchObject({
      type: "seer_check_selected",
      actorPlayerId: game.players[1].playerId,
    });
  });

  it("does not plan first-night actions outside first night", () => {
    const game = createGame();
    const events = [
      ...assignedRoleEvents(game.players.map((player) => player.ruleRole.id)),
      confirmedPhaseStarted("day", 1, game.players.length + 1),
    ];

    expect(
      planNextDraft({
        game,
        events,
        draftId: draftId(30),
        createdAt,
      }),
    ).toBeNull();
  });

  it("does not resolve night when active witch decisions violate rules", () => {
    const game = createGame();
    const wolf = playerByRole(game, "werewolf");
    const seer = playerByRole(game, "seer");
    const witch = playerByRole(game, "witch");
    const guard = playerByRole(game, "guard");
    const wolves = game.players.filter((player) => player.ruleRole.id === "werewolf");
    const killedPlayerId = game.players[0].playerId;
    const poisonTargetId = game.players[4].playerId;
    const events: readonly GameEvent[] = [
      ...assignedRoleEvents(game.players.map((player) => player.ruleRole.id)),
      confirmedPhaseStarted("night", 1, game.players.length + 1),
      confirmedEvent(
        createDraftEvent({
          id: draftId(8),
          gameId,
          type: "guard_protect_selected",
          phase: "night",
          actorPlayerId: guard.playerId,
          targetPlayerIds: [guard.playerId],
          visibility: { kind: "player_private", playerIds: [guard.playerId] },
          payload: { targetPlayerId: guard.playerId },
          createdAt,
        }),
        game.players.length + 2,
      ),
      confirmedEvent(createDraftEvent({
        id: draftId(9), gameId, type: "wolf_leader_selected", phase: "night",
        actorPlayerId: wolf.playerId, targetPlayerIds: [wolf.playerId],
        visibility: { kind: "host_only" }, payload: { leaderPlayerId: wolf.playerId }, createdAt,
      }), game.players.length + 3),
      confirmedEvent(createDraftEvent({
        id: draftId(90), gameId, type: "wolf_strategy_given", phase: "night",
        actorPlayerId: wolf.playerId, visibility: { kind: "faction_private", faction: "wolves" },
        payload: { playerId: wolf.playerId, text: "strategy", dayNumber: 1 }, createdAt,
      }), game.players.length + 30),
      ...wolves.filter((player) => player.playerId !== wolf.playerId).map((player, index) =>
        confirmedEvent(createDraftEvent({
          id: draftId(91 + index), gameId, type: "wolf_opinion_given", phase: "night",
          actorPlayerId: player.playerId, visibility: { kind: "faction_private", faction: "wolves" },
          payload: { playerId: player.playerId, text: "opinion", dayNumber: 1 }, createdAt,
        }), game.players.length + 31 + index),
      ),
      ...wolves.map((player, index) => confirmedEvent(createDraftEvent({
        id: draftId(95 + index), gameId, type: "wolf_vote_cast", phase: "night",
        actorPlayerId: player.playerId, targetPlayerIds: [killedPlayerId], visibility: { kind: "host_only" },
        payload: { voterPlayerId: player.playerId, targetPlayerId: killedPlayerId, dayNumber: 1 }, createdAt,
      }), game.players.length + 35 + index)),
      confirmedEvent(createDraftEvent({
        id: draftId(99), gameId, type: "wolf_vote_resolved", phase: "night",
        targetPlayerIds: [killedPlayerId], visibility: { kind: "faction_private", faction: "wolves" },
        payload: {
          votes: wolves.map((player) => ({ voterPlayerId: player.playerId, targetPlayerId: killedPlayerId })),
          tallies: [{ targetPlayerId: killedPlayerId, count: wolves.length }],
          tiedTargetPlayerIds: [killedPlayerId], targetPlayerId: killedPlayerId,
          resolution: "majority", dayNumber: 1,
        }, createdAt,
      }), game.players.length + 40),
      confirmedEvent(
        createDraftEvent({
          id: draftId(10),
          gameId,
          type: "seer_check_selected",
          phase: "night",
          actorPlayerId: seer.playerId,
          targetPlayerIds: [wolf.playerId],
          visibility: {
            kind: "player_private",
            playerIds: [seer.playerId],
          },
          payload: { targetPlayerId: wolf.playerId },
          createdAt,
        }),
        game.players.length + 4,
      ),
      confirmedEvent(
        createDraftEvent({
          id: draftId(11),
          gameId,
          type: "seer_check_result",
          phase: "night",
          actorPlayerId: seer.playerId,
          targetPlayerIds: [wolf.playerId],
          visibility: {
            kind: "player_private",
            playerIds: [seer.playerId],
          },
          payload: { targetPlayerId: wolf.playerId, result: "wolves" },
          createdAt,
        }),
        game.players.length + 5,
      ),
      confirmedEvent(
        createDraftEvent({
          id: draftId(12),
          gameId,
          type: "witch_death_info_shown",
          phase: "night",
          actorPlayerId: witch.playerId,
          targetPlayerIds: [killedPlayerId],
          visibility: {
            kind: "player_private",
            playerIds: [witch.playerId],
          },
          payload: { killedPlayerId },
          createdAt,
        }),
        game.players.length + 6,
      ),
      confirmedEvent(
        createDraftEvent({
          id: draftId(13),
          gameId,
          type: "witch_antidote_decided",
          phase: "night",
          actorPlayerId: witch.playerId,
          targetPlayerIds: [killedPlayerId],
          visibility: {
            kind: "player_private",
            playerIds: [witch.playerId],
          },
          payload: { used: true, targetPlayerId: killedPlayerId },
          createdAt,
        }),
        game.players.length + 7,
      ),
      confirmedEvent(
        createDraftEvent({
          id: draftId(14),
          gameId,
          type: "witch_poison_decided",
          phase: "night",
          actorPlayerId: witch.playerId,
          targetPlayerIds: [poisonTargetId],
          visibility: {
            kind: "player_private",
            playerIds: [witch.playerId],
          },
          payload: { used: true, targetPlayerId: poisonTargetId },
          createdAt,
        }),
        game.players.length + 8,
      ),
    ];

    expect(
      planNextDraft({
        game,
        events,
        draftId: draftId(15),
        createdAt,
      }),
    ).toBeNull();
  });
});

describe("complete deterministic game flow", () => {
  it("can advance from setup to game_ended without LLM, then returns null", () => {
    const game = createGame();
    let events: readonly GameEvent[] = [];
    const plannedTypes: GameEvent["type"][] = [];

    for (let step = 1; step <= 320; step += 1) {
      const draft = planNextDraft({
        game,
        events,
        draftId: draftId(100 + step),
        createdAt,
      });
      if (!draft) {
        break;
      }

      plannedTypes.push(draft.type);
      events = confirmNext(events, draft);

      if (draft.type === "game_ended") {
        break;
      }
    }

    expect(plannedTypes).toContain("last_words_given");
    expect(plannedTypes).toContain("day_speech_given");
    expect(plannedTypes).toContain("vote_cast");
    expect(plannedTypes).toContain("exile_resolved");
    expect(plannedTypes).toContain("game_ended");
    expect(
      plannedTypes.includes("phase_started") &&
        events.some(
          (event) =>
            event.type === "phase_started" &&
            event.payload.phase === "night" &&
            event.payload.dayNumber === 2,
        ),
    ).toBe(true);
    expect(plannedTypes.at(-1)).toBe("game_ended");
    expect(
      planNextDraft({
        game,
        events,
        draftId: draftId(999),
        createdAt,
      }),
    ).toBeNull();
  });

  it("renders public playback from structured facts with speakers, deaths, and winner", () => {
    const game = createGame();
    const events = confirmCompleteGame(game);
    const playback = compilePublicPlayback(events, game.players, {
      presenter: game.presenter,
    });

    expect(
      playback.some(
        (item) =>
          item.title.includes("发言") &&
          /\d+ 号 .+发言/.test(`${item.title} ${item.text}`),
      ),
    ).toBe(true);
    expect(
      playback.some(
        (item) =>
          item.title.includes("死讯") &&
          /倒下的是：\d+号/.test(`${item.title} ${item.text}`),
      ),
    ).toBe(true);
    expect(
      playback.some(
        (item) =>
          item.title.includes("游戏结束") &&
          /(好人|狼人)阵营胜利/.test(`${item.title} ${item.text}`),
      ),
    ).toBe(true);
  });

  it("plans game end after exile kills the last wolf", () => {
    const game = createGame();
    const wolfIds = game.players
      .filter((player) => player.ruleRole.id === "werewolf")
      .map((player) => player.playerId);
    const events: readonly GameEvent[] = [
      ...assignedRoleEvents(game.players.map((player) => player.ruleRole.id)),
      confirmedPhaseStarted("night", 1, game.players.length + 1),
      confirmedEvent(
        createDraftEvent({
          id: draftId(8),
          gameId,
          type: "night_resolved",
          phase: "night",
          targetPlayerIds: wolfIds.slice(0, -1),
          visibility: { kind: "host_only" },
          payload: { deadPlayerIds: wolfIds.slice(0, -1) },
          createdAt,
        }),
        game.players.length + 2,
      ),
      confirmedPhaseStarted("vote", 1, game.players.length + 3),
      confirmedEvent(
        createDraftEvent({
          id: draftId(10),
          gameId,
          type: "exile_resolved",
          phase: "vote",
          targetPlayerIds: [wolfIds.at(-1)!],
          visibility: { kind: "public" },
          payload: {
            exiledPlayerId: wolfIds.at(-1)!,
            tiedPlayerIds: [],
            voteType: "exile",
            voteTable: [],
            dayNumber: 1,
            round: 1,
            revealedRoles: [],
          },
          createdAt,
        }),
        game.players.length + 4,
      ),
    ];

    expect(
      planNextDraft({
        game,
        events,
        draftId: draftId(301),
        createdAt,
      }),
    ).toMatchObject({
      type: "game_ended",
      payload: {
        winner: "good",
        reason: "all_wolves_dead",
        dayNumber: 1,
      },
    });
  });

  it("plans PK speeches, revote, and PK exile resolution after a tied daily vote", () => {
    const game = createGame();
    let events = createEventsThroughDailyTie(game);

    const pkPhase = requireDraft(
      planNextDraft({
        game,
        events,
        draftId: draftId(400),
        createdAt,
      }),
    );
    expect(pkPhase).toMatchObject({
      type: "phase_started",
      payload: { phase: "pk", dayNumber: 1 },
    });
    events = confirmNext(events, pkPhase);

    const pkSpeech1 = requireDraft(
      planNextDraft({ game, events, draftId: draftId(401), createdAt }),
    );
    expect(pkSpeech1.type).toBe("pk_speech_given");
    events = confirmNext(events, pkSpeech1);

    const pkSpeech2 = requireDraft(
      planNextDraft({ game, events, draftId: draftId(402), createdAt }),
    );
    expect(pkSpeech2.type).toBe("pk_speech_given");
    events = confirmNext(events, pkSpeech2);

    const pkVote = requireDraft(
      planNextDraft({ game, events, draftId: draftId(403), createdAt }),
    );
    expect(pkVote).toMatchObject({
      type: "vote_cast",
      payload: { voteType: "pk", round: 2, dayNumber: 1 },
    });

    events = confirmAllCurrentVotes(game, confirmNext(events, pkVote), "pk", 2);
    const pkResolved = planNextDraft({
      game,
      events,
      draftId: draftId(404),
      createdAt,
    });

    expect(pkResolved).toMatchObject({
      type: "exile_resolved",
      payload: { voteType: "pk", round: 2, dayNumber: 1 },
    });
  });

  it("keeps individual votes sealed until the configured reveal point", () => {
    const game = createGame();
    const events = confirmAllUntil(game, "vote_cast");

    expect(events.at(-1)).toMatchObject({
      type: "vote_cast",
      visibility: { kind: "host_only" },
    });
  });

  it("reveals individual votes immediately when configured", () => {
    const baseGame = createGame();
    const game = {
      ...baseGame,
      ruleset: { ...baseGame.ruleset, voteReveal: "immediate" as const },
    };
    const events = confirmAllUntil(game, "vote_cast");

    expect(events.at(-1)).toMatchObject({
      type: "vote_cast",
      visibility: { kind: "public" },
    });
  });

  it("plans game end after night resolution kills all wolves", () => {
    const game = createGame();
    const wolfIds = game.players
      .filter((player) => player.ruleRole.id === "werewolf")
      .map((player) => player.playerId);
    const events: readonly GameEvent[] = [
      ...assignedRoleEvents(game.players.map((player) => player.ruleRole.id)),
      confirmedPhaseStarted("night", 1, game.players.length + 1),
      confirmedEvent(
        createDraftEvent({
          id: draftId(8),
          gameId,
          type: "night_resolved",
          phase: "night",
          targetPlayerIds: wolfIds,
          visibility: { kind: "host_only" },
          payload: {
            deadPlayerIds: wolfIds,
          },
          createdAt,
        }),
        game.players.length + 2,
      ),
    ];

    expect(
      planNextDraft({
        game,
        events,
        draftId: draftId(500),
        createdAt,
      }),
    ).toMatchObject({
      type: "game_ended",
      payload: {
        winner: "good",
        reason: "all_wolves_dead",
        dayNumber: 1,
      },
    });
  });

  it("does not draft a witch medicine again after it was used on a previous night", () => {
    const game = createGame();
    let events: readonly GameEvent[] = [];

    for (let step = 1; step <= 320; step += 1) {
      const draft = requireDraft(
        planNextDraft({
          game,
          events,
          draftId: draftId(900 + step),
          createdAt,
        }),
      );

      if (draft.type === "witch_antidote_decided") {
        const wolfKill = [...events].reverse().find(
          (event): event is Extract<GameEvent, { type: "wolf_vote_resolved" }> =>
            event.type === "wolf_vote_resolved" && event.payload.targetPlayerId !== null,
        );
        if (!wolfKill) {
          throw new Error("Expected wolf kill before witch antidote");
        }

        const killedPlayerId = wolfKill.payload.targetPlayerId!;
        events = confirmNext(events, {
          ...draft,
          targetPlayerIds: [killedPlayerId],
          payload: { used: true, targetPlayerId: killedPlayerId },
        });
        continue;
      }

      events = confirmNext(events, draft);

      if (
        draft.type === "phase_started" &&
        draft.payload.phase === "night" &&
        draft.payload.dayNumber === 2
      ) {
        break;
      }
    }

    const nightTwoDraftTypes: GameEvent["type"][] = [];
    for (let step = 1; step <= 20; step += 1) {
      const draft = planNextDraft({
        game,
        events,
        draftId: draftId(1100 + step),
        createdAt,
      });
      if (!draft) {
        break;
      }

      nightTwoDraftTypes.push(draft.type);
      events = confirmNext(events, draft);

      if (draft.type === "witch_poison_decided") {
        break;
      }
    }

    expect(nightTwoDraftTypes).toContain("witch_death_info_shown");
    expect(nightTwoDraftTypes).not.toContain("witch_antidote_decided");
    expect(nightTwoDraftTypes).toContain("witch_poison_decided");
  });
});

function assignedRoleEvents(roles: readonly RuleRoleId[]): readonly GameEvent[] {
  const game = createGame();
  return game.players.map((player, index) =>
    confirmedEvent(
      createDraftEvent({
        id: draftId(index + 1),
        gameId,
        type: "role_assigned",
        phase: "setup",
        targetPlayerIds: [player.playerId],
        visibility: { kind: "player_private", playerIds: [player.playerId] },
        payload: {
          playerId: player.playerId,
          role: roles[index],
          faction: factionForRuleRole(roles[index]),
        },
        createdAt,
      }),
      index + 1,
    ),
  );
}

function confirmedPhaseStarted(
  phase: Phase,
  dayNumber: number,
  index: number,
): GameEvent {
  return confirmedEvent(
    createDraftEvent({
      id: draftId(index),
      gameId,
      type: "phase_started",
      phase,
      visibility: { kind: "public" },
      payload: { phase, dayNumber },
      createdAt,
    }),
    index,
  );
}

function confirmedEvent(draft: DraftEvent, index: number): GameEvent {
  return confirmDraftEvent({
    draft,
    eventId: eventId(index),
    index,
    createdAt,
  });
}

function confirmAllUntil(
  game: Game,
  stopType: GameEvent["type"],
): readonly GameEvent[] {
  let events: readonly GameEvent[] = [];

  for (let step = 1; step <= 320; step += 1) {
    const draft = requireDraft(
      planNextDraft({
        game,
        events,
        draftId: draftId(600 + step),
        createdAt,
      }),
    );
    events = confirmNext(events, draft);

    if (draft.type === stopType) {
      return events;
    }
  }

  throw new Error(`Planner did not reach ${stopType}`);
}

function confirmCompleteGame(game: Game): readonly GameEvent[] {
  let events: readonly GameEvent[] = [];

  for (let step = 1; step <= 320; step += 1) {
    const draft = requireDraft(
      planNextDraft({
        game,
        events,
        draftId: draftId(1200 + step),
        createdAt,
      }),
    );
    events = confirmNext(events, draft);

    if (draft.type === "game_ended") {
      return events;
    }
  }

  throw new Error("Planner did not reach game_ended");
}

function createEventsThroughDailyTie(game: Game): readonly GameEvent[] {
  let events = confirmAllUntil(game, "vote_cast");
  events = confirmAllCurrentVotes(game, events, "exile", 1);

  const voteEvents = events.filter(
    (event): event is Extract<GameEvent, { type: "vote_cast" }> =>
      event.type === "vote_cast" &&
      event.payload.voteType === "exile" &&
      event.payload.round === 1,
  );
  const firstVoteIndex = events.findIndex((event) => event === voteEvents[0]);
  const tiedPlayerIds = [game.players[1].playerId, game.players[3].playerId];
  const voteTargets = [
    tiedPlayerIds[0],
    tiedPlayerIds[1],
    tiedPlayerIds[0],
    tiedPlayerIds[1],
    game.players[4].playerId,
  ];

  events = events.map((event, index) => {
    if (index < firstVoteIndex || event.type !== "vote_cast") {
      return event;
    }

    const voteOffset = index - firstVoteIndex;
    const targetPlayerId = voteTargets[voteOffset % voteTargets.length];
    return {
      ...event,
      targetPlayerIds: [targetPlayerId],
      payload: {
        ...event.payload,
        targetPlayerId,
      },
    };
  });

  const tieResolution = requireDraft(
    planNextDraft({
      game,
      events,
      draftId: draftId(799),
      createdAt,
    }),
  );
  expect(tieResolution).toMatchObject({
    type: "exile_resolved",
    payload: {
      exiledPlayerId: null,
      tiedPlayerIds,
      voteType: "exile",
      round: 1,
    },
  });

  return confirmNext(events, tieResolution);
}

function confirmAllCurrentVotes(
  game: Game,
  initialEvents: readonly GameEvent[],
  voteType: "exile" | "pk",
  round: number,
): readonly GameEvent[] {
  let events = initialEvents;

  for (let step = 1; step <= game.players.length; step += 1) {
    const draft = planNextDraft({
      game,
      events,
      draftId: draftId(700 + events.length + step),
      createdAt,
    });

    if (!draft || draft.type !== "vote_cast") {
      return events;
    }

    expect(draft.payload.voteType).toBe(voteType);
    expect(draft.payload.round).toBe(round);
    events = confirmNext(events, draft);
  }

  return events;
}

function playerByRole(game: Game, role: RuleRoleId) {
  const player = game.players.find((candidate) => candidate.ruleRole.id === role);
  if (!player) {
    throw new Error(`Missing player for role ${role}`);
  }

  return player;
}
