import { assertExactObjectKeys } from "./model-binding";
import {
  RULE_ROLE_IDS,
  type RuleRoleId,
} from "./rule-role";

export {
  RULE_ROLE_IDS,
  factionForRuleRole,
  type Faction,
  type RuleRoleId,
} from "./rule-role";

export type Brand<T, Name extends string> = T & { readonly __brand: Name };

export type GameId = Brand<string, "GameId">;
export type PlayerId = Brand<string, "PlayerId">;
export type EventId = Brand<string, "EventId">;
export type DraftId = Brand<string, "DraftId">;

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

export type RuleRoleCounts = {
  readonly werewolf: number;
  readonly seer: number;
  readonly witch: number;
  readonly hunter: number;
  readonly guard: number;
  readonly villager: number;
};

export const TWELVE_PLAYER_RULE_ROLE_COUNTS = {
  werewolf: 4,
  seer: 1,
  witch: 1,
  hunter: 1,
  guard: 1,
  villager: 4,
} as const satisfies RuleRoleCounts;

export type Ruleset = {
  readonly playerCount: number;
  readonly roleCounts: RuleRoleCounts;
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

export function isWerewolfRole(role: RuleRoleId): role is "werewolf" {
  return role === "werewolf";
}

export function createDefaultRuleset(): Ruleset {
  return createTwelvePlayerRuleset();
}

export function createTwelvePlayerRuleset(): Ruleset {
  return {
    playerCount: 12,
    roleCounts: { ...TWELVE_PLAYER_RULE_ROLE_COUNTS },
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

export function validateRuleset(value: unknown): Ruleset {
  if (!isRecord(value)) {
    throw new Error("Ruleset must be an object");
  }
  const roleCounts = value.roleCounts;
  if (!isRecord(roleCounts)) {
    throw new Error("Ruleset must be an object");
  }
  assertExactObjectKeys(value, "Ruleset", [
    "playerCount",
    "roleCounts",
    "winCondition",
    "witchFirstNightSelfSave",
    "witchAllowSameNightAntidoteAndPoison",
    "guardCanSelfProtect",
    "guardForbidConsecutiveSameTarget",
    "guardAndWitchSaveIsSafe",
    "voteReveal",
    "deadRoleReveal",
    "pkVoters",
    "allowAbstainVote",
  ]);
  assertExactObjectKeys(roleCounts, "Ruleset roleCounts", RULE_ROLE_IDS);
  if (!Number.isInteger(value.playerCount) || (value.playerCount as number) < 1) {
    throw new Error("Ruleset playerCount must be a positive integer");
  }
  let roleCountTotal = 0;
  for (const role of RULE_ROLE_IDS) {
    const count = roleCounts[role];
    if (!Number.isInteger(count) || (count as number) < 0) {
      throw new Error(`Ruleset roleCounts.${role} must be a non-negative integer`);
    }
    roleCountTotal += count as number;
  }
  if (roleCountTotal !== value.playerCount) {
    throw new Error("Ruleset role counts must sum to playerCount");
  }
  const currentBoard = createTwelvePlayerRuleset();
  if (
    value.playerCount !== currentBoard.playerCount ||
    RULE_ROLE_IDS.some(
      (role) => roleCounts[role] !== currentBoard.roleCounts[role],
    )
  ) {
    throw new Error("Unsupported ruleset board; expected the current 12-player board");
  }
  if (
    value.winCondition !== "slaughter_side" &&
    value.winCondition !== "slaughter_all"
  ) {
    throw new Error("Ruleset winCondition is invalid");
  }
  if (value.voteReveal !== "after_all_votes" && value.voteReveal !== "immediate") {
    throw new Error("Ruleset voteReveal is invalid");
  }
  if (value.deadRoleReveal !== "endgame" && value.deadRoleReveal !== "on_death") {
    throw new Error("Ruleset deadRoleReveal is invalid");
  }
  if (
    value.pkVoters !== "non_pk_only" &&
    value.pkVoters !== "all_living_non_self"
  ) {
    throw new Error("Ruleset pkVoters is invalid");
  }
  for (const field of [
    "witchFirstNightSelfSave",
    "witchAllowSameNightAntidoteAndPoison",
    "guardCanSelfProtect",
    "guardForbidConsecutiveSameTarget",
    "guardAndWitchSaveIsSafe",
    "allowAbstainVote",
  ] as const) {
    if (typeof value[field] !== "boolean") {
      throw new Error(`Ruleset ${field} must be a boolean`);
    }
  }

  return structuredClone(value) as Ruleset;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}
