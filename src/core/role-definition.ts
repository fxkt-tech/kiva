import type { ModelBindingSnapshot } from "./player";
import type { Faction } from "./types";

export type RoleAbilityId =
  | "werewolf_kill"
  | "seer_check"
  | "witch_antidote"
  | "witch_poison"
  | "hunter_shoot"
  | "vote"
  | "speak"
  | "last_words";

export type RoleVisibilityScope = "public" | "self" | "faction" | "system";

export type RoleDefinition = {
  id: string;
  name: string;
  faction: Faction;
  abilities: RoleAbilityId[];
  nightOrder: number | null;
  visibleTo: RoleVisibilityScope;
  rolePrompt: string;
  actionPrompt: string | null;
  defaultModelBinding: ModelBindingSnapshot | null;
  metadata: Record<string, string>;
};

const ROLE_ABILITY_IDS = new Set<string>([
  "werewolf_kill",
  "seer_check",
  "witch_antidote",
  "witch_poison",
  "hunter_shoot",
  "vote",
  "speak",
  "last_words",
]);

export function validateRoleDefinitions(roles: RoleDefinition[]): void {
  const roleIds = new Set<string>();

  for (const role of roles) {
    const roleId = role.id.trim();

    if (roleId.length === 0) {
      throw new Error("Role definition id must not be blank");
    }

    if (role.name.trim().length === 0) {
      throw new Error(`Role definition "${roleId}" name must not be blank`);
    }

    if (role.rolePrompt.trim().length === 0) {
      throw new Error(`Role definition "${roleId}" rolePrompt must not be blank`);
    }

    if (roleIds.has(roleId)) {
      throw new Error(`Duplicate role definition id "${roleId}"`);
    }
    roleIds.add(roleId);

    for (const ability of role.abilities) {
      if (!ROLE_ABILITY_IDS.has(ability)) {
        throw new Error(
          `Role definition "${roleId}" has unknown ability "${String(ability)}"`,
        );
      }
    }

    if (
      role.nightOrder !== null &&
      (!Number.isFinite(role.nightOrder) || role.nightOrder < 0)
    ) {
      throw new Error(
        `Role definition "${roleId}" nightOrder must be null or a finite number >= 0`,
      );
    }
  }
}
