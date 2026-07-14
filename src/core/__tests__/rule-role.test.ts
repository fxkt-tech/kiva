import { describe, expect, it } from "vitest";
import {
  RULE_ROLES,
  createRuleRoleSnapshot,
  ruleRoleById,
  validateRuleRoleSnapshot,
} from "../rule-role";

describe("Rule Role registry", () => {
  it("is the complete code-owned mechanics registry", () => {
    expect(RULE_ROLES.map((role) => role.id).sort()).toEqual(
      ["guard", "hunter", "seer", "villager", "werewolf", "witch"],
    );
    expect(ruleRoleById("werewolf")).toMatchObject({
      faction: "wolves",
      mechanicKey: "wolf_kill",
      initialPrivateKnowledge: ["own_role", "wolf_teammates"],
    });
  });

  it("accepts only snapshots matching the registry", () => {
    const snapshot = createRuleRoleSnapshot("seer");
    expect(validateRuleRoleSnapshot(snapshot)).toEqual(snapshot);
    expect(
      validateRuleRoleSnapshot({
        nightOrder: snapshot.nightOrder,
        initialPrivateKnowledge: snapshot.initialPrivateKnowledge,
        mechanicKey: snapshot.mechanicKey,
        team: snapshot.team,
        faction: snapshot.faction,
        name: snapshot.name,
        id: snapshot.id,
      }),
    ).toEqual(snapshot);
    expect(() =>
      validateRuleRoleSnapshot({ ...snapshot, name: "先知" }),
    ).toThrow("does not match registry");
    expect(snapshot).not.toHaveProperty("systemPrompt");
    expect(snapshot).not.toHaveProperty("defaultModelBinding");
  });
});
