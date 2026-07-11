import { createDraftEvent, type DraftEvent } from "./drafts";
import { getActiveEvents } from "./event-log";
import type { VoteType } from "./events";
import type { Game } from "./game";
import type { PlayerSnapshot } from "./player";
import {
  checkWinCondition,
  createEndgameReveal,
  getEligibleVoters,
  getLegalNightTargets,
  resolveVote,
  resolveNightDeathDetails,
  resolveWolfVote,
  validateWitchDecision,
} from "./rules";
import { deriveGameState, type DerivedGameState } from "./state";
import type { DraftId, GameRole, Phase, PlayerId } from "./types";
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

  const pendingEnd = planPendingEndDraft(input, effectivePlayers, state);
  if (pendingEnd) {
    return pendingEnd;
  }

  const afterHunterShot = planAfterHunterShotDraft(input, events, state);
  if (afterHunterShot) {
    return afterHunterShot;
  }

  const pendingHunterShot = planPendingHunterShotDraft(
    input,
    events,
    effectivePlayers,
    state,
  );
  if (pendingHunterShot) {
    return pendingHunterShot;
  }

  const afterExile = planAfterExileDraft(input, events, effectivePlayers, state);
  if (afterExile) {
    return afterExile;
  }

  const latestEvent = events.at(-1);
  if (!latestEvent || state.currentPhase === "setup") {
    return draftPhaseStarted(input, "night", 1);
  }

  if (latestEvent.type === "death_announced") {
    if (state.pendingLastWords.length > 0) {
      return draftPhaseStarted(input, "last_words", state.dayNumber);
    }

    return draftPhaseStarted(input, "speech", state.dayNumber);
  }

  if (state.currentPhase === "night") {
    return planNightDraft(input, events, effectivePlayers, state);
  }

  if (state.currentPhase === "last_words") {
    return planLastWordsDraft(input, events, effectivePlayers, state);
  }

  if (state.currentPhase === "speech") {
    return planDaySpeechDraft(input, effectivePlayers, state);
  }

  if (state.currentPhase === "vote") {
    return planDailyVoteDraft(input, effectivePlayers, state);
  }

  if (state.currentPhase === "pk") {
    return planPkDraft(input, effectivePlayers, state);
  }

  return null;
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
    createdAt: input.createdAt,
  });
}

