import type { CharacterDefinition } from "./character-definition";
import { validateGamePresets, type GamePreset } from "./game-preset";
import {
  createGamePresenterSnapshot,
  type GamePresenterSnapshot,
  type PresenterDefinition,
} from "./presenter-definition";
import {
  createPlayerSnapshot,
  validateBoard,
  type PlayerSnapshot,
} from "./player";
import { validateRoleDefinitions, type RoleDefinition } from "./role-definition";
import { seedCharacters } from "@/seeds/characters";
import { seedPresets } from "@/seeds/presets";
import { seedPresenters } from "@/seeds/presenters";
import { seedRoles } from "@/seeds/roles";
import {
  createDefaultRuleset,
  GAME_ROLES,
  type GameRole,
  type GameId,
  type PlayerId,
  type Ruleset,
} from "./types";

export type GameStatus = "drafting" | "ended";

export type Game = {
  readonly id: GameId;
  readonly title: string;
  readonly status: GameStatus;
  readonly ruleset: Ruleset;
  readonly presenter: GamePresenterSnapshot;
  readonly players: readonly PlayerSnapshot[];
  readonly createdAt: string;
  readonly updatedAt: string;
};

export type CreateSeedGameInput = {
  readonly gameId: GameId;
  readonly createdAt: string;
  readonly ruleset?: Ruleset;
};

export type CreateGameFromPresetInput = {
  readonly gameId: GameId;
  readonly title: string;
  readonly createdAt: string;
  readonly ruleset: Ruleset;
  readonly preset: GamePreset;
  readonly presenter: PresenterDefinition;
  readonly roles: readonly RoleDefinition[];
  readonly characters: readonly CharacterDefinition[];
};

export function createGameFromPreset(input: CreateGameFromPresetInput): Game {
  validateRoleDefinitions(input.roles);
  validateGamePresets([input.preset], {
    roles: input.roles,
    characters: input.characters,
  });

  if (input.preset.seatAssignments === null) {
    throw new Error(`Game preset ${input.preset.id} must include seatAssignments`);
  }

  const rolesById = new Map(input.roles.map((role) => [role.id, role]));
  const charactersById = new Map(
    input.characters.map((character) => [character.id, character]),
  );

  const players = input.preset.seatAssignments.map((seat) => {
    const role = rolesById.get(seat.roleId);
    if (!role) {
      throw new Error(`Role definition not found: ${seat.roleId}`);
    }
    const gameRole = toSupportedGameRole(role.id);
    const character = charactersById.get(seat.characterId);
    if (!character) {
      throw new Error(`Character definition not found: ${seat.characterId}`);
    }

    return createPlayerSnapshot({
      playerId: `p${seat.seatNo}` as PlayerId,
      seatNo: seat.seatNo,
      name: character.name,
      gameRole,
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
      voiceProfileSnapshot: character.voiceProfile,
      roleName: role.name,
      faction: role.faction,
      team: role.team,
      mechanicKey: role.mechanicKey,
    });
  });

  const game: Game = {
    id: input.gameId,
    title: input.title,
    status: "drafting",
    ruleset: input.ruleset,
    presenter: createGamePresenterSnapshot(input.presenter),
    players,
    createdAt: input.createdAt,
    updatedAt: input.createdAt,
  };

  const boardValidation = validateBoard(game.players, input.ruleset);
  if (!boardValidation.ok) {
    throw new Error(`Game preset ${input.preset.id} does not match ruleset`);
  }

  return game;
}

export function createSeedGame(input: CreateSeedGameInput): Game {
  return createGameFromPreset({
    gameId: input.gameId,
    title: "12人狼人杀标准局",
    createdAt: input.createdAt,
    ruleset: input.ruleset ?? createDefaultRuleset(),
    preset: seedPresets[0],
    presenter: seedPresenters[0]!,
    roles: seedRoles,
    characters: seedCharacters,
  });
}

function toSupportedGameRole(roleId: string): GameRole {
  if (!isGameRole(roleId)) {
    throw new Error(`Role is not supported by current ruleset: ${roleId}`);
  }

  return roleId;
}

function isGameRole(roleId: string): roleId is GameRole {
  return (GAME_ROLES as readonly string[]).includes(roleId);
}
