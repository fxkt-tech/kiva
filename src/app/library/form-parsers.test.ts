import { describe, expect, it } from "vitest";
import type { ModelBindingSnapshot } from "@/core/player";
import {
  characterFromFormData,
  presetFromFormData,
  roleFromFormData,
} from "./form-parsers";

const timestamp = "2026-06-27T00:00:00.000Z";

const modelBinding = {
  provider: "mock",
  model: "mock-model",
  temperature: 0.4,
  maxTokens: 800,
  responseFormat: "json",
} satisfies ModelBindingSnapshot;

describe("library form parsers", () => {
  it("parses a role form", () => {
    const form = new FormData();
    form.set("id", "werewolf");
    form.set("name", "狼人");
    form.set("enabled", "on");
    form.set("faction", "wolves");
    form.set("team", "wolf");
    form.set("mechanicKey", "wolf_kill");
    form.set("visibilityRules", "own_role,wolf_teammates");
    form.set("nightOrder", "10");
    form.set("systemPrompt", "你是狼人。");
    form.set("actionPrompt", "选择袭击目标。");

    expect(roleFromFormData(form, timestamp)).toMatchObject({
      id: "werewolf",
      enabled: true,
      visibilityRules: ["own_role", "wolf_teammates"],
      nightOrder: 10,
      actionPrompt: "选择袭击目标。",
      defaultModelBinding: null,
    });
  });

  it("parses a character form", () => {
    const form = new FormData();
    form.set("id", "qin");
    form.set("name", "秦川");
    form.set("enabled", "on");
    form.set("tags", "冷静,强势");
    form.set("persona", "冷静");
    form.set("speakingStyle", "短句");
    form.set("reasoningStyle", "证据优先");
    form.set("systemPrompt", "你是秦川。");

    expect(characterFromFormData(form, timestamp)).toMatchObject({
      id: "qin",
      tags: ["冷静", "强势"],
      avatar: null,
      enabled: true,
      defaultModelBinding: null,
    });
  });

  it("parses a preset seat table", () => {
    const form = new FormData();
    form.set("id", "six_player_standard");
    form.set("name", "6人狼人杀试运行");
    form.set("rulesetId", "six_player_v1");
    form.set("playerCount", "2");
    form.set("enabled", "on");
    form.set("seat.1.roleId", "werewolf");
    form.set("seat.1.characterId", "qin");
    form.set("seat.2.roleId", "seer");
    form.set("seat.2.characterId", "lin");

    expect(presetFromFormData(form, timestamp)).toMatchObject({
      roleIds: ["werewolf", "seer"],
      characterIds: ["qin", "lin"],
      seatAssignments: [
        {
          seatNo: 1,
          roleId: "werewolf",
          characterId: "qin",
          modelBindingOverride: null,
        },
        {
          seatNo: 2,
          roleId: "seer",
          characterId: "lin",
          modelBindingOverride: null,
        },
      ],
    });
  });

  it("parses missing enabled checkboxes as false", () => {
    expect(roleFromFormData(roleForm({ enabled: false }), timestamp).enabled).toBe(
      false,
    );
    expect(
      characterFromFormData(characterForm({ enabled: false }), timestamp).enabled,
    ).toBe(false);
    expect(presetFromFormData(presetForm({ enabled: false }), timestamp).enabled).toBe(
      false,
    );
  });

  it("preserves role and character default model bindings from hidden JSON", () => {
    const roleFormData = roleForm();
    roleFormData.set("defaultModelBinding", JSON.stringify(modelBinding));
    const characterFormData = characterForm();
    characterFormData.set("defaultModelBinding", JSON.stringify(modelBinding));

    expect(roleFromFormData(roleFormData, timestamp).defaultModelBinding).toEqual(
      modelBinding,
    );
    expect(
      characterFromFormData(characterFormData, timestamp).defaultModelBinding,
    ).toEqual(modelBinding);
  });

  it("preserves preset seat model binding overrides from hidden JSON", () => {
    const form = presetForm();
    form.set("seat.2.modelBindingOverride", JSON.stringify(modelBinding));

    expect(presetFromFormData(form, timestamp).seatAssignments).toEqual([
      {
        seatNo: 1,
        roleId: "werewolf",
        characterId: "qin",
        modelBindingOverride: null,
      },
      {
        seatNo: 2,
        roleId: "seer",
        characterId: "lin",
        modelBindingOverride: modelBinding,
      },
    ]);
  });

  it("throws the original invalid JSON parse message", () => {
    const form = roleForm();
    form.set("defaultModelBinding", "{");

    expect(() => roleFromFormData(form, timestamp)).toThrow(
      /Expected property name|Unexpected end|JSON/,
    );
  });
});

function roleForm(options: { readonly enabled?: boolean } = {}): FormData {
  const form = new FormData();
  form.set("id", "werewolf");
  form.set("name", "狼人");
  if (options.enabled !== false) {
    form.set("enabled", "on");
  }
  form.set("faction", "wolves");
  form.set("team", "wolf");
  form.set("mechanicKey", "wolf_kill");
  form.set("visibilityRules", "own_role,wolf_teammates");
  form.set("nightOrder", "");
  form.set("systemPrompt", "你是狼人。");
  form.set("actionPrompt", "");
  return form;
}

function characterForm(options: { readonly enabled?: boolean } = {}): FormData {
  const form = new FormData();
  form.set("id", "qin");
  form.set("name", "秦川");
  if (options.enabled !== false) {
    form.set("enabled", "on");
  }
  form.set("avatar", "");
  form.set("tags", "冷静,强势");
  form.set("persona", "冷静");
  form.set("speakingStyle", "短句");
  form.set("reasoningStyle", "证据优先");
  form.set("systemPrompt", "你是秦川。");
  return form;
}

function presetForm(options: { readonly enabled?: boolean } = {}): FormData {
  const form = new FormData();
  form.set("id", "six_player_standard");
  form.set("name", "6人狼人杀试运行");
  form.set("rulesetId", "six_player_v1");
  form.set("playerCount", "2");
  if (options.enabled !== false) {
    form.set("enabled", "on");
  }
  form.set("seat.1.roleId", "werewolf");
  form.set("seat.1.characterId", "qin");
  form.set("seat.2.roleId", "seer");
  form.set("seat.2.characterId", "lin");
  return form;
}
