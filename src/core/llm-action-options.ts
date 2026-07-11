import type { GameEvent } from "./events";
import { getActiveEvents } from "./event-log";
import type { Game } from "./game";
import type { LlmActionDraft } from "./llm-task-specs";
import { getEligibleVoters, getLegalNightTargets } from "./rules";
import { deriveGameState } from "./state";
import type { PlayerId } from "./types";

export type { LlmActionDraft } from "./llm-task-specs";

export type LegalActionOptions = {
  readonly targetPlayerIds: readonly PlayerId[];
  readonly allowNoTarget: boolean;
  readonly canUse?: boolean;
};

export function legalActionOptions(input: {
  readonly game: Game;
  readonly events: readonly GameEvent[];
  readonly draft: LlmActionDraft;
}): LegalActionOptions {
  const events = getActiveEvents(input.events);
  const state = deriveGameState(input.game.players, events);

  switch (input.draft.type) {
    case "guard_protect_selected": {
      const previousTargetId = previousGuardTargetId(
        events,
        state.dayNumber,
      );
      return {
        targetPlayerIds: getLegalNightTargets(
          "guard_protect",
          input.game.players,
          state.alivePlayerIds,
          input.draft.actorPlayerId,
        ).filter((targetPlayerId) =>
          input.game.ruleset.guardForbidConsecutiveSameTarget
            ? targetPlayerId !== previousTargetId
            : true,
        ),
        allowNoTarget: false,
      };
    }

    case "wolf_vote_cast":
      return {
        targetPlayerIds: getLegalNightTargets(
          "wolf_kill",
          input.game.players,
          state.alivePlayerIds,
          input.draft.actorPlayerId,
        ),
        allowNoTarget: false,
      };

    case "seer_check_selected": {
      const checkedPlayerIds = new Set(
        events
          .filter(
            (
              event,
            ): event is Extract<GameEvent, { type: "seer_check_result" }> =>
              event.type === "seer_check_result" &&
              event.actorPlayerId === input.draft.actorPlayerId,
          )
          .map((event) => event.payload.targetPlayerId),
      );
      return {
        targetPlayerIds: getLegalNightTargets(
          "seer_check",
          input.game.players,
          state.alivePlayerIds,
          input.draft.actorPlayerId,
        ).filter((targetPlayerId) => !checkedPlayerIds.has(targetPlayerId)),
        allowNoTarget: false,
      };
    }

    case "witch_antidote_decided": {
      const targetPlayerId = currentNightEvents(
        events,
        state.dayNumber,
      )
        .filter(
          (
            event,
          ): event is Extract<GameEvent, { type: "wolf_vote_resolved" }> =>
            event.type === "wolf_vote_resolved",
        )
        .at(-1)?.payload.targetPlayerId;
      const selfSaveForbidden =
        state.dayNumber === 1 &&
        targetPlayerId === input.draft.actorPlayerId &&
        !input.game.ruleset.witchFirstNightSelfSave;
      const canUse =
        state.witch.antidoteAvailable &&
        targetPlayerId !== null &&
        targetPlayerId !== undefined &&
        !selfSaveForbidden;

      return {
        targetPlayerIds: canUse ? [targetPlayerId] : [],
        allowNoTarget: true,
        canUse,
      };
    }

    case "witch_poison_decided": {
      const usedAntidoteThisNight = currentNightEvents(
        events,
        state.dayNumber,
      ).some(
        (event) =>
          event.type === "witch_antidote_decided" && event.payload.used,
      );
      const canUse =
        state.witch.poisonAvailable &&
        (input.game.ruleset.witchAllowSameNightAntidoteAndPoison ||
          !usedAntidoteThisNight);

      return {
        targetPlayerIds: canUse
          ? getLegalNightTargets(
              "witch_poison",
              input.game.players,
              state.alivePlayerIds,
              input.draft.actorPlayerId,
            )
          : [],
        allowNoTarget: true,
        canUse,
      };
    }

    case "hunter_shot_decided":
      return {
        targetPlayerIds: getLegalNightTargets(
          "hunter_shot",
          input.game.players,
          state.alivePlayerIds,
          input.draft.actorPlayerId,
        ),
        allowNoTarget: false,
      };

    case "vote_cast": {
      if (input.draft.payload.voteType === "sheriff") {
        return { targetPlayerIds: [], allowNoTarget: false };
      }

      const voterPlayerId = input.draft.payload.voterPlayerId;
      const targetPlayerIds =
        input.draft.payload.voteType === "pk"
          ? state.pk.status === "pending"
            ? state.pk.tiedPlayerIds.filter(
                (playerId) =>
                  state.alivePlayerIds.includes(playerId) &&
                  playerId !== voterPlayerId,
              )
            : []
          : state.alivePlayerIds.filter(
              (playerId) => playerId !== voterPlayerId,
            );

      return {
        targetPlayerIds,
        allowNoTarget: input.game.ruleset.allowAbstainVote,
      };
    }
  }
}

