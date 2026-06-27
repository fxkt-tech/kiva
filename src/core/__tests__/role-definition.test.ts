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
    abilities: ["seer_check", "vote", "speak", "last_words"],
    nightOrder: 1,
    visibleTo: "self",
    rolePrompt: "你是预言家，每晚可以查验一名玩家的阵营。",
    actionPrompt: "请选择今晚要查验的玩家。",
    defaultModelBinding: null,
    metadata: {},
    ...overrides,
  };
}

describe("role definitions", () => {
  it("accepts a valid role collection", () => {
    expect(() =>
      validateRoleDefinitions([
        validRole(),
        validRole({
          id: "werewolf",
          name: "狼人",
          faction: "wolves",
          abilities: ["werewolf_kill", "vote", "speak", "last_words"],
          nightOrder: 0,
          visibleTo: "faction",
          rolePrompt: "你是狼人，每晚与狼队友共同选择袭击目标。",
          actionPrompt: "请选择今晚要袭击的玩家。",
        }),
        validRole({
          id: "villager",
          name: "平民",
          abilities: ["vote", "speak", "last_words"],
          nightOrder: null,
          visibleTo: "self",
          rolePrompt: "你是平民，没有夜间技能，需要通过发言和投票找出狼人。",
          actionPrompt: null,
        }),
      ]),
    ).not.toThrow();
  });

  it.each([
    ["id", { id: "  " }],
    ["name", { name: "" }],
    ["rolePrompt", { rolePrompt: "\t\n" }],
  ] satisfies Array<[string, Partial<RoleDefinition>]>)(
    "rejects a blank %s",
    (_field, overrides) => {
      expect(() => validateRoleDefinitions([validRole(overrides)])).toThrow();
    },
  );

  it("rejects duplicate ids", () => {
    expect(() =>
      validateRoleDefinitions([validRole(), validRole({ name: "另一个预言家" })]),
    ).toThrow();
  });

  it("rejects unknown abilities at runtime", () => {
    expect(() =>
      validateRoleDefinitions([
        validRole({
          abilities: ["seer_check", "mind_read"] as RoleDefinition["abilities"],
        }),
      ]),
    ).toThrow();
  });

  it.each([
    ["negative", -1],
    ["infinite", Infinity],
    ["NaN", NaN],
  ])("rejects %s nightOrder", (_case, nightOrder) => {
    expect(() =>
      validateRoleDefinitions([validRole({ nightOrder })]),
    ).toThrow();
  });
});
