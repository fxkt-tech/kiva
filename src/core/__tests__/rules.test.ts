import { describe, expect, it } from "vitest";
import { createPlayerSnapshot } from "../player";
import {
  checkWinCondition,
  getLegalNightTargets,
  resolveNightDeaths,
} from "../rules";
import { createDefaultRuleset, type PlayerId } from "../types";

const p1 = "p1" as PlayerId;
const p2 = "p2" as PlayerId;
const p3 = "p3" as PlayerId;
const p4 = "p4" as PlayerId;
const p5 = "p5" as PlayerId;
const p6 = "p6" as PlayerId;

const players = [
  createPlayerSnapshot({
    playerId: p1,
    seatNo: 1,
    name: "P1",
    gameRole: "werewolf",
  }),
  createPlayerSnapshot({
    playerId: p2,
    seatNo: 2,
    name: "P2",
    gameRole: "werewolf",
  }),
  createPlayerSnapshot({
    playerId: p3,
    seatNo: 3,
    name: "P3",
    gameRole: "seer",
  }),
  createPlayerSnapshot({
    playerId: p4,
    seatNo: 4,
    name: "P4",
    gameRole: "witch",
  }),
  createPlayerSnapshot({
    playerId: p5,
    seatNo: 5,
    name: "P5",
    gameRole: "villager",
  }),
  createPlayerSnapshot({
    playerId: p6,
    seatNo: 6,
    name: "P6",
    gameRole: "villager",
  }),
];

describe("rules", () => {
  it("allows wolves to target living non-wolves", () => {
    expect(
      getLegalNightTargets("wolf_kill", players, [p1, p2, p3, p4, p5, p6]),
    ).toEqual([p3, p4, p5, p6]);
  });

  it("allows seer to check living players except self", () => {
    expect(
      getLegalNightTargets("seer_check", players, [p1, p2, p3, p4], p3),
    ).toEqual([p1, p2, p4]);
  });

  it("rescues wolf kill with antidote", () => {
    expect(
      resolveNightDeaths({
        wolfKillTargetId: p3,
        antidoteTargetId: p3,
        poisonTargetId: null,
      }),
    ).toEqual([]);
  });

  it("kills poison target even when wolf kill is rescued", () => {
    expect(
      resolveNightDeaths({
        wolfKillTargetId: p3,
        antidoteTargetId: p3,
        poisonTargetId: p5,
      }),
    ).toEqual([p5]);
  });

  it("good team wins when all wolves are dead", () => {
    expect(checkWinCondition(players, [p1, p2], createDefaultRuleset())).toEqual({
      ended: true,
      winner: "good",
      reason: "all_wolves_dead",
    });
  });

  it("wolves win by slaughter side when all gods are dead", () => {
    expect(checkWinCondition(players, [p3, p4], createDefaultRuleset())).toEqual({
      ended: true,
      winner: "wolves",
      reason: "all_gods_dead",
    });
  });
});
