import { describe, expect, it } from "vitest";
import { createSixPlayerRuleset, type PlayerId } from "../types";
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
    expect(first).toMatchObject({
      characterSourceId: null,
      roleSourceId: "werewolf",
      avatar: null,
      roleName: "狼人",
      team: "wolf",
      mechanicKey: "wolf_kill",
      characterSystemPromptSnapshot: "",
      roleSystemPromptSnapshot: "",
      roleActionPromptSnapshot: null,
    });
    expect(first.modelBindingSnapshot).toEqual({
      provider: "volcengine",
      model: "doubao-seed-1-6-flash-250828",
      temperature: 0.7,
      maxTokens: 1200,
      responseFormat: "json",
    });
    expect(first.modelBindingSnapshot).not.toBe(second.modelBindingSnapshot);
  });

  it("stores role library snapshot fields", () => {
    const snapshot = createPlayerSnapshot({
      playerId: "p1" as PlayerId,
      seatNo: 1,
      name: "P1",
      gameRole: "seer",
      characterSourceId: "character-seer",
      profileSourceId: "legacy-profile",
      roleSourceId: "custom-seer",
      avatar: "seer.png",
      persona: "careful",
      speakingStyle: "short",
      reasoningStyle: "deductive",
      characterSystemPromptSnapshot: "character prompt",
      roleSystemPromptSnapshot: "role prompt",
      roleActionPromptSnapshot: "action prompt",
      systemPrompt: "legacy prompt",
      roleName: "验人者",
      team: "god",
      mechanicKey: "seer_check",
    });

    expect(snapshot).toMatchObject({
      characterSourceId: "character-seer",
      profileSourceId: "legacy-profile",
      roleSourceId: "custom-seer",
      avatar: "seer.png",
      persona: "careful",
      speakingStyle: "short",
      reasoningStyle: "deductive",
      characterSystemPromptSnapshot: "character prompt",
      roleSystemPromptSnapshot: "role prompt",
      roleActionPromptSnapshot: "action prompt",
      systemPrompt: "legacy prompt",
      roleName: "验人者",
      team: "god",
      mechanicKey: "seer_check",
    });
  });

  it("fills character source from legacy profile source", () => {
    const snapshot = createPlayerSnapshot({
      playerId: "p1" as PlayerId,
      seatNo: 1,
      name: "P1",
      gameRole: "villager",
      profileSourceId: "profile-1",
    });

    expect(snapshot).toMatchObject({
      characterSourceId: "profile-1",
      profileSourceId: "profile-1",
    });
  });

  it("does not add a legacy profile source when only character source exists", () => {
    const snapshot = createPlayerSnapshot({
      playerId: "p1" as PlayerId,
      seatNo: 1,
      name: "P1",
      gameRole: "villager",
      characterSourceId: "character-1",
    });

    expect(snapshot.characterSourceId).toBe("character-1");
    expect(snapshot).not.toHaveProperty("profileSourceId");
  });

  it("defaults role metadata and prompt snapshots from legacy input", () => {
    const seer = createPlayerSnapshot({
      playerId: "p1" as PlayerId,
      seatNo: 1,
      name: "P1",
      gameRole: "seer",
      characterSystemPromptSnapshot: "character snapshot",
    });
    const witch = createPlayerSnapshot({
      playerId: "p2" as PlayerId,
      seatNo: 2,
      name: "P2",
      gameRole: "witch",
      systemPrompt: "legacy prompt",
    });
    const villager = createPlayerSnapshot({
      playerId: "p3" as PlayerId,
      seatNo: 3,
      name: "P3",
      gameRole: "villager",
    });

    expect(seer).toMatchObject({
      roleName: "预言家",
      team: "god",
      mechanicKey: "seer_check",
      avatar: null,
      characterSystemPromptSnapshot: "character snapshot",
      roleSystemPromptSnapshot: "",
      roleActionPromptSnapshot: null,
      systemPrompt: "character snapshot",
    });
    expect(witch).toMatchObject({
      roleName: "女巫",
      team: "god",
      mechanicKey: "witch_medicine",
      characterSystemPromptSnapshot: "legacy prompt",
      systemPrompt: "legacy prompt",
    });
    expect(villager).toMatchObject({
      roleName: "平民",
      team: "villager",
      mechanicKey: "none",
    });
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

  it("copies custom initial private knowledge into snapshots", () => {
    const initialPrivateKnowledge = ["own_role", "wolf_teammates"] as const;
    const snapshot = createPlayerSnapshot({
      playerId: "p1" as PlayerId,
      seatNo: 1,
      name: "P1",
      gameRole: "werewolf",
      initialPrivateKnowledge,
    });

    expect(snapshot.initialPrivateKnowledge).toEqual([
      "own_role",
      "wolf_teammates",
    ]);
    expect(snapshot.initialPrivateKnowledge).not.toBe(initialPrivateKnowledge);
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

    expect(validateSixPlayerBoard(players, createSixPlayerRuleset())).toEqual({
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

    expect(validateSixPlayerBoard(players, createSixPlayerRuleset())).toEqual({
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

    expect(validateSixPlayerBoard(players, createSixPlayerRuleset())).toEqual({
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

    expect(validateSixPlayerBoard(players, createSixPlayerRuleset())).toEqual({
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

    expect(validateSixPlayerBoard(players, createSixPlayerRuleset())).toEqual({
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

    expect(validateSixPlayerBoard(players, createSixPlayerRuleset())).toEqual({
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

    expect(validateSixPlayerBoard(players, createSixPlayerRuleset())).toEqual({
      ok: false,
      reason: "invalid_role_count",
      role: "werewolf",
      expected: 2,
      actual: 1,
    });
  });
});
