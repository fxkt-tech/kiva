import { describe, expect, it } from "vitest";
import {
  validateRoleDefinitions,
  type RoleDefinition,
} from "../role-definition";

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
    createdAt: "2026-06-27T00:00:00.000Z",
    updatedAt: "2026-06-27T00:00:00.000Z",
    ...overrides,
  };
}

describe("role definitions", () => {
  it("rejects a non-array collection", () => {
    expect(() => validateRoleDefinitions({})).toThrow(
      "Role definitions must be an array",
    );
  });

  it("accepts a valid role collection", () => {
    const roles = [
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
    ];

    expect(validateRoleDefinitions(roles)).toBe(roles);
  });

  it.each([
    ["id", { id: "  " }, "Role definition id must not be blank"],
    ["name", { name: "" }, "Role seer must include a name"],
    ["systemPrompt", { systemPrompt: "\t\n" }, "Role seer must include a systemPrompt"],
  ] satisfies Array<[string, Partial<RoleDefinition>, string]>)(
    "rejects a blank %s",
    (_field, overrides, message) => {
      expect(() => validateRoleDefinitions([validRole(overrides)])).toThrow(message);
    },
  );

  it("rejects role ids with surrounding whitespace", () => {
    expect(() => validateRoleDefinitions([validRole({ id: " seer" })])).toThrow(
      "Role definition id must not include leading or trailing whitespace",
    );
  });

  it.each([
    ["name", { name: 123 }, "Role seer must include a name"],
    ["systemPrompt", { systemPrompt: {} }, "Role seer must include a systemPrompt"],
    ["enabled", { enabled: "yes" }, "Role seer enabled must be a boolean"],
    ["createdAt", { createdAt: "not-a-date" }, "Role seer createdAt must be an ISO timestamp"],
    ["createdAt", { createdAt: "2026-06-27" }, "Role seer createdAt must be an ISO timestamp"],
    ["updatedAt", { updatedAt: "" }, "Role seer updatedAt must be an ISO timestamp"],
  ] satisfies Array<[string, Record<string, unknown>, string]>)(
    "rejects invalid %s field types",
    (_field, overrides, message) => {
      expect(() =>
        validateRoleDefinitions([
          {
            ...validRole(),
            ...overrides,
          },
        ]),
      ).toThrow(message);
    },
  );

  it("rejects duplicate ids", () => {
    expect(() =>
      validateRoleDefinitions([
        validRole(),
        validRole({ name: "另一个预言家" }),
      ]),
    ).toThrow("Duplicate role definition id: seer");
  });

  it.each([
    ["faction", { faction: "neutral" }, "Role seer has invalid faction: neutral"],
    ["team", { team: "third_party" }, "Role seer has invalid team: third_party"],
    [
      "mechanicKey",
      { mechanicKey: "guard_save" },
      "Role seer has invalid mechanicKey: guard_save",
    ],
    [
      "visibilityRules",
      { visibilityRules: ["hidden_partner"] },
      "Role seer has invalid visibility rule: hidden_partner",
    ],
  ] satisfies Array<[string, Record<string, unknown>, string]>)(
    "rejects invalid %s at runtime",
    (_field, overrides, message) => {
      expect(() =>
        validateRoleDefinitions([
          {
            ...validRole(),
            ...overrides,
          } as RoleDefinition,
        ]),
      ).toThrow(message);
    },
  );

  it.each([
    ["negative", -1],
    ["infinite", Infinity],
    ["NaN", NaN],
  ])("rejects %s nightOrder", (_case, nightOrder) => {
    expect(() =>
      validateRoleDefinitions([validRole({ nightOrder })]),
    ).toThrow("Role seer nightOrder must be null or a finite number >= 0");
  });

  it.each([
    [
      "werewolf faction",
      {
        id: "werewolf",
        name: "狼人",
        faction: "good",
        team: "wolf",
        mechanicKey: "wolf_kill",
      },
      "Role werewolf does not match the current ruleset contract",
    ],
    [
      "seer mechanic",
      { id: "seer", mechanicKey: "wolf_kill" },
      "Role seer does not match the current ruleset contract",
    ],
    [
      "villager team",
      {
        id: "villager",
        name: "平民",
        faction: "good",
        team: "god",
        mechanicKey: "none",
      },
      "Role villager does not match the current ruleset contract",
    ],
  ] satisfies Array<[string, Partial<RoleDefinition>, string]>)(
    "rejects %s mismatches for supported roles",
    (_case, overrides, message) => {
      expect(() => validateRoleDefinitions([validRole(overrides)])).toThrow(message);
    },
  );

  it.each([
    ["object", "invalid", "Role seer defaultModelBinding must be an object or null"],
    [
      "provider",
      {
        provider: "",
        model: "mock-model",
        responseFormat: "json",
      },
      "Role seer defaultModelBinding.provider must be set",
    ],
    [
      "provider whitespace",
      {
        provider: " mock",
        model: "mock-model",
        responseFormat: "json",
      },
      "Role seer defaultModelBinding.provider must be set",
    ],
    [
      "model",
      {
        provider: "mock",
        model: "",
        responseFormat: "json",
      },
      "Role seer defaultModelBinding.model must be set",
    ],
    [
      "model whitespace",
      {
        provider: "mock",
        model: "mock-model ",
        responseFormat: "json",
      },
      "Role seer defaultModelBinding.model must be set",
    ],
    [
      "responseFormat",
      {
        provider: "mock",
        model: "mock-model",
        responseFormat: "text",
      },
      "Role seer defaultModelBinding.responseFormat must be json",
    ],
    [
      "fallbackModel",
      {
        provider: "mock",
        model: "mock-model",
        responseFormat: "json",
        fallbackModel: "",
      },
      "Role seer defaultModelBinding.fallbackModel must be a non-empty string",
    ],
    [
      "fallbackModel whitespace",
      {
        provider: "mock",
        model: "mock-model",
        responseFormat: "json",
        fallbackModel: " mock-fallback",
      },
      "Role seer defaultModelBinding.fallbackModel must be a non-empty string",
    ],
  ] satisfies Array<[string, unknown, string]>)(
    "rejects invalid default model binding %s",
    (_field, defaultModelBinding, message) => {
      expect(() =>
        validateRoleDefinitions([
          {
            ...validRole(),
            defaultModelBinding,
          },
        ]),
      ).toThrow(message);
    },
  );

  it("allows disabled role definitions without prompts", () => {
    expect(
      validateRoleDefinitions([
        validRole({ enabled: false, systemPrompt: "" }),
      ]),
    ).toHaveLength(1);
  });
});
