import { describe, expect, test } from "vitest";

import { validateCharacterDefinitions } from "../../core/character-definition";
import { validateGamePresets } from "../../core/game-preset";
import { validateRoleDefinitions } from "../../core/role-definition";
import { seedCharacters } from "../characters";
import { seedPresets } from "../presets";
import { seedRoles } from "../roles";

const HARD_CODED_SEATS = [
  { seatNo: 1, roleId: "villager", characterId: "zhou_zhi" },
  { seatNo: 2, roleId: "seer", characterId: "chen_mo" },
  { seatNo: 3, roleId: "werewolf", characterId: "qin_chuan" },
  { seatNo: 4, roleId: "witch", characterId: "lin_xia" },
  { seatNo: 5, roleId: "villager", characterId: "xia_yu" },
  { seatNo: 6, roleId: "werewolf", characterId: "gu_qingyan" },
  { seatNo: 7, roleId: "guard", characterId: "shen_lan" },
  { seatNo: 8, roleId: "hunter", characterId: "xu_yan" },
  { seatNo: 9, roleId: "werewolf", characterId: "bai_qi" },
  { seatNo: 10, roleId: "villager", characterId: "tang_tang" },
  { seatNo: 11, roleId: "werewolf", characterId: "lu_zhao" },
  { seatNo: 12, roleId: "villager", characterId: "su_jin" },
] as const;

const CHARACTER_NAMES = [
  "秦川",
  "林夏",
  "周知",
  "夏宇",
  "陈墨",
  "顾清妍",
  "沈岚",
  "许砚",
  "白祁",
  "唐棠",
  "陆昭",
  "苏瑾",
];
const ROLE_ONLY_FIELDS = [
  "role",
  "faction",
  "team",
  "mechanicKey",
  "visibilityRules",
  "nightOrder",
  "rolePrompt",
] as const;

describe("seed library", () => {
  test("validates seed roles, characters, and presets", () => {
    expect(validateRoleDefinitions(seedRoles)).toBe(seedRoles);
    expect(validateCharacterDefinitions(seedCharacters)).toBe(seedCharacters);
    expect(
      validateGamePresets(seedPresets, {
        roles: seedRoles,
        characters: seedCharacters,
      }),
    ).toBe(seedPresets);
  });

  test("keeps the default preset aligned with the current hardcoded twelve-player seats", () => {
    const preset = seedPresets.find((item) => item.id === "twelve_player_standard");

    expect(preset).toBeDefined();
    expect(preset?.seatAssignments).toEqual(
      HARD_CODED_SEATS.map((assignment) => ({
        ...assignment,
        modelBindingOverride: null,
      })),
    );
    expect(preset?.characterIds).toEqual(
      HARD_CODED_SEATS.map((assignment) => assignment.characterId),
    );
  });

  test("keeps role ids as a multiset and character ids unique", () => {
    const preset = seedPresets.find((item) => item.id === "twelve_player_standard");
    const roleCounts = new Map<string, number>();

    for (const roleId of preset?.roleIds ?? []) {
      roleCounts.set(roleId, (roleCounts.get(roleId) ?? 0) + 1);
    }

    expect(Object.fromEntries(roleCounts)).toEqual({
      werewolf: 4,
      seer: 1,
      witch: 1,
      hunter: 1,
      guard: 1,
      villager: 4,
    });
    expect(new Set(preset?.characterIds).size).toBe(preset?.characterIds.length);
  });

  test("keeps role prompts separate from character identities", () => {
    for (const role of seedRoles) {
      const prompt = `${role.systemPrompt}\n${role.actionPrompt ?? ""}`;

      for (const characterName of CHARACTER_NAMES) {
        expect(prompt).not.toContain(characterName);
      }
    }
  });

  test("keeps character definitions free of role-only fields", () => {
    for (const character of seedCharacters) {
      for (const field of ROLE_ONLY_FIELDS) {
        expect(character).not.toHaveProperty(field);
      }
    }
  });
});
