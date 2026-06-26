import { describe, expect, it } from "vitest";
import { planNextDraft } from "../advance-planner";
import { confirmDraftEvent, createDraftEvent, type DraftEvent } from "../drafts";
import type { GameEvent } from "../events";
import { createSeedGame } from "../game";
import {
  factionForRole,
  type DraftId,
  type EventId,
  type GameId,
  type GameRole,
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
        role: game.players[0].gameRole,
        faction: game.players[0].faction,
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
      index <= game.players.length + 9;
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
      "wolf_kill_selected",
      "seer_check_selected",
      "seer_check_result",
      "witch_death_info_shown",
      "witch_antidote_decided",
      "witch_poison_decided",
      "night_resolved",
      "death_announced",
    ]);
    expect(planNextDraft({ game, events, draftId: draftId(16), createdAt })).toBe(
      null,
    );
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

  it("uses active role assignments as role truth after setup", () => {
    const game = createGame();
    const events = [
      ...assignedRoleEvents(game.players.map((player) => player.gameRole)),
      confirmedPhaseStarted("night", 1, game.players.length + 1),
    ];
    const swappedEvents = events.map((event) => {
      if (
        event.type === "role_assigned" &&
        event.payload.playerId === game.players[2].playerId
      ) {
        return {
          ...event,
          payload: {
            ...event.payload,
            role: "villager" as GameRole,
            faction: factionForRole("villager"),
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
            role: "seer" as GameRole,
            faction: factionForRole("seer"),
          },
        };
      }

      return event;
    });

    const wolfKill = requireDraft(
      planNextDraft({
        game,
        events: swappedEvents,
        draftId: draftId(20),
        createdAt,
      }),
    );
    const seerCheck = planNextDraft({
      game,
      events: confirmNext(swappedEvents, wolfKill),
      draftId: draftId(21),
      createdAt,
    });

    expect(seerCheck).toMatchObject({
      type: "seer_check_selected",
      actorPlayerId: game.players[4].playerId,
    });
  });

  it("does not plan first-night actions outside first night", () => {
    const game = createGame();
    const events = [
      ...assignedRoleEvents(game.players.map((player) => player.gameRole)),
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
    const killedPlayerId = game.players[2].playerId;
    const poisonTargetId = game.players[4].playerId;
    const events: readonly GameEvent[] = [
      ...assignedRoleEvents(game.players.map((player) => player.gameRole)),
      confirmedPhaseStarted("night", 1, 7),
      confirmedEvent(
        createDraftEvent({
          id: draftId(8),
          gameId,
          type: "wolf_kill_selected",
          phase: "night",
          actorPlayerId: game.players[0].playerId,
          targetPlayerIds: [killedPlayerId],
          visibility: { kind: "faction_private", faction: "wolves" },
          payload: { targetPlayerId: killedPlayerId },
          createdAt,
        }),
        8,
      ),
      confirmedEvent(
        createDraftEvent({
          id: draftId(9),
          gameId,
          type: "seer_check_selected",
          phase: "night",
          actorPlayerId: game.players[2].playerId,
          targetPlayerIds: [game.players[0].playerId],
          visibility: {
            kind: "player_private",
            playerIds: [game.players[2].playerId],
          },
          payload: { targetPlayerId: game.players[0].playerId },
          createdAt,
        }),
        9,
      ),
      confirmedEvent(
        createDraftEvent({
          id: draftId(10),
          gameId,
          type: "seer_check_result",
          phase: "night",
          actorPlayerId: game.players[2].playerId,
          targetPlayerIds: [game.players[0].playerId],
          visibility: {
            kind: "player_private",
            playerIds: [game.players[2].playerId],
          },
          payload: { targetPlayerId: game.players[0].playerId, result: "wolves" },
          createdAt,
        }),
        10,
      ),
      confirmedEvent(
        createDraftEvent({
          id: draftId(11),
          gameId,
          type: "witch_death_info_shown",
          phase: "night",
          actorPlayerId: game.players[3].playerId,
          targetPlayerIds: [killedPlayerId],
          visibility: {
            kind: "player_private",
            playerIds: [game.players[3].playerId],
          },
          payload: { killedPlayerId },
          createdAt,
        }),
        11,
      ),
      confirmedEvent(
        createDraftEvent({
          id: draftId(12),
          gameId,
          type: "witch_antidote_decided",
          phase: "night",
          actorPlayerId: game.players[3].playerId,
          targetPlayerIds: [killedPlayerId],
          visibility: {
            kind: "player_private",
            playerIds: [game.players[3].playerId],
          },
          payload: { used: true, targetPlayerId: killedPlayerId },
          createdAt,
        }),
        12,
      ),
      confirmedEvent(
        createDraftEvent({
          id: draftId(13),
          gameId,
          type: "witch_poison_decided",
          phase: "night",
          actorPlayerId: game.players[3].playerId,
          targetPlayerIds: [poisonTargetId],
          visibility: {
            kind: "player_private",
            playerIds: [game.players[3].playerId],
          },
          payload: { used: true, targetPlayerId: poisonTargetId },
          createdAt,
        }),
        13,
      ),
    ];

    expect(
      planNextDraft({
        game,
        events,
        draftId: draftId(14),
        createdAt,
      }),
    ).toBeNull();
  });
});

function assignedRoleEvents(roles: readonly GameRole[]): readonly GameEvent[] {
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
          faction: factionForRole(roles[index]),
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
