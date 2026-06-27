import {
  validateCharacterDefinitions,
  type CharacterDefinition,
} from "./character-definition";
import { createGameFromPreset } from "./game";
import { validateGamePresets, type GamePreset } from "./game-preset";
import {
  createPlayerSnapshot,
  type ModelBindingSnapshot,
} from "./player";
import { validateRoleDefinitions, type RoleDefinition } from "./role-definition";
import {
  createDefaultRuleset,
  type GameId,
  type GameRole,
  type PlayerId,
} from "./types";

export type LibraryDiagnostic = {
  readonly valid: boolean;
  readonly canCreateGame?: boolean;
  readonly references: readonly string[];
  readonly messages: readonly string[];
  readonly promptPreview: string;
};

type LibraryContext = {
  readonly roles: readonly RoleDefinition[];
  readonly characters: readonly CharacterDefinition[];
  readonly presets: readonly GamePreset[];
};

export function diagnoseRole(
  input: LibraryContext & { readonly role: RoleDefinition },
): LibraryDiagnostic {
  const validationMessages = roleCollectionValidationMessages(input.roles);
  const roleMessages = roleDefinitionValidationMessages(input.role);
  const messages = uniqueMessages([
    ...validationMessages,
    ...roleMessages,
    ...(isBuiltInRole(input.role.id) ? ["Built-in role contract locked"] : []),
    ...(input.role.enabled ? [] : ["Role is disabled"]),
  ]);

  return {
    valid:
      validationMessages.length === 0 &&
      roleMessages.length === 0 &&
      input.role.enabled,
    references: presetsReferencingRole(input.presets, input.role.id),
    messages,
    promptPreview: joinPromptParts([
      input.role.systemPrompt,
      input.role.actionPrompt,
    ]),
  };
}

export function diagnoseCharacter(
  input: LibraryContext & { readonly character: CharacterDefinition },
): LibraryDiagnostic {
  const validationMessages = characterCollectionValidationMessages(
    input.characters,
  );
  const characterMessages = characterDefinitionValidationMessages(
    input.character,
  );
  const messages = uniqueMessages([
    ...validationMessages,
    ...characterMessages,
    ...(input.character.enabled ? [] : ["Character is disabled"]),
  ]);

  return {
    valid:
      validationMessages.length === 0 &&
      characterMessages.length === 0 &&
      input.character.enabled,
    references: presetsReferencingCharacter(input.presets, input.character.id),
    messages,
    promptPreview: joinPromptParts([
      input.character.systemPrompt,
      input.character.persona,
      input.character.speakingStyle,
      input.character.reasoningStyle,
    ]),
  };
}

export function diagnosePreset(
  input: LibraryContext & { readonly preset: GamePreset },
): LibraryDiagnostic {
  const roleMessages = roleCollectionValidationMessages(input.roles);
  const characterMessages = characterCollectionValidationMessages(input.characters);
  const presetCollectionMessages = presetCollectionValidationMessages(input);
  const selectedPresetMessages = selectedPresetValidationMessages(input);
  const disabledMessages = input.preset.enabled ? [] : ["Preset is disabled"];
  const validationMessages = uniqueMessages([
    ...roleMessages,
    ...characterMessages,
    ...presetCollectionMessages,
    ...selectedPresetMessages,
    ...disabledMessages,
  ]);
  const presetStructureValid =
    roleMessages.length === 0 &&
    characterMessages.length === 0 &&
    selectedPresetMessages.length === 0;
  const creation = gameCreationDiagnostic(input);
  const messages = uniqueMessages([
    ...validationMessages,
    ...creation.messages,
  ]);

  return {
    valid: validationMessages.length === 0,
    canCreateGame: presetStructureValid && creation.canCreateGame,
    references: [],
    messages,
    promptPreview: promptPreviewForPresetSeat({
      preset: input.preset,
      seatNo: input.preset.seatAssignments?.[0]?.seatNo ?? 1,
      roles: input.roles,
      characters: input.characters,
    }),
  };
}

export function promptPreviewForPresetSeat(input: {
  readonly preset: GamePreset;
  readonly seatNo: number;
  readonly roles: readonly RoleDefinition[];
  readonly characters: readonly CharacterDefinition[];
}): string {
  const assignment = input.preset.seatAssignments?.find(
    (seat) => seat.seatNo === input.seatNo,
  );
  if (assignment === undefined) {
    return "No seat assignment selected.";
  }

  const role = input.roles.find((item) => item.id === assignment.roleId);
  const character = input.characters.find(
    (item) => item.id === assignment.characterId,
  );
  if (role === undefined || character === undefined) {
    return "Seat assignment references missing library items.";
  }

  return joinPromptParts([
    character.systemPrompt,
    role.systemPrompt,
    role.actionPrompt,
    `model: ${resolvedModelLabel({ assignment, character, role })}`,
  ]);
}

function presetsReferencingRole(
  presets: readonly GamePreset[],
  roleId: string,
): readonly string[] {
  return presets
    .filter((preset) => preset.roleIds.includes(roleId))
    .map((preset) => preset.id);
}

