import { isPlainObject, validateModelBindingSnapshot } from "./model-binding";
import type { ModelBindingSnapshot } from "./player";
import type { Faction } from "./types";

const ROLE_TEAMS = ["wolf", "god", "villager"] as const;
export type RoleTeam = (typeof ROLE_TEAMS)[number];

const ROLE_MECHANIC_KEYS = [
  "wolf_kill",
  "seer_check",
  "witch_medicine",
  "none",
] as const;
export type RoleMechanicKey = (typeof ROLE_MECHANIC_KEYS)[number];

const ROLE_KNOWLEDGE_RULES = [
  "own_role",
  "wolf_teammates",
  "witch_medicines",
] as const;
export type RoleKnowledgeRule = (typeof ROLE_KNOWLEDGE_RULES)[number];

export type RoleDefinition = {
  readonly id: string;
  readonly name: string;
  readonly faction: Faction;
  readonly team: RoleTeam;
  readonly mechanicKey: RoleMechanicKey;
  readonly systemPrompt: string;
  readonly actionPrompt: string | null;
  readonly visibilityRules: readonly RoleKnowledgeRule[];
  readonly nightOrder: number | null;
  readonly defaultModelBinding: ModelBindingSnapshot | null;
  readonly enabled: boolean;
  readonly createdAt: string;
  readonly updatedAt: string;
};

const FACTIONS = ["wolves", "good"] as const;

export function validateRoleDefinitions(roles: unknown): readonly RoleDefinition[] {
  if (!Array.isArray(roles)) {
    throw new Error("Role definitions must be an array");
  }

  const roleIds = new Set<string>();

  for (const role of roles) {
    assertRoleObject(role);
    const roleId = requireStableId(role.id, "Role definition id");

    if (roleIds.has(roleId)) {
      throw new Error(`Duplicate role definition id: ${roleId}`);
    }
    roleIds.add(roleId);

    if (!isOneOf(role.faction, FACTIONS)) {
      throw new Error(`Role ${roleId} has invalid faction: ${String(role.faction)}`);
    }

    if (!isOneOf(role.team, ROLE_TEAMS)) {
      throw new Error(`Role ${roleId} has invalid team: ${String(role.team)}`);
    }

    if (!isOneOf(role.mechanicKey, ROLE_MECHANIC_KEYS)) {
      throw new Error(
        `Role ${roleId} has invalid mechanicKey: ${String(role.mechanicKey)}`,
      );
    }

    if (!isNonBlankString(role.name)) {
      throw new Error(`Role ${roleId} must include a name`);
    }

    if (role.enabled && !isNonBlankString(role.systemPrompt)) {
      throw new Error(`Role ${roleId} must include a systemPrompt`);
    }

    if (role.actionPrompt !== null && typeof role.actionPrompt !== "string") {
      throw new Error(`Role ${roleId} actionPrompt must be a string or null`);
    }

    if (!Array.isArray(role.visibilityRules)) {
      throw new Error(`Role ${roleId} visibilityRules must be an array`);
    }

    for (const rule of role.visibilityRules) {
      if (!isOneOf(rule, ROLE_KNOWLEDGE_RULES)) {
        throw new Error(
          `Role ${roleId} has invalid visibility rule: ${String(rule)}`,
        );
      }
    }

    if (!isValidNightOrder(role.nightOrder)) {
      throw new Error(`Role ${roleId} nightOrder must be null or a finite number >= 0`);
    }

    if (typeof role.enabled !== "boolean") {
      throw new Error(`Role ${roleId} enabled must be a boolean`);
    }

    validateModelBindingSnapshot(
      role.defaultModelBinding,
      `Role ${roleId} defaultModelBinding`,
    );
    requireIsoTimestamp(role.createdAt, `Role ${roleId} createdAt`);
    requireIsoTimestamp(role.updatedAt, `Role ${roleId} updatedAt`);
  }

  return roles as readonly RoleDefinition[];
}

function assertRoleObject(role: unknown): asserts role is RoleDefinition {
  if (!isPlainObject(role)) {
    throw new Error("Role definition must be an object");
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

function requireIsoTimestamp(value: unknown, fieldName: string): void {
  if (
    typeof value !== "string" ||
    Number.isNaN(Date.parse(value)) ||
    new Date(value).toISOString() !== value
  ) {
    throw new Error(`${fieldName} must be an ISO timestamp`);
  }
}

function isValidNightOrder(value: number | null): boolean {
  return (
    value === null ||
    (typeof value === "number" && Number.isFinite(value) && value >= 0)
  );
}

function isNonBlankString(value: unknown): value is string {
  return typeof value === "string" && value.trim().length > 0;
}

function isOneOf<T extends readonly string[]>(
  value: unknown,
  values: T,
): value is T[number] {
  return typeof value === "string" && values.includes(value);
}
