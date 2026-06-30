import { describe, expect, expectTypeOf, it } from "vitest";
import {
  createDefaultRuleset,
  createSixPlayerRuleset,
  factionForRole,
  GAME_ROLES,
  isWerewolfRole,
  type GameRole,
  type Ruleset,
  type SixPlayerRoleCounts,
} from "../types";

describe("core types", () => {
  it("defines the fixed first-version roles", () => {
    expect(GAME_ROLES).toEqual([
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

  it("keeps an explicit legacy six-player ruleset for existing games", () => {
    expect(createSixPlayerRuleset()).toMatchObject({
      playerCount: 6,
      roleCounts: {
        werewolf: 2,
        seer: 1,
        witch: 1,
        hunter: 0,
        guard: 0,
        villager: 2,
      },
    });
  });

  it("maps roles to factions", () => {
    expect(factionForRole("werewolf")).toBe("wolves");
    expect(factionForRole("seer")).toBe("good");
    expect(factionForRole("witch")).toBe("good");
    expect(factionForRole("hunter")).toBe("good");
    expect(factionForRole("guard")).toBe("good");
    expect(factionForRole("villager")).toBe("good");
  });

  it("keeps GameRole as a narrow union", () => {
    expectTypeOf<GameRole>().toEqualTypeOf<
      "werewolf" | "seer" | "witch" | "hunter" | "guard" | "villager"
    >();
  });

  it("keeps ruleset role counts fixed for the first-version board", () => {
    expectTypeOf<Ruleset["roleCounts"]>().toEqualTypeOf<SixPlayerRoleCounts>();
  });
});