function planNightDraft(
  input: PlanNextDraftInput,
  events: readonly GameEvent[],
  players: readonly PlayerSnapshot[],
  state: DerivedGameState,
): DraftEvent | null {
  const phaseEvents = getCurrentPhaseEvents(events, "night", state.dayNumber);
  const guard = firstAlivePlayerIdByRole(players, state.alivePlayerIds, "guard");
  const guardProtect = findEvent(phaseEvents, "guard_protect_selected");
  if (!guardProtect && guard) {
    const previousGuardTarget = previousGuardTargetId(events, state.dayNumber);
    const targets = getLegalNightTargets(
      "guard_protect",
      players,
      state.alivePlayerIds,
      guard,
    ).filter((target) =>
      input.game.ruleset.guardForbidConsecutiveSameTarget
        ? target !== previousGuardTarget
        : true,
    );
    const target = targets[0];

    if (target) {
      return createDraftEvent({
        id: input.draftId,
        gameId: input.game.id,
        type: "guard_protect_selected",
        phase: "night",
        actorPlayerId: guard,
        targetPlayerIds: [target],
        visibility: { kind: "player_private", playerIds: [guard] },
        payload: { targetPlayerId: target },
        createdAt: input.createdAt,
      });
    }
  }

  const wolves = players
    .filter((player) => player.gameRole === "werewolf")
    .filter((player) => state.alivePlayerIds.includes(player.playerId));
  if (wolves.length === 0) {
    return draftGameEndIfNeeded(input, players, state.deadPlayerIds, state.dayNumber);
  }

  const leaderSelection = events.find(
    (event): event is EventOf<"wolf_leader_selected"> =>
      event.type === "wolf_leader_selected",
  );
  if (state.dayNumber === 1 && !leaderSelection) {
    const leaderPlayerId = wolves[0]!.playerId;
    return createDraftEvent({
      id: input.draftId,
      gameId: input.game.id,
      type: "wolf_leader_selected",
      phase: "night",
      actorPlayerId: leaderPlayerId,
      targetPlayerIds: [leaderPlayerId],
      visibility: { kind: "host_only" },
      payload: { leaderPlayerId },
      createdAt: input.createdAt,
    });
  }

  const leaderPlayerId = leaderSelection?.payload.leaderPlayerId;
  if (state.dayNumber === 1 && !hasEvent(phaseEvents, "wolf_strategy_given")) {
    if (!leaderPlayerId) return null;
    return createDraftEvent({
      id: input.draftId,
      gameId: input.game.id,
      type: "wolf_strategy_given",
      phase: "night",
      actorPlayerId: leaderPlayerId,
      visibility: { kind: "faction_private", faction: "wolves" },
      payload: {
        playerId: leaderPlayerId,
        text: "今晚先建立狼队的整体战术。",
        dayNumber: 1,
      },
      createdAt: input.createdAt,
    });
  }

  const opinionWolves = state.dayNumber === 1
    ? wolves.filter((wolf) => wolf.playerId !== leaderPlayerId)
    : wolves;
  const opinionActors = new Set(
    phaseEvents
      .filter((event): event is EventOf<"wolf_opinion_given"> =>
        event.type === "wolf_opinion_given",
      )
      .map((event) => event.actorPlayerId),
  );
  const opinionWolf = opinionWolves.find(
    (wolf) => !opinionActors.has(wolf.playerId),
  );
  if (opinionWolf) {
    return createDraftEvent({
      id: input.draftId,
      gameId: input.game.id,
      type: "wolf_opinion_given",
      phase: "night",
      actorPlayerId: opinionWolf.playerId,
      visibility: { kind: "faction_private", faction: "wolves" },
      payload: {
        playerId: opinionWolf.playerId,
        text: "我结合当前局势提出自己的刀人意见。",
        dayNumber: state.dayNumber,
      },
      createdAt: input.createdAt,
    });
  }

  const wolfVotes = phaseEvents.filter(
    (event): event is EventOf<"wolf_vote_cast"> => event.type === "wolf_vote_cast",
  );
  const votedWolfIds = new Set(wolfVotes.map((event) => event.payload.voterPlayerId));
  const votingWolf = wolves.find((wolf) => !votedWolfIds.has(wolf.playerId));
  if (votingWolf) {
    const targetPlayerId = getLegalNightTargets(
      "wolf_kill",
      players,
      state.alivePlayerIds,
      votingWolf.playerId,
    )[0];
    if (!targetPlayerId) return null;
    return createDraftEvent({
      id: input.draftId,
      gameId: input.game.id,
      type: "wolf_vote_cast",
      phase: "night",
      actorPlayerId: votingWolf.playerId,
      targetPlayerIds: [targetPlayerId],
      visibility: { kind: "host_only" },
      payload: {
        voterPlayerId: votingWolf.playerId,
        targetPlayerId,
        dayNumber: state.dayNumber,
      },
      createdAt: input.createdAt,
    });
  }

  const wolfResolution = findEvent(phaseEvents, "wolf_vote_resolved");
  if (!wolfResolution) {
    const vote = resolveWolfVote(wolfVotes.map((event) => event.payload));
    return createDraftEvent({
      id: input.draftId,
      gameId: input.game.id,
      type: "wolf_vote_resolved",
      phase: "night",
      targetPlayerIds: vote.targetPlayerId ? [vote.targetPlayerId] : [],
      visibility: { kind: "faction_private", faction: "wolves" },
      payload: {
        ...vote,
        resolution: vote.targetPlayerId ? "majority" : null,
        dayNumber: state.dayNumber,
      },
      createdAt: input.createdAt,
    });
  }
  if (!wolfResolution.payload.targetPlayerId) return null;

  const seer = firstAlivePlayerIdByRole(players, state.alivePlayerIds, "seer");
  const seerCheck = findEvent(phaseEvents, "seer_check_selected");
  const seerCanAct =
    seer !== undefined &&
    getLegalNightTargets("seer_check", players, state.alivePlayerIds, seer)
      .length > 0;

  if (!seerCheck) {
    if (seerCanAct && seer) {
      const target = chooseSeerTarget(
        players,
        state.alivePlayerIds,
        events,
        seer,
      );

      if (target) {
        return createDraftEvent({
          id: input.draftId,
          gameId: input.game.id,
          type: "seer_check_selected",
          phase: "night",
          actorPlayerId: seer,
          targetPlayerIds: [target],
          visibility: { kind: "player_private", playerIds: [seer] },
          payload: { targetPlayerId: target },
          createdAt: input.createdAt,
        });
      }
    }
  }

  if (seerCheck && !hasEvent(phaseEvents, "seer_check_result")) {
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
      createdAt: input.createdAt,
    });
  }

  const witch = firstAlivePlayerIdByRole(players, state.alivePlayerIds, "witch");
  if (witch && !hasEvent(phaseEvents, "witch_death_info_shown")) {

    return createDraftEvent({
      id: input.draftId,
      gameId: input.game.id,
      type: "witch_death_info_shown",
      phase: "night",
      actorPlayerId: witch,
      targetPlayerIds: [wolfResolution.payload.targetPlayerId],
      visibility: { kind: "player_private", playerIds: [witch] },
      payload: { killedPlayerId: wolfResolution.payload.targetPlayerId },
      createdAt: input.createdAt,
    });
  }

  if (
    witch &&
    state.witch.antidoteAvailable &&
    !hasEvent(phaseEvents, "witch_antidote_decided")
  ) {

    return createDraftEvent({
      id: input.draftId,
      gameId: input.game.id,
      type: "witch_antidote_decided",
      phase: "night",
      actorPlayerId: witch,
      visibility: { kind: "player_private", playerIds: [witch] },
      payload: { used: false, targetPlayerId: null },
      createdAt: input.createdAt,
    });
  }

  if (
    witch &&
    state.witch.poisonAvailable &&
    !hasEvent(phaseEvents, "witch_poison_decided")
  ) {

    return createDraftEvent({
      id: input.draftId,
      gameId: input.game.id,
      type: "witch_poison_decided",
      phase: "night",
      actorPlayerId: witch,
      visibility: { kind: "player_private", playerIds: [witch] },
      payload: { used: false, targetPlayerId: null },
      createdAt: input.createdAt,
    });
  }

  if (!hasEvent(phaseEvents, "night_resolved")) {
    const guardProtect = findEvent(phaseEvents, "guard_protect_selected");
    const antidote = findEvent(phaseEvents, "witch_antidote_decided");
    const poison = findEvent(phaseEvents, "witch_poison_decided");
    if (
      !isLegalTarget(
        wolfResolution.payload.targetPlayerId,
        getLegalNightTargets(
          "wolf_kill",
          players,
          state.alivePlayerIds,
          wolves[0]?.playerId,
        ),
      )
    ) {
      return null;
    }

    if (
      guardProtect &&
      !isLegalTarget(
        guardProtect.payload.targetPlayerId,
        getLegalNightTargets(
          "guard_protect",
          players,
          state.alivePlayerIds,
          guardProtect.actorPlayerId,
        ),
      )
    ) {
      return null;
    }

    if (
      guardProtect &&
      input.game.ruleset.guardForbidConsecutiveSameTarget &&
      guardProtect.payload.targetPlayerId ===
        previousGuardTargetId(events, state.dayNumber)
    ) {
      return null;
    }

    const antidoteTargetId =
      antidote?.payload.used === true ? antidote.payload.targetPlayerId : null;
    const poisonTargetId =
      poison?.payload.used === true ? poison.payload.targetPlayerId : null;
    const guardTargetId = guardProtect?.payload.targetPlayerId ?? null;
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
        getLegalNightTargets("witch_poison", players, state.alivePlayerIds, witch),
      )
    ) {
      return null;
    }

    if (witch) {
      const witchDecision = validateWitchDecision(
        {
          nightNumber: state.dayNumber,
          witchPlayerId: witch,
          killedPlayerId: wolfResolution.payload.targetPlayerId,
          antidoteTargetId,
          poisonTargetId,
        },
        input.game.ruleset,
      );
      if (!witchDecision.ok) {
        return null;
      }
    }

    const deaths = resolveNightDeathDetails({
      wolfKillTargetId: wolfResolution.payload.targetPlayerId,
      guardTargetId,
      antidoteTargetId,
      poisonTargetId,
    });
    const deadPlayerIds = deaths.map((death) => death.playerId);

    return createDraftEvent({
      id: input.draftId,
      gameId: input.game.id,
      type: "night_resolved",
      phase: "night",
      targetPlayerIds: deadPlayerIds,
      visibility: { kind: "host_only" },
      payload: { deadPlayerIds, deaths },
      createdAt: input.createdAt,
    });
  }

  return planAfterNightResolvedDraft(input, phaseEvents, players, state);
}

