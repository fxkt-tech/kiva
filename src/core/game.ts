import { createPlayerSnapshot, type PlayerSnapshot } from "./player";
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
  readonly ruleset: Ruleset;
  readonly players: readonly PlayerSnapshot[];
  readonly createdAt: string;
  readonly updatedAt: string;
};

export type CreateSeedGameInput = {
  readonly gameId: GameId;
  readonly createdAt: string;
  readonly ruleset?: Ruleset;
};

export function createSeedGame(input: CreateSeedGameInput): Game {
  const ruleset = input.ruleset ?? createDefaultRuleset();
  const players = [
    ["p1", 1, "秦川", "werewolf"],
    ["p2", 2, "林夏", "werewolf"],
    ["p3", 3, "周知", "seer"],
    ["p4", 4, "许棠", "witch"],
    ["p5", 5, "陈墨", "villager"],
    ["p6", 6, "沈岚", "villager"],
  ] as const;

  return {
    id: input.gameId,
    title: "6人狼人杀试运行",
    status: "drafting",
    ruleset,
    players: players.map(([suffix, seatNo, name, role]) =>
      createPlayerSnapshot({
        playerId: `${input.gameId}_${suffix}` as PlayerId,
        seatNo,
        name,
        gameRole: role,
        persona: "冷静、愿意表达判断",
        speakingStyle: "短句、直接、有推进感",
        reasoningStyle: "根据自己可见的信息给出结论",
        systemPrompt: "你是狼人杀对局中的一名玩家，只能依据你可见的信息行动。",
      }),
    ),
    createdAt: input.createdAt,
    updatedAt: input.createdAt,
  };
}
