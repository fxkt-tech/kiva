import { describe, expect, it } from "vitest";
import type { GameEvent } from "../events";
import { createSeedGame } from "../game";
import { createGamePresenterSnapshot } from "../presenter-definition";
import { resolvePresenter } from "../presenter";
import type { EventId, GameId, PlayerId } from "../types";
import { seedPresenters } from "@/seeds/presenters";

const gameId = "presenter_game" as GameId;
const game = createSeedGame({
  gameId,
  createdAt: "2026-07-11T00:00:00.000Z",
});
const nightWatch = createGamePresenterSnapshot(seedPresenters[0]!);
const judge = createGamePresenterSnapshot(seedPresenters[1]!);

describe("presenter resolution", () => {
  it("resolves one event through distinct presenter-owned scripts", () => {
    const phase = event({
      type: "phase_started",
      payload: { phase: "night", dayNumber: 1 },
    });

    const atmospheric = resolvePresenter(nightWatch, phase, game.players);
    const directive = resolvePresenter(judge, phase, game.players);

    expect(atmospheric.transcriptText).toContain("所有玩家保持安静");
    expect(directive.transcriptText).toBe("天黑请闭眼。");
    expect(atmospheric.cue.copyKey).toBe("phase.night");
  });

  it("renders typed player labels into action templates", () => {
    const action = event({
      type: "wolf_vote_resolved",
      payload: {
        votes: [], tallies: [], tiedTargetPlayerIds: ["p4" as PlayerId],
        targetPlayerId: "p4" as PlayerId, resolution: "majority", dayNumber: 1,
      },
    });

    expect(resolvePresenter(nightWatch, action, game.players).transcriptText)
      .toContain("4号林夏");
  });

  it("keeps authored speech player-owned while resolving a ready presenter prompt", () => {
    const speech = event({
      type: "day_speech_given",
      phase: "speech",
      payload: {
        playerId: "p4" as PlayerId,
        text: "我认为三号的发言有问题。",
        dayNumber: 1,
        round: 1,
      },
    });

    expect(resolvePresenter(nightWatch, speech, game.players)).toMatchObject({
      presenterName: "守夜人",
      transcriptSpeaker: "player",
      transcriptText: "我认为三号的发言有问题。",
      cue: {
        copyKey: "prompt.speech",
        text: "4号玩家请发言。",
      },
    });
  });

  it("uses presenter-owned fallback copy for empty player speech", () => {
    const speech = event({
      type: "day_speech_given",
      phase: "speech",
      payload: {
        playerId: "p4" as PlayerId,
        text: "   ",
        dayNumber: 1,
        round: 1,
      },
    });

    expect(resolvePresenter(nightWatch, speech, game.players)).toMatchObject({
      transcriptSpeaker: "presenter",
      transcriptText: "该玩家选择沉默。沉默，有时也是一种答案。",
    });
  });

});

function event(overrides: Partial<GameEvent>): GameEvent {
  return {
    id: "event_presenter" as EventId,
    gameId,
    index: 1,
    status: "active",
    type: "phase_started",
    phase: "night",
    visibility: { kind: "public" },
    payload: { phase: "night", dayNumber: 1 },
    createdAt: "2026-07-11T00:00:00.000Z",
    ...overrides,
  } as GameEvent;
}
