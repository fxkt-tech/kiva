export type Brand<T, Name extends string> = T & { readonly __brand: Name };

export type GameId = Brand<string, "GameId">;
export type PlayerId = Brand<string, "PlayerId">;
export type EventId = Brand<string, "EventId">;
export type DraftId = Brand<string, "DraftId">;

export const GAME_ROLES = [
  "werewolf",
  "seer",
  "witch",
  "hunter",
  "guard",
  "villager",
] as const;
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

export type RoleCounts = {
  readonly werewolf: number;
  readonly seer: number;
  readonly witch: number;
  readonly hunter: number;
  readonly guard: number;
  readonly villager: number;
};

export type SixPlayerRoleCounts = RoleCounts;

export type Ruleset = {
  readonly playerCount: number;
  readonly roleCounts: RoleCounts;
  readonly winCondition: WinCondition;
  readonly witchFirstNightSelfSave: boolean;
  readonly witchAllowSameNightAntidoteAndPoison: boolean;
  readonly guardCanSelfProtect: boolean;
  readonly guardForbidConsecutiveSameTarget: boolean;
  readonly guardAndWitchSaveIsSafe: boolean;
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
  hunter: "good",
  guard: "good",
  villager: "good",
} satisfies Record<GameRole, Faction>;

export function factionForRole(role: GameRole): Faction {
  return FACTION_BY_ROLE[role];
}

export function createDefaultRuleset(): Ruleset {
  return createTwelvePlayerRuleset();
}

export function createSixPlayerRuleset(): Ruleset {
  return {
    playerCount: 6,
    roleCounts: {
      werewolf: 2,
      seer: 1,
      witch: 1,
      hunter: 0,
      guard: 0,
      villager: 2,
    },
    winCondition: "slaughter_side",
    witchFirstNightSelfSave: true,
    witchAllowSameNightAntidoteAndPoison: false,
    guardCanSelfProtect: true,
    guardForbidConsecutiveSameTarget: true,
    guardAndWitchSaveIsSafe: true,
    voteReveal: "after_all_votes",
    deadRoleReveal: "endgame",
    pkVoters: "non_pk_only",
    allowAbstainVote: true,
  };
}

export function createTwelvePlayerRuleset(): Ruleset {
  return {
    ...createSixPlayerRuleset(),
    playerCount: 12,
    roleCounts: {
      werewolf: 4,
      seer: 1,
      witch: 1,
      hunter: 1,
      guard: 1,
      villager: 4,
    },
  };
}
