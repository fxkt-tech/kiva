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
import type { GameRunMode } from "@/core/game-run-mode";
import {
  actorBriefForStep,
  assertEpisodeScriptMatchesGame,
  assertEpisodeDraftMatchesStep,
  episodeInputHash,
  episodeScriptReport,
  planNextEpisodeDraft,
  type EpisodeAuthorWorkspace,
  type EpisodeScriptState,
} from "@/core/episode-script";
import {
  authorEpisodeScript,
  createEpisodeAuthorWorkspace,
  EpisodeAuthoringError,
} from "@/core/episode-author";
import type { ModelBindingSnapshot } from "@/core/player";
import {
  createDefaultRuleset,
  type DraftId,
  type GameId,
} from "@/core/types";
import { createDraftId, createEventId, createGameId } from "@/core/id";
import type { LlmClient } from "@/core/llm";
import { isLlmSpeechDraft } from "@/core/llm-task-specs";
import { generateSpeechDraft } from "@/core/speech-generation";
import { evaluateSpeech, speechBudgetForDraft } from "@/core/speech-budget";
import { generateActionDraft } from "@/core/action-generation";
import { getLegalNightTargets } from "@/core/rules";
import { deriveGameState } from "@/core/state";
import { seedCharacters } from "@/seeds/characters";
import { seedPresets } from "@/seeds/presets";
import { seedPresenters } from "@/seeds/presenters";
import { seedRoles } from "@/seeds/roles";
import { seedScripts } from "@/seeds/scripts";
import {
  GAME_RECORD_SCHEMA_VERSION,
  type GameRecord,
  type GameRepository,
} from "./game-repository";
import type { LibraryRecord, LibraryRepository } from "./library-repository";

export type GameActions = ReturnType<typeof createGameActions>;

