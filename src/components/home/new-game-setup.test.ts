import { describe, expect, it } from "vitest";
import { RULE_ROLES } from "@/core/rule-role";
import { seedActors } from "@/seeds/actors";
import {
  createRandomLineupSeats,
  ruleRoleCountMessages,
  validateLineupSeats,
} from "./new-game-setup";

describe("new game lineup setup", () => {
  it("builds 12 unique Actors with the required Rule Role composition", () => {
    const seats = createRandomLineupSeats({
      ruleRoles: RULE_ROLES,
      actors: seedActors,
      random: () => 0,
    });
    expect(seats).toHaveLength(12);
    expect(new Set(seats.map((seat) => seat.actorId)).size).toBe(12);
    expect(ruleRoleCountMessages(seats)).toEqual([]);
    expect(validateLineupSeats(seats, RULE_ROLES, seedActors)).toEqual([]);
  });

  it("does not force Qin Chuan into an arbitrary lineup", () => {
    const seats = createRandomLineupSeats({
      ruleRoles: RULE_ROLES,
      actors: seedActors,
      random: () => 0,
    });
    expect(seats).toHaveLength(12);
    expect(seats.some((seat) => seat.actorId === "qin_chuan")).toBe(false);
  });
});
