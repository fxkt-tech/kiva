import { getActiveEvents } from "./event-log";
import type { GameEvent } from "./events";
import type { PlayerSnapshot } from "./player";
import type { Phase, PlayerId } from "./types";

export type DerivedGameState = {
  readonly currentPhase: Phase;
  readonly dayNumber: number;
  readonly alivePlayerIds: readonly PlayerId[];
  readonly deadPlayerIds: readonly PlayerId[];
  readonly witch: {
    readonly antidoteAvailable: boolean;
    readonly poisonAvailable: boolean;
  };
};

export function deriveGameState(
  players: readonly PlayerSnapshot[],
  events: readonly GameEvent[],
): DerivedGameState {
  let currentPhase: Phase = "setup";
  let dayNumber = 0;
  const dead = new Set<PlayerId>();
  let antidoteAvailable = true;
  let poisonAvailable = true;

  for (const event of getActiveEvents(events)) {
    if (event.type === "phase_started") {
      currentPhase = event.payload.phase;
      dayNumber = event.payload.dayNumber;
    }

    if (event.type === "night_resolved") {
      for (const playerId of event.payload.deadPlayerIds) {
        dead.add(playerId);
      }
    }

    if (event.type === "witch_antidote_decided" && event.payload.used) {
      antidoteAvailable = false;
    }

    if (event.type === "witch_poison_decided" && event.payload.used) {
      poisonAvailable = false;
    }
  }

  const allPlayerIds = players.map((player) => player.playerId);

  return {
    currentPhase,
    dayNumber,
    alivePlayerIds: allPlayerIds.filter((playerId) => !dead.has(playerId)),
    deadPlayerIds: allPlayerIds.filter((playerId) => dead.has(playerId)),
    witch: {
      antidoteAvailable,
      poisonAvailable,
    },
  };
}
