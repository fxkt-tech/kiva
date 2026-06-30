import { getActiveEvents } from "./event-log";
import type {
  GameEndReason,
  GameEvent,
  LastWordsReason,
  RevealedRole,
  VoteType,
} from "./events";
import type { PlayerSnapshot } from "./player";
import type { Phase, PlayerId } from "./types";

export type PendingLastWords = {
  readonly playerId: PlayerId;
  readonly reason: LastWordsReason;
};

export type DerivedVote = {
  readonly voterPlayerId: PlayerId;
  readonly targetPlayerId: PlayerId | null;
};

export type DerivedVoteGroup = {
  readonly dayNumber: number;
  readonly round: number;
  readonly voteType: VoteType;
  readonly votes: readonly DerivedVote[];
};

export type DerivedDaySpeech = {
  readonly dayNumber: number;
  readonly round: number;
  readonly spokenPlayerIds: readonly PlayerId[];
  readonly pkSpokenPlayerIds: readonly PlayerId[];
};

export type DerivedPkState =
  | { readonly status: "none" }
  | {
      readonly status: "pending";
      readonly dayNumber: number;
      readonly round: number;
      readonly tiedPlayerIds: readonly PlayerId[];
    }
  | {
      readonly status: "no_exile";
      readonly dayNumber: number;
      readonly round: number;
      readonly tiedPlayerIds: readonly PlayerId[];
      readonly reason: "no_exile" | "second_tie";
    };

export type DerivedEndedState = {
  readonly winner: "wolves" | "good";
  readonly reason: GameEndReason;
  readonly revealedRoles: readonly RevealedRole[];
};

export type DerivedGameState = {
  readonly currentPhase: Phase;
  readonly dayNumber: number;
  readonly alivePlayerIds: readonly PlayerId[];
  readonly deadPlayerIds: readonly PlayerId[];
  readonly pendingLastWords: readonly PendingLastWords[];
  readonly daySpeech: DerivedDaySpeech;
  readonly votes: readonly DerivedVoteGroup[];
  readonly pk: DerivedPkState;
  readonly ended: DerivedEndedState | null;
  readonly witch: {
    readonly antidoteAvailable: boolean;
    readonly poisonAvailable: boolean;
  };
};

