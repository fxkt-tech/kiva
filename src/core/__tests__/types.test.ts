import { describe, expect, it } from "vitest";
import {
  createDefaultRuleset,
  GAME_ROLES,
  isWerewolfRole,
  type GameRole,
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
    expect(createDefaultRuleset()).toMatchObject({
      winCondition: "slaughter_side",
      witchFirstNightSelfSave: true,
      witchAllowSameNightAntidoteAndPoison: false,
      voteReveal: "after_all_votes",
      deadRoleReveal: "endgame",
      pkVoters: "non_pk_only",
    });
  });

  it("keeps GameRole as a narrow union", () => {
    const role: GameRole = "witch";
    expect(role).toBe("witch");
  });
});
