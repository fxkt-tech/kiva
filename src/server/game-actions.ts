import { randomUUID } from "node:crypto";
import { planNextDraft } from "@/core/advance-planner";
import {
  applyDraftPayloadEdit,
  type DraftPayloadEdit,
} from "@/core/draft-edit";
import { confirmDraftEvent } from "@/core/drafts";
import {
  appendEvent,
  getActiveEvents,
  rollbackAfterIndex,
} from "@/core/event-log";
import { createSeedGame } from "@/core/game";
import { createDefaultRuleset, type GameId } from "@/core/types";
import { createDraftId, createEventId, createGameId } from "@/core/id";
import type { LlmClient } from "@/core/llm";
import { generateSpeechDraft } from "@/core/speech-generation";
import { generateActionDraft } from "@/core/action-generation";
import type { GameRecord, GameRepository } from "./game-repository";

export type GameActions = ReturnType<typeof createGameActions>;

export type CreateGameActionsOptions = {
  readonly llmClient?: LlmClient;
};

export function createGameActions(
  repository: GameRepository,
  options: CreateGameActionsOptions = {},
) {
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
      const record: GameRecord = {
        game,
        events: [],
        draft: null,
        generations: [],
      };

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
      return repository.withGameLock(gameId, async () => {
        const record = await loadGame(gameId);
        if (record.draft) {
          return record;
        }

        const updatedAt = now();
        const plannedDraft = planNextDraft({
          game: record.game,
          events: record.events,
          draftId: createDraftId(),
          createdAt: updatedAt,
        });
        const generatedDraft = plannedDraft
          ? await maybeGenerateDraft({
              record,
              draft: plannedDraft,
              llmClient: options.llmClient,
              createdAt: updatedAt,
            })
          : { draft: null, generation: null };
        const nextRecord: GameRecord = {
          ...record,
          game: { ...record.game, updatedAt },
          draft: generatedDraft.draft,
          generations: generatedDraft.generation
            ? [...record.generations, generatedDraft.generation]
            : record.generations,
        };

        await repository.save(nextRecord);
        return nextRecord;
      });
    },

    async confirmDraft(gameId: GameId): Promise<GameRecord> {
      return repository.withGameLock(gameId, async () => {
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
        const nextEvents = appendEvent(record.events, nextEvent);
        const updatedGame = { ...record.game, updatedAt };
        const confirmedRecord: GameRecord = {
          game: updatedGame,
          events: nextEvents,
          draft: null,
          generations: record.generations,
        };
        await repository.save(confirmedRecord);
        return confirmedRecord;
      });
    },

    async editDraftPayload(
      gameId: GameId,
      edit: DraftPayloadEdit,
    ): Promise<GameRecord> {
      return repository.withGameLock(gameId, async () => {
        const record = await loadGame(gameId);
        if (!record.draft) {
          return record;
        }

        const updatedAt = now();
        const nextRecord: GameRecord = {
          ...record,
          game: { ...record.game, updatedAt },
          draft: applyDraftPayloadEdit(record.draft, edit),
        };

        await repository.save(nextRecord);
        return nextRecord;
      });
    },

    async regenerateDraft(gameId: GameId): Promise<GameRecord> {
      return repository.withGameLock(gameId, async () => {
        const record = await loadGame(gameId);
        if (!record.draft || !options.llmClient) {
          return record;
        }

        const updatedAt = now();
        const generatedDraft = await maybeGenerateDraft({
          record,
          draft: record.draft,
          llmClient: options.llmClient,
          createdAt: updatedAt,
        });
        if (!generatedDraft.generation) {
          return record;
        }

        const nextRecord: GameRecord = {
          ...record,
          game: { ...record.game, updatedAt },
          draft: generatedDraft.draft,
          generations: [...record.generations, generatedDraft.generation],
        };

        await repository.save(nextRecord);
        return nextRecord;
      });
    },

    async deleteDraft(gameId: GameId): Promise<GameRecord> {
      return repository.withGameLock(gameId, async () => {
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

      return repository.withGameLock(gameId, async () => {
        const record = await loadGame(gameId);
        const updatedAt = now();
        const nextRecord: GameRecord = {
          game: { ...record.game, updatedAt },
          events: rollbackAfterIndex(record.events, index),
          draft: null,
          generations: record.generations,
        };

        await repository.save(nextRecord);
        return nextRecord;
      });
    },
  };
}

async function maybeGenerateDraft(input: {
  readonly record: GameRecord;
  readonly draft: NonNullable<GameRecord["draft"]>;
  readonly llmClient: LlmClient | undefined;
  readonly createdAt: string;
}) {
  if (!input.llmClient) {
    return { draft: input.draft, generation: null };
  }

  const speechResult = await generateSpeechDraft({
    game: input.record.game,
    events: input.record.events,
    draft: input.draft,
    llmClient: input.llmClient,
    generationId: createGenerationId(),
    createdAt: input.createdAt,
  });
  if (speechResult.generation) {
    return speechResult;
  }

  return generateActionDraft({
    game: input.record.game,
    events: input.record.events,
    draft: input.draft,
    llmClient: input.llmClient,
    generationId: createGenerationId(),
    createdAt: input.createdAt,
  });
}

function createGenerationId(): string {
  return `generation_${randomUUID()}`;
}

function now(): string {
  return new Date().toISOString();
}

function validateRollbackIndex(index: number): void {
  if (!Number.isInteger(index) || index < 0) {
    throw new Error(`Invalid rollback index: ${index}`);
  }
}
