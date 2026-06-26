import { planNextDraft } from "@/core/advance-planner";
import { confirmDraftEvent } from "@/core/drafts";
import {
  appendEvent,
  getActiveEvents,
  rollbackAfterIndex,
} from "@/core/event-log";
import { createSeedGame } from "@/core/game";
import { createDefaultRuleset, type GameId } from "@/core/types";
import { createDraftId, createEventId, createGameId } from "@/core/id";
import type { GameRecord, GameRepository } from "./game-repository";

export type GameActions = ReturnType<typeof createGameActions>;

type DraftDisplayInput = {
  readonly title: string;
  readonly text: string;
};

const mutationLocks = new Map<string, Promise<void>>();

export function createGameActions(repository: GameRepository) {
  async function loadGame(gameId: GameId): Promise<GameRecord> {
    const record = await repository.get(gameId);
    if (!record) {
      throw new Error(`Game not found: ${gameId}`);
    }

    return record;
  }

  return {
    async createGame(): Promise<GameRecord> {
      const createdAt = now();
      const game = createSeedGame({
        gameId: createGameId(),
        createdAt,
        ruleset: createDefaultRuleset(),
      });
      const record: GameRecord = { game, events: [], draft: null };

      await repository.save(record);
      return record;
    },

    async getGame(gameId: GameId): Promise<GameRecord | null> {
      return repository.get(gameId);
    },

    async listGames(): Promise<readonly GameRecord[]> {
      return repository.list();
    },

    async continueGame(gameId: GameId): Promise<GameRecord> {
      return withGameMutationLock(gameId, async () => {
        const record = await loadGame(gameId);
        if (record.draft) {
          return record;
        }

        const updatedAt = now();
        const nextRecord: GameRecord = {
          ...record,
          game: { ...record.game, updatedAt },
          draft: planNextDraft({
            game: record.game,
            events: record.events,
            draftId: createDraftId(),
            createdAt: updatedAt,
          }),
        };

        await repository.save(nextRecord);
        return nextRecord;
      });
    },

    async confirmDraft(gameId: GameId): Promise<GameRecord> {
      return withGameMutationLock(gameId, async () => {
        const record = await loadGame(gameId);
        if (!record.draft) {
          return record;
        }

        const updatedAt = now();
        const nextIndex = getActiveEvents(record.events).length + 1;
        const nextEvent = confirmDraftEvent({
          draft: record.draft,
          eventId: createEventId(nextIndex),
          index: nextIndex,
          createdAt: updatedAt,
        });
        const nextRecord: GameRecord = {
          game: { ...record.game, updatedAt },
          events: appendEvent(record.events, nextEvent),
          draft: null,
        };

        await repository.save(nextRecord);
        return nextRecord;
      });
    },

    async editDraftDisplay(
      gameId: GameId,
      display: DraftDisplayInput,
    ): Promise<GameRecord> {
      return withGameMutationLock(gameId, async () => {
        const record = await loadGame(gameId);
        if (!record.draft) {
          return record;
        }

        const updatedAt = now();
        const nextRecord: GameRecord = {
          ...record,
          game: { ...record.game, updatedAt },
          draft: {
            ...record.draft,
            display: {
              title: display.title.trim(),
              text: display.text.trim(),
            },
          },
        };

        await repository.save(nextRecord);
        return nextRecord;
      });
    },

    async deleteDraft(gameId: GameId): Promise<GameRecord> {
      return withGameMutationLock(gameId, async () => {
        const record = await loadGame(gameId);
        const updatedAt = now();
        const nextRecord: GameRecord = {
          ...record,
          game: { ...record.game, updatedAt },
          draft: null,
        };

        await repository.save(nextRecord);
        return nextRecord;
      });
    },

    async rollbackAfter(gameId: GameId, index: number): Promise<GameRecord> {
      validateRollbackIndex(index);

      return withGameMutationLock(gameId, async () => {
        const record = await loadGame(gameId);
        const updatedAt = now();
        const nextRecord: GameRecord = {
          game: { ...record.game, updatedAt },
          events: rollbackAfterIndex(record.events, index),
          draft: null,
        };

        await repository.save(nextRecord);
        return nextRecord;
      });
    },
  };
}

function now(): string {
  return new Date().toISOString();
}

async function withGameMutationLock<T>(
  gameId: GameId,
  operation: () => Promise<T>,
): Promise<T> {
  const key = gameId.toString();
  const previous = mutationLocks.get(key) ?? Promise.resolve();
  let releaseCurrent!: () => void;
  const current = new Promise<void>((resolve) => {
    releaseCurrent = resolve;
  });
  const tail = previous.catch(() => undefined).then(() => current);

  mutationLocks.set(key, tail);
  await previous.catch(() => undefined);

  try {
    return await operation();
  } finally {
    releaseCurrent();
    if (mutationLocks.get(key) === tail) {
      mutationLocks.delete(key);
    }
  }
}

function validateRollbackIndex(index: number): void {
  if (!Number.isInteger(index) || index < 0) {
    throw new Error(`Invalid rollback index: ${index}`);
  }
}
