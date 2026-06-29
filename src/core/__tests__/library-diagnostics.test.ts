import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import type { CharacterDefinition } from "../character-definition";
import type { GamePreset } from "../game-preset";
import {
  diagnoseCharacter,
  diagnosePreset,
  diagnoseRole,
  promptPreviewForPresetSeat,
} from "../library-diagnostics";
import type { ModelBindingSnapshot } from "../player";
import type { RoleDefinition } from "../role-definition";

const timestamp = "2026-06-27T00:00:00.000Z";

const werewolf = role({
  id: "werewolf",
  name: "狼人",
  faction: "wolves",
  team: "wolf",
  mechanicKey: "wolf_kill",
  systemPrompt: "你是狼人。",
  actionPrompt: "选择袭击目标。",
});
const seer = role({
  id: "seer",
  name: "预言家",
  faction: "good",
  team: "god",
  mechanicKey: "seer_check",
  systemPrompt: "你是预言家。",
  actionPrompt: "选择查验目标。",
});
const witch = role({
  id: "witch",
  name: "女巫",
  faction: "good",
  team: "god",
  mechanicKey: "witch_medicine",
  systemPrompt: "你是女巫。",
  actionPrompt: "选择是否用药。",
});
const villager = role({
  id: "villager",
  name: "平民",
  faction: "good",
  team: "villager",
  mechanicKey: "none",
  systemPrompt: "你是平民。",
  actionPrompt: null,
});
const qin = character({ id: "qin", name: "秦川", systemPrompt: "你是秦川。" });
const lin = character({ id: "lin", name: "林夏", systemPrompt: "你是林夏。" });
const zhou = character({ id: "zhou", name: "周知", systemPrompt: "你是周知。" });
const xu = character({ id: "xu", name: "夏宇", systemPrompt: "你是夏宇。" });
const chen = character({ id: "chen", name: "陈墨", systemPrompt: "你是陈墨。" });
const shen = character({ id: "shen", name: "顾清妍", systemPrompt: "你是顾清妍。" });

