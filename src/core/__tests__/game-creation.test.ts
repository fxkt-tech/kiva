import { describe, expect, it } from "vitest";
import type { GamePreset } from "../game-preset";
import { createGameFromPreset, createSeedGame } from "../game";
import type { RoleDefinition } from "../role-definition";
import { createDefaultRuleset, type GameId } from "../types";
import { seedCharacters } from "@/seeds/characters";
import { seedPresets } from "@/seeds/presets";
import { seedPresenters } from "@/seeds/presenters";
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
      presenter: seedPresenters[0]!,

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
      "周知",
      "陈墨",
      "秦川",
      "林夏",
      "夏宇",
      "顾清妍",
      "沈岚",
      "许砚",
      "白祁",
      "唐棠",
      "陆昭",
      "苏瑾",
    ]);
    expect(game.players.map((player) => player.gameRole)).toEqual([
      "villager",
      "seer",
      "werewolf",
      "witch",
      "villager",
      "werewolf",
      "guard",
      "hunter",
      "werewolf",
      "villager",
      "werewolf",
      "villager",
    ]);
    expect(game.players.map((player) => player.playerId)).toEqual([
      "p1",
      "p2",
      "p3",
      "p4",
      "p5",
      "p6",
      "p7",
      "p8",
      "p9",
      "p10",
      "p11",
      "p12",
    ]);

    expect(game.players[0]).toMatchObject({
      characterSourceId: "zhou_zhi",
      roleSourceId: "villager",
      roleName: "平民",
      roleSystemPromptSnapshot: seedRoles.find((role) => role.id === "villager")!
        .systemPrompt,
      characterSystemPromptSnapshot: seedCharacters.find(
        (character) => character.id === "zhou_zhi",
      )!.systemPrompt,
      mechanicKey: "none",
      team: "villager",
    });
    expect(game.players[0]?.systemPrompt).toBe(
      seedCharacters.find((character) => character.id === "zhou_zhi")!
        .systemPrompt,
    );
  });

  it("keeps createSeedGame compatible with the seat, name, role, and playerId format", () => {
    const game = createSeedGame({ gameId, createdAt });

    expect(
      game.players.map((player) => ({
        playerId: player.playerId,
        seatNo: player.seatNo,
        name: player.name,
        gameRole: player.gameRole,
      })),
    ).toEqual([
      { playerId: "p1", seatNo: 1, name: "周知", gameRole: "villager" },
      { playerId: "p2", seatNo: 2, name: "陈墨", gameRole: "seer" },
      { playerId: "p3", seatNo: 3, name: "秦川", gameRole: "werewolf" },
      { playerId: "p4", seatNo: 4, name: "林夏", gameRole: "witch" },
      { playerId: "p5", seatNo: 5, name: "夏宇", gameRole: "villager" },
      { playerId: "p6", seatNo: 6, name: "顾清妍", gameRole: "werewolf" },
      { playerId: "p7", seatNo: 7, name: "沈岚", gameRole: "guard" },
      { playerId: "p8", seatNo: 8, name: "许砚", gameRole: "hunter" },
      { playerId: "p9", seatNo: 9, name: "白祁", gameRole: "werewolf" },
      { playerId: "p10", seatNo: 10, name: "唐棠", gameRole: "villager" },
      { playerId: "p11", seatNo: 11, name: "陆昭", gameRole: "werewolf" },
      { playerId: "p12", seatNo: 12, name: "苏瑾", gameRole: "villager" },
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
        presenter: seedPresenters[0]!,

        roles: seedRoles,
        characters: seedCharacters,
      }),
    ).toThrow("Game preset twelve_player_standard must include seatAssignments");
  });

  it("rejects role definitions that are not supported by the current ruleset", () => {
    const unsupportedRole: RoleDefinition = {
      ...seedRoles[0],
      id: "idiot",
      name: "白痴",
    };
    const preset = {
      ...seedPresets[0]!,
      seatAssignments: seedPresets[0]!.seatAssignments!.map((seat, index) =>
        index === 0 ? { ...seat, roleId: "idiot" } : seat,
      ),
      roleIds: ["idiot", ...seedPresets[0]!.roleIds.slice(1)],
    } satisfies GamePreset;

    expect(() =>
      createGameFromPreset({
        gameId,
        title: "Unsupported role",
        createdAt,
        ruleset: createDefaultRuleset(),
        preset,
        presenter: seedPresenters[0]!,

        roles: [unsupportedRole, ...seedRoles],
        characters: seedCharacters,
      }),
    ).toThrow("Role is not supported by current ruleset: idiot");
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
        presenter: seedPresenters[0]!,

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
        presenter: seedPresenters[0]!,

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
        presenter: seedPresenters[0]!,

        roles: seedRoles,
        characters: seedCharacters,
      }),
    ).toThrow("references unknown character: missing_character");
  });
});