function presetsReferencingCharacter(
  presets: readonly GamePreset[],
  characterId: string,
): readonly string[] {
  return presets
    .filter((preset) => preset.characterIds.includes(characterId))
    .map((preset) => preset.id);
}

function roleCollectionValidationMessages(
  roles: readonly RoleDefinition[],
): readonly string[] {
  return validateCollection(() => validateRoleDefinitions(roles));
}

function roleDefinitionValidationMessages(
  role: RoleDefinition,
): readonly string[] {
  return validateCollection(() => validateRoleDefinitions([role]));
}

function characterCollectionValidationMessages(
  characters: readonly CharacterDefinition[],
): readonly string[] {
  return validateCollection(() => validateCharacterDefinitions(characters));
}

function characterDefinitionValidationMessages(
  character: CharacterDefinition,
): readonly string[] {
  return validateCollection(() => validateCharacterDefinitions([character]));
}

function validateCollection(validate: () => void): readonly string[] {
  const messages: string[] = [];

  try {
    validate();
  } catch (error) {
    messages.push(errorMessage(error));
  }

  return uniqueMessages(messages);
}

function presetCollectionValidationMessages(
  input: LibraryContext & { readonly preset: GamePreset },
): readonly string[] {
  return gamePresetValidationMessages(input.presets, input);
}

function selectedPresetValidationMessages(
  input: LibraryContext & { readonly preset: GamePreset },
): readonly string[] {
  return gamePresetValidationMessages([input.preset], input);
}

function gamePresetValidationMessages(
  presets: readonly GamePreset[],
  libraries: {
    readonly roles: readonly RoleDefinition[];
    readonly characters: readonly CharacterDefinition[];
  },
): readonly string[] {
  const messages: string[] = [];

  try {
    validateGamePresets(presets, {
      roles: libraries.roles,
      characters: libraries.characters,
    });
  } catch (error) {
    messages.push(errorMessage(error));
  }

  return messages;
}

function gameCreationDiagnostic(
  input: LibraryContext & { readonly preset: GamePreset },
): { readonly canCreateGame: boolean; readonly messages: readonly string[] } {
  try {
    createGameFromPreset({
      gameId: "library_diagnostic_preview" as GameId,
      title: input.preset.name,
      createdAt: "2026-06-27T00:00:00.000Z",
      ruleset: createDefaultRuleset(),
      preset: input.preset,
      roles: input.roles,
      characters: input.characters,
    });
    return { canCreateGame: true, messages: [] };
  } catch (error) {
    return { canCreateGame: false, messages: [errorMessage(error)] };
  }
}

function uniqueMessages(messages: readonly string[]): readonly string[] {
  return [...new Set(messages)];
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

function joinPromptParts(parts: readonly (string | null)[]): string {
  return parts.filter(isPresentText).join("\n");
}

function resolvedModelLabel(input: {
  readonly assignment: NonNullable<GamePreset["seatAssignments"]>[number];
  readonly character: CharacterDefinition;
  readonly role: RoleDefinition;
}): string {
  const modelBinding =
    input.assignment.modelBindingOverride ??
    input.character.defaultModelBinding ??
    input.role.defaultModelBinding ??
    defaultModelBindingForSupportedRole(input);

  return modelBinding === null ? "system/default" : modelLabel(modelBinding);
}

function defaultModelBindingForSupportedRole(input: {
  readonly assignment: NonNullable<GamePreset["seatAssignments"]>[number];
  readonly character: CharacterDefinition;
  readonly role: RoleDefinition;
}): ModelBindingSnapshot | null {
  if (!isBuiltInRole(input.role.id)) {
    return null;
  }

  return createPlayerSnapshot({
    playerId: "library_diagnostic_preview_p1" as PlayerId,
    seatNo: input.assignment.seatNo,
    name: input.character.name,
    gameRole: input.role.id,
    characterSourceId: input.character.id,
    roleSourceId: input.role.id,
    avatar: input.character.avatar,
    persona: input.character.persona,
    speakingStyle: input.character.speakingStyle,
    reasoningStyle: input.character.reasoningStyle,
    characterSystemPromptSnapshot: input.character.systemPrompt,
    roleSystemPromptSnapshot: input.role.systemPrompt,
    roleActionPromptSnapshot: input.role.actionPrompt,
    systemPrompt: input.character.systemPrompt,
    roleName: input.role.name,
    faction: input.role.faction,
    team: input.role.team,
    mechanicKey: input.role.mechanicKey,
  }).modelBindingSnapshot;
}

function modelLabel(modelBinding: ModelBindingSnapshot): string {
  return `${modelBinding.provider}/${modelBinding.model}`;
}

function isPresentText(value: string | null): value is string {
  return value !== null && value.length > 0;
}

function isBuiltInRole(roleId: string): roleId is GameRole {
  return roleId === "werewolf" ||
    roleId === "seer" ||
    roleId === "witch" ||
    roleId === "villager";
}