describe("library diagnostics", () => {
  it("reports role references, built-in contract status, and role prompt preview", () => {
    const preset = presetWithSeats({
      roles: ["werewolf", "seer"],
      characters: ["qin", "lin"],
    });

    expect(
      diagnoseRole({
        role: werewolf,
        roles: [werewolf, seer],
        characters: [qin, lin],
        presets: [preset],
      }),
    ).toMatchObject({
      valid: true,
      references: ["two_player_test"],
      messages: expect.arrayContaining(["Built-in role contract locked"]),
      promptPreview: expect.stringContaining("你是狼人。"),
    });
  });

  it("marks role diagnostics invalid when built-in contract fields mismatch", () => {
    expect(
      diagnoseRole({
        role: { ...werewolf, faction: "good" },
        roles: [werewolf, seer],
        characters: [qin, lin],
        presets: [],
      }),
    ).toMatchObject({
      valid: false,
      messages: expect.arrayContaining([
        "Role werewolf does not match the current ruleset contract",
      ]),
    });
  });

  it("marks role diagnostics invalid when the role collection has duplicate ids", () => {
    const duplicateWerewolf = role({
      ...werewolf,
      name: "另一个狼人",
      createdAt: "2026-06-27T00:01:00.000Z",
      updatedAt: "2026-06-27T00:01:00.000Z",
    });

    expect(
      diagnoseRole({
        role: werewolf,
        roles: [werewolf, duplicateWerewolf],
        characters: [qin, lin],
        presets: [],
      }),
    ).toMatchObject({
      valid: false,
      messages: expect.arrayContaining([
        "Duplicate role definition id: werewolf",
      ]),
    });
  });

  it("reports character references and builds prompt preview without role fields", () => {
    const preset = presetWithSeats({
      roles: ["werewolf", "seer"],
      characters: ["qin", "lin"],
    });

    const diagnostic = diagnoseCharacter({
      character: qin,
      roles: [werewolf, seer],
      characters: [qin, lin],
      presets: [preset],
    });

    expect(diagnostic).toMatchObject({
      valid: true,
      references: ["two_player_test"],
      promptPreview: expect.stringContaining("你是秦川。"),
    });
    expect(diagnostic.promptPreview).not.toContain("你是狼人。");
    expect(diagnostic.promptPreview).not.toContain("选择袭击目标。");
  });

  it("labels character prompt preview sections", () => {
    const diagnostic = diagnoseCharacter({
      character: qin,
      roles: [werewolf, seer],
      characters: [qin, lin],
      presets: [],
    });

    expect(diagnostic.promptPreview).toBe(
      [
        "角色基础提示：",
        "你是秦川。",
        "",
        "人设：",
        "冷静",
        "",
        "发言风格：",
        "短句",
        "",
        "推理风格：",
        "证据优先",
      ].join("\n"),
    );
  });

  it("marks character diagnostics invalid when enabled character prompt is blank", () => {
    expect(
      diagnoseCharacter({
        character: { ...qin, systemPrompt: "" },
        roles: [werewolf, seer],
        characters: [qin, lin],
        presets: [],
      }),
    ).toMatchObject({
      valid: false,
      messages: expect.arrayContaining([
        "Character qin must include a systemPrompt",
      ]),
    });
  });

  it("marks character diagnostics invalid when the character collection has duplicate ids", () => {
    const duplicateQin = character({
      ...qin,
      name: "另一个秦川",
      createdAt: "2026-06-27T00:01:00.000Z",
      updatedAt: "2026-06-27T00:01:00.000Z",
    });

    expect(
      diagnoseCharacter({
        character: qin,
        roles: [werewolf, seer],
        characters: [qin, duplicateQin],
        presets: [],
      }),
    ).toMatchObject({
      valid: false,
      messages: expect.arrayContaining([
        "Duplicate character definition id: qin",
      ]),
    });
  });

  it("reports disabled role and character diagnostics", () => {
    expect(
      diagnoseRole({
        role: { ...werewolf, enabled: false },
        roles: [werewolf, seer],
        characters: [qin, lin],
        presets: [],
      }),
    ).toMatchObject({
      valid: false,
      messages: expect.arrayContaining(["Role is disabled"]),
    });

    expect(
      diagnoseCharacter({
        character: { ...qin, enabled: false },
        roles: [werewolf, seer],
        characters: [qin, lin],
        presets: [],
      }),
    ).toMatchObject({
      valid: false,
      messages: ["Character is disabled"],
    });
  });

  it("marks complete presets valid and creatable", () => {
    const preset = sixPlayerPreset();

    expect(
      diagnosePreset({
        preset,
        roles: [werewolf, seer, witch, villager],
        characters: [qin, lin, zhou, xu, chen, shen],
        presets: [preset],
      }),
    ).toMatchObject({
      valid: true,
      canCreateGame: true,
      messages: [],
    });
  });

  it("marks disabled presets invalid while preserving structural create-game readiness", () => {
    const preset = sixPlayerPreset({ enabled: false });

    expect(
      diagnosePreset({
        preset,
        roles: [werewolf, seer, witch, villager],
        characters: [qin, lin, zhou, xu, chen, shen],
        presets: [preset],
      }),
    ).toMatchObject({
      valid: false,
      canCreateGame: true,
      messages: expect.arrayContaining(["Preset is disabled"]),
    });
  });

  it("keeps presets without seat assignments valid but not creatable", () => {
    const preset = { ...sixPlayerPreset(), seatAssignments: null };

    expect(
      diagnosePreset({
        preset,
        roles: [werewolf, seer, witch, villager],
        characters: [qin, lin, zhou, xu, chen, shen],
        presets: [preset],
      }),
    ).toMatchObject({
      valid: true,
      canCreateGame: false,
      messages: expect.arrayContaining([
        "Game preset six_player_test must include seatAssignments",
      ]),
    });
  });

  it("marks preset diagnostics invalid when the preset collection has duplicate ids", () => {
    const preset = sixPlayerPreset();
    const duplicatePreset = {
      ...sixPlayerPreset({ name: "Duplicate six player test" }),
    };

    expect(
      diagnosePreset({
        preset,
        roles: [werewolf, seer, witch, villager],
        characters: [qin, lin, zhou, xu, chen, shen],
        presets: [preset, duplicatePreset],
      }),
    ).toMatchObject({
      valid: false,
      messages: expect.arrayContaining([
        "Duplicate game preset id: six_player_test",
      ]),
    });
  });

  it("keeps presets valid but not creatable when create game rejects an otherwise valid role reference", () => {
    const hunter = role({
      id: "hunter",
      name: "猎人",
      faction: "good",
      team: "villager",
      mechanicKey: "none",
      systemPrompt: "你是猎人。",
      actionPrompt: null,
    });
    const preset = sixPlayerPreset({
      roleIds: ["hunter", "werewolf", "seer", "witch", "villager", "villager"],
      seatAssignments: [
        seatAssignment({ seatNo: 1, roleId: "hunter", characterId: "qin" }),
        seatAssignment({ seatNo: 2, roleId: "werewolf", characterId: "lin" }),
        seatAssignment({ seatNo: 3, roleId: "seer", characterId: "zhou" }),
        seatAssignment({ seatNo: 4, roleId: "witch", characterId: "xu" }),
        seatAssignment({ seatNo: 5, roleId: "villager", characterId: "chen" }),
        seatAssignment({ seatNo: 6, roleId: "villager", characterId: "shen" }),
      ],
    });

    expect(
      diagnosePreset({
        preset,
        roles: [hunter, werewolf, seer, witch, villager],
        characters: [qin, lin, zhou, xu, chen, shen],
        presets: [preset],
      }),
    ).toMatchObject({
      valid: true,
      canCreateGame: false,
      messages: expect.arrayContaining([
        "Role is not supported by current ruleset: hunter",
      ]),
    });
  });

  it("marks presets invalid and not creatable when referenced characters are structurally invalid", () => {
    const invalidQin = { ...qin, systemPrompt: "" };
    const preset = sixPlayerPreset();

    expect(
      diagnosePreset({
        preset,
        roles: [werewolf, seer, witch, villager],
        characters: [invalidQin, lin, zhou, xu, chen, shen],
        presets: [preset],
      }),
    ).toMatchObject({
      valid: false,
      canCreateGame: false,
      messages: expect.arrayContaining([
        "Character qin must include a systemPrompt",
      ]),
    });
  });

  it("marks presets invalid and not creatable when they reference unknown roles", () => {
    const preset = presetWithSeats({
      roles: ["werewolf", "missing_role"],
      characters: ["qin", "lin"],
    });

    expect(
      diagnosePreset({
        preset,
        roles: [werewolf, seer],
        characters: [qin, lin],
        presets: [preset],
      }),
    ).toMatchObject({
      valid: false,
      canCreateGame: false,
      messages: expect.arrayContaining([
        "Game preset two_player_test roleIds[1] references unknown role: missing_role",
      ]),
    });
  });

  it("marks selected presets invalid even when they are absent from the preset collection", () => {
    const preset = presetWithSeats({
      roles: ["werewolf", "missing_role"],
      characters: ["qin", "lin"],
    });

    expect(
      diagnosePreset({
        preset,
        roles: [werewolf, seer],
        characters: [qin, lin],
        presets: [],
      }),
    ).toMatchObject({
      valid: false,
      messages: expect.arrayContaining([
        "Game preset two_player_test roleIds[1] references unknown role: missing_role",
      ]),
    });
  });

  it("reports multiple independent preset reference errors together", () => {
    const preset = presetWithSeats({
      roles: ["werewolf", "missing_role"],
      characters: ["qin", "missing_character"],
    });

    expect(
      diagnosePreset({
        preset,
        roles: [werewolf, seer],
        characters: [qin, lin],
        presets: [preset],
      }),
    ).toMatchObject({
      valid: false,
      canCreateGame: false,
      messages: expect.arrayContaining([
        "Game preset two_player_test roleIds[1] references unknown role: missing_role",
        "Game preset two_player_test characterIds[1] references unknown character: missing_character",
      ]),
    });
  });

  it("returns diagnostics instead of throwing for malformed preset reference lists", () => {
    const preset = {
      ...presetWithSeats({
        roles: ["werewolf", "seer"],
        characters: ["qin", "lin"],
      }),
      roleIds: "werewolf",
    } as unknown as GamePreset;

    expect(() =>
      diagnosePreset({
        preset,
        roles: [werewolf, seer],
        characters: [qin, lin],
        presets: [preset],
      }),
    ).not.toThrow();
    expect(
      diagnosePreset({
        preset,
        roles: [werewolf, seer],
        characters: [qin, lin],
        presets: [preset],
      }),
    ).toMatchObject({
      valid: false,
      canCreateGame: false,
      messages: expect.arrayContaining([
        "Game preset two_player_test roleIds must be an array",
      ]),
    });
  });

  it("returns diagnostics instead of throwing for malformed seat assignments", () => {
    const preset = {
      ...presetWithSeats({
        roles: ["werewolf", "seer"],
        characters: ["qin", "lin"],
      }),
      seatAssignments: 42,
    } as unknown as GamePreset;

    expect(() =>
      diagnosePreset({
        preset,
        roles: [werewolf, seer],
        characters: [qin, lin],
        presets: [preset],
      }),
    ).not.toThrow();
    expect(
      diagnosePreset({
        preset,
        roles: [werewolf, seer],
        characters: [qin, lin],
        presets: [preset],
      }),
    ).toMatchObject({
      valid: false,
      canCreateGame: false,
      messages: expect.arrayContaining([
        "Game preset two_player_test seatAssignments must be an array or null",
      ]),
    });
  });

  it("marks presets invalid and not creatable when they reference unknown characters", () => {
    const preset = presetWithSeats({
      roles: ["werewolf", "seer"],
      characters: ["qin", "missing_character"],
    });

    expect(
      diagnosePreset({
        preset,
        roles: [werewolf, seer],
        characters: [qin, lin],
        presets: [preset],
      }),
    ).toMatchObject({
      valid: false,
      canCreateGame: false,
      messages: expect.arrayContaining([
        "Game preset two_player_test characterIds[1] references unknown character: missing_character",
      ]),
    });
  });

  it("marks presets invalid and not creatable when they reference disabled roles", () => {
    const disabledSeer = { ...seer, enabled: false };
    const preset = presetWithSeats({
      roles: ["werewolf", "seer"],
      characters: ["qin", "lin"],
    });

    expect(
      diagnosePreset({
        preset,
        roles: [werewolf, disabledSeer],
        characters: [qin, lin],
        presets: [preset],
      }),
    ).toMatchObject({
      valid: false,
      canCreateGame: false,
      messages: expect.arrayContaining([
        "Game preset two_player_test roleIds[1] references disabled role: seer",
      ]),
    });
  });

  it("marks presets invalid and not creatable when they reference disabled characters", () => {
    const disabledLin = { ...lin, enabled: false };
    const preset = presetWithSeats({
      roles: ["werewolf", "seer"],
      characters: ["qin", "lin"],
    });

    expect(
      diagnosePreset({
        preset,
        roles: [werewolf, seer],
        characters: [qin, disabledLin],
        presets: [preset],
      }),
    ).toMatchObject({
      valid: false,
      canCreateGame: false,
      messages: expect.arrayContaining([
        "Game preset two_player_test characterIds[1] references disabled character: lin",
      ]),
    });
  });

  it.each([
    [
      "seat assignment length mismatch",
      {
        seatAssignments: [
          seatAssignment({
            seatNo: 1,
            roleId: "werewolf",
            characterId: "qin",
          }),
        ],
      },
      "Game preset two_player_test seatAssignments length must equal playerCount",
    ],
    [
      "duplicate seat number",
      {
        seatAssignments: [
          seatAssignment({
            seatNo: 1,
            roleId: "werewolf",
            characterId: "qin",
          }),
          seatAssignment({
            seatNo: 1,
            roleId: "seer",
            characterId: "lin",
          }),
        ],
      },
      "Game preset two_player_test seatAssignments[1] duplicates seatNo: 1",
    ],
    [
      "role multiset mismatch",
      {
        seatAssignments: [
          seatAssignment({
            seatNo: 1,
            roleId: "werewolf",
            characterId: "qin",
          }),
          seatAssignment({
            seatNo: 2,
            roleId: "werewolf",
            characterId: "lin",
          }),
        ],
      },
      "Game preset two_player_test seatAssignments roleId multiset must match roleIds",
    ],
    [
      "character multiset mismatch",
      {
        seatAssignments: [
          seatAssignment({
            seatNo: 1,
            roleId: "werewolf",
            characterId: "qin",
          }),
          seatAssignment({
            seatNo: 2,
            roleId: "seer",
            characterId: "qin",
          }),
        ],
      },
      "Game preset two_player_test seatAssignments characterId multiset must match characterIds",
    ],
  ] satisfies Array<[string, Partial<GamePreset>, string]>)(
    "marks presets invalid and not creatable for %s",
    (_case, overrides, message) => {
      const preset = {
        ...presetWithSeats({
          roles: ["werewolf", "seer"],
          characters: ["qin", "lin"],
        }),
        ...overrides,
      };

      expect(
        diagnosePreset({
          preset,
          roles: [werewolf, seer],
          characters: [qin, lin],
          presets: [preset],
        }),
      ).toMatchObject({
        valid: false,
        canCreateGame: false,
        messages: expect.arrayContaining([message]),
      });
    },
  );

  it("builds preset seat prompt preview from character, role, action, and resolved model label", () => {
    const preset = presetWithSeats({
      roles: ["werewolf", "seer"],
      characters: ["qin", "lin"],
    });

    expect(
      promptPreviewForPresetSeat({
        preset,
        seatNo: 1,
        roles: [werewolf, seer],
        characters: [qin, lin],
      }),
    ).toContain("你是秦川。\n你是狼人。\n选择袭击目标。\nmodel: mock/mock-model");
  });

  it("resolves seat model with override before character, role, then core default", () => {
    const roleDefault = modelBinding({ provider: "role", model: "default" });
    const characterDefault = modelBinding({
      provider: "character",
      model: "default",
    });
    const seatOverride = modelBinding({ provider: "seat", model: "override" });
    const preset = presetWithSeats({
      roles: ["werewolf", "seer"],
      characters: ["qin", "lin"],
      seatOverrides: [seatOverride, null],
    });

    const overridePreview = promptPreviewForPresetSeat({
      preset,
      seatNo: 1,
      roles: [role({ ...werewolf, defaultModelBinding: roleDefault }), seer],
      characters: [character({ ...qin, defaultModelBinding: characterDefault }), lin],
    });
    expect(overridePreview).toContain("model: seat/override");
    expect(overridePreview).not.toContain("model: character/default");

    expect(
      promptPreviewForPresetSeat({
        preset: presetWithSeats({ roles: ["werewolf", "seer"], characters: ["qin", "lin"] }),
        seatNo: 1,
        roles: [role({ ...werewolf, defaultModelBinding: roleDefault }), seer],
        characters: [character({ ...qin, defaultModelBinding: characterDefault }), lin],
      }),
    ).toContain("model: character/default");

    expect(
      promptPreviewForPresetSeat({
        preset: presetWithSeats({ roles: ["werewolf", "seer"], characters: ["qin", "lin"] }),
        seatNo: 1,
        roles: [role({ ...werewolf, defaultModelBinding: roleDefault }), seer],
        characters: [character({ ...qin, defaultModelBinding: null }), lin],
      }),
    ).toContain("model: role/default");

    expect(
      promptPreviewForPresetSeat({
        preset: presetWithSeats({ roles: ["werewolf", "seer"], characters: ["qin", "lin"] }),
        seatNo: 1,
        roles: [role({ ...werewolf, defaultModelBinding: null }), seer],
        characters: [character({ ...qin, defaultModelBinding: null }), lin],
      }),
    ).toContain("model: volcengine/doubao-seed-1-6-flash-250828");
  });

  it("keeps diagnostics independent from game seed helpers", () => {
    const source = readFileSync(
      "src/core/library-diagnostics.ts",
      "utf8",
    );

    expect(source).not.toContain('from "./game"');
  });
});

