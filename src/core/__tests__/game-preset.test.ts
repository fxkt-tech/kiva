import { describe, expect, it } from "vitest";
import type { CharacterDefinition } from "../character-definition";
import {
  validateGamePresets,
  type GamePreset,
  type GamePresetSeatAssignment,
} from "../game-preset";
import type { ModelBindingSnapshot } from "../player";
import type { RoleDefinition } from "../role-definition";
import { edgeVoiceProfile } from "../voice";

const timestamp = "2026-06-27T00:00:00.000Z";

function validRole(overrides: Partial<RoleDefinition> = {}): RoleDefinition {
  return {
    id: "seer",
    name: "预言家",
    faction: "good",
    team: "god",
    mechanicKey: "seer_check",
    systemPrompt: "你是预言家，每晚可以查验一名玩家的阵营。",
    actionPrompt: "请选择今晚要查验的玩家。",
    visibilityRules: ["own_role"],
    nightOrder: 20,
    defaultModelBinding: null,
    enabled: true,
    createdAt: timestamp,
    updatedAt: timestamp,
    ...overrides,
  };
}

function validCharacter(
  overrides: Partial<CharacterDefinition> = {},
): CharacterDefinition {
  return {
    id: "calm-analyst",
    name: "冷静分析师",
    avatar: null,
    tags: ["logical"],
    persona: "沉着、克制，优先基于事实判断局势。",
    speakingStyle: "短句为主，直接给出判断和理由。",
    reasoningStyle: "先列事实，再排除低概率解释。",
    systemPrompt: "你是一名冷静分析师，需要以稳定、理性的方式参与发言。",
    defaultModelBinding: null,
    voiceProfile: edgeVoiceProfile("zh-CN-XiaoxiaoNeural"),
    enabled: true,
    createdAt: timestamp,
    updatedAt: timestamp,
    ...overrides,
  };
}

function libraries(
  overrides: {
    roles?: readonly RoleDefinition[];
    characters?: readonly CharacterDefinition[];
  } = {},
): { roles: readonly RoleDefinition[]; characters: readonly CharacterDefinition[] } {
  return {
    roles: [
      validRole(),
      validRole({
        id: "werewolf",
        name: "狼人",
        faction: "wolves",
        team: "wolf",
        mechanicKey: "wolf_kill",
        systemPrompt: "你是狼人，每晚与狼队友共同选择袭击目标。",
        actionPrompt: "请选择今晚要袭击的玩家。",
        visibilityRules: ["own_role", "wolf_teammates"],
        nightOrder: 10,
      }),
      validRole({
        id: "villager",
        name: "平民",
        team: "villager",
        mechanicKey: "none",
        systemPrompt: "你是平民，没有夜间技能，需要通过发言和投票找出狼人。",
        actionPrompt: null,
        visibilityRules: ["own_role"],
        nightOrder: null,
      }),
    ],
    characters: [
      validCharacter(),
      validCharacter({
        id: "warm-mediator",
        name: "温和调停者",
        tags: ["warm"],
      }),
      validCharacter({
        id: "direct-skeptic",
        name: "直接怀疑者",
        tags: ["direct"],
      }),
    ],
    ...overrides,
  };
}

