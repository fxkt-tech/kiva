import { createPlayerSnapshot, type PlayerSnapshot } from "../player";
import type { RuleRoleId, PlayerId } from "../types";
import { seedActors } from "@/seeds/actors";

export function testPlayer(
  playerId: PlayerId,
  seatNo: number,
  ruleRoleId: RuleRoleId,
): PlayerSnapshot {
  const actor = seedActors[(seatNo - 1) % seedActors.length];
  if (!actor) throw new Error(`Missing test Actor for seat ${seatNo}`);
  return createPlayerSnapshot({ playerId, seatNo, actor, ruleRoleId });
}
