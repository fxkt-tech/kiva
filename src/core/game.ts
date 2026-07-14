import type { ActorDefinition } from "./actor-definition";
import type { GameRunMode } from "./game-run-mode";
import {
  createGameScriptSnapshot,
  type GameScriptDefinition,
  type GameScriptSnapshot,
} from "./game-script";
import {
  createGamePresenterSnapshot,
  type GamePresenterSnapshot,
  type PresenterDefinition,
} from "./presenter-definition";
import { validateLineups, type Lineup } from "./lineup";
import {
  createPlayerSnapshot,
  validateBoard,
  type PlayerSnapshot,
} from "./player";
import { seedActors } from "@/seeds/actors";
import { seedLineups } from "@/seeds/lineups";
import { seedPresenters } from "@/seeds/presenters";
import { seedScripts } from "@/seeds/scripts";
import {
  createDefaultRuleset,
  type GameId,
  type PlayerId,
  type Ruleset,
} from "./types";

export type GameStatus = "drafting" | "ended";

export type Game = {
  readonly id: GameId;
  readonly title: string;
  readonly status: GameStatus;
  readonly runMode: GameRunMode;
  readonly ruleset: Ruleset;
  readonly presenter: GamePresenterSnapshot;
  readonly script: GameScriptSnapshot;
  readonly players: readonly PlayerSnapshot[];
  readonly createdAt: string;
  readonly updatedAt: string;
};

export type CreateSeedGameInput = {
  readonly gameId: GameId;
  readonly createdAt: string;
  readonly ruleset?: Ruleset;
};

export type CreateGameFromLineupInput = {
  readonly gameId: GameId;
  readonly title: string;
  readonly createdAt: string;
  readonly ruleset: Ruleset;
  readonly lineup: Lineup;
  readonly presenter: PresenterDefinition;
  readonly script: GameScriptDefinition;
  readonly actors: readonly ActorDefinition[];
  readonly runMode: GameRunMode;
};

export function createGameFromLineup(input: CreateGameFromLineupInput): Game {
  validateLineups([input.lineup], input.actors);
  const actorsById = new Map(input.actors.map((actor) => [actor.id, actor]));

  const players = [...input.lineup.seats]
    .sort((left, right) => left.seatNo - right.seatNo)
    .map((seat) => {
      const actor = actorsById.get(seat.actorId);
      if (!actor) {
        throw new Error(`Actor definition not found: ${seat.actorId}`);
      }
      return createPlayerSnapshot({
        playerId: `p${seat.seatNo}` as PlayerId,
        seatNo: seat.seatNo,
        actor,
        ruleRoleId: seat.ruleRoleId,
      });
    });

  const game: Game = {
    id: input.gameId,
    title: input.title,
    status: "drafting",
    runMode: input.runMode,
    ruleset: input.ruleset,
    presenter: createGamePresenterSnapshot(input.presenter),
    script: createGameScriptSnapshot(input.script),
    players,
    createdAt: input.createdAt,
    updatedAt: input.createdAt,
  };

  const boardValidation = validateBoard(game.players, input.ruleset);
  if (!boardValidation.ok) {
    throw new Error(`Lineup ${input.lineup.id} does not match ruleset`);
  }

  return game;
}

export function createSeedGame(input: CreateSeedGameInput): Game {
  return createGameFromLineup({
    gameId: input.gameId,
    title: "12人狼人杀标准局",
    createdAt: input.createdAt,
    ruleset: input.ruleset ?? createDefaultRuleset(),
    lineup: seedLineups[0]!,
    presenter: seedPresenters[0]!,
    script: seedScripts[0]!,
    actors: seedActors,
    runMode: "game",
  });
}
