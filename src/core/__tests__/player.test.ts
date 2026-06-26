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

  it("assigns private knowledge from role", () => {
    expect(player("p1", 1, "werewolf").initialPrivateKnowledge).toEqual([
      "own_role",
      "wolf_teammates",
    ]);
    expect(player("p2", 2, "seer").initialPrivateKnowledge).toEqual([
      "own_role",
    ]);
    expect(player("p3", 3, "witch").initialPrivateKnowledge).toEqual([
      "own_role",
      "witch_medicines",
    ]);
    expect(player("p4", 4, "villager").initialPrivateKnowledge).toEqual([
      "own_role",
    ]);
  });

  it("uses default empty profile text fields and independent model bindings", () => {
    const first = player("p1", 1, "werewolf");
    const second = player("p2", 2, "seer");

    expect(first.persona).toBe("");
    expect(first.speakingStyle).toBe("");
    expect(first.reasoningStyle).toBe("");
    expect(first.systemPrompt).toBe("");
    expect(first.modelBindingSnapshot).toEqual({
      provider: "mock",
      model: "mock-role-model",
      temperature: 0.7,
      maxTokens: 1200,
      responseFormat: "json",
    });
    expect(first.modelBindingSnapshot).not.toBe(second.modelBindingSnapshot);
  });

  it("copies custom model bindings into snapshots", () => {
    const modelBinding = {
      provider: "test-provider",
      model: "test-model",
      temperature: 0.2,
      maxTokens: 800,
      responseFormat: "json" as const,
    };

    const snapshot = createPlayerSnapshot({
      playerId: "p1" as PlayerId,
      seatNo: 1,
      name: "P1",
      gameRole: "werewolf",
      modelBindingSnapshot: modelBinding,
    });

    modelBinding.provider = "mutated-provider";
    modelBinding.model = "mutated-model";
    modelBinding.temperature = 1;
    modelBinding.maxTokens = 10;

    expect(snapshot.modelBindingSnapshot).toEqual({
      provider: "test-provider",
      model: "test-model",
      temperature: 0.2,
      maxTokens: 800,
      responseFormat: "json",
    });
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

  it("rejects invalid player count", () => {
    const players = [
      player("p1", 1, "werewolf"),
      player("p2", 2, "werewolf"),
      player("p3", 3, "seer"),
      player("p4", 4, "witch"),
      player("p5", 5, "villager"),
    ];

    expect(validateSixPlayerBoard(players, createDefaultRuleset())).toEqual({
      ok: false,
      reason: "invalid_player_count",
      expected: 6,
      actual: 5,
    });
  });

  it("rejects duplicate player IDs", () => {
    const players = [
      player("p1", 1, "werewolf"),
      player("p1", 2, "werewolf"),
      player("p3", 3, "seer"),
      player("p4", 4, "witch"),
      player("p5", 5, "villager"),
      player("p6", 6, "villager"),
    ];

    expect(validateSixPlayerBoard(players, createDefaultRuleset())).toEqual({
      ok: false,
      reason: "duplicate_player_id",
    });
  });

  it("rejects seats outside the board range before checking duplicates", () => {
    const players = [
      player("p1", 0, "werewolf"),
      player("p2", 0, "werewolf"),
      player("p3", 3, "seer"),
      player("p4", 4, "witch"),
      player("p5", 5, "villager"),
      player("p6", 6, "villager"),
    ];

    expect(validateSixPlayerBoard(players, createDefaultRuleset())).toEqual({
      ok: false,
      reason: "invalid_seat",
      seatNo: 0,
    });
  });

  it("rejects non-integer seats", () => {
    const players = [
      player("p1", 1.5, "werewolf"),
      player("p2", 2, "werewolf"),
      player("p3", 3, "seer"),
      player("p4", 4, "witch"),
      player("p5", 5, "villager"),
      player("p6", 6, "villager"),
    ];

    expect(validateSixPlayerBoard(players, createDefaultRuleset())).toEqual({
      ok: false,
      reason: "invalid_seat",
      seatNo: 1.5,
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
