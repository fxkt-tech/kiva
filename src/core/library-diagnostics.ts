import {
  validateCharacterDefinitions,
  type CharacterDefinition,
} from "./character-definition";
import { validateGamePresets, type GamePreset } from "./game-preset";
import {
  createPlayerSnapshot,
  validateSixPlayerBoard,
  type ModelBindingSnapshot,
  type PlayerSnapshot,
} from "./player";
import { validateRoleDefinitions, type RoleDefinition } from "./role-definition";
import {
  createDefaultRuleset,
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
  if (!Array.isArray(input.preset.seatAssignments)) {
    return "No seat assignment selected.";
  }

  const assignment = input.preset.seatAssignments.find(
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

  messages.push(...presetReferenceMessages(presets, libraries));

  return uniqueMessages(messages);
}

function presetReferenceMessages(
  presets: readonly GamePreset[],
  libraries: {
    readonly roles: readonly RoleDefinition[];
    readonly characters: readonly CharacterDefinition[];
  },
): readonly string[] {
  const rolesById = new Map(libraries.roles.map((role) => [role.id, role]));
  const charactersById = new Map(
    libraries.characters.map((character) => [character.id, character]),
  );
  const messages: string[] = [];

  for (const preset of presets) {
    if (!Array.isArray(preset.roleIds) || !Array.isArray(preset.characterIds)) {
      continue;
    }

    for (const [index, roleId] of preset.roleIds.entries()) {
      const role = rolesById.get(roleId);
      if (role === undefined) {
        messages.push(
          `Game preset ${preset.id} roleIds[${index}] references unknown role: ${roleId}`,
        );
      } else if (!role.enabled) {
        messages.push(
          `Game preset ${preset.id} roleIds[${index}] references disabled role: ${roleId}`,
        );
      }
    }

    for (const [index, characterId] of preset.characterIds.entries()) {
      const character = charactersById.get(characterId);
      if (character === undefined) {
        messages.push(
          `Game preset ${preset.id} characterIds[${index}] references unknown character: ${characterId}`,
        );
      } else if (!character.enabled) {
        messages.push(
          `Game preset ${preset.id} characterIds[${index}] references disabled character: ${characterId}`,
        );
      }
    }
  }

  return messages;
}

function gameCreationDiagnostic(
  input: LibraryContext & { readonly preset: GamePreset },
): { readonly canCreateGame: boolean; readonly messages: readonly string[] } {
  const messages: string[] = [];

  if (input.preset.seatAssignments === null) {
    messages.push(`Game preset ${input.preset.id} must include seatAssignments`);
    return { canCreateGame: false, messages };
  }

  if (!Array.isArray(input.preset.seatAssignments)) {
    return { canCreateGame: false, messages };
  }

  const rolesById = new Map(input.roles.map((role) => [role.id, role]));
  const charactersById = new Map(
    input.characters.map((character) => [character.id, character]),
  );
  const players: PlayerSnapshot[] = [];

  for (const seat of input.preset.seatAssignments) {
    const role = rolesById.get(seat.roleId);
    if (role === undefined) {
      messages.push(`Role definition not found: ${seat.roleId}`);
      continue;
    }

    if (!isBuiltInRole(role.id)) {
      messages.push(`Role is not supported by current ruleset: ${role.id}`);
      continue;
    }

    const character = charactersById.get(seat.characterId);
    if (character === undefined) {
      messages.push(`Character definition not found: ${seat.characterId}`);
      continue;
    }

    players.push(
      createPlayerSnapshot({
        playerId: `library_diagnostic_preview_p${seat.seatNo}` as PlayerId,
        seatNo: seat.seatNo,
        name: character.name,
        gameRole: role.id,
        characterSourceId: character.id,
        roleSourceId: role.id,
        avatar: character.avatar,
        persona: character.persona,
        speakingStyle: character.speakingStyle,
        reasoningStyle: character.reasoningStyle,
        characterSystemPromptSnapshot: character.systemPrompt,
        roleSystemPromptSnapshot: role.systemPrompt,
        roleActionPromptSnapshot: role.actionPrompt,
        systemPrompt: character.systemPrompt,
        modelBindingSnapshot:
          seat.modelBindingOverride ??
          character.defaultModelBinding ??
          role.defaultModelBinding ??
          undefined,
        roleName: role.name,
        faction: role.faction,
        team: role.team,
        mechanicKey: role.mechanicKey,
      }),
    );
  }

  if (messages.length === 0) {
    const boardValidation = validateSixPlayerBoard(players, createDefaultRuleset());
    if (!boardValidation.ok) {
      messages.push(`Game preset ${input.preset.id} does not match ruleset`);
    }
  }

  return { canCreateGame: messages.length === 0, messages: uniqueMessages(messages) };
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
