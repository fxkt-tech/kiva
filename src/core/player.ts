import {
  createActorSnapshot,
  validateActorSnapshot,
  type ActorDefinition,
  type ActorSnapshot,
} from "./actor-definition";
import { assertExactObjectKeys, isPlainObject } from "./model-binding";
import {
  createRuleRoleSnapshot,
  RULE_ROLE_IDS,
  validateRuleRoleSnapshot,
  type RuleRole,
  type RuleRoleId,
} from "./rule-role";
import type { PlayerId, Ruleset } from "./types";

export type PlayerSnapshot = {
  readonly playerId: PlayerId;
  readonly seatNo: number;
  readonly actor: ActorSnapshot;
  readonly ruleRole: RuleRole;
};

export type CreatePlayerSnapshotInput = {
  readonly playerId: PlayerId;
  readonly seatNo: number;
  readonly actor: ActorDefinition | ActorSnapshot;
  readonly ruleRoleId: RuleRoleId;
};

export type BoardValidationResult =
  | { readonly ok: true }
  | {
      readonly ok: false;
      readonly reason: "invalid_player_count";
      readonly expected: number;
      readonly actual: number;
    }
  | { readonly ok: false; readonly reason: "duplicate_player_id" }
  | { readonly ok: false; readonly reason: "duplicate_actor" }
  | {
      readonly ok: false;
      readonly reason: "invalid_seat";
      readonly seatNo: number;
    }
  | { readonly ok: false; readonly reason: "duplicate_seat" }
  | {
      readonly ok: false;
      readonly reason: "invalid_role_count";
      readonly role: RuleRoleId;
      readonly expected: number;
      readonly actual: number;
    };

export function createPlayerSnapshot(
  input: CreatePlayerSnapshotInput,
): PlayerSnapshot {
  return {
    playerId: input.playerId,
    seatNo: input.seatNo,
    actor:
      "id" in input.actor
        ? createActorSnapshot(input.actor)
        : validateActorSnapshot(input.actor),
    ruleRole: createRuleRoleSnapshot(input.ruleRoleId),
  };
}

export function validatePlayerSnapshot(input: unknown): PlayerSnapshot {
  if (!isPlainObject(input)) {
    throw new Error("Player snapshot must be an object");
  }
  assertExactObjectKeys(input, "Player snapshot", [
    "playerId",
    "seatNo",
    "actor",
    "ruleRole",
  ]);
  if (!isNonBlankString(input.playerId)) {
    throw new Error("Player snapshot playerId must be set");
  }
  if (!Number.isInteger(input.seatNo) || (input.seatNo as number) < 1) {
    throw new Error("Player snapshot seatNo must be a positive integer");
  }

  return {
    playerId: input.playerId as PlayerId,
    seatNo: input.seatNo as number,
    actor: validateActorSnapshot(input.actor),
    ruleRole: validateRuleRoleSnapshot(input.ruleRole),
  };
}

export function validateBoard(
  players: readonly PlayerSnapshot[],
  ruleset: Ruleset,
): BoardValidationResult {
  if (players.length !== ruleset.playerCount) {
    return {
      ok: false,
      reason: "invalid_player_count",
      expected: ruleset.playerCount,
      actual: players.length,
    };
  }

  if (new Set(players.map((player) => player.playerId)).size !== players.length) {
    return { ok: false, reason: "duplicate_player_id" };
  }
  if (
    new Set(players.map((player) => player.actor.sourceId)).size !==
    players.length
  ) {
    return { ok: false, reason: "duplicate_actor" };
  }

  const invalidSeat = players.find(
    (player) =>
      !Number.isInteger(player.seatNo) ||
      player.seatNo < 1 ||
      player.seatNo > ruleset.playerCount,
  );
  if (invalidSeat) {
    return { ok: false, reason: "invalid_seat", seatNo: invalidSeat.seatNo };
  }

  if (new Set(players.map((player) => player.seatNo)).size !== players.length) {
    return { ok: false, reason: "duplicate_seat" };
  }

  for (const role of RULE_ROLE_IDS) {
    const actual = players.filter(
      (player) => player.ruleRole.id === role,
    ).length;
    const expected = ruleset.roleCounts[role];
    if (actual !== expected) {
      return {
        ok: false,
        reason: "invalid_role_count",
        role,
        expected,
        actual,
      };
    }
  }

  return { ok: true };
}

function isNonBlankString(value: unknown): value is string {
  return typeof value === "string" && value.trim().length > 0;
}
