import { describe, expect, it } from "vitest";
import { seedCharacters } from "@/seeds/characters";
import { seedRoles } from "@/seeds/roles";
import {
  createRandomSeatSetup,
  roleCountMessages,
  validateSeatSetup,
} from "./new-game-setup";

describe("new game setup helpers", () => {
  it("generates a twelve-player setup with fixed role counts and unique characters", () => {
    const seats = createRandomSeatSetup({
      roles: seedRoles,
      characters: seedCharacters,
      random: () => 0.42,
    });

    expect(seats).toHaveLength(12);
    expect(roleCounts(seats.map((seat) => seat.roleId))).toEqual({
      werewolf: 4,
      seer: 1,
      witch: 1,
      hunter: 1,
      guard: 1,
      villager: 4,
    });
    expect(new Set(seats.map((seat) => seat.characterId)).size).toBe(12);
    expect(validateSeatSetup(seats, seedRoles, seedCharacters)).toEqual([]);
  });

  it("rejects changed role counts", () => {
    const seats = createRandomSeatSetup({
      roles: seedRoles,
      characters: seedCharacters,
      random: () => 0,
    }).map((seat) =>
      seat.seatNo === 1 ? { ...seat, roleId: "seer" } : seat,
    );

    expect(roleCountMessages(seats)).toContain("狼人需要 4 个，当前 3 个。");
    expect(roleCountMessages(seats)).toContain("预言家需要 1 个，当前 2 个。");
  });

  it("rejects duplicate characters", () => {
    const seats = createRandomSeatSetup({
      roles: seedRoles,
      characters: seedCharacters,
      random: () => 0,
    });
    const duplicate = seats.map((seat) =>
      seat.seatNo === 2 ? { ...seat, characterId: seats[0]!.characterId } : seat,
    );

    expect(validateSeatSetup(duplicate, seedRoles, seedCharacters)).toContain(
      "玩家角色不能重复。",
    );
  });
});

function roleCounts(roleIds: readonly string[]) {
  return Object.fromEntries(
    ["werewolf", "seer", "witch", "hunter", "guard", "villager"].map((roleId) => [
      roleId,
      roleIds.filter((value) => value === roleId).length,
    ]),
  );
}
