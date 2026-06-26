import { describe, expect, it } from "vitest";
import { createDefaultRuleset, type PlayerId } from "../types";
import {
  createPlayerSnapshot,
  validateSixPlayerBoard,
  type PlayerSnapshot,
} from "../player";

function player(id: string, seatNo: number, role: PlayerSnapshot["gameRole"]) {
  return createPlayerSnapshot({
    playerId: id as PlayerId,
    seatNo,
    name: `P${seatNo}`,
    gameRole: role,
  });
}

describe("player snapshots", () => {
  it("assigns faction from role", () => {
    expect(player("p1", 1, "werewolf").faction).toBe("wolves");
    expect(player("p2", 2, "seer").faction).toBe("good");
  });

  it("accepts the approved fixed six player board", () => {
    const players = [
      player("p1", 1, "werewolf"),
      player("p2", 2, "werewolf"),
      player("p3", 3, "seer"),
      player("p4", 4, "witch"),
      player("p5", 5, "villager"),
      player("p6", 6, "villager"),
    ];

    expect(validateSixPlayerBoard(players, createDefaultRuleset())).toEqual({
      ok: true,
    });
  });

  it("rejects duplicate seats", () => {
    const players = [
      player("p1", 1, "werewolf"),
      player("p2", 1, "werewolf"),
      player("p3", 3, "seer"),
      player("p4", 4, "witch"),
      player("p5", 5, "villager"),
      player("p6", 6, "villager"),
    ];

    expect(validateSixPlayerBoard(players, createDefaultRuleset())).toEqual({
      ok: false,
      reason: "duplicate_seat",
    });
  });

  it("rejects wrong role counts", () => {
    const players = [
      player("p1", 1, "werewolf"),
      player("p2", 2, "villager"),
      player("p3", 3, "seer"),
      player("p4", 4, "witch"),
      player("p5", 5, "villager"),
      player("p6", 6, "villager"),
    ];

    expect(validateSixPlayerBoard(players, createDefaultRuleset())).toEqual({
      ok: false,
      reason: "invalid_role_count",
      role: "werewolf",
      expected: 2,
      actual: 1,
    });
  });
});
