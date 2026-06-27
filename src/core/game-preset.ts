import type { CharacterDefinition } from "./character-definition";
import type { ModelBindingSnapshot } from "./player";
import type { RoleDefinition } from "./role-definition";

export type GamePresetSeatAssignment = {
  readonly seatNo: number;
  readonly roleId: string;
  readonly characterId: string;
  readonly modelBindingOverride: ModelBindingSnapshot | null;
};

export type GamePreset = {
  readonly id: string;
  readonly name: string;
  readonly rulesetId: string;
  readonly playerCount: number;
  readonly roleIds: readonly string[];
  readonly characterIds: readonly string[];
  readonly seatAssignments: readonly GamePresetSeatAssignment[] | null;
  readonly enabled: boolean;
  readonly createdAt: string;
  readonly updatedAt: string;
};

export function validateGamePresets(
  input: unknown,
  libraries: {
    roles: readonly RoleDefinition[];
    characters: readonly CharacterDefinition[];
  },
): readonly GamePreset[] {
  if (!Array.isArray(input)) {
    throw new Error("Game presets must be an array");
  }

  const rolesById = new Map(libraries.roles.map((role) => [role.id, role]));
  const charactersById = new Map(
    libraries.characters.map((character) => [character.id, character]),
  );
  const presetIds = new Set<string>();

  for (const [index, preset] of input.entries()) {
    const path = `Game preset[${index}]`;
    assertPresetObject(preset, path);
    const presetId = requireStableId(preset.id, `${path} id`);

    if (presetIds.has(presetId)) {
      throw new Error(`Duplicate game preset id: ${presetId}`);
    }
    presetIds.add(presetId);

    if (!isNonBlankString(preset.name)) {
      throw new Error(`Game preset ${presetId} must include a name`);
    }

    if (!isNonBlankString(preset.rulesetId)) {
      throw new Error(`Game preset ${presetId} must include a rulesetId`);
    }

    if (!isPositiveInteger(preset.playerCount)) {
      throw new Error(
        `Game preset ${presetId} playerCount must be a positive integer`,
      );
    }

    validateIdList({
      ids: preset.roleIds,
      fieldName: "roleIds",
      playerCount: preset.playerCount,
      presetId,
      lookup: rolesById,
      entityName: "role",
    });
    validateIdList({
      ids: preset.characterIds,
      fieldName: "characterIds",
      playerCount: preset.playerCount,
      presetId,
      lookup: charactersById,
      entityName: "character",
    });

    validateSeatAssignments({
      preset,
      presetId,
      rolesById,
      charactersById,
    });

    if (typeof preset.enabled !== "boolean") {
      throw new Error(`Game preset ${presetId} enabled must be a boolean`);
    }

    requireIsoTimestamp(
      preset.createdAt,
      `Game preset ${presetId} createdAt`,
    );
    requireIsoTimestamp(
      preset.updatedAt,
      `Game preset ${presetId} updatedAt`,
    );
  }

  return input as readonly GamePreset[];
}

function assertPresetObject(
  preset: unknown,
  path: string,
): asserts preset is GamePreset {
  if (preset === null || typeof preset !== "object") {
    throw new Error(`${path} must be an object`);
  }
}

function validateIdList<T extends { readonly enabled: boolean }>({
  ids,
  fieldName,
  playerCount,
  presetId,
  lookup,
  entityName,
}: {
  ids: unknown;
  fieldName: "roleIds" | "characterIds";
  playerCount: number;
  presetId: string;
  lookup: ReadonlyMap<string, T>;
  entityName: "role" | "character";
}): void {
  if (!Array.isArray(ids)) {
    throw new Error(`Game preset ${presetId} ${fieldName} must be an array`);
  }

  if (ids.length !== playerCount) {
    throw new Error(
      `Game preset ${presetId} ${fieldName} length must equal playerCount`,
    );
  }

  for (const [index, value] of ids.entries()) {
    const id = requireStableId(
      value,
      `Game preset ${presetId} ${fieldName}[${index}]`,
    );
    const definition = lookup.get(id);

    if (definition === undefined) {
      throw new Error(
        `Game preset ${presetId} ${fieldName}[${index}] references unknown ${entityName}: ${id}`,
      );
    }

    if (!definition.enabled) {
      throw new Error(
        `Game preset ${presetId} ${fieldName}[${index}] references disabled ${entityName}: ${id}`,
      );
    }
  }
}