function role(overrides: Partial<RoleDefinition> = {}): RoleDefinition {
  return {
    id: "werewolf",
    name: "狼人",
    faction: "wolves",
    team: "wolf",
    mechanicKey: "wolf_kill",
    systemPrompt: "role prompt",
    actionPrompt: null,
    visibilityRules: ["own_role"],
    nightOrder: null,
    defaultModelBinding: {
      provider: "mock",
      model: "mock-model",
      temperature: 0.7,
      maxTokens: 1000,
      responseFormat: "json",
    },
    enabled: true,
    createdAt: timestamp,
    updatedAt: timestamp,
    ...overrides,
  };
}

function character(
  overrides: Partial<CharacterDefinition> = {},
): CharacterDefinition {
  return {
    id: "qin",
    name: "秦川",
    avatar: null,
    tags: [],
    persona: "冷静",
    speakingStyle: "短句",
    reasoningStyle: "证据优先",
    systemPrompt: "character prompt",
    defaultModelBinding: null,
    enabled: true,
    createdAt: timestamp,
    updatedAt: timestamp,
    ...overrides,
  };
}

function presetWithSeats(input: {
  roles: readonly string[];
  characters: readonly string[];
  seatOverrides?: readonly (ModelBindingSnapshot | null)[];
}): GamePreset {
  return {
    id: "two_player_test",
    name: "Two player test",
    rulesetId: "test_ruleset",
    playerCount: 2,
    roleIds: input.roles,
    characterIds: input.characters,
    seatAssignments: input.roles.map((roleId, index) => ({
      seatNo: index + 1,
      roleId,
      characterId: input.characters[index]!,
      modelBindingOverride: input.seatOverrides?.[index] ?? null,
    })),
    enabled: true,
    createdAt: timestamp,
    updatedAt: timestamp,
  };
}

