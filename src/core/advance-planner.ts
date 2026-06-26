import { createDraftEvent, type DraftEvent } from "./drafts";
import { getActiveEvents } from "./event-log";
import type { GameEndReason, VoteType } from "./events";
import type { Game } from "./game";
import type { PlayerSnapshot } from "./player";
import {
  checkWinCondition,
  createEndgameReveal,
  getEligibleVoters,
  getLegalNightTargets,
  resolveVote,
  resolveNightDeaths,
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

  const afterExile = planAfterExileDraft(input, events, effectivePlayers, state);
  if (afterExile) {
    return afterExile;
  }

  const latestEvent = events.at(-1);
  if (!latestEvent || state.currentPhase === "setup") {
    return draftPhaseStarted(input, "night", 1, "第 1 夜开始", "夜晚开始。");
  }

  if (latestEvent.type === "death_announced") {
    if (state.pendingLastWords.length > 0) {
      return draftPhaseStarted(
        input,
        "last_words",
        state.dayNumber,
        "遗言阶段",
        "死亡玩家发表遗言。",
      );
    }

    return draftPhaseStarted(
      input,
      "speech",
      state.dayNumber,
      "发言阶段",
      "存活玩家依次发言。",
    );
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
    display: { title: "身份牌", text: `你的身份是 ${player.gameRole}。` },
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
  const wolfKill = findEvent(phaseEvents, "wolf_kill_selected");
  if (!wolfKill) {
    const wolf = firstAlivePlayerIdByRole(
      players,
      state.alivePlayerIds,
      "werewolf",
    );
    const target = getLegalNightTargets(
      "wolf_kill",
      players,
      state.alivePlayerIds,
    )[0];

    if (!wolf) {
      return draftGameEndIfNeeded(
        input,
        players,
        state.deadPlayerIds,
        state.dayNumber,
      );
    }

    if (!target) {
      return null;
    }

    return createDraftEvent({
      id: input.draftId,
      gameId: input.game.id,
      type: "wolf_kill_selected",
      phase: "night",
      actorPlayerId: wolf,
      targetPlayerIds: [target],
      visibility: { kind: "faction_private", faction: "wolves" },
      payload: { targetPlayerId: target },
      display: { title: "狼人刀人", text: "狼人选择夜间击杀目标。" },
      createdAt: input.createdAt,
    });
  }

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
          display: { title: "预言家查验", text: "预言家选择查验目标。" },
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
      display: { title: "查验结果", text: "预言家收到查验结果。" },
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
      targetPlayerIds: [wolfKill.payload.targetPlayerId],
      visibility: { kind: "player_private", playerIds: [witch] },
      payload: { killedPlayerId: wolfKill.payload.targetPlayerId },
      display: { title: "女巫信息", text: "女巫得知夜间死亡信息。" },
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
      display: { title: "女巫解药", text: "女巫默认不使用解药。" },
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
      display: { title: "女巫毒药", text: "女巫默认不使用毒药。" },
      createdAt: input.createdAt,
    });
  }

  if (!hasEvent(phaseEvents, "night_resolved")) {
    const antidote = findEvent(phaseEvents, "witch_antidote_decided");
    const poison = findEvent(phaseEvents, "witch_poison_decided");
    if (
      !isLegalTarget(
        wolfKill.payload.targetPlayerId,
        getLegalNightTargets("wolf_kill", players, state.alivePlayerIds),
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
          killedPlayerId: wolfKill.payload.targetPlayerId,
          antidoteTargetId,
          poisonTargetId,
        },
        input.game.ruleset,
      );
      if (!witchDecision.ok) {
        return null;
      }
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
    display: {
      title: "昨夜死讯",
      text:
        nightResolved.payload.deadPlayerIds.length > 0
          ? `昨夜死亡：${playerListLabel(players, nightResolved.payload.deadPlayerIds)}。`
          : "昨夜平安夜，没有玩家死亡。",
    },
    createdAt: input.createdAt,
  });
}

function planLastWordsDraft(
  input: PlanNextDraftInput,
  events: readonly GameEvent[],
  players: readonly PlayerSnapshot[],
  state: DerivedGameState,
): DraftEvent | null {
  const pending = state.pendingLastWords[0];
  if (pending) {
    const speaker = playerLabel(players, pending.playerId);
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
      display: {
        title: `${speaker}遗言`,
        text: `${speaker}发表遗言：我的遗言先到这里。`,
      },
      createdAt: input.createdAt,
    });
  }

  if (lastWordsStartedAfterExile(events, state.dayNumber)) {
    return draftPhaseStarted(
      input,
      "night",
      state.dayNumber + 1,
      `第 ${state.dayNumber + 1} 夜开始`,
      "进入下一夜。",
    );
  }

  return draftPhaseStarted(
    input,
    "speech",
    state.dayNumber,
    "发言阶段",
    "存活玩家依次发言。",
  );
}

