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
  readonly display?: {
    readonly title?: string;
    readonly text?: string;
  };
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

export type WolfKillSelectedEvent = GameEventBase<
  "wolf_kill_selected",
  { readonly targetPlayerId: PlayerId }
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

export type NightResolvedEvent = GameEventBase<
  "night_resolved",
  { readonly deadPlayerIds: readonly PlayerId[] }
>;

export type DeathAnnouncedEvent = GameEventBase<
  "death_announced",
  { readonly deadPlayerIds: readonly PlayerId[] }
>;

export type GameEvent =
  | PhaseStartedEvent
  | RoleAssignedEvent
  | WolfKillSelectedEvent
  | SeerCheckSelectedEvent
  | SeerCheckResultEvent
  | WitchDeathInfoShownEvent
  | WitchAntidoteDecidedEvent
  | WitchPoisonDecidedEvent
  | NightResolvedEvent
  | DeathAnnouncedEvent;
