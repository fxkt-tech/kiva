import { createDraftEvent, type DraftEvent } from "./drafts";
import { getActiveEvents } from "./event-log";
import type { Game } from "./game";
import type { PlayerSnapshot } from "./player";
import {
  getLegalNightTargets,
  resolveNightDeaths,
  validateWitchDecision,
} from "./rules";
import { deriveGameState } from "./state";
import type { DraftId, GameRole, PlayerId } from "./types";
import type { GameEvent } from "./events";

export type PlanNextDraftInput = {
  readonly game: Game;
  readonly events: readonly GameEvent[];
  readonly draftId: DraftId;
  readonly createdAt: string;
};

type EventOf<Type extends GameEvent["type"]> = Extract<
  GameEvent,
  { type: Type }
>;

export function planNextDraft(input: PlanNextDraftInput): DraftEvent | null {
  const events = getActiveEvents(input.events);
  if (hasEvent(events, "game_ended")) {
    return null;
  }

  const roleAssignment = planRoleAssignment(input, events);
  if (roleAssignment) {
    return roleAssignment;
  }

  const effectivePlayers = derivePlayersFromRoleAssignments(input.game, events);
  const state = deriveGameState(effectivePlayers, events);

  if (!hasNightOneStarted(events)) {
    if (state.currentPhase !== "setup") {
      return null;
    }

    return createDraftEvent({
      id: input.draftId,
      gameId: input.game.id,
      type: "phase_started",
      phase: "night",
      visibility: { kind: "public" },
      payload: { phase: "night", dayNumber: 1 },
      display: { title: "第 1 夜开始", text: "夜晚开始。" },
      createdAt: input.createdAt,
    });
  }

  if (state.currentPhase !== "night" || state.dayNumber !== 1) {
    return null;
  }

  return planFirstNightDraft(input, events, effectivePlayers, state.alivePlayerIds);
}

function planRoleAssignment(
  input: PlanNextDraftInput,
  events: readonly GameEvent[],
): DraftEvent | null {
  const assignedPlayerIds = new Set(
    events
      .filter((event): event is EventOf<"role_assigned"> =>
        event.type === "role_assigned",
      )
      .map((event) => event.payload.playerId),
  );
  const player = input.game.players.find(
    (candidate) => !assignedPlayerIds.has(candidate.playerId),
  );

  if (!player) {
    return null;
  }

  return createDraftEvent({
    id: input.draftId,
    gameId: input.game.id,
    type: "role_assigned",
    phase: "setup",
    targetPlayerIds: [player.playerId],
    visibility: {
      kind: "player_private",
      playerIds: [player.playerId],
    },
    payload: {
      playerId: player.playerId,
      role: player.gameRole,
      faction: player.faction,
    },
    display: { title: "身份牌", text: `你的身份是 ${player.gameRole}。` },
    createdAt: input.createdAt,
  });
}

