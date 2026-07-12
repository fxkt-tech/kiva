import {
  factionForRole,
  GAME_ROLES,
  type Faction,
  type GameRole,
  type PlayerId,
  type Ruleset,
} from "./types";
import type { RoleMechanicKey, RoleTeam } from "./role-definition";
import { edgeVoiceProfile, type VoiceProfileSnapshot } from "./voice";

export type ModelBindingSnapshot = {
  readonly provider: string;
  readonly model: string;
  readonly temperature: number;
  readonly maxTokens: number;
  readonly responseFormat: "json";
  readonly fallbackModel?: string;
};

export type PrivateKnowledgeKey =
  | "own_role"
  | "wolf_teammates"
  | "witch_medicines";

export type PlayerTeam = RoleTeam;

export type PlayerMechanicKey = RoleMechanicKey;

export type PlayerSnapshot = {
  readonly playerId: PlayerId;
  readonly seatNo: number;
  readonly characterSourceId: string | null;
  readonly profileSourceId?: string;
  readonly name: string;
  readonly roleSourceId: string;
  readonly avatar: string | null;
  readonly persona: string;
  readonly speakingStyle: string;
  readonly reasoningStyle: string;
  readonly characterSystemPromptSnapshot: string;
  readonly roleSystemPromptSnapshot: string;
  readonly roleActionPromptSnapshot: string | null;
  readonly systemPrompt: string;
  readonly modelBindingSnapshot: ModelBindingSnapshot;
  readonly voiceProfileSnapshot: VoiceProfileSnapshot;
  readonly gameRole: GameRole;
  readonly roleName: string;
  readonly faction: Faction;
  readonly team: PlayerTeam;
  readonly mechanicKey: PlayerMechanicKey;
  readonly initialPrivateKnowledge: readonly PrivateKnowledgeKey[];
};

export type CreatePlayerSnapshotInput = {
  readonly playerId: PlayerId;
  readonly seatNo: number;
  readonly name: string;
  readonly gameRole: GameRole;
  readonly characterSourceId?: string | null;
  readonly profileSourceId?: string;
  readonly roleSourceId?: string;
  readonly avatar?: string | null;
  readonly persona?: string;
  readonly speakingStyle?: string;
  readonly reasoningStyle?: string;
  readonly characterSystemPromptSnapshot?: string;
  readonly roleSystemPromptSnapshot?: string;
  readonly roleActionPromptSnapshot?: string | null;
  readonly systemPrompt?: string;
  readonly modelBindingSnapshot?: ModelBindingSnapshot;
  readonly voiceProfileSnapshot?: VoiceProfileSnapshot;
  readonly roleName?: string;
  readonly faction?: Faction;
  readonly team?: PlayerTeam;
  readonly mechanicKey?: PlayerMechanicKey;
  readonly initialPrivateKnowledge?: readonly PrivateKnowledgeKey[];
};

export type BoardValidationResult =
  | { readonly ok: true }
  | {
      readonly ok: false;
      readonly reason: "invalid_player_count";
      readonly expected: number;
      readonly actual: number;
    }
  | { readonly ok: false; readonly reason: "duplicate_player_id" }
  | {
      readonly ok: false;
      readonly reason: "invalid_seat";
      readonly seatNo: number;
    }
  | { readonly ok: false; readonly reason: "duplicate_seat" }
  | {
      readonly ok: false;
      readonly reason: "invalid_role_count";
      readonly role: GameRole;
      readonly expected: number;
      readonly actual: number;
    };

const defaultModelBinding: ModelBindingSnapshot = {
  provider: "volcengine",
  model: "doubao-seed-1-6-flash-250828",
  temperature: 0.7,
  maxTokens: 1200,
  responseFormat: "json",
};

const PRIVATE_KNOWLEDGE_BY_ROLE = {
  werewolf: ["own_role", "wolf_teammates"],
  seer: ["own_role"],
  witch: ["own_role", "witch_medicines"],
  hunter: ["own_role"],
  guard: ["own_role"],
  villager: ["own_role"],
} satisfies Record<GameRole, readonly PrivateKnowledgeKey[]>;

