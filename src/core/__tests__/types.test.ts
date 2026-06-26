import { describe, expect, expectTypeOf, it } from "vitest";
import {
  createDefaultRuleset,
  factionForRole,
  GAME_ROLES,
  isWerewolfRole,
  type GameRole,
  type Ruleset,
  type SixPlayerRoleCounts,
} from "../types";

describe("core types", () => {
  it("defines the fixed first-version roles", () => {
    expect(GAME_ROLES).toEqual(["werewolf", "seer", "witch", "villager"]);
  });

  it("detects werewolf role", () => {
    expect(isWerewolfRole("werewolf")).toBe(true);
    expect(isWerewolfRole("seer")).toBe(false);
  });

  it("creates default ruleset from approved spec", () => {
    expect(createDefaultRuleset()).toEqual({
      playerCount: 6,
      roleCounts: {
        werewolf: 2,
        seer: 1,
        witch: 1,
        villager: 2,
      },
      winCondition: "slaughter_side",
      witchFirstNightSelfSave: true,
      witchAllowSameNightAntidoteAndPoison: false,
      voteReveal: "after_all_votes",
      deadRoleReveal: "endgame",
      pkVoters: "non_pk_only",
      allowAbstainVote: true,
    });
  });

  it("maps roles to factions", () => {
    expect(factionForRole("werewolf")).toBe("wolves");
    expect(factionForRole("seer")).toBe("good");
    expect(factionForRole("witch")).toBe("good");
    expect(factionForRole("villager")).toBe("good");
  });

  it("keeps GameRole as a narrow union", () => {
    expectTypeOf<GameRole>().toEqualTypeOf<
      "werewolf" | "seer" | "witch" | "villager"
    >();
  });

  it("keeps ruleset role counts fixed for the first-version board", () => {
    expectTypeOf<Ruleset["roleCounts"]>().toEqualTypeOf<SixPlayerRoleCounts>();
  });
});