function planFirstNightDraft(
  input: PlanNextDraftInput,
  events: readonly GameEvent[],
  players: readonly PlayerSnapshot[],
  alivePlayerIds: readonly PlayerId[],
): DraftEvent | null {
  const wolfKill = findEvent(events, "wolf_kill_selected");
  if (!wolfKill) {
    const wolf = firstPlayerIdByRole(players, "werewolf");
    const target = players.find((player) => player.gameRole !== "werewolf");

    if (!wolf || !target) {
      return null;
    }

    return createDraftEvent({
      id: input.draftId,
      gameId: input.game.id,
      type: "wolf_kill_selected",
      phase: "night",
      actorPlayerId: wolf,
      targetPlayerIds: [target.playerId],
      visibility: { kind: "faction_private", faction: "wolves" },
      payload: { targetPlayerId: target.playerId },
      display: { title: "狼人刀人", text: "狼人选择夜间击杀目标。" },
      createdAt: input.createdAt,
    });
  }

  const seerCheck = findEvent(events, "seer_check_selected");
  if (!seerCheck) {
    const seer = firstPlayerIdByRole(players, "seer");
    const target = firstPlayerIdByRole(players, "werewolf");

    if (!seer || !target) {
      return null;
    }

    return createDraftEvent({
      id: input.draftId,
      gameId: input.game.id,
      type: "seer_check_selected",
      phase: "night",
      actorPlayerId: seer,
      targetPlayerIds: [target],
      visibility: { kind: "player_private", playerIds: [seer] },
      payload: { targetPlayerId: target },
      display: { title: "预言家查验", text: "预言家选择查验目标。" },
      createdAt: input.createdAt,
    });
  }

  if (!hasEvent(events, "seer_check_result")) {
    const seer = firstPlayerIdByRole(players, "seer");
    const checkedPlayer = players.find(
      (player) => player.playerId === seerCheck.payload.targetPlayerId,
    );

    if (!seer || !checkedPlayer) {
      return null;
    }

    return createDraftEvent({
      id: input.draftId,
      gameId: input.game.id,
      type: "seer_check_result",
      phase: "night",
      actorPlayerId: seer,
      targetPlayerIds: [checkedPlayer.playerId],
      visibility: { kind: "player_private", playerIds: [seer] },
      payload: {
        targetPlayerId: checkedPlayer.playerId,
        result: checkedPlayer.faction,
      },
      display: { title: "查验结果", text: "预言家收到查验结果。" },
      createdAt: input.createdAt,
    });
  }

  if (!hasEvent(events, "witch_death_info_shown")) {
    const witch = firstPlayerIdByRole(players, "witch");
    if (!witch) {
      return null;
    }

    return createDraftEvent({
      id: input.draftId,
      gameId: input.game.id,
      type: "witch_death_info_shown",
      phase: "night",
      actorPlayerId: witch,
      targetPlayerIds: [wolfKill.payload.targetPlayerId],
      visibility: { kind: "player_private", playerIds: [witch] },
      payload: { killedPlayerId: wolfKill.payload.targetPlayerId },
      display: { title: "女巫信息", text: "女巫得知夜间死亡信息。" },
      createdAt: input.createdAt,
    });
  }

  if (!hasEvent(events, "witch_antidote_decided")) {
    const witch = firstPlayerIdByRole(players, "witch");
    if (!witch) {
      return null;
    }

    return createDraftEvent({
      id: input.draftId,
      gameId: input.game.id,
      type: "witch_antidote_decided",
      phase: "night",
      actorPlayerId: witch,
      visibility: { kind: "player_private", playerIds: [witch] },
      payload: { used: false, targetPlayerId: null },
      display: { title: "女巫解药", text: "女巫默认不使用解药。" },
      createdAt: input.createdAt,
    });
  }

  if (!hasEvent(events, "witch_poison_decided")) {
    const witch = firstPlayerIdByRole(players, "witch");
    if (!witch) {
      return null;
    }

    return createDraftEvent({
      id: input.draftId,
      gameId: input.game.id,
      type: "witch_poison_decided",
      phase: "night",
      actorPlayerId: witch,
      visibility: { kind: "player_private", playerIds: [witch] },
      payload: { used: false, targetPlayerId: null },
      display: { title: "女巫毒药", text: "女巫默认不使用毒药。" },
      createdAt: input.createdAt,
    });
  }

  if (!hasEvent(events, "night_resolved")) {
    const antidote = findEvent(events, "witch_antidote_decided");
    const poison = findEvent(events, "witch_poison_decided");
    const witch = firstPlayerIdByRole(players, "witch");
    if (
      !witch ||
      !isLegalTarget(
        wolfKill.payload.targetPlayerId,
        getLegalNightTargets("wolf_kill", players, alivePlayerIds),
      )
    ) {
      return null;
    }

    const antidoteTargetId =
      antidote?.payload.used === true ? antidote.payload.targetPlayerId : null;
    const poisonTargetId =
      poison?.payload.used === true ? poison.payload.targetPlayerId : null;
    if (
      (antidote?.payload.used === true && antidoteTargetId === null) ||
      (poison?.payload.used === true && poisonTargetId === null)
    ) {
      return null;
    }

    if (
      poisonTargetId !== null &&
      !isLegalTarget(
        poisonTargetId,
        getLegalNightTargets("witch_poison", players, alivePlayerIds, witch),
      )
    ) {
      return null;
    }

    const witchDecision = validateWitchDecision(
      {
        nightNumber: 1,
        witchPlayerId: witch,
        killedPlayerId: wolfKill.payload.targetPlayerId,
        antidoteTargetId,
        poisonTargetId,
      },
      input.game.ruleset,
    );
    if (!witchDecision.ok) {
      return null;
    }

    const deadPlayerIds = resolveNightDeaths({
      wolfKillTargetId: wolfKill.payload.targetPlayerId,
      antidoteTargetId,
      poisonTargetId,
    });

    return createDraftEvent({
      id: input.draftId,
      gameId: input.game.id,
      type: "night_resolved",
      phase: "night",
      targetPlayerIds: deadPlayerIds,
      visibility: { kind: "host_only" },
      payload: { deadPlayerIds },
      display: { title: "夜间结算", text: "系统结算夜间死亡。" },
      createdAt: input.createdAt,
    });
  }

  if (!hasEvent(events, "death_announced")) {
    const nightResolved = findEvent(events, "night_resolved");
    if (!nightResolved) {
      return null;
    }

    return createDraftEvent({
      id: input.draftId,
      gameId: input.game.id,
      type: "death_announced",
      phase: "day",
      targetPlayerIds: nightResolved.payload.deadPlayerIds,
      visibility: { kind: "public" },
      payload: { deadPlayerIds: nightResolved.payload.deadPlayerIds },
      display: { title: "昨夜死讯", text: "公布昨夜死亡玩家。" },
      createdAt: input.createdAt,
    });
  }

  return null;
}

function firstPlayerIdByRole(
  players: readonly PlayerSnapshot[],
  role: GameRole,
): PlayerId | undefined {
  return players.find((player) => player.gameRole === role)?.playerId;
}

function findEvent<Type extends GameEvent["type"]>(
  events: readonly GameEvent[],
  type: Type,
): EventOf<Type> | undefined {
  return events.find((event): event is EventOf<Type> => event.type === type);
}

function hasEvent(events: readonly GameEvent[], type: GameEvent["type"]) {
  return events.some((event) => event.type === type);
}

function hasNightOneStarted(events: readonly GameEvent[]): boolean {
  return events.some(
    (event) =>
      event.type === "phase_started" &&
      event.payload.phase === "night" &&
      event.payload.dayNumber === 1,
  );
}

function derivePlayersFromRoleAssignments(
  game: Game,
  events: readonly GameEvent[],
): readonly PlayerSnapshot[] {
  const assignments = new Map(
    events
      .filter((event): event is EventOf<"role_assigned"> =>
        event.type === "role_assigned",
      )
      .map((event) => [event.payload.playerId, event.payload] as const),
  );

  return game.players.map((player) => {
    const assignment = assignments.get(player.playerId);
    if (!assignment) {
      return player;
    }

    return {
      ...player,
      gameRole: assignment.role,
      faction: assignment.faction,
    };
  });
}

function isLegalTarget(
  targetPlayerId: PlayerId,
  legalTargetIds: readonly PlayerId[],
): boolean {
  return legalTargetIds.includes(targetPlayerId);
}