function planAfterNightResolvedDraft(
  input: PlanNextDraftInput,
  phaseEvents: readonly GameEvent[],
  players: readonly PlayerSnapshot[],
  state: DerivedGameState,
): DraftEvent | null {
  const nightResolved = findEvent(phaseEvents, "night_resolved");
  if (!nightResolved) {
    return null;
  }

  const end = draftGameEndIfNeeded(
    input,
    players,
    state.deadPlayerIds,
    state.dayNumber,
  );
  if (end) {
    return end;
  }

  if (hasEvent(phaseEvents, "death_announced")) {
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
    createdAt: input.createdAt,
  });
}

function planPendingHunterShotDraft(
  input: PlanNextDraftInput,
  events: readonly GameEvent[],
  players: readonly PlayerSnapshot[],
  state: DerivedGameState,
): DraftEvent | null {
  const hunter = players.find(
    (player) =>
      player.gameRole === "hunter" &&
      state.deadPlayerIds.includes(player.playerId),
  );
  if (!hunter) {
    return null;
  }

  if (
    events.some(
      (event) =>
        event.type === "hunter_shot_decided" &&
        event.actorPlayerId === hunter.playerId,
    )
  ) {
    return null;
  }

  const deathReason = hunterDeathReason(events, hunter.playerId);
  if (deathReason !== "wolf_kill" && deathReason !== "exile") {
    return null;
  }

  const target = getLegalNightTargets(
    "hunter_shot",
    players,
    state.alivePlayerIds,
    hunter.playerId,
  )[0];
  if (!target) {
    return null;
  }

  return createDraftEvent({
    id: input.draftId,
    gameId: input.game.id,
    type: "hunter_shot_decided",
    phase: "last_words",
    actorPlayerId: hunter.playerId,
    targetPlayerIds: [target],
    visibility: { kind: "public" },
    payload: { targetPlayerId: target },
    createdAt: input.createdAt,
  });
}