export type CreateGameActionsOptions = {
  readonly llmClient?: LlmClient;
  readonly episodeAuthorModelBinding?: ModelBindingSnapshot;
  readonly libraryRepository?: LibraryRepository;
  readonly defaultPresetId?: string;
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
    scriptId?: string,
    runMode: GameRunMode = "game",
  ): Promise<GameRecord> {
    const createdAt = now();
    const library = await loadGameLibrary(options.libraryRepository);
    const preset = library.presets.find((item) => item.id === presetId);
    if (!preset) {
      throw new Error(`Game preset not found: ${presetId}`);
    }
    const presenter = requireEnabledPresenter(library, presenterId);
    const script = requireEnabledScript(library, scriptId);
    const game = createGameFromPreset({
      gameId: createGameId(),
      title: preset.name,
      createdAt,
      ruleset: createDefaultRuleset(),
      preset,
      presenter,
      script,
      roles: library.roles,
      characters: library.characters,
      runMode,
    });
    const record: GameRecord = {
      schemaVersion: GAME_RECORD_SCHEMA_VERSION,
      game,
      events: [],
      draft: null,
      generations: [],
      voiceArtifactsByEventId: {},
      episodeScript: runMode === "scripted" ? { status: "idle" } : null,
    };

    await repository.save(record);
    return record;
  }

  async function createGameFromPresetRecord(
    preset: LibraryRecord["presets"][number],
    library: Pick<LibraryRecord, "roles" | "characters" | "presenters" | "scripts">,
    presenterId: string,
    scriptId?: string,
    runMode: GameRunMode = "game",
  ): Promise<GameRecord> {
    const createdAt = now();
    const presenter = requireEnabledPresenter(library, presenterId);
    const script = requireEnabledScript(library, scriptId);
    const game = createGameFromPreset({
      gameId: createGameId(),
      title: preset.name,
      createdAt,
      ruleset: createDefaultRuleset(),
      preset,
      presenter,
      script,
      roles: library.roles,
      characters: library.characters,
      runMode,
    });
    const record: GameRecord = {
      schemaVersion: GAME_RECORD_SCHEMA_VERSION,
      game,
      events: [],
      draft: null,
      generations: [],
      voiceArtifactsByEventId: {},
      episodeScript: runMode === "scripted" ? { status: "idle" } : null,
    };

    await repository.save(record);
    return record;
  }

  async function startEpisodeScriptGeneration(gameId: GameId): Promise<string> {
    const jobId = `episode_job_${randomUUID()}`;
    const startedAt = now();
    await repository.withGameLock(gameId, async () => {
      const record = await loadGame(gameId);
      if (record.game.runMode !== "scripted") {
        throw new Error("Episode scripts are only available in scripted mode");
      }
      if (getActiveEvents(record.events).length > 0 || record.draft) {
        throw new Error("Cannot generate an episode script after game execution starts");
      }
      if (record.episodeScript?.status === "approved") {
        throw new Error("Approved episode script cannot be regenerated");
      }
      const nextRecord: GameRecord = {
        ...record,
        game: { ...record.game, updatedAt: startedAt },
        episodeScript: {
          status: "generating",
          jobId,
          startedAt,
          workspace: resumableEpisodeAuthorWorkspace(
            record.episodeScript,
            episodeInputHash(record.game),
          ) ??
            createEpisodeAuthorWorkspace({
              game: record.game,
              modelBinding: options.episodeAuthorModelBinding,
            }),
          requests: episodeAuthorRequests(record.episodeScript),
        },
      };
      await repository.save(nextRecord);
    });
    return jobId;
  }

  async function runEpisodeScriptGeneration(
    gameId: GameId,
    jobId: string,
  ): Promise<GameRecord> {
    const generating = await loadGame(gameId);
    if (
      generating.episodeScript?.status !== "generating" ||
      generating.episodeScript.jobId !== jobId
    ) {
      return generating;
    }

    try {
      if (!options.llmClient) {
        throw new Error("Script Author LLM is not configured");
      }
      const authored = await authorEpisodeScript({
        game: generating.game,
        llmClient: options.llmClient,
        createdAt: now(),
        workspace: generating.episodeScript.workspace,
        onCheckpoint: async ({ workspace, request }) => {
          await repository.withGameLock(gameId, async () => {
            const current = await loadGame(gameId);
            if (
              current.episodeScript?.status !== "generating" ||
              current.episodeScript.jobId !== jobId
            ) {
              throw new StaleEpisodeAuthorJobError();
            }
            await repository.save({
              ...current,
              game: { ...current.game, updatedAt: now() },
              episodeScript: {
                ...current.episodeScript,
                workspace,
                requests: [...current.episodeScript.requests, request],
              },
            });
          });
        },
      });
      return repository.withGameLock(gameId, async () => {
        const current = await loadGame(gameId);
        if (
          current.episodeScript?.status !== "generating" ||
          current.episodeScript.jobId !== jobId
        ) {
          return current;
        }
        const report = episodeScriptReport(authored.script);
        const nextRecord: GameRecord = {
          ...current,
          game: { ...current.game, updatedAt: now() },
          episodeScript: {
            status: "review",
            jobId,
            candidate: authored.script,
            report,
            requests: current.episodeScript.requests,
          },
        };
        await repository.save(nextRecord);
        return nextRecord;
      });
    } catch (error) {
      if (error instanceof StaleEpisodeAuthorJobError) {
        return loadGame(gameId);
      }
      return repository.withGameLock(gameId, async () => {
        const current = await loadGame(gameId);
        if (
          current.episodeScript?.status !== "generating" ||
          current.episodeScript.jobId !== jobId
        ) {
          return current;
        }
        const nextRecord: GameRecord = {
          ...current,
          game: { ...current.game, updatedAt: now() },
          episodeScript: {
            status: "failed",
            jobId,
            error: error instanceof Error ? error.message : String(error),
            failedAt: now(),
            workspace:
              error instanceof EpisodeAuthoringError
                ? error.workspace
                : current.episodeScript.workspace,
            requests: current.episodeScript.requests,
          },
        };
        await repository.save(nextRecord);
        return nextRecord;
      });
    }
  }

  async function generateEpisodeScript(gameId: GameId): Promise<GameRecord> {
    const jobId = await startEpisodeScriptGeneration(gameId);
    return runEpisodeScriptGeneration(gameId, jobId);
  }

  return {
    startEpisodeScriptGeneration,
    runEpisodeScriptGeneration,
    generateEpisodeScript,
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
      return createGameFromPresetRecord(preset, library, presenterId, undefined, "game");
    },

    async createGameFromPresetId(
      presetId: string,
      presenterId: string,
      scriptId?: string,
      runMode: GameRunMode = "game",
    ): Promise<GameRecord> {
      return createGameFromPresetId(presetId, presenterId, scriptId, runMode);
    },

    async createGameFromPresetRecord(
      preset: LibraryRecord["presets"][number],
      library: Pick<LibraryRecord, "roles" | "characters" | "presenters" | "scripts">,
      presenterId: string,
      scriptId?: string,
      runMode: GameRunMode = "game",
    ): Promise<GameRecord> {
      return createGameFromPresetRecord(
        preset,
        library,
        presenterId,
        scriptId,
        runMode,
      );
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
        if (
          record.game.runMode === "scripted" &&
          record.episodeScript?.status !== "approved"
        ) {
          throw new Error("Scripted game cannot advance before script approval");
        }
        if (record.draft) {
          return record;
        }

        const updatedAt = now();
        const plannedDraft = record.game.runMode === "scripted"
          ? planScriptedDraft(record, updatedAt)
          : planNextDraft({
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

    async approveEpisodeScript(
      gameId: GameId,
      expectedJobId: string,
      expectedScriptId: string,
    ): Promise<GameRecord> {
      return repository.withGameLock(gameId, async () => {
        const record = await loadGame(gameId);
        const state = record.episodeScript;
        if (
          state?.status !== "review" ||
          state.jobId !== expectedJobId ||
          state.candidate.id !== expectedScriptId
        ) {
          throw new Error("Episode script candidate changed before approval");
        }
        if (getActiveEvents(record.events).length > 0 || record.draft) {
          throw new Error("Cannot approve an episode script after execution starts");
        }
        assertEpisodeScriptMatchesGame({
          game: record.game,
          script: state.candidate,
        });
        if (!state.report.valid) {
          throw new Error("Episode script validation report did not pass");
        }
        const approvedAt = now();
        const nextRecord: GameRecord = {
          ...record,
          game: { ...record.game, updatedAt: approvedAt },
          episodeScript: {
            status: "approved",
            jobId: state.jobId,
            script: structuredClone(state.candidate),
            report: structuredClone(state.report),
            approvedAt,
            requests: structuredClone(state.requests),
          },
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

        validateSpeechDraft(record, record.draft);
        validateEpisodeDraft(record, record.draft);
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
          schemaVersion: GAME_RECORD_SCHEMA_VERSION,
          game: updatedGame,
          events: nextEvents,
          draft: null,
          generations: record.generations,
          voiceArtifactsByEventId: record.voiceArtifactsByEventId,
          episodeScript: record.episodeScript,
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
        validateScriptedEdit(record, edit);
        const editedDraft = applyDraftPayloadEdit(record.draft, edit);
        validateSpeechDraft(record, editedDraft);
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
        if (
          record.game.runMode === "scripted" &&
          !isLlmSpeechDraft(record.draft)
        ) {
          return record;
        }

        const updatedAt = now();
        const generatedDraft = await maybeGenerateDraft({
          record,
          draft: record.draft,
          llmClient: options.llmClient,
          createdAt: updatedAt,
          actorBrief:
            record.episodeScript?.status === "approved"
              ? actorBriefForStep(
                  record.episodeScript.script,
                  getActiveEvents(record.events).length + 1,
                )
              : null,
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
          schemaVersion: GAME_RECORD_SCHEMA_VERSION,
          game: { ...record.game, updatedAt },
          events: rollbackAfterIndex(record.events, index),
          draft: null,
          generations: record.generations,
          voiceArtifactsByEventId: Object.fromEntries(
            Object.entries(record.voiceArtifactsByEventId).filter(([eventId]) =>
              rollbackAfterIndex(record.events, index).some(
                (event) => event.id === eventId,
              ),
            ),
          ),
          episodeScript: record.episodeScript,
        };

        await repository.save(nextRecord);
        return nextRecord;
      });
    },
  };
}

function validateSpeechDraft(
  record: GameRecord,
  draft: NonNullable<GameRecord["draft"]>,
): void {
  if (!isLlmSpeechDraft(draft)) return;

  const hasPriorDaySpeech =
    draft.type === "day_speech_given" &&
    getActiveEvents(record.events).some(
      (event) =>
        event.type === "day_speech_given" &&
        event.payload.dayNumber === draft.payload.dayNumber,
    );
  const scriptedBudget =
    record.episodeScript?.status === "approved"
      ? actorBriefForStep(
          record.episodeScript.script,
          getActiveEvents(record.events).length + 1,
        )?.budget
      : null;
  const budget =
    scriptedBudget ?? speechBudgetForDraft({ draft, hasPriorDaySpeech });
  const evaluation = evaluateSpeech(draft.payload.text, budget);
  if (!evaluation.withinHardLimit) {
    throw new Error(
      `Speech text exceeds hard limit: ${evaluation.characterCount} > ${budget.hardMaxCharacters} non-whitespace characters`,
    );
  }
}

function validateScriptedEdit(record: GameRecord, edit: DraftPayloadEdit): void {
  if (record.game.runMode !== "scripted") return;
  if (
    !record.draft ||
    !isLlmSpeechDraft(record.draft) ||
    Object.keys(edit).some((key) => key !== "text")
  ) {
    throw new Error("Scripted game structure is locked; only speech text can be edited");
  }
}

function validateEpisodeDraft(
  record: GameRecord,
  draft: NonNullable<GameRecord["draft"]>,
): void {
  if (record.game.runMode !== "scripted") return;
  if (record.episodeScript?.status !== "approved") {
    throw new Error("Scripted game cannot confirm before script approval");
  }
  assertEpisodeDraftMatchesStep({
    script: record.episodeScript.script,
    activeEventCount: getActiveEvents(record.events).length,
    draft,
  });
}

function planScriptedDraft(
  record: GameRecord,
  createdAt: string,
): NonNullable<GameRecord["draft"]> | null {
  if (record.episodeScript?.status !== "approved") {
    throw new Error("Scripted game cannot advance before script approval");
  }
  return planNextEpisodeDraft({
    game: record.game,
    events: record.events,
    script: record.episodeScript.script,
    draftId: createDraftId(),
    createdAt,
  }).draft;
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
    scripts: seedScripts,
  };
}

function requireEnabledPresenter(
  library: Pick<LibraryRecord, "presenters">,
  presenterId: string,
): LibraryRecord["presenters"][number] {
  const enabled = library.presenters.filter((candidate) => candidate.enabled);
  if (enabled.length !== 1) {
    throw new Error(`Exactly one enabled presenter is required; found ${enabled.length}`);
  }
  const presenter = enabled[0]!;
  if (presenterId && presenter.id !== presenterId) {
    throw new Error(`Presenter not found or disabled: ${presenterId}`);
  }
  return presenter;
}

function requireEnabledScript(
  library: Pick<LibraryRecord, "scripts">,
  scriptId?: string,
): LibraryRecord["scripts"][number] {
  const enabled = library.scripts.filter((script) => script.enabled);
  const script = scriptId
    ? enabled.find((candidate) => candidate.id === scriptId)
    : enabled[0];
  if (!script) throw new Error(`Game script not found or disabled: ${scriptId ?? "default"}`);
  return script;
}

async function maybeGenerateDraft(input: {
  readonly record: GameRecord;
  readonly draft: NonNullable<GameRecord["draft"]>;
  readonly llmClient: LlmClient | undefined;
  readonly createdAt: string;
  readonly actorBrief?: import("@/core/episode-script").EpisodeActorBrief | null;
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
    actorBrief: input.actorBrief,
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

function episodeAuthorRequests(state: EpisodeScriptState | null) {
  return state && "requests" in state ? state.requests : [];
}

function resumableEpisodeAuthorWorkspace(
  state: EpisodeScriptState | null,
  inputHash: string,
): EpisodeAuthorWorkspace | null {
  if (
    (state?.status === "generating" || state?.status === "failed") &&
    state.workspace.inputHash === inputHash
  ) {
    return structuredClone(state.workspace);
  }
  return null;
}

class StaleEpisodeAuthorJobError extends Error {}

function now(): string {
  return new Date().toISOString();
}

function validateRollbackIndex(index: number): void {
  if (!Number.isInteger(index) || index < 0) {
    throw new Error(`Invalid rollback index: ${index}`);
  }
}
