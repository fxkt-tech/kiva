import { describe, expect, it } from "vitest";
import type { GamePreset } from "../game-preset";
import { createGameFromPreset, createSeedGame } from "../game";
import type { RoleDefinition } from "../role-definition";
import { createDefaultRuleset, type GameId } from "../types";
import { seedCharacters } from "@/seeds/characters";
import { seedPresets } from "@/seeds/presets";
import { seedRoles } from "@/seeds/roles";

const gameId = "game_creation" as GameId;
const createdAt = "2026-06-27T12:00:00.000Z";

describe("game creation", () => {
  it("creates immutable player snapshots from a preset seat assignment", () => {
    const game = createGameFromPreset({
      gameId,
      title: "Library game",
      createdAt,
      ruleset: createDefaultRuleset(),
      preset: seedPresets[0]!,
      roles: seedRoles,
      characters: seedCharacters,
    });

    expect(game).toMatchObject({
      id: gameId,
      title: "Library game",
      status: "drafting",
      createdAt,
      updatedAt: createdAt,
    });
    expect(game.players.map((player) => player.name)).toEqual([
      "秦川",
      "林夏",
      "周知",
      "许棠",
      "陈墨",
      "沈岚",
    ]);
    expect(game.players.map((player) => player.gameRole)).toEqual([
      "werewolf",
      "werewolf",
      "seer",
      "witch",
      "villager",
      "villager",
    ]);
    expect(game.players.map((player) => player.playerId)).toEqual([
      `${gameId}_p1`,
      `${gameId}_p2`,
      `${gameId}_p3`,
      `${gameId}_p4`,
      `${gameId}_p5`,
      `${gameId}_p6`,
    ]);

    expect(game.players[0]).toMatchObject({
      characterSourceId: "qin_chuan",
      roleSourceId: "werewolf",
      roleName: "狼人",
      roleSystemPromptSnapshot: seedRoles[0].systemPrompt,
      characterSystemPromptSnapshot: seedCharacters[0].systemPrompt,
      mechanicKey: "wolf_kill",
      team: "wolf",
    });
    expect(game.players[0]?.systemPrompt).toBe(seedCharacters[0].systemPrompt);
  });

  it("keeps createSeedGame compatible with the old seat, name, role, and playerId format", () => {
    const game = createSeedGame({ gameId, createdAt });

    expect(
      game.players.map((player) => ({
        playerId: player.playerId,
        seatNo: player.seatNo,
        name: player.name,
        gameRole: player.gameRole,
      })),
    ).toEqual([
      { playerId: `${gameId}_p1`, seatNo: 1, name: "秦川", gameRole: "werewolf" },
      { playerId: `${gameId}_p2`, seatNo: 2, name: "林夏", gameRole: "werewolf" },
      { playerId: `${gameId}_p3`, seatNo: 3, name: "周知", gameRole: "seer" },
      { playerId: `${gameId}_p4`, seatNo: 4, name: "许棠", gameRole: "witch" },
      { playerId: `${gameId}_p5`, seatNo: 5, name: "陈墨", gameRole: "villager" },
      { playerId: `${gameId}_p6`, seatNo: 6, name: "沈岚", gameRole: "villager" },
    ]);
  });

  it("rejects presets without explicit seat assignments", () => {
    const preset = { ...seedPresets[0]!, seatAssignments: null } satisfies GamePreset;

    expect(() =>
      createGameFromPreset({
        gameId,
        title: "Missing seats",
        createdAt,
        ruleset: createDefaultRuleset(),
        preset,
        roles: seedRoles,
        characters: seedCharacters,
      }),
    ).toThrow("Game preset six_player_standard must include seatAssignments");
  });

  it("rejects role definitions that are not supported by the current ruleset", () => {
    const unsupportedRole: RoleDefinition = {
      ...seedRoles[0],
      id: "hunter",
      name: "猎人",
    };
    const preset = {
      ...seedPresets[0]!,
      seatAssignments: seedPresets[0]!.seatAssignments!.map((seat, index) =>
        index === 0 ? { ...seat, roleId: "hunter" } : seat,
      ),
      roleIds: ["hunter", ...seedPresets[0]!.roleIds.slice(1)],
    } satisfies GamePreset;

    expect(() =>
      createGameFromPreset({
        gameId,
        title: "Unsupported role",
        createdAt,
        ruleset: createDefaultRuleset(),
        preset,
        roles: [unsupportedRole, ...seedRoles],
        characters: seedCharacters,
      }),
    ).toThrow("Role is not supported by current ruleset: hunter");
  });

  it("rejects supported role definitions whose contract fields are inconsistent", () => {
    const mismatchedRole: RoleDefinition = {
      ...seedRoles[0],
      id: "werewolf",
      faction: "good",
    };

    expect(() =>
      createGameFromPreset({
        gameId,
        title: "Mismatched role",
        createdAt,
        ruleset: createDefaultRuleset(),
        preset: seedPresets[0]!,
        roles: [mismatchedRole, ...seedRoles.slice(1)],
        characters: seedCharacters,
      }),
    ).toThrow("Role werewolf does not match the current ruleset contract");
  });

  it("rejects presets that reference missing role definitions", () => {
    const preset = {
      ...seedPresets[0]!,
      seatAssignments: seedPresets[0]!.seatAssignments!.map((seat, index) =>
        index === 0 ? { ...seat, roleId: "missing_role" } : seat,
      ),
      roleIds: ["missing_role", ...seedPresets[0]!.roleIds.slice(1)],
    } satisfies GamePreset;

    expect(() =>
      createGameFromPreset({
        gameId,
        title: "Missing role",
        createdAt,
        ruleset: createDefaultRuleset(),
        preset,
        roles: seedRoles,
        characters: seedCharacters,
      }),
    ).toThrow("references unknown role: missing_role");
  });

  it("rejects presets that reference missing character definitions", () => {
    const preset = {
      ...seedPresets[0]!,
      seatAssignments: seedPresets[0]!.seatAssignments!.map((seat, index) =>
        index === 0 ? { ...seat, characterId: "missing_character" } : seat,
      ),
      characterIds: [
        "missing_character",
        ...seedPresets[0]!.characterIds.slice(1),
      ],
    } satisfies GamePreset;

    expect(() =>
      createGameFromPreset({
        gameId,
        title: "Missing character",
        createdAt,
        ruleset: createDefaultRuleset(),
        preset,
        roles: seedRoles,
        characters: seedCharacters,
      }),
    ).toThrow("references unknown character: missing_character");
  });
});