function planDaySpeechDraft(
  input: PlanNextDraftInput,
  players: readonly PlayerSnapshot[],
  state: DerivedGameState,
): DraftEvent | null {
  const spoken = new Set(state.daySpeech.spokenPlayerIds);
  const speaker = state.alivePlayerIds.find((playerId) => !spoken.has(playerId));
  if (speaker) {
    const speakerLabel = playerLabel(players, speaker);
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
      display: {
        title: `${speakerLabel}发言`,
        text: `${speakerLabel}发言：我先给出自己的判断。`,
      },
      createdAt: input.createdAt,
    });
  }

  return draftPhaseStarted(
    input,
    "vote",
    state.dayNumber,
    "放逐投票",
    "进入本日放逐投票。",
  );
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
    const voterLabel = playerLabel(players, voter);
    const targetText = target ? playerLabel(players, target) : "弃票";
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
      display: {
        title: `${voterLabel}投票`,
        text: `${voterLabel}投给 ${targetText}。`,
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
    display: {
      title: "投票结算",
      text: exileResolutionText(
        players,
        resolution.exiledPlayerId,
        resolution.tiedPlayerIds,
      ),
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
    const speakerLabel = playerLabel(players, speaker);
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
      display: {
        title: `${speakerLabel}PK 发言`,
        text: `${speakerLabel}PK 发言：我补充自己的 PK 发言。`,
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
    const voterLabel = playerLabel(players, voter);
    const targetText = target ? playerLabel(players, target) : "弃票";
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
      display: {
        title: `${voterLabel}PK 投票`,
        text: `${voterLabel}投给 ${targetText}。`,
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
    display: {
      title: "PK 结算",
      text: exileResolutionText(
        players,
        resolution.exiledPlayerId,
        resolution.tiedPlayerIds,
      ),
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
    return draftPhaseStarted(
      input,
      "pk",
      latestEvent.payload.dayNumber,
      "PK 阶段",
      "平票玩家进入 PK。",
    );
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
    return draftPhaseStarted(
      input,
      "last_words",
      latestEvent.payload.dayNumber,
      "遗言阶段",
      "出局玩家发表遗言。",
    );
  }

  return draftPhaseStarted(
    input,
    "night",
    latestEvent.payload.dayNumber + 1,
    `第 ${latestEvent.payload.dayNumber + 1} 夜开始`,
    "进入下一夜。",
  );
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
    display: {
      title: `游戏结束：${winnerLabel(result.winner)}胜利`,
      text: `${winnerLabel(result.winner)}胜利，原因：${winReasonLabel(result.reason)}。`,
    },
    createdAt: input.createdAt,
  });
}

function draftPhaseStarted(
  input: PlanNextDraftInput,
  phase: Phase,
  dayNumber: number,
  title: string,
  text: string,
): DraftEvent {
  return createDraftEvent({
    id: input.draftId,
    gameId: input.game.id,
    type: "phase_started",
    phase,
    visibility: { kind: "public" },
    payload: { phase, dayNumber },
    display: { title, text },
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

function lastWordsStartedAfterExile(
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

  return events[lastWordsStartIndex - 1]?.type === "exile_resolved";
}

function playerLabel(
  players: readonly PlayerSnapshot[],
  playerId: PlayerId,
): string {
  const player = players.find((candidate) => candidate.playerId === playerId);
  if (!player) {
    return `未知玩家 ${playerId}`;
  }

  return `${player.seatNo} 号 ${player.name}`;
}

function playerListLabel(
  players: readonly PlayerSnapshot[],
  playerIds: readonly PlayerId[],
): string {
  return playerIds.map((playerId) => playerLabel(players, playerId)).join("、");
}

function exileResolutionText(
  players: readonly PlayerSnapshot[],
  exiledPlayerId: PlayerId | null,
  tiedPlayerIds: readonly PlayerId[],
): string {
  if (exiledPlayerId) {
    return `放逐出局：${playerLabel(players, exiledPlayerId)}。`;
  }

  if (tiedPlayerIds.length > 0) {
    return `平票：${playerListLabel(players, tiedPlayerIds)}。`;
  }

  return "本轮无人被放逐。";
}

function winnerLabel(winner: "wolves" | "good"): string {
  return winner === "wolves" ? "狼人阵营" : "好人阵营";
}

function winReasonLabel(reason: GameEndReason): string {
  switch (reason) {
    case "all_wolves_dead":
      return "所有狼人出局";
    case "all_gods_dead":
      return "所有神职出局";
    case "all_villagers_dead":
      return "所有平民出局";
    case "all_good_dead":
      return "所有好人出局";
  }
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