function sixPlayerPreset(overrides: Partial<GamePreset> = {}): GamePreset {
  return {
    id: "six_player_test",
    name: "Six player test",
    rulesetId: "test_ruleset",
    playerCount: 6,
    roleIds: ["werewolf", "werewolf", "seer", "witch", "villager", "villager"],
    characterIds: ["qin", "lin", "zhou", "xu", "chen", "shen"],
    seatAssignments: [
      seatAssignment({ seatNo: 1, roleId: "werewolf", characterId: "qin" }),
      seatAssignment({ seatNo: 2, roleId: "werewolf", characterId: "lin" }),
      seatAssignment({ seatNo: 3, roleId: "seer", characterId: "zhou" }),
      seatAssignment({ seatNo: 4, roleId: "witch", characterId: "xu" }),
      seatAssignment({ seatNo: 5, roleId: "villager", characterId: "chen" }),
      seatAssignment({ seatNo: 6, roleId: "villager", characterId: "shen" }),
    ],
    enabled: true,
    createdAt: timestamp,
    updatedAt: timestamp,
    ...overrides,
  };
}

function seatAssignment(
  overrides: Omit<
    NonNullable<GamePreset["seatAssignments"]>[number],
    "modelBindingOverride"
  > &
    Partial<
      Pick<
        NonNullable<GamePreset["seatAssignments"]>[number],
        "modelBindingOverride"
      >
    >,
): NonNullable<GamePreset["seatAssignments"]>[number] {
  return {
    modelBindingOverride: null,
    ...overrides,
  };
}

function modelBinding(
  overrides: Partial<ModelBindingSnapshot> = {},
): ModelBindingSnapshot {
  return {
    provider: "mock",
    model: "mock-model",
    temperature: 0.7,
    maxTokens: 1000,
    responseFormat: "json",
    ...overrides,
  };
}
