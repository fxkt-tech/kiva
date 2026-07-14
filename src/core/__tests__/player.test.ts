import { describe, expect, it } from "vitest";
import { seedActors } from "@/seeds/actors";
import {
  createPlayerSnapshot,
  validateBoard,
  validatePlayerSnapshot,
  type PlayerSnapshot,
} from "../player";
import {
  createDefaultRuleset,
  type PlayerId,
  type RuleRoleId,
} from "../types";
import { testPlayer } from "./test-player";

const roles: readonly RuleRoleId[] = [
  "werewolf", "werewolf", "werewolf", "werewolf",
  "seer", "witch", "hunter", "guard",
  "villager", "villager", "villager", "villager",
];

function board(): PlayerSnapshot[] {
  return roles.map((role, index) =>
    testPlayer(`p${index + 1}` as PlayerId, index + 1, role),
  );
}

describe("PlayerSnapshot v2", () => {
  it("freezes one Actor and one code-owned Rule Role per seat", () => {
    const snapshot = createPlayerSnapshot({
      playerId: "p1" as PlayerId,
      seatNo: 1,
      actor: seedActors[0]!,
      ruleRoleId: "werewolf",
    });
    expect(snapshot).toMatchObject({
      playerId: "p1",
      seatNo: 1,
      actor: { sourceId: seedActors[0]!.id },
      ruleRole: {
        id: "werewolf",
        faction: "wolves",
        team: "wolf",
        mechanicKey: "wolf_kill",
        initialPrivateKnowledge: ["own_role", "wolf_teammates"],
      },
    });
    expect(Object.keys(snapshot).sort()).toEqual([
      "actor",
      "playerId",
      "ruleRole",
      "seatNo",
    ]);
  });

  it("copies Actor content into an immutable-by-convention snapshot", () => {
    const actor = structuredClone(seedActors[0]!);
    const snapshot = createPlayerSnapshot({
      playerId: "p1" as PlayerId,
      seatNo: 1,
      actor,
      ruleRoleId: "seer",
    });
    (actor.core as { stableCore: string }).stableCore = "mutated";
    expect(snapshot.actor.core.stableCore).not.toBe("mutated");
    expect(validatePlayerSnapshot(snapshot)).toEqual(snapshot);
  });

  it("rejects legacy flattened fields", () => {
    const snapshot = board()[0]!;
    expect(() =>
      validatePlayerSnapshot({ ...snapshot, name: "legacy" }),
    ).toThrow("keys are invalid");
  });

  it("validates the fixed 12-seat composition", () => {
    expect(validateBoard(board(), createDefaultRuleset())).toEqual({ ok: true });
  });

  it("rejects duplicate Actors and wrong role counts", () => {
    const duplicateActor = board();
    duplicateActor[1] = {
      ...duplicateActor[1]!,
      actor: duplicateActor[0]!.actor,
    };
    expect(validateBoard(duplicateActor, createDefaultRuleset())).toEqual({
      ok: false,
      reason: "duplicate_actor",
    });

    const wrongRole = board();
    wrongRole[0] = testPlayer("p1" as PlayerId, 1, "villager");
    expect(validateBoard(wrongRole, createDefaultRuleset())).toEqual({
      ok: false,
      reason: "invalid_role_count",
      role: "werewolf",
      expected: 4,
      actual: 3,
    });
  });
});