function planAfterHunterShotDraft(
  input: PlanNextDraftInput,
  events: readonly GameEvent[],
  state: DerivedGameState,
): DraftEvent | null {
  if (events.at(-1)?.type !== "hunter_shot_decided") {
    return null;
  }

  if (state.pendingLastWords.length > 0) {
    return draftPhaseStarted(input, "last_words", state.dayNumber);
  }

  return draftPhaseStarted(input, "night", state.dayNumber + 1);
}

function planLastWordsDraft(
  input: PlanNextDraftInput,
  events: readonly GameEvent[],
  players: readonly PlayerSnapshot[],
  state: DerivedGameState,
): DraftEvent | null {
  const pending = state.pendingLastWords[0];
  if (pending) {
    return createDraftEvent({
      id: input.draftId,
      gameId: input.game.id,
      type: "last_words_given",
      phase: "last_words",
      actorPlayerId: pending.playerId,
      targetPlayerIds: [pending.playerId],
      visibility: { kind: "public" },
      payload: {
        playerId: pending.playerId,
        text: "我的遗言先到这里。",
        dayNumber: state.dayNumber,
        reason: pending.reason,
      },
      createdAt: input.createdAt,
    });
  }

  if (lastWordsShouldAdvanceToNextNight(events, state.dayNumber)) {
    return draftPhaseStarted(input, "night", state.dayNumber + 1);
  }

  return draftPhaseStarted(input, "speech", state.dayNumber);
}

function planDaySpeechDraft(
  input: PlanNextDraftInput,
  players: readonly PlayerSnapshot[],
  state: DerivedGameState,
): DraftEvent | null {
  const spoken = new Set(state.daySpeech.spokenPlayerIds);
  const speaker = state.alivePlayerIds.find((playerId) => !spoken.has(playerId));
  if (speaker) {
    return createDraftEvent({
      id: input.draftId,
      gameId: input.game.id,
      type: "day_speech_given",
      phase: "speech",
      actorPlayerId: speaker,
      visibility: { kind: "public" },
      payload: {
        playerId: speaker,
        text: "我先给出自己的判断。",
        dayNumber: state.dayNumber,
        round: 1,
      },
      createdAt: input.createdAt,
    });
  }

  return draftPhaseStarted(input, "vote", state.dayNumber);
}

