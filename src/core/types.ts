export type Brand<T, Name extends string> = T & { readonly __brand: Name };

export type GameId = Brand<string, "GameId">;
export type PlayerId = Brand<string, "PlayerId">;
export type EventId = Brand<string, "EventId">;
export type DraftId = Brand<string, "DraftId">;

export const GAME_ROLES = ["werewolf", "seer", "witch", "villager"] as const;
export type GameRole = (typeof GAME_ROLES)[number];

export type Faction = "wolves" | "good";
export type Phase =
  | "setup"
  | "night"
  | "day"
  | "last_words"
  | "speech"
  | "vote"
  | "pk"
  | "ended";

export type WinCondition = "slaughter_side" | "slaughter_all";
export type VoteReveal = "after_all_votes" | "immediate";
export type DeadRoleReveal = "endgame" | "on_death";
export type PkVoters = "non_pk_only" | "all_living_non_self";

export type SixPlayerRoleCounts = {
  readonly werewolf: 2;
  readonly seer: 1;
  readonly witch: 1;
  readonly villager: 2;
};

export type Ruleset = {
  readonly playerCount: 6;
  readonly roleCounts: SixPlayerRoleCounts;
  readonly winCondition: WinCondition;
  readonly witchFirstNightSelfSave: boolean;
  readonly witchAllowSameNightAntidoteAndPoison: boolean;
  readonly voteReveal: VoteReveal;
  readonly deadRoleReveal: DeadRoleReveal;
  readonly pkVoters: PkVoters;
  readonly allowAbstainVote: boolean;
};

export function isWerewolfRole(role: GameRole): role is "werewolf" {
  return role === "werewolf";
}

const FACTION_BY_ROLE = {
  werewolf: "wolves",
  seer: "good",
  witch: "good",
  villager: "good",
} satisfies Record<GameRole, Faction>;

export function factionForRole(role: GameRole): Faction {
  return FACTION_BY_ROLE[role];
}

export function createDefaultRuleset(): Ruleset {
  return {
    playerCount: 6,
    roleCounts: {
      werewolf: 2,
      seer: 1,
      witch: 1,
      villager: 2,
    },
    winCondition: "slaughter_side",
    witchFirstNightSelfSave: true,
    witchAllowSameNightAntidoteAndPoison: false,
    voteReveal: "after_all_votes",
    deadRoleReveal: "endgame",
    pkVoters: "non_pk_only",
    allowAbstainVote: true,
  };
}
