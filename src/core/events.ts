import type {
  DraftId,
  EventId,
  Faction,
  GameId,
  GameRole,
  Phase,
  PlayerId,
} from "./types";

export type EventStatus = "active" | "superseded";

export type EventVisibility =
  | { readonly kind: "public" }
  | { readonly kind: "host_only" }
  | { readonly kind: "player_private"; readonly playerIds: readonly PlayerId[] }
  | { readonly kind: "faction_private"; readonly faction: "wolves" }
  | { readonly kind: "custom"; readonly playerIds: readonly PlayerId[] };

export type GameEventBase<Type extends string, Payload> = {
  readonly id: EventId;
  readonly gameId: GameId;
  readonly index: number;
  readonly status: EventStatus;
  readonly type: Type;
  readonly phase: Phase;
  readonly actorPlayerId?: PlayerId;
  readonly targetPlayerIds?: readonly PlayerId[];
  readonly visibility: EventVisibility;
  readonly payload: Payload;
  readonly createdFromDraftId?: DraftId;
  readonly createdAt: string;
};

export type PhaseStartedEvent = GameEventBase<
  "phase_started",
  { readonly phase: Phase; readonly dayNumber: number }
>;

export type RoleAssignedEvent = GameEventBase<
  "role_assigned",
  {
    readonly playerId: PlayerId;
    readonly role: GameRole;
    readonly faction: Faction;
  }
>;

export type WolfLeaderSelectedEvent = GameEventBase<
  "wolf_leader_selected",
  { readonly leaderPlayerId: PlayerId }
>;

export type WolfStrategyGivenEvent = GameEventBase<
  "wolf_strategy_given",
  { readonly playerId: PlayerId; readonly text: string; readonly dayNumber: number }
>;

export type WolfOpinionGivenEvent = GameEventBase<
  "wolf_opinion_given",
  { readonly playerId: PlayerId; readonly text: string; readonly dayNumber: number }
>;

export type WolfVoteCastEvent = GameEventBase<
  "wolf_vote_cast",
  {
    readonly voterPlayerId: PlayerId;
    readonly targetPlayerId: PlayerId;
    readonly dayNumber: number;
  }
>;

export type WolfVoteEntry = {
  readonly voterPlayerId: PlayerId;
  readonly targetPlayerId: PlayerId;
};

export type WolfVoteTally = {
  readonly targetPlayerId: PlayerId;
  readonly count: number;
};

export type WolfVoteResolvedEvent = GameEventBase<
  "wolf_vote_resolved",
  {
    readonly votes: readonly WolfVoteEntry[];
    readonly tallies: readonly WolfVoteTally[];
    readonly tiedTargetPlayerIds: readonly PlayerId[];
    readonly targetPlayerId: PlayerId | null;
    readonly resolution: "majority" | "host_tiebreak" | null;
    readonly dayNumber: number;
  }
>;

export type SeerCheckSelectedEvent = GameEventBase<
  "seer_check_selected",
  { readonly targetPlayerId: PlayerId }
>;

export type SeerCheckResultEvent = GameEventBase<
  "seer_check_result",
  { readonly targetPlayerId: PlayerId; readonly result: "wolves" | "good" }
>;

export type WitchDeathInfoShownEvent = GameEventBase<
  "witch_death_info_shown",
  { readonly killedPlayerId: PlayerId | null }
>;

export type WitchAntidoteDecidedEvent = GameEventBase<
  "witch_antidote_decided",
  { readonly used: boolean; readonly targetPlayerId: PlayerId | null }
>;

export type WitchPoisonDecidedEvent = GameEventBase<
  "witch_poison_decided",
  { readonly used: boolean; readonly targetPlayerId: PlayerId | null }
>;

export type GuardProtectSelectedEvent = GameEventBase<
  "guard_protect_selected",
  { readonly targetPlayerId: PlayerId }
>;

export type HunterShotDecidedEvent = GameEventBase<
  "hunter_shot_decided",
  { readonly targetPlayerId: PlayerId }
>;

export type NightDeathReason = "wolf_kill" | "witch_poison";

export type NightDeath = {
  readonly playerId: PlayerId;
  readonly reason: NightDeathReason;
};

export type NightResolvedEvent = GameEventBase<
  "night_resolved",
  {
    readonly deadPlayerIds: readonly PlayerId[];
    readonly deaths: readonly NightDeath[];
  }
>;

export type DeathAnnouncedEvent = GameEventBase<
  "death_announced",
  { readonly deadPlayerIds: readonly PlayerId[] }
>;

export type VoteType = "sheriff" | "exile" | "pk";

export type GameEndReason =
  | "all_wolves_dead"
  | "all_gods_dead"
  | "all_villagers_dead"
  | "all_good_dead";

export type VoteTableEntry = {
  readonly voterPlayerId: PlayerId;
  readonly targetPlayerId: PlayerId | null;
};

export type RevealedRole = {
  readonly playerId: PlayerId;
  readonly roleId: GameRole;
  readonly roleName: string;
  readonly faction: Faction;
};

export type LastWordsReason = "night_death" | "exile" | "hunter_shot";

export type LastWordsGivenEvent = GameEventBase<
  "last_words_given",
  {
    readonly playerId: PlayerId;
    readonly text: string;
    readonly dayNumber: number;
    readonly reason: LastWordsReason;
  }
>;

export type DaySpeechGivenEvent = GameEventBase<
  "day_speech_given",
  {
    readonly playerId: PlayerId;
    readonly text: string;
    readonly dayNumber: number;
    readonly round: number;
  }
>;

export type VoteCastEvent = GameEventBase<
  "vote_cast",
  {
    readonly voterPlayerId: PlayerId;
    readonly targetPlayerId: PlayerId | null;
    readonly dayNumber: number;
    readonly round: number;
    readonly voteType: VoteType;
  }
>;

export type PkSpeechGivenEvent = GameEventBase<
  "pk_speech_given",
  {
    readonly playerId: PlayerId;
    readonly text: string;
    readonly dayNumber: number;
    readonly round: number;
  }
>;

export type ExileResolvedEvent = GameEventBase<
  "exile_resolved",
  {
    readonly exiledPlayerId: PlayerId | null;
    readonly tiedPlayerIds: readonly PlayerId[];
    readonly voteType: VoteType;
    readonly voteTable: readonly VoteTableEntry[];
    readonly dayNumber: number;
    readonly round: number;
    readonly revealedRoles: readonly RevealedRole[];
  }
>;

export type GameEndedEvent = GameEventBase<
  "game_ended",
  {
    readonly winner: "wolves" | "good";
    readonly reason: GameEndReason;
    readonly dayNumber: number;
    readonly revealedRoles: readonly RevealedRole[];
  }
>;

export type GameEvent =
  | PhaseStartedEvent
  | RoleAssignedEvent
  | WolfLeaderSelectedEvent
  | WolfStrategyGivenEvent
  | WolfOpinionGivenEvent
  | WolfVoteCastEvent
  | WolfVoteResolvedEvent
  | SeerCheckSelectedEvent
  | SeerCheckResultEvent
  | WitchDeathInfoShownEvent
  | WitchAntidoteDecidedEvent
  | WitchPoisonDecidedEvent
  | GuardProtectSelectedEvent
  | HunterShotDecidedEvent
  | NightResolvedEvent
  | DeathAnnouncedEvent
  | LastWordsGivenEvent
  | DaySpeechGivenEvent
  | PkSpeechGivenEvent
  | VoteCastEvent
  | ExileResolvedEvent
  | GameEndedEvent;