function planDailyVoteDraft(
  input: PlanNextDraftInput,
  players: readonly PlayerSnapshot[],
  state: DerivedGameState,
): DraftEvent | null {
  const existingVotes = getVoteGroup(state, "exile", 1)?.votes ?? [];
  const voted = new Set(existingVotes.map((vote) => vote.voterPlayerId));
  const voter = state.alivePlayerIds.find((playerId) => !voted.has(playerId));
  if (voter) {
    const target = chooseVoteTarget(voter, state.alivePlayerIds);
    return createDraftEvent({
      id: input.draftId,
      gameId: input.game.id,
      type: "vote_cast",
      phase: "vote",
      actorPlayerId: voter,
      targetPlayerIds: target ? [target] : [],
      visibility: { kind: "public" },
      payload: {
        voterPlayerId: voter,
        targetPlayerId: target,
        dayNumber: state.dayNumber,
        round: 1,
        voteType: "exile",
      },
      createdAt: input.createdAt,
    });
  }

  const resolution = resolveVote({
    votes: existingVotes,
    allowAbstainVote: input.game.ruleset.allowAbstainVote,
  });
  return createDraftEvent({
    id: input.draftId,
    gameId: input.game.id,
    type: "exile_resolved",
    phase: "vote",
    targetPlayerIds: resolution.exiledPlayerId ? [resolution.exiledPlayerId] : [],
    visibility: { kind: "public" },
    payload: {
      ...resolution,
      voteType: "exile",
      dayNumber: state.dayNumber,
      round: 1,
      revealedRoles: [],
    },
    createdAt: input.createdAt,
  });
}

function planPkDraft(
  input: PlanNextDraftInput,
  players: readonly PlayerSnapshot[],
  state: DerivedGameState,
): DraftEvent | null {
  if (state.pk.status !== "pending") {
    return null;
  }

  const round = state.pk.round + 1;
  const spoken = new Set(state.daySpeech.pkSpokenPlayerIds);
  const speaker = state.pk.tiedPlayerIds.find((playerId) => !spoken.has(playerId));
  if (speaker) {
    return createDraftEvent({
      id: input.draftId,
      gameId: input.game.id,
      type: "pk_speech_given",
      phase: "pk",
      actorPlayerId: speaker,
      visibility: { kind: "public" },
      payload: {
        playerId: speaker,
        text: "我补充自己的 PK 发言。",
        dayNumber: state.dayNumber,
        round,
      },
      createdAt: input.createdAt,
    });
  }

  const eligibleVoters = getEligibleVoters({
    alivePlayerIds: state.alivePlayerIds,
    voteType: "pk",
    pkPlayerIds: state.pk.tiedPlayerIds,
    pkVoters: input.game.ruleset.pkVoters,
  });
  const existingVotes = getVoteGroup(state, "pk", round)?.votes ?? [];
  const voted = new Set(existingVotes.map((vote) => vote.voterPlayerId));
  const voter = eligibleVoters.find((playerId) => !voted.has(playerId));
  if (voter) {
    const target = choosePkVoteTarget(voter, state.pk.tiedPlayerIds);
    return createDraftEvent({
      id: input.draftId,
      gameId: input.game.id,
      type: "vote_cast",
      phase: "vote",
      actorPlayerId: voter,
      targetPlayerIds: target ? [target] : [],
      visibility: { kind: "public" },
      payload: {
        voterPlayerId: voter,
        targetPlayerId: target,
        dayNumber: state.dayNumber,
        round,
        voteType: "pk",
      },
      createdAt: input.createdAt,
    });
  }

  const resolution = resolveVote({
    votes: existingVotes,
    allowAbstainVote: input.game.ruleset.allowAbstainVote,
  });
  return createDraftEvent({
    id: input.draftId,
    gameId: input.game.id,
    type: "exile_resolved",
    phase: "vote",
    targetPlayerIds: resolution.exiledPlayerId ? [resolution.exiledPlayerId] : [],
    visibility: { kind: "public" },
    payload: {
      exiledPlayerId: resolution.exiledPlayerId,
      tiedPlayerIds: resolution.exiledPlayerId ? [] : resolution.tiedPlayerIds,
      voteTable: resolution.voteTable,
      voteType: "pk",
      dayNumber: state.dayNumber,
      round,
      revealedRoles: [],
    },
    createdAt: input.createdAt,
  });
}