function validModelBinding(
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

function validSeatAssignment(
  overrides: Partial<GamePresetSeatAssignment> = {},
): GamePresetSeatAssignment {
  return {
    seatNo: 1,
    roleId: "seer",
    characterId: "calm-analyst",
    modelBindingOverride: null,
    ...overrides,
  };
}

function validPreset(overrides: Partial<GamePreset> = {}): GamePreset {
  return {
    id: "six-player-standard",
    name: "六人标准局",
    rulesetId: "classic-six",
    playerCount: 3,
    roleIds: ["werewolf", "seer", "villager"],
    characterIds: ["calm-analyst", "warm-mediator", "direct-skeptic"],
    seatAssignments: [
      validSeatAssignment({
        seatNo: 1,
        roleId: "werewolf",
        characterId: "calm-analyst",
      }),
      validSeatAssignment({
        seatNo: 2,
        roleId: "seer",
        characterId: "warm-mediator",
        modelBindingOverride: validModelBinding(),
      }),
      validSeatAssignment({
        seatNo: 3,
        roleId: "villager",
        characterId: "direct-skeptic",
      }),
    ],
    enabled: true,
    createdAt: timestamp,
    updatedAt: timestamp,
    ...overrides,
  };
}

describe("game presets", () => {
  it("accepts valid presets and returns the original array", () => {
    const presets = [
      validPreset(),
      validPreset({
        id: "six-player-random-seats",
        seatAssignments: null,
      }),
    ];

    expect(validateGamePresets(presets, libraries())).toBe(presets);
  });

  it("rejects a non-array collection", () => {
    expect(() => validateGamePresets({}, libraries())).toThrow(
      "Game presets must be an array",
    );
  });

  it("rejects non-object entries", () => {
    expect(() => validateGamePresets([null], libraries())).toThrow(
      "Game preset[0] must be an object",
    );
  });

  it("rejects array entries", () => {
    expect(() => validateGamePresets([[]], libraries())).toThrow(
      "Game preset[0] must be an object",
    );
  });

  it.each([
    ["blank id", { id: "" }, "Game preset[0] id must not be blank"],
    [
      "whitespace id",
      { id: " six-player-standard" },
      "Game preset[0] id must not include leading or trailing whitespace",
    ],
    ["blank name", { name: "" }, "Game preset six-player-standard must include a name"],
    [
      "blank rulesetId",
      { rulesetId: "  " },
      "Game preset six-player-standard must include a rulesetId",
    ],
    [
      "zero playerCount",
      { playerCount: 0 },
      "Game preset six-player-standard playerCount must be a positive integer",
    ],
    [
      "decimal playerCount",
      { playerCount: 3.5 },
      "Game preset six-player-standard playerCount must be a positive integer",
    ],
    [
      "enabled type",
      { enabled: "yes" },
      "Game preset six-player-standard enabled must be a boolean",
    ],
    [
      "createdAt",
      { createdAt: "2026-06-27" },
      "Game preset six-player-standard createdAt must be an ISO timestamp",
    ],
    [
      "updatedAt",
      { updatedAt: "" },
      "Game preset six-player-standard updatedAt must be an ISO timestamp",
    ],
  ] satisfies Array<[string, Record<string, unknown>, string]>)(
    "rejects invalid %s",
    (_case, overrides, message) => {
      expect(() =>
        validateGamePresets([{ ...validPreset(), ...overrides }], libraries()),
      ).toThrow(message);
    },
  );

  it("rejects duplicate ids", () => {
    expect(() =>
      validateGamePresets(
        [validPreset(), validPreset({ name: "另一套配置" })],
        libraries(),
      ),
    ).toThrow("Duplicate game preset id: six-player-standard");
  });

  it.each([
    [
      "roleIds type",
      { roleIds: "seer" },
      "Game preset six-player-standard roleIds must be an array",
    ],
    [
      "characterIds type",
      { characterIds: "calm-analyst" },
      "Game preset six-player-standard characterIds must be an array",
    ],
    [
      "roleIds length",
      { roleIds: ["werewolf", "seer"] },
      "Game preset six-player-standard roleIds length must equal playerCount",
    ],
    [
      "characterIds length",
      { characterIds: ["calm-analyst"] },
      "Game preset six-player-standard characterIds length must equal playerCount",
    ],
    [
      "blank role id",
      { roleIds: ["werewolf", "", "villager"] },
      "Game preset six-player-standard roleIds[1] must not be blank",
    ],
    [
      "trimmed character id",
      { characterIds: ["calm-analyst", "warm-mediator ", "direct-skeptic"] },
      "Game preset six-player-standard characterIds[1] must not include leading or trailing whitespace",
    ],
  ] satisfies Array<[string, Record<string, unknown>, string]>)(
    "rejects invalid %s",
    (_case, overrides, message) => {
      expect(() =>
        validateGamePresets([{ ...validPreset(), ...overrides }], libraries()),
      ).toThrow(message);
    },
  );

  it("rejects missing role references", () => {
    expect(() =>
      validateGamePresets(
        [validPreset({ roleIds: ["werewolf", "seer", "witch"] })],
        libraries(),
      ),
    ).toThrow("Game preset six-player-standard roleIds[2] references unknown role: witch");
  });

  it("rejects missing character references", () => {
    expect(() =>
      validateGamePresets(
        [
          validPreset({
            characterIds: ["calm-analyst", "warm-mediator", "quiet-observer"],
          }),
        ],
        libraries(),
      ),
    ).toThrow(
      "Game preset six-player-standard characterIds[2] references unknown character: quiet-observer",
    );
  });

  it("rejects disabled role references", () => {
    expect(() =>
      validateGamePresets(
        [validPreset()],
        libraries({
          roles: [
            validRole({ id: "werewolf" }),
            validRole(),
            validRole({ id: "villager", enabled: false }),
          ],
        }),
      ),
    ).toThrow(
      "Game preset six-player-standard roleIds[2] references disabled role: villager",
    );
  });

  it("rejects disabled character references", () => {
    expect(() =>
      validateGamePresets(
        [validPreset()],
        libraries({
          characters: [
            validCharacter(),
            validCharacter({ id: "warm-mediator" }),
            validCharacter({ id: "direct-skeptic", enabled: false }),
          ],
        }),
      ),
    ).toThrow(
      "Game preset six-player-standard characterIds[2] references disabled character: direct-skeptic",
    );
  });

  it.each([
    [
      "seatAssignments type",
      { seatAssignments: "seat-1" },
      "Game preset six-player-standard seatAssignments must be an array or null",
    ],
    [
      "length",
      { seatAssignments: [validSeatAssignment()] },
      "Game preset six-player-standard seatAssignments length must equal playerCount",
    ],
    [
      "non-object entry",
      { seatAssignments: [null, validSeatAssignment({ seatNo: 2 }), validSeatAssignment({ seatNo: 3 })] },
      "Game preset six-player-standard seatAssignments[0] must be an object",
    ],
    [
      "array entry",
      { seatAssignments: [[], validSeatAssignment({ seatNo: 2 }), validSeatAssignment({ seatNo: 3 })] },
      "Game preset six-player-standard seatAssignments[0] must be an object",
    ],
    [
      "duplicate seat",
      {
        seatAssignments: [
          validSeatAssignment({ seatNo: 1, roleId: "werewolf" }),
          validSeatAssignment({ seatNo: 1 }),
          validSeatAssignment({ seatNo: 3, roleId: "villager", characterId: "direct-skeptic" }),
        ],
      },
      "Game preset six-player-standard seatAssignments[1] duplicates seatNo: 1",
    ],
    [
      "out of range seat",
      {
        seatAssignments: [
          validSeatAssignment({ seatNo: 1, roleId: "werewolf" }),
          validSeatAssignment({ seatNo: 2 }),
          validSeatAssignment({ seatNo: 4, roleId: "villager", characterId: "direct-skeptic" }),
        ],
      },
      "Game preset six-player-standard seatAssignments[2] seatNo must be an integer from 1 to playerCount",
    ],
  ] satisfies Array<[string, Record<string, unknown>, string]>)(
    "rejects invalid %s",
    (_case, overrides, message) => {
      expect(() =>
        validateGamePresets([{ ...validPreset(), ...overrides }], libraries()),
      ).toThrow(message);
    },
  );

  it("rejects seat assignment role multisets that do not match preset roleIds", () => {
    expect(() =>
      validateGamePresets(
        [
          validPreset({
            roleIds: ["werewolf", "villager", "villager"],
            seatAssignments: [
              validSeatAssignment({ seatNo: 1, roleId: "werewolf" }),
              validSeatAssignment({ seatNo: 2, roleId: "seer", characterId: "warm-mediator" }),
              validSeatAssignment({
                seatNo: 3,
                roleId: "villager",
                characterId: "direct-skeptic",
              }),
            ],
          }),
        ],
        libraries(),
      ),
    ).toThrow(
      "Game preset six-player-standard seatAssignments roleId multiset must match roleIds",
    );
  });

  it("rejects seat assignment character multisets that do not match preset characterIds", () => {
    expect(() =>
      validateGamePresets(
        [
          validPreset({
            characterIds: ["calm-analyst", "warm-mediator", "warm-mediator"],
            seatAssignments: [
              validSeatAssignment({ seatNo: 1, roleId: "werewolf" }),
              validSeatAssignment({ seatNo: 2, roleId: "seer", characterId: "warm-mediator" }),
              validSeatAssignment({
                seatNo: 3,
                roleId: "villager",
                characterId: "direct-skeptic",
              }),
            ],
          }),
        ],
        libraries(),
      ),
    ).toThrow(
      "Game preset six-player-standard seatAssignments characterId multiset must match characterIds",
    );
  });

  it("rejects seat assignments that reference unknown role entries", () => {
    expect(() =>
      validateGamePresets(
        [
          validPreset({
            roleIds: ["werewolf", "seer", "villager"],
            seatAssignments: [
              validSeatAssignment({ seatNo: 1, roleId: "werewolf" }),
              validSeatAssignment({ seatNo: 2, roleId: "witch", characterId: "warm-mediator" }),
              validSeatAssignment({
                seatNo: 3,
                roleId: "villager",
                characterId: "direct-skeptic",
              }),
            ],
          }),
        ],
        libraries(),
      ),
    ).toThrow(
      "Game preset six-player-standard seatAssignments[1] roleId references unknown role: witch",
    );
  });

  it("rejects seat assignments that reference disabled role entries", () => {
    expect(() =>
      validateGamePresets(
        [
          validPreset({
            roleIds: ["werewolf", "seer", "villager"],
            characterIds: ["calm-analyst", "warm-mediator", "direct-skeptic"],
            seatAssignments: [
              validSeatAssignment({ seatNo: 1, roleId: "werewolf" }),
              validSeatAssignment({
                seatNo: 2,
                roleId: "villager-disabled",
                characterId: "warm-mediator",
              }),
              validSeatAssignment({
                seatNo: 3,
                roleId: "seer",
                characterId: "direct-skeptic",
              }),
            ],
          }),
        ],
        libraries({
          roles: [
            validRole({ id: "werewolf" }),
            validRole(),
            validRole({ id: "villager-disabled", enabled: false }),
            validRole({ id: "villager" }),
          ],
        }),
      ),
    ).toThrow(
      "Game preset six-player-standard seatAssignments[1] roleId references disabled role: villager-disabled",
    );
  });

  it("rejects seat assignments that reference disabled character entries", () => {
    expect(() =>
      validateGamePresets(
        [
          validPreset({
            characterIds: ["calm-analyst", "warm-mediator", "direct-skeptic"],
            seatAssignments: [
              validSeatAssignment({ seatNo: 1, roleId: "werewolf" }),
              validSeatAssignment({
                seatNo: 2,
                roleId: "seer",
                characterId: "disabled-character",
              }),
              validSeatAssignment({
                seatNo: 3,
                roleId: "villager",
                characterId: "direct-skeptic",
              }),
            ],
          }),
        ],
        libraries({
          characters: [
            validCharacter(),
            validCharacter({ id: "warm-mediator" }),
            validCharacter({ id: "direct-skeptic" }),
            validCharacter({ id: "disabled-character", enabled: false }),
          ],
        }),
      ),
    ).toThrow(
      "Game preset six-player-standard seatAssignments[1] characterId references disabled character: disabled-character",
    );
  });

  it("rejects seat assignments that reference unknown character entries", () => {
    expect(() =>
      validateGamePresets(
        [
          validPreset({
            characterIds: ["calm-analyst", "warm-mediator", "direct-skeptic"],
            seatAssignments: [
              validSeatAssignment({ seatNo: 1, roleId: "werewolf" }),
              validSeatAssignment({
                seatNo: 2,
                roleId: "seer",
                characterId: "quiet-observer",
              }),
              validSeatAssignment({
                seatNo: 3,
                roleId: "villager",
                characterId: "direct-skeptic",
              }),
            ],
          }),
        ],
        libraries(),
      ),
    ).toThrow(
      "Game preset six-player-standard seatAssignments[1] characterId references unknown character: quiet-observer",
    );
  });

  it("rejects invalid modelBindingOverride", () => {
    expect(() =>
      validateGamePresets(
        [
          validPreset({
            seatAssignments: [
              validSeatAssignment({ seatNo: 1, roleId: "werewolf" }),
              validSeatAssignment({
                seatNo: 2,
                roleId: "seer",
                characterId: "warm-mediator",
                modelBindingOverride: validModelBinding({ maxTokens: 0 }),
              }),
              validSeatAssignment({
                seatNo: 3,
                roleId: "villager",
                characterId: "direct-skeptic",
              }),
            ],
          }),
        ],
        libraries(),
      ),
    ).toThrow(
      "Game preset six-player-standard seatAssignments[1] modelBindingOverride.maxTokens must be a positive integer",
    );
  });

  it("rejects array modelBindingOverride values", () => {
    expect(() =>
      validateGamePresets(
        [
          validPreset({
            seatAssignments: [
              validSeatAssignment({ seatNo: 1, roleId: "werewolf" }),
              {
                ...validSeatAssignment({
                  seatNo: 2,
                  roleId: "seer",
                  characterId: "warm-mediator",
                }),
                modelBindingOverride: [],
              } as unknown as GamePresetSeatAssignment,
              validSeatAssignment({
                seatNo: 3,
                roleId: "villager",
                characterId: "direct-skeptic",
              }),
            ],
          }),
        ],
        libraries(),
      ),
    ).toThrow(
      "Game preset six-player-standard seatAssignments[1] modelBindingOverride must be an object or null",
    );
  });
});
