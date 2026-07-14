import { describe, expect, it } from "vitest";
import { compileActorRuntimeCard } from "../actor-definition";
import { createGameFromLineup, createSeedGame } from "../game";
import {
  RULE_ROLE_IDS,
  createDefaultRuleset,
  type GameId,
} from "../types";
import { seedActors } from "@/seeds/actors";
import { seedLineups } from "@/seeds/lineups";
import { seedPresenters } from "@/seeds/presenters";
import { seedScripts } from "@/seeds/scripts";

const createdAt = "2026-07-14T00:00:00.000Z";

describe("game creation from Lineup", () => {
  it("freezes the exact selected 12 Actors and their Rule Roles", () => {
    const game = createGameFromLineup({
      gameId: "game_lineup" as GameId,
      title: "本局阵容",
      createdAt,
      ruleset: createDefaultRuleset(),
      lineup: seedLineups[0]!,
      presenter: seedPresenters[0]!,
      script: seedScripts[0]!,
      actors: seedActors,
      runMode: "scripted",
    });
    expect(game.players).toHaveLength(12);
    expect(game.players.map((player) => ({
      actorId: player.actor.sourceId,
      roleId: player.ruleRole.id,
    }))).toEqual(
      [...seedLineups[0]!.seats]
        .sort((a, b) => a.seatNo - b.seatNo)
        .map((seat) => ({ actorId: seat.actorId, roleId: seat.ruleRoleId })),
    );
    expect(game.players.every((player) => Object.keys(player).length === 4)).toBe(true);
  });

  it("does not require Qin Chuan in every game", () => {
    const replacement = seedActors.find((actor) => actor.id === "bai_qi")!;
    const lineup = {
      ...seedLineups[0]!,
      id: "without_qin",
      seats: seedLineups[0]!.seats.map((seat) =>
        seat.actorId === "qin_chuan"
          ? { ...seat, actorId: replacement.id }
          : seat,
      ),
    };
    const game = createGameFromLineup({
      gameId: "without_qin" as GameId,
      title: "无秦川对局",
      createdAt,
      ruleset: createDefaultRuleset(),
      lineup,
      presenter: seedPresenters[0]!,
      script: seedScripts[0]!,
      actors: seedActors,
      runMode: "game",
    });
    expect(game.players.some((player) => player.actor.sourceId === "qin_chuan")).toBe(false);
  });

  it("creates a valid seed game through the same path", () => {
    expect(createSeedGame({ gameId: "seed" as GameId, createdAt }).players).toHaveLength(12);
  });

  it("keeps one Actor contract unchanged across every Rule Role", () => {
    const sourceActor = seedActors.find((actor) => actor.id === "qin_chuan")!;
    const expectedCard = compileActorRuntimeCard(sourceActor);

    for (const ruleRoleId of RULE_ROLE_IDS) {
      const sourceSeat = seedLineups[0]!.seats.find(
        (seat) => seat.actorId === sourceActor.id,
      )!;
      const targetSeat = seedLineups[0]!.seats.find(
        (seat) => seat.ruleRoleId === ruleRoleId,
      )!;
      const lineup = {
        ...seedLineups[0]!,
        id: `qin_as_${ruleRoleId}`,
        seats: seedLineups[0]!.seats.map((seat) => {
          if (seat.seatNo === sourceSeat.seatNo) {
            return { ...seat, actorId: targetSeat.actorId };
          }
          if (seat.seatNo === targetSeat.seatNo) {
            return { ...seat, actorId: sourceSeat.actorId };
          }
          return seat;
        }),
      };
      const game = createGameFromLineup({
        gameId: `qin_as_${ruleRoleId}` as GameId,
        title: `秦川 ${ruleRoleId}`,
        createdAt,
        ruleset: createDefaultRuleset(),
        lineup,
        presenter: seedPresenters[0]!,
        script: seedScripts[0]!,
        actors: seedActors,
        runMode: "game",
      });
      const player = game.players.find(
        (candidate) => candidate.actor.sourceId === sourceActor.id,
      )!;

      expect(player.ruleRole.id).toBe(ruleRoleId);
      expect(compileActorRuntimeCard(player.actor)).toEqual(expectedCard);
    }
  });
});
