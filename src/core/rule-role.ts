import { assertExactObjectKeys, isPlainObject } from "./model-binding";

export const RULE_ROLE_IDS = [
  "werewolf",
  "seer",
  "witch",
  "hunter",
  "guard",
  "villager",
] as const;

export type RuleRoleId = (typeof RULE_ROLE_IDS)[number];
export type Faction = "wolves" | "good";
export type RuleRoleTeam = "wolf" | "god" | "villager";
export type RuleRoleMechanicKey =
  | "wolf_kill"
  | "seer_check"
  | "witch_medicine"
  | "hunter_shot"
  | "guard_protect"
  | "none";
export type PrivateKnowledgeKey =
  | "own_role"
  | "wolf_teammates"
  | "witch_medicines";

export type RuleRole = {
  readonly id: RuleRoleId;
  readonly name: string;
  readonly faction: Faction;
  readonly team: RuleRoleTeam;
  readonly mechanicKey: RuleRoleMechanicKey;
  readonly initialPrivateKnowledge: readonly PrivateKnowledgeKey[];
  readonly nightOrder: number | null;
};

const registry = [
  {
    id: "guard",
    name: "守卫",
    faction: "good",
    team: "god",
    mechanicKey: "guard_protect",
    initialPrivateKnowledge: ["own_role"],
    nightOrder: 5,
  },
  {
    id: "werewolf",
    name: "狼人",
    faction: "wolves",
    team: "wolf",
    mechanicKey: "wolf_kill",
    initialPrivateKnowledge: ["own_role", "wolf_teammates"],
    nightOrder: 10,
  },
  {
    id: "seer",
    name: "预言家",
    faction: "good",
    team: "god",
    mechanicKey: "seer_check",
    initialPrivateKnowledge: ["own_role"],
    nightOrder: 20,
  },
  {
    id: "witch",
    name: "女巫",
    faction: "good",
    team: "god",
    mechanicKey: "witch_medicine",
    initialPrivateKnowledge: ["own_role", "witch_medicines"],
    nightOrder: 30,
  },
  {
    id: "hunter",
    name: "猎人",
    faction: "good",
    team: "god",
    mechanicKey: "hunter_shot",
    initialPrivateKnowledge: ["own_role"],
    nightOrder: null,
  },
  {
    id: "villager",
    name: "平民",
    faction: "good",
    team: "villager",
    mechanicKey: "none",
    initialPrivateKnowledge: ["own_role"],
    nightOrder: null,
  },
] as const satisfies readonly RuleRole[];

export const RULE_ROLES: readonly RuleRole[] = registry;

export function ruleRoleById(id: RuleRoleId): RuleRole {
  const role = RULE_ROLES.find((candidate) => candidate.id === id);
  if (!role) {
    throw new Error(`Unsupported Rule Role: ${id}`);
  }
  return role;
}

export function isRuleRoleId(value: unknown): value is RuleRoleId {
  return (
    typeof value === "string" &&
    (RULE_ROLE_IDS as readonly string[]).includes(value)
  );
}

export function createRuleRoleSnapshot(id: RuleRoleId): RuleRole {
  return structuredClone(ruleRoleById(id));
}

export function validateRuleRoleSnapshot(value: unknown): RuleRole {
  if (!isPlainObject(value)) {
    throw new Error("Rule Role snapshot must be an object");
  }
  assertExactObjectKeys(value, "Rule Role snapshot", [
    "id",
    "name",
    "faction",
    "team",
    "mechanicKey",
    "initialPrivateKnowledge",
    "nightOrder",
  ]);
  if (!isRuleRoleId(value.id)) {
    throw new Error("Rule Role snapshot id is invalid");
  }
  const expected = ruleRoleById(value.id);
  if (
    value.name !== expected.name ||
    value.faction !== expected.faction ||
    value.team !== expected.team ||
    value.mechanicKey !== expected.mechanicKey ||
    value.nightOrder !== expected.nightOrder ||
    !Array.isArray(value.initialPrivateKnowledge) ||
    value.initialPrivateKnowledge.length !==
      expected.initialPrivateKnowledge.length ||
    value.initialPrivateKnowledge.some(
      (item, index) => item !== expected.initialPrivateKnowledge[index],
    )
  ) {
    throw new Error(`Rule Role snapshot ${value.id} does not match registry`);
  }
  return structuredClone(value) as RuleRole;
}

export function factionForRuleRole(id: RuleRoleId): Faction {
  return ruleRoleById(id).faction;
}
