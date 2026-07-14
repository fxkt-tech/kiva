import type { ActorDefinition } from "./actor-definition";
import { assertExactObjectKeys, isPlainObject } from "./model-binding";
import { isRuleRoleId, type RuleRoleId } from "./rule-role";
import { createTwelvePlayerRuleset } from "./types";

export const CURRENT_RULESET_ID = "classic_twelve" as const;

export type LineupSeat = {
  readonly seatNo: number;
  readonly ruleRoleId: RuleRoleId;
  readonly actorId: string;
};

export type Lineup = {
  readonly id: string;
  readonly name: string;
  readonly rulesetId: typeof CURRENT_RULESET_ID;
  readonly seats: readonly LineupSeat[];
  readonly enabled: boolean;
  readonly revision: number;
};

export function validateLineups(
  value: unknown,
  actors: readonly ActorDefinition[],
): readonly Lineup[] {
  if (!Array.isArray(value)) {
    throw new Error("Lineups must be an array");
  }
  const actorsById = new Map(actors.map((actor) => [actor.id, actor]));
  const ids = new Set<string>();

  for (const [index, lineup] of value.entries()) {
    const path = `Lineup[${index}]`;
    if (!isPlainObject(lineup)) {
      throw new Error(`${path} must be an object`);
    }
    assertExactObjectKeys(lineup, path, [
      "id",
      "name",
      "rulesetId",
      "seats",
      "enabled",
      "revision",
    ]);
    const id = stableText(lineup.id, `${path}.id`);
    if (ids.has(id)) {
      throw new Error(`Duplicate Lineup id: ${id}`);
    }
    ids.add(id);
    stableText(lineup.name, `${path}.name`);
    if (lineup.rulesetId !== CURRENT_RULESET_ID) {
      throw new Error(`${path}.rulesetId is unsupported`);
    }
    if (typeof lineup.enabled !== "boolean") {
      throw new Error(`${path}.enabled must be a boolean`);
    }
    if (!Number.isInteger(lineup.revision) || (lineup.revision as number) < 1) {
      throw new Error(`${path}.revision must be a positive integer`);
    }
    validateSeats(lineup.seats, actorsById, path);
  }

  return value as readonly Lineup[];
}

function validateSeats(
  value: unknown,
  actorsById: ReadonlyMap<string, ActorDefinition>,
  path: string,
): void {
  const ruleset = createTwelvePlayerRuleset();
  if (!Array.isArray(value) || value.length !== ruleset.playerCount) {
    throw new Error(`${path}.seats must contain exactly ${ruleset.playerCount} seats`);
  }
  const seatNumbers = new Set<number>();
  const actorIds = new Set<string>();
  const roleCounts = new Map<RuleRoleId, number>();

  for (const [index, seat] of value.entries()) {
    const seatPath = `${path}.seats[${index}]`;
    if (!isPlainObject(seat)) {
      throw new Error(`${seatPath} must be an object`);
    }
    assertExactObjectKeys(seat, seatPath, [
      "seatNo",
      "ruleRoleId",
      "actorId",
    ]);
    if (
      !Number.isInteger(seat.seatNo) ||
      (seat.seatNo as number) < 1 ||
      (seat.seatNo as number) > ruleset.playerCount
    ) {
      throw new Error(`${seatPath}.seatNo is invalid`);
    }
    const seatNo = seat.seatNo as number;
    if (seatNumbers.has(seatNo)) {
      throw new Error(`${seatPath} duplicates seat ${seatNo}`);
    }
    seatNumbers.add(seatNo);

    if (!isRuleRoleId(seat.ruleRoleId)) {
      throw new Error(`${seatPath}.ruleRoleId is unsupported`);
    }
    roleCounts.set(
      seat.ruleRoleId,
      (roleCounts.get(seat.ruleRoleId) ?? 0) + 1,
    );

    const actorId = stableText(seat.actorId, `${seatPath}.actorId`);
    const actor = actorsById.get(actorId);
    if (!actor) {
      throw new Error(`${seatPath}.actorId references unknown Actor: ${actorId}`);
    }
    if (!actor.enabled) {
      throw new Error(`${seatPath}.actorId references disabled Actor: ${actorId}`);
    }
    if (actorIds.has(actorId)) {
      throw new Error(`${seatPath}.actorId duplicates Actor: ${actorId}`);
    }
    actorIds.add(actorId);
  }

  for (const [roleId, expected] of Object.entries(ruleset.roleCounts)) {
    const actual = roleCounts.get(roleId as RuleRoleId) ?? 0;
    if (actual !== expected) {
      throw new Error(
        `${path}.seats must contain ${expected} ${roleId} Rule Roles; received ${actual}`,
      );
    }
  }
}

function stableText(value: unknown, path: string): string {
  if (
    typeof value !== "string" ||
    value.trim().length === 0 ||
    value !== value.trim()
  ) {
    throw new Error(`${path} must be a stable non-blank string`);
  }
  return value;
}