function validateSeatAssignments({
  preset,
  presetId,
  rolesById,
  charactersById,
}: {
  preset: GamePreset;
  presetId: string;
  rolesById: ReadonlyMap<string, RoleDefinition>;
  charactersById: ReadonlyMap<string, CharacterDefinition>;
}): void {
  if (preset.seatAssignments === null) {
    return;
  }

  if (!Array.isArray(preset.seatAssignments)) {
    throw new Error(
      `Game preset ${presetId} seatAssignments must be an array or null`,
    );
  }

  if (preset.seatAssignments.length !== preset.playerCount) {
    throw new Error(
      `Game preset ${presetId} seatAssignments length must equal playerCount`,
    );
  }

  const seenSeats = new Set<number>();
  const assignedRoleIds: string[] = [];
  const assignedCharacterIds: string[] = [];

  for (const [index, assignment] of preset.seatAssignments.entries()) {
    const assignmentPath = `Game preset ${presetId} seatAssignments[${index}]`;
    assertSeatAssignmentObject(assignment, assignmentPath);

    if (
      !Number.isInteger(assignment.seatNo) ||
      assignment.seatNo < 1 ||
      assignment.seatNo > preset.playerCount
    ) {
      throw new Error(
        `${assignmentPath} seatNo must be an integer from 1 to playerCount`,
      );
    }

    if (seenSeats.has(assignment.seatNo)) {
      throw new Error(
        `${assignmentPath} duplicates seatNo: ${assignment.seatNo}`,
      );
    }
    seenSeats.add(assignment.seatNo);

    const roleId = requireStableId(
      assignment.roleId,
      `${assignmentPath} roleId`,
    );
    validateEnabledReference({
      id: roleId,
      lookup: rolesById,
      entityName: "role",
      path: `${assignmentPath} roleId`,
    });
    assignedRoleIds.push(roleId);

    const characterId = requireStableId(
      assignment.characterId,
      `${assignmentPath} characterId`,
    );
    validateEnabledReference({
      id: characterId,
      lookup: charactersById,
      entityName: "character",
      path: `${assignmentPath} characterId`,
    });
    assignedCharacterIds.push(characterId);

    validateModelBinding(
      assignment.modelBindingOverride,
      `${assignmentPath} modelBindingOverride`,
    );
  }

  if (!haveSameMultiset(preset.roleIds, assignedRoleIds)) {
    throw new Error(
      `Game preset ${presetId} seatAssignments roleId multiset must match roleIds`,
    );
  }

  if (!haveSameMultiset(preset.characterIds, assignedCharacterIds)) {
    throw new Error(
      `Game preset ${presetId} seatAssignments characterId multiset must match characterIds`,
    );
  }
}

function assertSeatAssignmentObject(
  assignment: unknown,
  path: string,
): asserts assignment is GamePresetSeatAssignment {
  if (assignment === null || typeof assignment !== "object") {
    throw new Error(`${path} must be an object`);
  }
}

function validateEnabledReference<T extends { readonly enabled: boolean }>({
  id,
  lookup,
  entityName,
  path,
}: {
  id: string;
  lookup: ReadonlyMap<string, T>;
  entityName: "role" | "character";
  path: string;
}): void {
  const definition = lookup.get(id);

  if (definition === undefined) {
    throw new Error(`${path} references unknown ${entityName}: ${id}`);
  }

  if (!definition.enabled) {
    throw new Error(`${path} references disabled ${entityName}: ${id}`);
  }
}

function validateModelBinding(value: unknown, path: string): void {
  if (value === null) {
    return;
  }

  if (typeof value !== "object") {
    throw new Error(`${path} must be an object or null`);
  }

  const binding = value as Partial<ModelBindingSnapshot>;

  if (!isStableString(binding.provider)) {
    throw new Error(`${path}.provider must be set`);
  }

  if (!isStableString(binding.model)) {
    throw new Error(`${path}.model must be set`);
  }

  if (
    typeof binding.temperature !== "number" ||
    !Number.isFinite(binding.temperature)
  ) {
    throw new Error(`${path}.temperature must be a finite number`);
  }

  if (
    typeof binding.maxTokens !== "number" ||
    !Number.isInteger(binding.maxTokens) ||
    binding.maxTokens <= 0
  ) {
    throw new Error(`${path}.maxTokens must be a positive integer`);
  }

  if (binding.responseFormat !== "json") {
    throw new Error(`${path}.responseFormat must be json`);
  }

  if (
    binding.fallbackModel !== undefined &&
    !isStableString(binding.fallbackModel)
  ) {
    throw new Error(`${path}.fallbackModel must be a non-empty string`);
  }
}

function haveSameMultiset(
  expected: readonly string[],
  actual: readonly string[],
): boolean {
  if (expected.length !== actual.length) {
    return false;
  }

  const counts = new Map<string, number>();

  for (const id of expected) {
    counts.set(id, (counts.get(id) ?? 0) + 1);
  }

  for (const id of actual) {
    const count = counts.get(id);

    if (count === undefined) {
      return false;
    }

    if (count === 1) {
      counts.delete(id);
    } else {
      counts.set(id, count - 1);
    }
  }

  return counts.size === 0;
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

function requireIsoTimestamp(value: unknown, fieldName: string): void {
  if (
    typeof value !== "string" ||
    Number.isNaN(Date.parse(value)) ||
    new Date(value).toISOString() !== value
  ) {
    throw new Error(`${fieldName} must be an ISO timestamp`);
  }
}

function isPositiveInteger(value: unknown): value is number {
  return typeof value === "number" && Number.isInteger(value) && value > 0;
}

function isNonBlankString(value: unknown): value is string {
  return typeof value === "string" && value.trim().length > 0;
}

function isStableString(value: unknown): value is string {
  return isNonBlankString(value) && value === value.trim();
}