function planPendingEndDraft(
  input: PlanNextDraftInput,
  players: readonly PlayerSnapshot[],
  state: DerivedGameState,
): DraftEvent | null {
  if (state.currentPhase !== "night" && state.currentPhase !== "vote") {
    return null;
  }

  return draftGameEndIfNeeded(
    input,
    players,
    state.deadPlayerIds,
    state.dayNumber,
  );
}

function planAfterExileDraft(
  input: PlanNextDraftInput,
  events: readonly GameEvent[],
  players: readonly PlayerSnapshot[],
  state: DerivedGameState,
): DraftEvent | null {
  const latestEvent = events.at(-1);
  if (latestEvent?.type !== "exile_resolved") {
    return null;
  }

  if (
    latestEvent.payload.voteType === "exile" &&
    latestEvent.payload.tiedPlayerIds.length > 0
  ) {
    return draftPhaseStarted(input, "pk", latestEvent.payload.dayNumber);
  }

  const end = draftGameEndIfNeeded(
    input,
    players,
    state.deadPlayerIds,
    latestEvent.payload.dayNumber,
  );
  if (end) {
    return end;
  }

  if (state.pendingLastWords.length > 0) {
    return draftPhaseStarted(input, "last_words", latestEvent.payload.dayNumber);
  }

  return draftPhaseStarted(
    input,
    "night",
    latestEvent.payload.dayNumber + 1,
  );
}

function hunterDeathReason(
  events: readonly GameEvent[],
  hunterPlayerId: PlayerId,
): "wolf_kill" | "witch_poison" | "exile" | "hunter_shot" | null {
  for (const event of [...events].reverse()) {
    if (
      event.type === "hunter_shot_decided" &&
      event.payload.targetPlayerId === hunterPlayerId
    ) {
      return "hunter_shot";
    }

    if (
      event.type === "exile_resolved" &&
      event.payload.exiledPlayerId === hunterPlayerId
    ) {
      return "exile";
    }

    if (event.type === "night_resolved") {
      const matchingDeath = event.payload.deaths?.find(
        (death) => death.playerId === hunterPlayerId,
      );
      if (matchingDeath) {
        return matchingDeath.reason;
      }

      if (event.payload.deadPlayerIds.includes(hunterPlayerId)) {
        return "wolf_kill";
      }
    }
  }

  return null;
}

function draftGameEndIfNeeded(
  input: PlanNextDraftInput,
  players: readonly PlayerSnapshot[],
  deadPlayerIds: readonly PlayerId[],
  dayNumber: number,
): DraftEvent | null {
  const result = checkWinCondition(players, deadPlayerIds, input.game.ruleset);
  if (!result.ended) {
    return null;
  }

  return createDraftEvent({
    id: input.draftId,
    gameId: input.game.id,
    type: "game_ended",
    phase: "ended",
    visibility: { kind: "public" },
    payload: {
      winner: result.winner,
      reason: result.reason,
      dayNumber,
      revealedRoles: createEndgameReveal(players),
    },
    createdAt: input.createdAt,
  });
}

function draftPhaseStarted(
  input: PlanNextDraftInput,
  phase: Phase,
  dayNumber: number,
): DraftEvent {
  return createDraftEvent({
    id: input.draftId,
    gameId: input.game.id,
    type: "phase_started",
    phase,
    visibility: { kind: "public" },
    payload: { phase, dayNumber },
    createdAt: input.createdAt,
  });
}

function firstPlayerIdByRole(
  players: readonly PlayerSnapshot[],
  role: GameRole,
): PlayerId | undefined {
  return players.find((player) => player.gameRole === role)?.playerId;
}