const ROLE_NAME_BY_ROLE = {
  werewolf: "狼人",
  seer: "预言家",
  witch: "女巫",
  hunter: "猎人",
  guard: "守卫",
  villager: "平民",
} satisfies Record<GameRole, string>;

const TEAM_BY_ROLE = {
  werewolf: "wolf",
  seer: "god",
  witch: "god",
  hunter: "god",
  guard: "god",
  villager: "villager",
} satisfies Record<GameRole, PlayerTeam>;

const MECHANIC_KEY_BY_ROLE = {
  werewolf: "wolf_kill",
  seer: "seer_check",
  witch: "witch_medicine",
  hunter: "hunter_shot",
  guard: "guard_protect",
  villager: "none",
} satisfies Record<GameRole, PlayerMechanicKey>;

export function createPlayerSnapshot(
  input: CreatePlayerSnapshotInput,
): PlayerSnapshot {
  const characterSystemPromptSnapshot =
    input.characterSystemPromptSnapshot ?? input.systemPrompt ?? "";

  return {
    playerId: input.playerId,
    seatNo: input.seatNo,
    characterSourceId: input.characterSourceId ?? input.profileSourceId ?? null,
    ...(input.profileSourceId === undefined
      ? {}
      : { profileSourceId: input.profileSourceId }),
    name: input.name,
    roleSourceId: input.roleSourceId ?? input.gameRole,
    avatar: input.avatar ?? null,
    persona: input.persona ?? "",
    speakingStyle: input.speakingStyle ?? "",
    reasoningStyle: input.reasoningStyle ?? "",
    characterSystemPromptSnapshot,
    roleSystemPromptSnapshot: input.roleSystemPromptSnapshot ?? "",
    roleActionPromptSnapshot: input.roleActionPromptSnapshot ?? null,
    systemPrompt: input.systemPrompt ?? input.characterSystemPromptSnapshot ?? "",
    modelBindingSnapshot: {
      ...(input.modelBindingSnapshot ?? defaultModelBinding),
    },
    voiceProfileSnapshot:
      input.voiceProfileSnapshot ?? edgeVoiceProfile("zh-CN-XiaoxiaoNeural"),
    gameRole: input.gameRole,
    roleName: input.roleName ?? ROLE_NAME_BY_ROLE[input.gameRole],
    faction: input.faction ?? factionForRole(input.gameRole),
    team: input.team ?? TEAM_BY_ROLE[input.gameRole],
    mechanicKey: input.mechanicKey ?? MECHANIC_KEY_BY_ROLE[input.gameRole],
    initialPrivateKnowledge:
      input.initialPrivateKnowledge !== undefined
        ? [...input.initialPrivateKnowledge]
        : createInitialPrivateKnowledge(input.gameRole),
  };
}

function createInitialPrivateKnowledge(
  role: GameRole,
): readonly PrivateKnowledgeKey[] {
  return [...PRIVATE_KNOWLEDGE_BY_ROLE[role]];
}

export function validateBoard(
  players: readonly PlayerSnapshot[],
  ruleset: Ruleset,
): BoardValidationResult {
  if (players.length !== ruleset.playerCount) {
    return {
      ok: false,
      reason: "invalid_player_count",
      expected: ruleset.playerCount,
      actual: players.length,
    };
  }

  if (new Set(players.map((p) => p.playerId)).size !== players.length) {
    return { ok: false, reason: "duplicate_player_id" };
  }

  const invalidSeat = players.find(
    (p) =>
      !Number.isInteger(p.seatNo) ||
      p.seatNo < 1 ||
      p.seatNo > ruleset.playerCount,
  );
  if (invalidSeat) {
    return { ok: false, reason: "invalid_seat", seatNo: invalidSeat.seatNo };
  }

  if (new Set(players.map((p) => p.seatNo)).size !== players.length) {
    return { ok: false, reason: "duplicate_seat" };
  }

  for (const role of GAME_ROLES) {
    const actual = players.filter((p) => p.gameRole === role).length;
    const expected = ruleset.roleCounts[role];
    if (actual !== expected) {
      return {
        ok: false,
        reason: "invalid_role_count",
        role,
        expected,
        actual,
      };
    }
  }

  return { ok: true };
}

export const validateSixPlayerBoard = validateBoard;
