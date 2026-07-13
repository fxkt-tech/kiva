import { describe, expect, it } from "vitest";
import {
  createDefaultRuleset,
  type GameRole,
  type PlayerId,
} from "../types";
import {
  createPlayerSnapshot,
  validateBoard,
  validatePlayerSnapshot,
  type PlayerSnapshot,
} from "../player";

function player(id: string, seatNo: number, gameRole: GameRole) {
  return createPlayerSnapshot({
    playerId: id as PlayerId,
    seatNo,
    name: `P${seatNo}`,
    gameRole,
  });
}

function currentPlayer(): PlayerSnapshot {
  return createPlayerSnapshot({
    playerId: "p1" as PlayerId,
    seatNo: 1,
    name: "林夏",
    gameRole: "seer",
    characterSourceId: "character-linxia",
    roleSourceId: "seer",
    avatar: "linxia.png",
    persona: "谨慎而敏锐",
    speakingStyle: "简短直接",
    reasoningStyle: "先核验事实再下结论",
    characterSystemPromptSnapshot: "保持人物性格一致。",
    roleSystemPromptSnapshot: "你是预言家。",
    roleActionPromptSnapshot: "选择查验目标。",
    roleName: "预言家",
    team: "god",
    mechanicKey: "seer_check",
  });
}

function currentBoard(): PlayerSnapshot[] {
  const roles: GameRole[] = [
    "werewolf",
    "werewolf",
    "werewolf",
    "werewolf",
    "seer",
    "witch",
    "hunter",
    "guard",
    "villager",
    "villager",
    "villager",
    "villager",
  ];
  return roles.map((role, index) => player(`p${index + 1}`, index + 1, role));
}

describe("player snapshots", () => {
  it("derives role-owned fields for newly created snapshots", () => {
    expect(player("p1", 1, "werewolf")).toMatchObject({
      faction: "wolves",
      roleSourceId: "werewolf",
      roleName: "狼人",
      team: "wolf",
      mechanicKey: "wolf_kill",
      initialPrivateKnowledge: ["own_role", "wolf_teammates"],
    });
    expect(player("p2", 2, "witch").initialPrivateKnowledge).toEqual([
      "own_role",
      "witch_medicines",
    ]);
  });

  it("copies mutable snapshot inputs", () => {
    const modelBinding = {
      provider: "test-provider",
      model: "test-model",
      responseFormat: "json" as const,
    };
    const privateKnowledge = ["own_role", "wolf_teammates"] as const;
    const snapshot = createPlayerSnapshot({
      playerId: "p1" as PlayerId,
      seatNo: 1,
      name: "P1",
      gameRole: "werewolf",
      modelBindingSnapshot: modelBinding,
      initialPrivateKnowledge: privateKnowledge,
    });

    modelBinding.provider = "mutated-provider";
    expect(snapshot.modelBindingSnapshot.provider).toBe("test-provider");
    expect(snapshot.initialPrivateKnowledge).toEqual(privateKnowledge);
    expect(snapshot.initialPrivateKnowledge).not.toBe(privateKnowledge);
  });

  it("accepts only a complete current persisted snapshot", () => {
    const snapshot = currentPlayer();

    expect(validatePlayerSnapshot(snapshot)).toEqual(snapshot);
    expect(() =>
      validatePlayerSnapshot({
        ...snapshot,
        characterSourceId: undefined,
        profileSourceId: "legacy-profile",
      }),
    ).toThrow("keys are invalid");
    expect(() =>
      validatePlayerSnapshot({ ...snapshot, systemPrompt: "legacy prompt" }),
    ).toThrow("keys are invalid");
  });

  it("accepts the current twelve-player board", () => {
    expect(validateBoard(currentBoard(), createDefaultRuleset())).toEqual({
      ok: true,
    });
  });

  it("rejects invalid player count", () => {
    expect(validateBoard(currentBoard().slice(0, 11), createDefaultRuleset())).toEqual({
      ok: false,
      reason: "invalid_player_count",
      expected: 12,
      actual: 11,
    });
  });

  it("rejects duplicate player IDs", () => {
    const players = currentBoard();
    players[1] = { ...players[1]!, playerId: players[0]!.playerId };

    expect(validateBoard(players, createDefaultRuleset())).toEqual({
      ok: false,
      reason: "duplicate_player_id",
    });
  });

  it("rejects invalid and duplicate seats", () => {
    const invalid = currentBoard();
    invalid[0] = { ...invalid[0]!, seatNo: 0 };
    expect(validateBoard(invalid, createDefaultRuleset())).toEqual({
      ok: false,
      reason: "invalid_seat",
      seatNo: 0,
    });

    const duplicate = currentBoard();
    duplicate[1] = { ...duplicate[1]!, seatNo: duplicate[0]!.seatNo };
    expect(validateBoard(duplicate, createDefaultRuleset())).toEqual({
      ok: false,
      reason: "duplicate_seat",
    });
  });

  it("rejects wrong role counts", () => {
    const players = currentBoard();
    players[0] = { ...players[0]!, gameRole: "villager" };

    expect(validateBoard(players, createDefaultRuleset())).toEqual({
      ok: false,
      reason: "invalid_role_count",
      role: "werewolf",
      expected: 4,
      actual: 3,
    });
  });
});
