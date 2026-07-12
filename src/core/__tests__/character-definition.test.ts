import { describe, expect, it } from "vitest";
import {
  validateCharacterDefinitions,
  type CharacterDefinition,
} from "../character-definition";
import { edgeVoiceProfile } from "../voice";

function validCharacter(
  overrides: Partial<CharacterDefinition> = {},
): CharacterDefinition {
  return {
    id: "calm-analyst",
    name: "冷静分析师",
    avatar: "https://example.com/avatar.png",
    tags: ["logical", "concise"],
    persona: "沉着、克制，优先基于事实判断局势。",
    speakingStyle: "短句为主，直接给出判断和理由。",
    reasoningStyle: "先列事实，再排除低概率解释。",
    systemPrompt: "你是一名冷静分析师，需要以稳定、理性的方式参与发言。",
    defaultModelBinding: null,
    voiceProfile: edgeVoiceProfile("zh-CN-XiaoxiaoNeural"),
    enabled: true,
    createdAt: "2026-06-27T00:00:00.000Z",
    updatedAt: "2026-06-27T00:00:00.000Z",
    ...overrides,
  };
}

describe("character definitions", () => {
  it("accepts a valid character collection and returns the original array", () => {
    const characters = [
      validCharacter(),
      validCharacter({
        id: "warm-mediator",
        name: "温和调停者",
        avatar: null,
        tags: ["warm", "social"],
        persona: "友善、愿意倾听，擅长缓和对抗。",
        speakingStyle: "语气温和，经常确认他人的观点。",
        reasoningStyle: "从关系和发言动机推断风险。",
        systemPrompt: "你是一名温和调停者，需要推动讨论但避免过早下结论。",
        defaultModelBinding: {
          provider: "mock",
          model: "mock-model",
          temperature: 0.7,
          maxTokens: 1000,
          responseFormat: "json",
          fallbackModel: "mock-fallback",
        },
      }),
    ];

    expect(validateCharacterDefinitions(characters)).toBe(characters);
  });

  it("rejects a non-array collection", () => {
    expect(() => validateCharacterDefinitions({})).toThrow(
      "Character definitions must be an array",
    );
  });

  it("rejects non-object entries", () => {
    expect(() => validateCharacterDefinitions([null])).toThrow(
      "Character definition[0] must be an object",
    );
  });

  it.each([
    ["blank id", { id: "" }, "Character definition[0] id must not be blank"],
    [
      "whitespace id",
      { id: " calm-analyst" },
      "Character definition[0] id must not include leading or trailing whitespace",
    ],
    ["blank name", { name: "" }, "Character calm-analyst must include a name"],
  ] satisfies Array<[string, Partial<CharacterDefinition>, string]>)(
    "rejects %s",
    (_case, overrides, message) => {
      expect(() =>
        validateCharacterDefinitions([validCharacter(overrides)]),
      ).toThrow(message);
    },
  );

  it("rejects duplicate ids", () => {
    expect(() =>
      validateCharacterDefinitions([
        validCharacter(),
        validCharacter({ name: "另一个分析师" }),
      ]),
    ).toThrow("Duplicate character definition id: calm-analyst");
  });

  it.each([
    ["persona", { persona: "" }, "Character calm-analyst must include a persona"],
    [
      "speakingStyle",
      { speakingStyle: "\t\n" },
      "Character calm-analyst must include a speakingStyle",
    ],
    [
      "reasoningStyle",
      { reasoningStyle: 123 },
      "Character calm-analyst reasoningStyle must be a string",
    ],
    [
      "systemPrompt",
      { systemPrompt: {} },
      "Character calm-analyst systemPrompt must be a string",
    ],
  ] satisfies Array<[string, Record<string, unknown>, string]>)(
    "rejects invalid enabled %s",
    (_field, overrides, message) => {
      expect(() =>
        validateCharacterDefinitions([{ ...validCharacter(), ...overrides }]),
      ).toThrow(message);
    },
  );

  it("allows disabled character definitions without persona or prompt text", () => {
    expect(
      validateCharacterDefinitions([
        validCharacter({
          enabled: false,
          persona: "",
          speakingStyle: "",
          reasoningStyle: "",
          systemPrompt: "",
        }),
      ]),
    ).toHaveLength(1);
  });

  it("rejects disabled character definitions with non-string persona fields", () => {
    expect(() =>
      validateCharacterDefinitions([
        {
          ...validCharacter({
            enabled: false,
            persona: "",
            speakingStyle: "",
            reasoningStyle: "",
            systemPrompt: "",
          }),
          persona: 123,
        },
      ]),
    ).toThrow("Character calm-analyst persona must be a string");
  });

  it.each([
    ["avatar type", { avatar: 123 }, "Character calm-analyst avatar must be a string or null"],
    ["blank avatar", { avatar: "  " }, "Character calm-analyst avatar must not be blank"],
    ["tags type", { tags: "logical" }, "Character calm-analyst tags must be an array"],
    ["tag type", { tags: ["logical", 123] }, "Character calm-analyst tags must contain only strings"],
    ["blank tag", { tags: ["logical", ""] }, "Character calm-analyst tags must not contain blank values"],
    ["trimmed tag", { tags: [" logical"] }, "Character calm-analyst tags must not include leading or trailing whitespace"],
    ["enabled type", { enabled: "yes" }, "Character calm-analyst enabled must be a boolean"],
  ] satisfies Array<[string, Record<string, unknown>, string]>)(
    "rejects invalid %s",
    (_field, overrides, message) => {
      expect(() =>
        validateCharacterDefinitions([{ ...validCharacter(), ...overrides }]),
      ).toThrow(message);
    },
  );

  it.each([
    ["object", "invalid", "Character calm-analyst defaultModelBinding must be an object or null"],
    [
      "provider",
      {
        provider: "",
        model: "mock-model",
        temperature: 0.7,
        maxTokens: 1000,
        responseFormat: "json",
      },
      "Character calm-analyst defaultModelBinding.provider must be set",
    ],
    [
      "provider whitespace",
      {
        provider: " mock",
        model: "mock-model",
        temperature: 0.7,
        maxTokens: 1000,
        responseFormat: "json",
      },
      "Character calm-analyst defaultModelBinding.provider must be set",
    ],
    [
      "model",
      {
        provider: "mock",
        model: "",
        temperature: 0.7,
        maxTokens: 1000,
        responseFormat: "json",
      },
      "Character calm-analyst defaultModelBinding.model must be set",
    ],
    [
      "model whitespace",
      {
        provider: "mock",
        model: "mock-model ",
        temperature: 0.7,
        maxTokens: 1000,
        responseFormat: "json",
      },
      "Character calm-analyst defaultModelBinding.model must be set",
    ],
    [
      "temperature",
      {
        provider: "mock",
        model: "mock-model",
        temperature: Infinity,
        maxTokens: 1000,
        responseFormat: "json",
      },
      "Character calm-analyst defaultModelBinding.temperature must be a finite number",
    ],
    [
      "temperature NaN",
      {
        provider: "mock",
        model: "mock-model",
        temperature: NaN,
        maxTokens: 1000,
        responseFormat: "json",
      },
      "Character calm-analyst defaultModelBinding.temperature must be a finite number",
    ],
    [
      "maxTokens",
      {
        provider: "mock",
        model: "mock-model",
        temperature: 0.7,
        maxTokens: 0,
        responseFormat: "json",
      },
      "Character calm-analyst defaultModelBinding.maxTokens must be a positive integer",
    ],
    [
      "maxTokens decimal",
      {
        provider: "mock",
        model: "mock-model",
        temperature: 0.7,
        maxTokens: 1000.5,
        responseFormat: "json",
      },
      "Character calm-analyst defaultModelBinding.maxTokens must be a positive integer",
    ],
    [
      "responseFormat",
      {
        provider: "mock",
        model: "mock-model",
        temperature: 0.7,
        maxTokens: 1000,
        responseFormat: "text",
      },
      "Character calm-analyst defaultModelBinding.responseFormat must be json",
    ],
    [
      "fallbackModel",
      {
        provider: "mock",
        model: "mock-model",
        temperature: 0.7,
        maxTokens: 1000,
        responseFormat: "json",
        fallbackModel: "",
      },
      "Character calm-analyst defaultModelBinding.fallbackModel must be a non-empty string",
    ],
    [
      "fallbackModel whitespace",
      {
        provider: "mock",
        model: "mock-model",
        temperature: 0.7,
        maxTokens: 1000,
        responseFormat: "json",
        fallbackModel: " mock-fallback",
      },
      "Character calm-analyst defaultModelBinding.fallbackModel must be a non-empty string",
    ],
  ] satisfies Array<[string, unknown, string]>)(
    "rejects invalid default model binding %s",
    (_field, defaultModelBinding, message) => {
      expect(() =>
        validateCharacterDefinitions([
          {
            ...validCharacter(),
            defaultModelBinding,
          },
        ]),
      ).toThrow(message);
    },
  );

  it.each([
    ["createdAt", { createdAt: "not-a-date" }, "Character calm-analyst createdAt must be an ISO timestamp"],
    ["createdAt", { createdAt: "2026-06-27" }, "Character calm-analyst createdAt must be an ISO timestamp"],
    ["updatedAt", { updatedAt: "" }, "Character calm-analyst updatedAt must be an ISO timestamp"],
  ] satisfies Array<[string, Record<string, unknown>, string]>)(
    "rejects invalid %s timestamps",
    (_field, overrides, message) => {
      expect(() =>
        validateCharacterDefinitions([{ ...validCharacter(), ...overrides }]),
      ).toThrow(message);
    },
  );

  it("keeps character definitions separate from gameplay role fields", () => {
    const character: CharacterDefinition = validCharacter();

    expect(character).not.toHaveProperty("faction");
    expect(character).not.toHaveProperty("team");
    expect(character).not.toHaveProperty("mechanicKey");
    expect(character).not.toHaveProperty("visibilityRules");
  });

  it.each([
    ["role", { role: "werewolf" }],
    ["faction", { faction: "good" }],
    ["team", { team: "god" }],
    ["mechanicKey", { mechanicKey: "seer_check" }],
    ["visibilityRules", { visibilityRules: ["own_role"] }],
    ["nightOrder", { nightOrder: 20 }],
    ["rolePrompt", { rolePrompt: "你是预言家。" }],
  ])("rejects gameplay role field %s at runtime", (field, roleField) => {
    expect(() =>
      validateCharacterDefinitions([{ ...validCharacter(), ...roleField }]),
    ).toThrow(`Character definition[0] must not include role field: ${field}`);
  });
});