function firstAlivePlayerIdByRole(
  players: readonly PlayerSnapshot[],
  alivePlayerIds: readonly PlayerId[],
  role: GameRole,
): PlayerId | undefined {
  const alive = new Set(alivePlayerIds);
  return players.find(
    (player) => player.gameRole === role && alive.has(player.playerId),
  )?.playerId;
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

function getCurrentPhaseEvents(
  events: readonly GameEvent[],
  phase: Phase,
  dayNumber: number,
): readonly GameEvent[] {
  const startIndex = findLastIndex(
    events,
    (event) =>
      event.type === "phase_started" &&
      event.payload.phase === phase &&
      event.payload.dayNumber === dayNumber,
  );
  if (startIndex < 0) {
    return [];
  }

  const endIndex = events.findIndex(
    (event, index) => index > startIndex && event.type === "phase_started",
  );
  return events.slice(startIndex + 1, endIndex < 0 ? undefined : endIndex);
}

function chooseSeerTarget(
  players: readonly PlayerSnapshot[],
  alivePlayerIds: readonly PlayerId[],
  events: readonly GameEvent[],
  seer: PlayerId,
): PlayerId | null {
  const checked = new Set(
    events
      .filter((event): event is EventOf<"seer_check_selected"> =>
        event.type === "seer_check_selected",
      )
      .map((event) => event.payload.targetPlayerId),
  );
  const legalTargets = getLegalNightTargets(
    "seer_check",
    players,
    alivePlayerIds,
    seer,
  ).filter((playerId) => !checked.has(playerId));
  const wolfTarget = legalTargets.find(
    (playerId) =>
      players.find((player) => player.playerId === playerId)?.gameRole ===
      "werewolf",
  );

  return wolfTarget ?? legalTargets[0] ?? null;
}

function getVoteGroup(
  state: DerivedGameState,
  voteType: VoteType,
  round: number,
) {
  return state.votes.find(
    (group) =>
      group.dayNumber === state.dayNumber &&
      group.voteType === voteType &&
      group.round === round,
  );
}

function chooseVoteTarget(
  voter: PlayerId,
  alivePlayerIds: readonly PlayerId[],
): PlayerId | null {
  return alivePlayerIds.find((playerId) => playerId !== voter) ?? null;
}

function choosePkVoteTarget(
  voter: PlayerId,
  tiedPlayerIds: readonly PlayerId[],
): PlayerId | null {
  return (
    tiedPlayerIds.find((playerId) => playerId !== voter) ??
    tiedPlayerIds[0] ??
    null
  );
}

function previousGuardTargetId(
  events: readonly GameEvent[],
  currentDayNumber: number,
): PlayerId | null {
  const previousNightStartIndex = findLastIndex(
    events,
    (event) =>
      event.type === "phase_started" &&
      event.payload.phase === "night" &&
      event.payload.dayNumber < currentDayNumber,
  );
  if (previousNightStartIndex < 0) {
    return null;
  }

  const nextPhaseIndex = events.findIndex(
    (event, index) =>
      index > previousNightStartIndex && event.type === "phase_started",
  );
  const previousNightEvents = events.slice(
    previousNightStartIndex + 1,
    nextPhaseIndex < 0 ? undefined : nextPhaseIndex,
  );
  const guardProtect = previousNightEvents.find(
    (event): event is EventOf<"guard_protect_selected"> =>
      event.type === "guard_protect_selected",
  );

  return guardProtect?.payload.targetPlayerId ?? null;
}

function lastWordsShouldAdvanceToNextNight(
  events: readonly GameEvent[],
  dayNumber: number,
): boolean {
  const lastWordsStartIndex = findLastIndex(
    events,
    (event) =>
      event.type === "phase_started" &&
      event.payload.phase === "last_words" &&
      event.payload.dayNumber === dayNumber,
  );
  if (lastWordsStartIndex < 1) {
    return false;
  }

  const previousEvent = events[lastWordsStartIndex - 1];
  if (previousEvent?.type === "exile_resolved") {
    return true;
  }

  return (
    previousEvent?.type === "hunter_shot_decided" &&
    previousEvent.actorPlayerId !== undefined &&
    hunterDeathReason(events.slice(0, lastWordsStartIndex), previousEvent.actorPlayerId) ===
      "exile"
  );
}

function findLastIndex<T>(
  items: readonly T[],
  predicate: (item: T) => boolean,
): number {
  for (let index = items.length - 1; index >= 0; index -= 1) {
    if (predicate(items[index])) {
      return index;
    }
  }

  return -1;
}
