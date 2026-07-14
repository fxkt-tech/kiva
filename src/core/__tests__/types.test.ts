import { describe, expect, expectTypeOf, it } from "vitest";
import {
  createDefaultRuleset,
  factionForRuleRole,
  RULE_ROLE_IDS,
  isWerewolfRole,
  type RuleRoleId,
  type RuleRoleCounts,
  type Ruleset,
  validateRuleset,
} from "../types";

describe("core types", () => {
  it("defines the fixed first-version roles", () => {
    expect(RULE_ROLE_IDS).toEqual([
      "werewolf",
      "seer",
      "witch",
      "hunter",
      "guard",
      "villager",
    ]);
  });

  it("detects werewolf role", () => {
    expect(isWerewolfRole("werewolf")).toBe(true);
    expect(isWerewolfRole("seer")).toBe(false);
  });

  it("creates default ruleset from approved spec", () => {
    expect(createDefaultRuleset()).toEqual({
      playerCount: 12,
      roleCounts: {
        werewolf: 4,
        seer: 1,
        witch: 1,
        hunter: 1,
        guard: 1,
        villager: 4,
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
    });
  });

  it("validates the complete current ruleset", () => {
    const ruleset = createDefaultRuleset();
    expect(validateRuleset(ruleset)).toEqual(ruleset);
    expect(() =>
      validateRuleset({ ...ruleset, roleCounts: { ...ruleset.roleCounts, guard: 0 } }),
    ).toThrow("sum to playerCount");
    expect(() =>
      validateRuleset({
        ...ruleset,
        playerCount: 6,
        roleCounts: {
          werewolf: 2,
          seer: 1,
          witch: 1,
          hunter: 0,
          guard: 0,
          villager: 2,
        },
      }),
    ).toThrow("expected the current 12-player board");
  });

  it("maps roles to factions", () => {
    expect(factionForRuleRole("werewolf")).toBe("wolves");
    expect(factionForRuleRole("seer")).toBe("good");
    expect(factionForRuleRole("witch")).toBe("good");
    expect(factionForRuleRole("hunter")).toBe("good");
    expect(factionForRuleRole("guard")).toBe("good");
    expect(factionForRuleRole("villager")).toBe("good");
  });

  it("keeps RuleRoleId as a narrow union", () => {
    expectTypeOf<RuleRoleId>().toEqualTypeOf<
      "werewolf" | "seer" | "witch" | "hunter" | "guard" | "villager"
    >();
  });

  it("keeps ruleset role counts fixed for the first-version board", () => {
    expectTypeOf<Ruleset["roleCounts"]>().toEqualTypeOf<RuleRoleCounts>();
  });
});
