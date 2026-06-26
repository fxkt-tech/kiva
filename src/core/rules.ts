import type { PlayerSnapshot } from "./player";
import type { GameRole, PlayerId, Ruleset } from "./types";

export type NightActionKind = "wolf_kill" | "seer_check" | "witch_poison";

const GOD_ROLES = new Set<GameRole>(["seer", "witch"]);

export type NightResolutionInput = {
  readonly wolfKillTargetId: PlayerId | null;
  readonly antidoteTargetId: PlayerId | null;
  readonly poisonTargetId: PlayerId | null;
};

export type WitchDecisionInput = {
  readonly nightNumber: number;
  readonly witchPlayerId: PlayerId;
  readonly killedPlayerId: PlayerId | null;
  readonly antidoteTargetId: PlayerId | null;
  readonly poisonTargetId: PlayerId | null;
};

export type WitchDecisionValidationResult =
  | { readonly ok: true }
  | {
      readonly ok: false;
      readonly reason:
        | "same_night_antidote_and_poison_forbidden"
        | "first_night_self_save_forbidden"
        | "antidote_without_death"
        | "antidote_target_mismatch";
    };

export type WinCheckResult =
  | { readonly ended: false }
  | {
      readonly ended: true;
      readonly winner: "wolves" | "good";
      readonly reason:
        | "all_wolves_dead"
        | "all_gods_dead"
        | "all_villagers_dead"
        | "all_good_dead";
    };

export function getLegalNightTargets(
  action: NightActionKind,
  players: readonly PlayerSnapshot[],
  alivePlayerIds: readonly PlayerId[],
  actorPlayerId?: PlayerId,
): readonly PlayerId[] {
  const alive = new Set(alivePlayerIds);

  if (action === "wolf_kill") {
    return players
      .filter((player) => alive.has(player.playerId))
      .filter((player) => player.gameRole !== "werewolf")
      .map((player) => player.playerId);
  }

  if (action === "seer_check") {
    return getActorSpecificTargets(players, alive, "seer", actorPlayerId);
  }

  return getActorSpecificTargets(players, alive, "witch", actorPlayerId);
}

export function validateWitchDecision(
  input: WitchDecisionInput,
  ruleset: Ruleset,
): WitchDecisionValidationResult {
  if (
    input.antidoteTargetId &&
    input.poisonTargetId &&
    !ruleset.witchAllowSameNightAntidoteAndPoison
  ) {
    return { ok: false, reason: "same_night_antidote_and_poison_forbidden" };
  }

  if (input.antidoteTargetId && !input.killedPlayerId) {
    return { ok: false, reason: "antidote_without_death" };
  }

  if (
    input.antidoteTargetId &&
    input.antidoteTargetId !== input.killedPlayerId
  ) {
    return { ok: false, reason: "antidote_target_mismatch" };
  }

  if (
    input.nightNumber === 1 &&
    input.antidoteTargetId === input.witchPlayerId &&
    !ruleset.witchFirstNightSelfSave
  ) {
    return { ok: false, reason: "first_night_self_save_forbidden" };
  }

  return { ok: true };
}

function getActorSpecificTargets(
  players: readonly PlayerSnapshot[],
  alive: ReadonlySet<PlayerId>,
  actorRole: GameRole,
  actorPlayerId?: PlayerId,
): readonly PlayerId[] {
  if (!actorPlayerId) {
    return [];
  }

  const actor = players.find((player) => player.playerId === actorPlayerId);
  if (!actor || !alive.has(actor.playerId) || actor.gameRole !== actorRole) {
    return [];
  }

  return players
    .filter((player) => alive.has(player.playerId))
    .filter((player) => player.playerId !== actorPlayerId)
    .map((player) => player.playerId);
}

export function resolveNightDeaths(
  input: NightResolutionInput,
): readonly PlayerId[] {
  const dead = new Set<PlayerId>();

  if (
    input.wolfKillTargetId &&
    input.wolfKillTargetId !== input.antidoteTargetId
  ) {
    dead.add(input.wolfKillTargetId);
  }

  if (input.poisonTargetId) {
    dead.add(input.poisonTargetId);
  }

  return [...dead];
}

export function checkWinCondition(
  players: readonly PlayerSnapshot[],
  deadPlayerIds: readonly PlayerId[],
  ruleset: Ruleset,
): WinCheckResult {
  const dead = new Set(deadPlayerIds);
  const livingPlayers = players.filter((player) => !dead.has(player.playerId));
  const livingWolves = livingPlayers.filter(
    (player) => player.gameRole === "werewolf",
  );

  if (livingWolves.length === 0) {
    return { ended: true, winner: "good", reason: "all_wolves_dead" };
  }

  if (ruleset.winCondition === "slaughter_all") {
    const livingGood = livingPlayers.filter(
      (player) => player.faction === "good",
    );
    if (livingGood.length === 0) {
      return { ended: true, winner: "wolves", reason: "all_good_dead" };
    }
    return { ended: false };
  }

  const livingGods = livingPlayers.filter((player) =>
    GOD_ROLES.has(player.gameRole),
  );
  if (livingGods.length === 0) {
    return { ended: true, winner: "wolves", reason: "all_gods_dead" };
  }

  const livingVillagers = livingPlayers.filter(
    (player) => player.gameRole === "villager",
  );
  if (livingVillagers.length === 0) {
    return { ended: true, winner: "wolves", reason: "all_villagers_dead" };
  }

  return { ended: false };
}
