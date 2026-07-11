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
import { createGameFromPreset } from "@/core/game";
import {
  createDefaultRuleset,
  type DraftId,
  type GameId,
} from "@/core/types";
import { createDraftId, createEventId, createGameId } from "@/core/id";
import type { LlmClient } from "@/core/llm";
import type { LlmPromptMode } from "@/core/prompt-builders";
import { generateSpeechDraft } from "@/core/speech-generation";
import { generateActionDraft } from "@/core/action-generation";
import { getLegalNightTargets } from "@/core/rules";
import { deriveGameState } from "@/core/state";
import { seedCharacters } from "@/seeds/characters";
import { seedPresets } from "@/seeds/presets";
import { seedPresenters } from "@/seeds/presenters";
import { seedRoles } from "@/seeds/roles";
import type { GameRecord, GameRepository } from "./game-repository";
import type { LibraryRecord, LibraryRepository } from "./library-repository";

export type GameActions = ReturnType<typeof createGameActions>;

export type CreateGameActionsOptions = {
  readonly llmClient?: LlmClient;
  readonly libraryRepository?: LibraryRepository;
  readonly defaultPresetId?: string;
  readonly promptMode?: LlmPromptMode;
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

  async function createGameFromPresetId(
    presetId: string,
    presenterId: string,
  ): Promise<GameRecord> {
    const createdAt = now();
    const library = await loadGameLibrary(options.libraryRepository);
    const preset = library.presets.find((item) => item.id === presetId);
    if (!preset) {
      throw new Error(`Game preset not found: ${presetId}`);
    }
    const presenter = requireEnabledPresenter(library, presenterId);
    const game = createGameFromPreset({
      gameId: createGameId(),
      title: preset.name,
      createdAt,
      ruleset: createDefaultRuleset(),
      preset,
      presenter,
      roles: library.roles,
      characters: library.characters,
    });
    const record: GameRecord = {
      game,
      events: [],
      draft: null,
      generations: [],
    };

    await repository.save(record);
    return record;
  }

  async function createGameFromPresetRecord(
    preset: LibraryRecord["presets"][number],
    library: Pick<LibraryRecord, "roles" | "characters" | "presenters">,
    presenterId: string,
  ): Promise<GameRecord> {
    const createdAt = now();
    const presenter = requireEnabledPresenter(library, presenterId);
    const game = createGameFromPreset({
      gameId: createGameId(),
      title: preset.name,
      createdAt,
      ruleset: createDefaultRuleset(),
      preset,
      presenter,
      roles: library.roles,
      characters: library.characters,
    });
    const record: GameRecord = {
      game,
      events: [],
      draft: null,
      generations: [],
    };

    await repository.save(record);
    return record;
  }

  return {
    async createGame(): Promise<GameRecord> {
      const presetId = options.defaultPresetId ?? "twelve_player_standard";
      const library = await loadGameLibrary(options.libraryRepository);
      const preset = library.presets.find((candidate) => candidate.id === presetId);
      if (!preset) {
        throw new Error(`Game preset not found: ${presetId}`);
      }
      const presenterId = library.presenters[0]?.id;
      if (!presenterId) {
        throw new Error("No presenter definitions available");
      }
      return createGameFromPresetRecord(preset, library, presenterId);
    },

    async createGameFromPresetId(
      presetId: string,
      presenterId: string,
    ): Promise<GameRecord> {
      return createGameFromPresetId(presetId, presenterId);
    },

    async createGameFromPresetRecord(
      preset: LibraryRecord["presets"][number],
      library: Pick<LibraryRecord, "roles" | "characters" | "presenters">,
      presenterId: string,
    ): Promise<GameRecord> {
      return createGameFromPresetRecord(preset, library, presenterId);
    },

    async getGame(gameId: GameId): Promise<GameRecord | null> {
      return repository.get(gameId);
    },

    async listGames(): Promise<readonly GameRecord[]> {
      return repository.list();
    },

    async renameGame(gameId: GameId, title: string): Promise<GameRecord> {
      const normalizedTitle = title.trim();
      if (normalizedTitle.length === 0) {
        throw new Error("Game title cannot be blank");
      }

      return repository.withGameLock(gameId, async () => {
        const record = await loadGame(gameId);
        const updatedRecord: GameRecord = {
          ...record,
          game: {
            ...record.game,
            title: normalizedTitle,
            updatedAt: now(),
          },
        };

        await repository.save(updatedRecord);
        return updatedRecord;
      });
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
        const nextRecord: GameRecord = {
          ...record,
          game: { ...record.game, updatedAt },
          draft: plannedDraft,
          generations: record.generations,
        };

        await repository.save(nextRecord);
        return nextRecord;
      });
    },

    async confirmDraft(
      gameId: GameId,
      expectedDraftId?: DraftId,
    ): Promise<GameRecord> {
      return repository.withGameLock(gameId, async () => {
        const record = await loadGame(gameId);
        if (
          !record.draft ||
          (expectedDraftId !== undefined && record.draft.id !== expectedDraftId)
        ) {
          return record;
        }

        validateWolfDraft(record, record.draft);

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
        const editedDraft = applyDraftPayloadEdit(record.draft, edit);
        validateWolfDraft(record, editedDraft);
        const nextRecord: GameRecord = {
          ...record,
          game: { ...record.game, updatedAt },
          draft: editedDraft,
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
          promptMode: options.promptMode,
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

    async deleteGame(gameId: GameId): Promise<void> {
      return repository.withGameLock(gameId, async () => {
        await repository.delete(gameId);
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

function validateWolfDraft(
  record: GameRecord,
  draft: NonNullable<GameRecord["draft"]>,
): void {
  const state = deriveGameState(record.game.players, record.events);
  if (draft.type === "wolf_leader_selected") {
    const leader = record.game.players.find(
      (player) => player.playerId === draft.payload.leaderPlayerId,
    );
    if (
      state.dayNumber !== 1 ||
      !leader ||
      leader.gameRole !== "werewolf" ||
      !state.alivePlayerIds.includes(leader.playerId)
    ) {
      throw new Error("Wolf leader must be an alive werewolf on the first night");
    }
  }
  if (draft.type === "wolf_vote_cast") {
    const legalTargets = getLegalNightTargets(
      "wolf_kill",
      record.game.players,
      state.alivePlayerIds,
      draft.actorPlayerId,
    );
    if (!legalTargets.includes(draft.payload.targetPlayerId)) {
      throw new Error("Wolf vote target must be an alive non-werewolf");
    }
  }
  if (draft.type === "wolf_vote_resolved") {
    if (!draft.payload.targetPlayerId) {
      throw new Error("Wolf vote tiebreak requires a host-selected target");
    }
    if (
      draft.payload.resolution === "host_tiebreak" &&
      !draft.payload.tiedTargetPlayerIds.includes(draft.payload.targetPlayerId)
    ) {
      throw new Error("Wolf vote tiebreak target must be tied for the highest vote");
    }
  }
}

async function loadGameLibrary(
  libraryRepository: LibraryRepository | undefined,
): Promise<LibraryRecord> {
  if (libraryRepository) {
    return libraryRepository.loadAll();
  }

  return {
    roles: seedRoles,
    characters: seedCharacters,
    presets: seedPresets,
    presenters: seedPresenters,
  };
}

function requireEnabledPresenter(
  library: Pick<LibraryRecord, "presenters">,
  presenterId: string,
): LibraryRecord["presenters"][number] {
  const presenter = library.presenters.find(
    (candidate) => candidate.id === presenterId,
  );
  if (!presenter) {
    throw new Error(`Presenter not found: ${presenterId}`);
  }
  if (!presenter.enabled) {
    throw new Error(`Presenter is disabled: ${presenterId}`);
  }
  return presenter;
}

async function maybeGenerateDraft(input: {
  readonly record: GameRecord;
  readonly draft: NonNullable<GameRecord["draft"]>;
  readonly llmClient: LlmClient | undefined;
  readonly createdAt: string;
  readonly promptMode?: LlmPromptMode;
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
    promptMode: input.promptMode,
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
    promptMode: input.promptMode,
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
