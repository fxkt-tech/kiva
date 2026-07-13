import { describe, expect, it } from "vitest";
import type { GamePreset } from "../game-preset";
import { createGameFromPreset, createSeedGame } from "../game";
import type { RoleDefinition } from "../role-definition";
import { createDefaultRuleset, type GameId } from "../types";
import { seedCharacters } from "@/seeds/characters";
import { seedPresets } from "@/seeds/presets";
import { seedPresenters } from "@/seeds/presenters";
import { seedScripts } from "@/seeds/scripts";
import { seedRoles } from "@/seeds/roles";

const gameId = "game_creation" as GameId;
const createdAt = "2026-06-27T12:00:00.000Z";

describe("game creation", () => {
  it("snapshots the selected run mode", () => {
    const scripted = createGameFromPreset({
      gameId,
      title: "Scripted game",
      createdAt,
      ruleset: createDefaultRuleset(),
      preset: seedPresets[0]!,
      presenter: seedPresenters[0]!,
      script: seedScripts[0]!,
      roles: seedRoles,
      characters: seedCharacters,
      runMode: "scripted",
    });

    expect(scripted.runMode).toBe("scripted");
  });
  it("creates immutable player snapshots from a preset seat assignment", () => {
    const game = createGameFromPreset({
      gameId,
      title: "Library game",
      createdAt,
      ruleset: createDefaultRuleset(),
      preset: seedPresets[0]!,
      presenter: seedPresenters[0]!,
      script: seedScripts[0]!,
      runMode: "game",
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
      "周序",
      "乔可",
      "秦川",
      "夏弥",
      "任野",
      "顾绫",
      "程雾",
      "叶忱",
      "迟木",
      "唐梨",
      "陆燃",
      "苏弦",
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
      characterSourceId: "zhou_xu",
      roleSourceId: "villager",
      roleName: "平民",
      roleSystemPromptSnapshot: seedRoles.find((role) => role.id === "villager")!
        .systemPrompt,
      characterSystemPromptSnapshot: seedCharacters.find(
        (character) => character.id === "zhou_xu",
      )!.systemPrompt,
      mechanicKey: "none",
      team: "villager",
    });
    expect(game.players[0]?.modelBindingSnapshot).toEqual(
      seedCharacters.find((character) => character.id === "zhou_xu")!
        .defaultModelBinding,
    );
  });

  it("creates the seeded game with the current seat and identity format", () => {
    const game = createSeedGame({ gameId, createdAt });

    expect(
      game.players.map((player) => ({
        playerId: player.playerId,
        seatNo: player.seatNo,
        name: player.name,
        gameRole: player.gameRole,
      })),
    ).toEqual([
      { playerId: "p1", seatNo: 1, name: "周序", gameRole: "villager" },
      { playerId: "p2", seatNo: 2, name: "乔可", gameRole: "seer" },
      { playerId: "p3", seatNo: 3, name: "秦川", gameRole: "werewolf" },
      { playerId: "p4", seatNo: 4, name: "夏弥", gameRole: "witch" },
      { playerId: "p5", seatNo: 5, name: "任野", gameRole: "villager" },
      { playerId: "p6", seatNo: 6, name: "顾绫", gameRole: "werewolf" },
      { playerId: "p7", seatNo: 7, name: "程雾", gameRole: "guard" },
      { playerId: "p8", seatNo: 8, name: "叶忱", gameRole: "hunter" },
      { playerId: "p9", seatNo: 9, name: "迟木", gameRole: "werewolf" },
      { playerId: "p10", seatNo: 10, name: "唐梨", gameRole: "villager" },
      { playerId: "p11", seatNo: 11, name: "陆燃", gameRole: "werewolf" },
      { playerId: "p12", seatNo: 12, name: "苏弦", gameRole: "villager" },
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
        script: seedScripts[0]!,
        runMode: "game",
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
        script: seedScripts[0]!,
        runMode: "game",
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
        script: seedScripts[0]!,
        runMode: "game",
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
        script: seedScripts[0]!,
        runMode: "game",
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
        script: seedScripts[0]!,
        runMode: "game",
        roles: seedRoles,
        characters: seedCharacters,
      }),
    ).toThrow("references unknown character: missing_character");
  });
});