export function deriveGameState(
  players: readonly PlayerSnapshot[],
  events: readonly GameEvent[],
): DerivedGameState {
  let currentPhase: Phase = "setup";
  let dayNumber = 0;
  const dead = new Set<PlayerId>();
  const allPlayerIds = players.map((player) => player.playerId);
  const knownPlayerIds = new Set(allPlayerIds);
  let antidoteAvailable = true;
  let poisonAvailable = true;
  const pendingLastWordsByPlayerId = new Map<PlayerId, PendingLastWords>();
  const daySpeechByKey = new Map<string, PlayerId[]>();
  const pkSpeechByKey = new Map<string, PlayerId[]>();
  const votesByKey = new Map<string, DerivedVoteGroup>();
  let latestDaySpeechDayNumber = 0;
  let latestDaySpeechRound = 1;
  let latestPkSpeechDayNumber = 0;
  let latestPkSpeechRound = 1;
  let pk: DerivedPkState = { status: "none" };
  let ended: DerivedEndedState | null = null;

  for (const event of getActiveEvents(events)) {
    if (event.type === "phase_started") {
      currentPhase = event.payload.phase;
      dayNumber = event.payload.dayNumber;
      if (pk.status !== "none" && pk.dayNumber < dayNumber) {
        pk = { status: "none" };
      }
    }

    if (event.type === "night_resolved") {
      for (const playerId of event.payload.deadPlayerIds) {
        assertKnownPlayerId(playerId, knownPlayerIds, "dead");
        dead.add(playerId);
      }
    }

    if (event.type === "death_announced") {
      for (const playerId of event.payload.deadPlayerIds) {
        assertKnownPlayerId(playerId, knownPlayerIds, "dead");
        if (dead.has(playerId)) {
          pendingLastWordsByPlayerId.set(playerId, {
            playerId,
            reason: "night_death",
          });
        }
      }
    }

    if (event.type === "last_words_given") {
      assertKnownPlayerId(event.payload.playerId, knownPlayerIds, "last words");
      pendingLastWordsByPlayerId.delete(event.payload.playerId);
    }

    if (event.type === "day_speech_given") {
      assertKnownPlayerId(event.payload.playerId, knownPlayerIds, "speaker");
      latestDaySpeechDayNumber = event.payload.dayNumber;
      latestDaySpeechRound = event.payload.round;
      addUnique(
        daySpeechByKey,
        speechKey(event.payload.dayNumber, event.payload.round),
        event.payload.playerId,
      );
    }

    if (event.type === "pk_speech_given") {
      assertKnownPlayerId(event.payload.playerId, knownPlayerIds, "speaker");
      latestPkSpeechDayNumber = event.payload.dayNumber;
      latestPkSpeechRound = event.payload.round;
      addUnique(
        pkSpeechByKey,
        speechKey(event.payload.dayNumber, event.payload.round),
        event.payload.playerId,
      );
    }

    if (event.type === "vote_cast") {
      assertKnownPlayerId(event.payload.voterPlayerId, knownPlayerIds, "voter");
      if (event.payload.targetPlayerId !== null) {
        assertKnownPlayerId(event.payload.targetPlayerId, knownPlayerIds, "vote target");
      }

      const key = voteKey(
        event.payload.dayNumber,
        event.payload.round,
        event.payload.voteType,
      );
      const existing = votesByKey.get(key);
      const vote = {
        voterPlayerId: event.payload.voterPlayerId,
        targetPlayerId: event.payload.targetPlayerId,
      };

      votesByKey.set(key, {
        dayNumber: event.payload.dayNumber,
        round: event.payload.round,
        voteType: event.payload.voteType,
        votes: [...(existing?.votes ?? []), vote],
      });
    }

    if (event.type === "exile_resolved") {
      if (event.payload.exiledPlayerId !== null) {
        assertKnownPlayerId(event.payload.exiledPlayerId, knownPlayerIds, "dead");
        dead.add(event.payload.exiledPlayerId);
        pendingLastWordsByPlayerId.set(event.payload.exiledPlayerId, {
          playerId: event.payload.exiledPlayerId,
          reason: "exile",
        });
      }

      for (const playerId of event.payload.tiedPlayerIds) {
        assertKnownPlayerId(playerId, knownPlayerIds, "tied");
      }

      for (const vote of event.payload.voteTable) {
        assertKnownPlayerId(vote.voterPlayerId, knownPlayerIds, "voter");
        if (vote.targetPlayerId !== null) {
          assertKnownPlayerId(vote.targetPlayerId, knownPlayerIds, "vote target");
        }
      }

      if (
        event.payload.exiledPlayerId === null &&
        event.payload.tiedPlayerIds.length > 0 &&
        event.payload.voteType === "exile"
      ) {
        pk = {
          status: "pending",
          dayNumber: event.payload.dayNumber,
          round: event.payload.round,
          tiedPlayerIds: event.payload.tiedPlayerIds,
        };
      } else if (
        event.payload.exiledPlayerId === null &&
        event.payload.tiedPlayerIds.length > 0 &&
        event.payload.voteType === "pk"
      ) {
        pk = {
          status: "no_exile",
          dayNumber: event.payload.dayNumber,
          round: event.payload.round,
          tiedPlayerIds: event.payload.tiedPlayerIds,
          reason: "second_tie",
        };
      } else if (event.payload.exiledPlayerId === null) {
        pk = {
          status: "no_exile",
          dayNumber: event.payload.dayNumber,
          round: event.payload.round,
          tiedPlayerIds: [],
          reason: "no_exile",
        };
      } else {
        pk = { status: "none" };
      }
    }

    if (event.type === "hunter_shot_decided") {
      assertKnownPlayerId(event.payload.targetPlayerId, knownPlayerIds, "dead");
      dead.add(event.payload.targetPlayerId);
      pendingLastWordsByPlayerId.set(event.payload.targetPlayerId, {
        playerId: event.payload.targetPlayerId,
        reason: "hunter_shot",
      });
    }

    if (event.type === "game_ended") {
      currentPhase = "ended";
      dayNumber = event.payload.dayNumber;
      ended = {
        winner: event.payload.winner,
        reason: event.payload.reason,
        revealedRoles: event.payload.revealedRoles,
      };
    }

    if (event.type === "witch_antidote_decided" && event.payload.used) {
      antidoteAvailable = false;
    }

    if (event.type === "witch_poison_decided" && event.payload.used) {
      poisonAvailable = false;
    }
  }

  const currentSpeechRound = getCurrentSpeechRound({
    currentDayNumber: dayNumber,
    latestDaySpeechDayNumber,
    latestDaySpeechRound,
    latestPkSpeechDayNumber,
    latestPkSpeechRound,
    pk,
  });

  return {
    currentPhase,
    dayNumber,
    alivePlayerIds: allPlayerIds.filter((playerId) => !dead.has(playerId)),
    deadPlayerIds: allPlayerIds.filter((playerId) => dead.has(playerId)),
    pendingLastWords: allPlayerIds.flatMap((playerId) => {
      const pending = pendingLastWordsByPlayerId.get(playerId);
      return pending ? [pending] : [];
    }),
    daySpeech: {
      dayNumber,
      round: currentSpeechRound,
      spokenPlayerIds:
        daySpeechByKey.get(speechKey(dayNumber, currentSpeechRound)) ?? [],
      pkSpokenPlayerIds:
        pkSpeechByKey.get(speechKey(dayNumber, currentSpeechRound)) ?? [],
    },
    votes: [...votesByKey.values()],
    pk,
    ended,
    witch: {
      antidoteAvailable,
      poisonAvailable,
    },
  };
}

function getCurrentSpeechRound(input: {
  readonly currentDayNumber: number;
  readonly latestDaySpeechDayNumber: number;
  readonly latestDaySpeechRound: number;
  readonly latestPkSpeechDayNumber: number;
  readonly latestPkSpeechRound: number;
  readonly pk: DerivedPkState;
}): number {
  if (
    input.latestPkSpeechDayNumber === input.currentDayNumber &&
    input.latestPkSpeechRound > 1
  ) {
    return input.latestPkSpeechRound;
  }

  if (
    input.pk.status === "pending" &&
    input.pk.dayNumber === input.currentDayNumber
  ) {
    return input.pk.round + 1;
  }

  if (input.latestDaySpeechDayNumber === input.currentDayNumber) {
    return input.latestDaySpeechRound;
  }

  return 1;
}

function assertKnownPlayerId(
  playerId: PlayerId,
  knownPlayerIds: ReadonlySet<PlayerId>,
  label: string,
): void {
  if (!knownPlayerIds.has(playerId)) {
    throw new Error(`Unknown ${label} player id ${playerId}`);
  }
}

function speechKey(dayNumber: number, round: number): string {
  return `${dayNumber}:${round}`;
}

function voteKey(dayNumber: number, round: number, voteType: VoteType): string {
  return `${dayNumber}:${round}:${voteType}`;
}

function addUnique(
  map: Map<string, PlayerId[]>,
  key: string,
  playerId: PlayerId,
): void {
  const existing = map.get(key) ?? [];
  if (existing.includes(playerId)) {
    return;
  }

  map.set(key, [...existing, playerId]);
}
