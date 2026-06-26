import {
  factionForRole,
  GAME_ROLES,
  type Faction,
  type GameRole,
  type PlayerId,
  type Ruleset,
} from "./types";

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

export type PlayerSnapshot = {
  readonly playerId: PlayerId;
  readonly seatNo: number;
  readonly profileSourceId?: string;
  readonly name: string;
  readonly avatar?: string;
  readonly persona: string;
  readonly speakingStyle: string;
  readonly reasoningStyle: string;
  readonly systemPrompt: string;
  readonly modelBindingSnapshot: ModelBindingSnapshot;
  readonly gameRole: GameRole;
  readonly faction: Faction;
  readonly initialPrivateKnowledge: readonly PrivateKnowledgeKey[];
};

export type CreatePlayerSnapshotInput = {
  readonly playerId: PlayerId;
  readonly seatNo: number;
  readonly name: string;
  readonly gameRole: GameRole;
  readonly profileSourceId?: string;
  readonly avatar?: string;
  readonly persona?: string;
  readonly speakingStyle?: string;
  readonly reasoningStyle?: string;
  readonly systemPrompt?: string;
  readonly modelBindingSnapshot?: ModelBindingSnapshot;
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
  | { readonly ok: false; readonly reason: "invalid_seat"; readonly seatNo: number }
  | { readonly ok: false; readonly reason: "duplicate_seat" }
  | {
      readonly ok: false;
      readonly reason: "invalid_role_count";
      readonly role: GameRole;
      readonly expected: number;
      readonly actual: number;
    };

const defaultModelBinding: ModelBindingSnapshot = {
  provider: "mock",
  model: "mock-role-model",
  temperature: 0.7,
  maxTokens: 1200,
  responseFormat: "json",
};

const PRIVATE_KNOWLEDGE_BY_ROLE = {
  werewolf: ["own_role", "wolf_teammates"],
  seer: ["own_role"],
  witch: ["own_role", "witch_medicines"],
  villager: ["own_role"],
} satisfies Record<GameRole, readonly PrivateKnowledgeKey[]>;

export function createPlayerSnapshot(
  input: CreatePlayerSnapshotInput,
): PlayerSnapshot {
  const faction = factionForRole(input.gameRole);

  return {
    playerId: input.playerId,
    seatNo: input.seatNo,
    profileSourceId: input.profileSourceId,
    name: input.name,
    avatar: input.avatar,
    persona: input.persona ?? "",
    speakingStyle: input.speakingStyle ?? "",
    reasoningStyle: input.reasoningStyle ?? "",
    systemPrompt: input.systemPrompt ?? "",
    modelBindingSnapshot: {
      ...(input.modelBindingSnapshot ?? defaultModelBinding),
    },
    gameRole: input.gameRole,
    faction,
    initialPrivateKnowledge: createInitialPrivateKnowledge(input.gameRole),
  };
}

function createInitialPrivateKnowledge(
  role: GameRole,
): readonly PrivateKnowledgeKey[] {
  return [...PRIVATE_KNOWLEDGE_BY_ROLE[role]];
}

export function validateSixPlayerBoard(
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
