import React from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import type { GameEvent } from "@/core/events";
import { createSeedGame } from "@/core/game";
import type { EventId, GameId } from "@/core/types";
import { EventTimeline } from "./event-timeline";

const gameId = "game_1" as GameId;
const createdAt = "2026-06-26T00:00:00.000Z";
const game = createSeedGame({ gameId, createdAt });

describe("EventTimeline", () => {
  it("renders newest events first", () => {
    const html = renderToStaticMarkup(
      React.createElement(EventTimeline, {
        gameId,
        events: [
          phaseStartedEvent(1),
          phaseStartedEvent(3),
          phaseStartedEvent(2),
        ],
        players: game.players,
      }),
    );

    expect(html.indexOf("#3")).toBeLessThan(html.indexOf("#2"));
    expect(html.indexOf("#2")).toBeLessThan(html.indexOf("#1"));
  });
});

function phaseStartedEvent(index: number): GameEvent {
  return {
    id: `event_${index}` as EventId,
    gameId,
    index,
    status: "active",
    type: "phase_started",
    phase: "night",
    visibility: { kind: "public" },
    payload: { phase: "night", dayNumber: index },
    createdAt: `2026-06-26T00:0${index}:00.000Z`,
  };
}