export function canActorUseActionOptions(input: {
  readonly game: Game;
  readonly events: readonly GameEvent[];
  readonly draft: LlmActionDraft;
}): boolean {
  const state = deriveGameState(input.game.players, input.events);
  const actorPlayerId =
    input.draft.type === "vote_cast"
      ? input.draft.payload.voterPlayerId
      : input.draft.actorPlayerId;
  const actor = input.game.players.find(
    (player) => player.playerId === actorPlayerId,
  );
  if (!actor) return false;

  switch (input.draft.type) {
    case "guard_protect_selected":
      return actor.gameRole === "guard" && actor.mechanicKey === "guard_protect";
    case "wolf_vote_cast":
      return actor.gameRole === "werewolf" && actor.mechanicKey === "wolf_kill";
    case "seer_check_selected":
      return actor.gameRole === "seer" && actor.mechanicKey === "seer_check";
    case "witch_antidote_decided":
    case "witch_poison_decided":
      return actor.gameRole === "witch" && actor.mechanicKey === "witch_medicine";
    case "hunter_shot_decided":
      return actor.gameRole === "hunter" && actor.mechanicKey === "hunter_shot";
    case "vote_cast": {
      if (!state.alivePlayerIds.includes(actor.playerId)) return false;
      if (input.draft.payload.voteType !== "pk") return true;
      if (state.pk.status !== "pending") return false;
      return getEligibleVoters({
        alivePlayerIds: state.alivePlayerIds,
        voteType: "pk",
        pkPlayerIds: state.pk.tiedPlayerIds,
        pkVoters: input.game.ruleset.pkVoters,
      }).includes(actor.playerId);
    }
  }
}

function currentNightEvents(
  events: readonly GameEvent[],
  dayNumber: number,
): readonly GameEvent[] {
  const startIndex = currentNightStartIndex(events, dayNumber);
  return startIndex >= 0 ? events.slice(startIndex + 1) : [];
}

function previousGuardTargetId(
  events: readonly GameEvent[],
  currentDayNumber: number,
): PlayerId | null {
  const currentNightStart = currentNightStartIndex(events, currentDayNumber);
  const priorEvents =
    currentNightStart >= 0 ? events.slice(0, currentNightStart) : events;
  return (
    [...priorEvents]
      .reverse()
      .find(
        (
          event,
        ): event is Extract<GameEvent, { type: "guard_protect_selected" }> =>
          event.type === "guard_protect_selected",
      )?.payload.targetPlayerId ?? null
  );
}

function currentNightStartIndex(
  events: readonly GameEvent[],
  dayNumber: number,
): number {
  for (let index = events.length - 1; index >= 0; index -= 1) {
    const event = events[index];
    if (
      event?.type === "phase_started" &&
      event.payload.phase === "night" &&
      event.payload.dayNumber === dayNumber
    ) {
      return index;
    }
  }

  return -1;
}
