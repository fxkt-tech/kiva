import { isPlainObject, validateModelBindingSnapshot } from "./model-binding";
import type { ModelBindingSnapshot } from "./player";

export type CharacterDefinition = {
  readonly id: string;
  readonly name: string;
  readonly avatar: string | null;
  readonly tags: readonly string[];
  readonly persona: string;
  readonly speakingStyle: string;
  readonly reasoningStyle: string;
  readonly systemPrompt: string;
  readonly defaultModelBinding: ModelBindingSnapshot | null;
  readonly enabled: boolean;
  readonly createdAt: string;
  readonly updatedAt: string;
};

const ROLE_ONLY_FIELDS = [
  "role",
  "faction",
  "team",
  "mechanicKey",
  "visibilityRules",
  "nightOrder",
  "rolePrompt",
] as const;

export function validateCharacterDefinitions(
  characters: unknown,
): readonly CharacterDefinition[] {
  if (!Array.isArray(characters)) {
    throw new Error("Character definitions must be an array");
  }

  const characterIds = new Set<string>();

  for (const [index, character] of characters.entries()) {
    const path = `Character definition[${index}]`;
    assertCharacterObject(character, path);
    rejectRoleOnlyFields(character, path);
    const characterId = requireStableId(
      character.id,
      `${path} id`,
    );

    if (characterIds.has(characterId)) {
      throw new Error(`Duplicate character definition id: ${characterId}`);
    }
    characterIds.add(characterId);

    if (!isNonBlankString(character.name)) {
      throw new Error(`Character ${characterId} must include a name`);
    }

    validateAvatar(character.avatar, characterId);
    validateTags(character.tags, characterId);
    requireTextField(character.persona, characterId, "persona");
    requireTextField(character.speakingStyle, characterId, "speakingStyle");
    requireTextField(character.reasoningStyle, characterId, "reasoningStyle");
    requireTextField(character.systemPrompt, characterId, "systemPrompt");

    if (typeof character.enabled !== "boolean") {
      throw new Error(`Character ${characterId} enabled must be a boolean`);
    }

    if (character.enabled) {
      requireEnabledText(character.persona, characterId, "persona");
      requireEnabledText(character.speakingStyle, characterId, "speakingStyle");
      requireEnabledText(
        character.reasoningStyle,
        characterId,
        "reasoningStyle",
      );
      requireEnabledText(character.systemPrompt, characterId, "systemPrompt");
    }

    validateModelBindingSnapshot(
      character.defaultModelBinding,
      `Character ${characterId} defaultModelBinding`,
    );
    requireIsoTimestamp(
      character.createdAt,
      `Character ${characterId} createdAt`,
    );
    requireIsoTimestamp(
      character.updatedAt,
      `Character ${characterId} updatedAt`,
    );
  }

  return characters as readonly CharacterDefinition[];
}

function assertCharacterObject(
  character: unknown,
  path: string,
): asserts character is CharacterDefinition {
  if (!isPlainObject(character)) {
    throw new Error(`${path} must be an object`);
  }
}

function rejectRoleOnlyFields(
  character: CharacterDefinition,
  path: string,
): void {
  for (const field of ROLE_ONLY_FIELDS) {
    if (field in character) {
      throw new Error(`${path} must not include role field: ${field}`);
    }
  }
}

function requireStableId(value: unknown, fieldName: string): string {
  if (typeof value !== "string" || value.trim().length === 0) {
    throw new Error(`${fieldName} must not be blank`);
  }

  if (value !== value.trim()) {
    throw new Error(`${fieldName} must not include leading or trailing whitespace`);
  }

  return value;
}

function validateAvatar(value: unknown, characterId: string): void {
  if (value === null) {
    return;
  }

  if (typeof value !== "string") {
    throw new Error(`Character ${characterId} avatar must be a string or null`);
  }

  if (value.trim().length === 0) {
    throw new Error(`Character ${characterId} avatar must not be blank`);
  }
}

function validateTags(value: unknown, characterId: string): void {
  if (!Array.isArray(value)) {
    throw new Error(`Character ${characterId} tags must be an array`);
  }

  for (const tag of value) {
    if (typeof tag !== "string") {
      throw new Error(`Character ${characterId} tags must contain only strings`);
    }

    if (tag.trim().length === 0) {
      throw new Error(`Character ${characterId} tags must not contain blank values`);
    }

    if (tag !== tag.trim()) {
      throw new Error(
        `Character ${characterId} tags must not include leading or trailing whitespace`,
      );
    }
  }
}

function requireEnabledText(
  value: unknown,
  characterId: string,
  fieldName: "persona" | "speakingStyle" | "reasoningStyle" | "systemPrompt",
): void {
  if (!isNonBlankString(value)) {
    throw new Error(`Character ${characterId} must include a ${fieldName}`);
  }
}

function requireTextField(
  value: unknown,
  characterId: string,
  fieldName: "persona" | "speakingStyle" | "reasoningStyle" | "systemPrompt",
): void {
  if (typeof value !== "string") {
    throw new Error(`Character ${characterId} ${fieldName} must be a string`);
  }
}

function requireIsoTimestamp(value: unknown, fieldName: string): void {
  if (
    typeof value !== "string" ||
    Number.isNaN(Date.parse(value)) ||
    new Date(value).toISOString() !== value
  ) {
    throw new Error(`${fieldName} must be an ISO timestamp`);
  }
}

function isNonBlankString(value: unknown): value is string {
  return typeof value === "string" && value.trim().length > 0;
}
