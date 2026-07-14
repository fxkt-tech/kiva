import type { ActorDefinition } from "@/core/actor-definition";
import type { LineupSeat } from "@/core/lineup";
import { ruleRoleById, type RuleRole } from "@/core/rule-role";
import { TWELVE_PLAYER_RULE_ROLE_COUNTS } from "@/core/types";

export const requiredTwelvePlayerRuleRoleCounts =
  TWELVE_PLAYER_RULE_ROLE_COUNTS;

export function createRandomLineupSeats(input: {
  readonly ruleRoles: readonly RuleRole[];
  readonly actors: readonly ActorDefinition[];
  readonly random?: () => number;
}): readonly LineupSeat[] {
  const random = input.random ?? Math.random;
  const availableRoleIds = new Set(input.ruleRoles.map((role) => role.id));
  const rolePool = Object.entries(requiredTwelvePlayerRuleRoleCounts).flatMap(
    ([roleId, count]) =>
      availableRoleIds.has(roleId as RuleRole["id"])
        ? Array.from({ length: count }, () => roleId as RuleRole["id"])
        : [],
  );
  const actorPool = shuffle(
    input.actors.filter((actor) => actor.enabled),
    random,
  ).slice(0, 12);

  return shuffle(rolePool, random).map((ruleRoleId, index) => ({
    seatNo: index + 1,
    ruleRoleId,
    actorId: actorPool[index]?.id ?? "",
  }));
}

export function validateLineupSeats(
  seats: readonly LineupSeat[],
  ruleRoles: readonly RuleRole[],
  actors: readonly ActorDefinition[],
): readonly string[] {
  const messages = [
    ...ruleRoleCountMessages(seats),
    ...seatReferenceMessages(seats, ruleRoles, actors),
  ];
  return [...new Set(messages)];
}

export function ruleRoleCountMessages(
  seats: readonly Pick<LineupSeat, "ruleRoleId">[],
): readonly string[] {
  const messages: string[] = [];
  for (const [roleId, expected] of Object.entries(
    requiredTwelvePlayerRuleRoleCounts,
  )) {
    const actual = seats.filter((seat) => seat.ruleRoleId === roleId).length;
    if (actual !== expected) {
      messages.push(
        `${ruleRoleById(roleId as RuleRole["id"]).name}需要 ${expected} 个，当前 ${actual} 个。`,
      );
    }
  }
  return messages;
}

function seatReferenceMessages(
  seats: readonly LineupSeat[],
  ruleRoles: readonly RuleRole[],
  actors: readonly ActorDefinition[],
): readonly string[] {
  const ruleRoleIds = new Set(ruleRoles.map((role) => role.id));
  const actorIds = new Set(
    actors.filter((actor) => actor.enabled).map((actor) => actor.id),
  );
  const selectedActorIds = seats.map((seat) => seat.actorId).filter(Boolean);
  const messages: string[] = [];

  if (seats.length !== 12) {
    messages.push(`需要 12 个座位，当前 ${seats.length} 个。`);
  }
  for (const seat of seats) {
    if (!ruleRoleIds.has(seat.ruleRoleId)) {
      messages.push(`Seat ${seat.seatNo} 选择了未知规则身份。`);
    }
    if (!actorIds.has(seat.actorId)) {
      messages.push(`Seat ${seat.seatNo} 选择了不可用 Actor：${seat.actorId || "空"}`);
    }
  }
  if (new Set(selectedActorIds).size !== selectedActorIds.length) {
    messages.push("Actor 不能重复。");
  }
  return messages;
}

function shuffle<T>(items: readonly T[], random: () => number): T[] {
  const shuffled = [...items];
  for (let index = shuffled.length - 1; index > 0; index -= 1) {
    const swapIndex = Math.floor(random() * (index + 1));
    [shuffled[index], shuffled[swapIndex]] = [
      shuffled[swapIndex]!,
      shuffled[index]!,
    ];
  }
  return shuffled;
}
