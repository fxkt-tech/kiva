import { mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { afterEach, describe, expect, it } from "vitest";
import { getActiveEvents } from "@/core/event-log";
import { MockLlmClient, type LlmClient } from "@/core/llm";
import type { GameId } from "@/core/types";
import { seedCharacters } from "@/seeds/characters";
import { seedPresets } from "@/seeds/presets";
import { seedRoles } from "@/seeds/roles";
import { seedPresenters } from "@/seeds/presenters";
import { createGameActions } from "../game-actions";
import {
  createGameRepository,
  type GameRecord,
  type GameRepository,
} from "../game-repository";
import type { LibraryRepository } from "../library-repository";

const tempDirs: string[] = [];
const defaultPresetId = "twelve_player_standard";
const zhouCharacter = seedCharacters.find((character) => character.id === "zhou_zhi")!;

afterEach(async () => {
  await Promise.all(
    tempDirs.splice(0).map((dir) => rm(dir, { recursive: true, force: true })),
  );
});

async function createTempDir(): Promise<string> {
  const dir = await mkdtemp(join(tmpdir(), "kiva-game-actions-"));
  tempDirs.push(dir);
  return dir;
}

async function createActions() {
  const repository = createGameRepository(await createTempDir());
  return { actions: createGameActions(repository), repository };
}

describe("game actions", () => {
  it("creates a game record and it can be retrieved", async () => {
    const { actions, repository } = await createActions();

    const created = await actions.createGame();

    expect(created).toMatchObject({
      game: {
        id: expect.stringMatching(
          /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/,
        ),
      },
      events: [],
      draft: null,
    });
    expect(created.game.players[0]).toMatchObject({
      characterSourceId: "zhou_zhi",
      roleSourceId: "villager",
      characterSystemPromptSnapshot: zhouCharacter.systemPrompt,
    });
    await expect(repository.get(created.game.id)).resolves.toEqual(created);
    await expect(actions.getGame(created.game.id)).resolves.toEqual(created);
  });

  it("deletes a game record", async () => {
    const { actions, repository } = await createActions();
    const created = await actions.createGame();

    await actions.deleteGame(created.game.id);

    await expect(repository.get(created.game.id)).resolves.toBeNull();
    await expect(actions.getGame(created.game.id)).resolves.toBeNull();
    await expect(actions.listGames()).resolves.toEqual([]);
  });

  it("renames a game record", async () => {
    const { actions, repository } = await createActions();
    const created = await actions.createGame();

    const renamed = await actions.renameGame(created.game.id, "  新标题  ");

    expect(renamed.game.title).toBe("新标题");
    expect(renamed.game.updatedAt).not.toBe(created.game.updatedAt);
    await expect(repository.get(created.game.id)).resolves.toMatchObject({
      game: {
        id: created.game.id,
        title: "新标题",
        updatedAt: renamed.game.updatedAt,
      },
    });
  });

  it("rejects blank game names", async () => {
    const { actions } = await createActions();
    const created = await actions.createGame();

    await expect(actions.renameGame(created.game.id, "   ")).rejects.toThrow(
      "Game title cannot be blank",
    );
  });

  it("creates a game from an injected role library repository", async () => {
    const repository = createGameRepository(await createTempDir());
    let loadAllCalls = 0;
    const libraryRepository = fakeLibraryRepository({
      loadAll: async () => {
        loadAllCalls += 1;
        return {
          roles: seedRoles,
          characters: seedCharacters,
          presets: seedPresets,
          presenters: seedPresenters,
        };
      },
    });
    const actions = createGameActions(repository, { libraryRepository });

    const created = await actions.createGame();

    expect(loadAllCalls).toBe(1);
    expect(created.game.title).toBe("12人狼人杀标准局");
    expect(created.game.players.map((player) => player.name)).toEqual([
      "周知",
      "陈墨",
      "秦川",
      "林夏",
      "夏宇",
      "顾清妍",
      "沈岚",
      "许砚",
      "白祁",
      "唐棠",
      "陆昭",
      "苏瑾",
    ]);
    expect(created.game.players[0]).toMatchObject({
      characterSourceId: "zhou_zhi",
      roleSourceId: "villager",
      characterSystemPromptSnapshot: zhouCharacter.systemPrompt,
      mechanicKey: "none",
      team: "villager",
    });
    await expect(repository.get(created.game.id)).resolves.toEqual(created);
  });

  it("throws when the configured default preset id is not in the library", async () => {
    const repository = createGameRepository(await createTempDir());
    const libraryRepository = fakeLibraryRepository({
      loadAll: async () => ({
        roles: seedRoles,
        characters: seedCharacters,
        presets: seedPresets,
          presenters: seedPresenters,
      }),
    });
    const actions = createGameActions(repository, {
      libraryRepository,
      defaultPresetId: "missing_preset",
    });

    await expect(actions.createGame()).rejects.toThrow(
      "Game preset not found: missing_preset",
    );
  });

  it("creates a game from an explicit preset id", async () => {
    const repository = createGameRepository(await createTempDir());
    const libraryRepository = fakeLibraryRepository({
      loadAll: async () => ({
        roles: seedRoles,
        characters: seedCharacters,
        presets: seedPresets,
          presenters: seedPresenters,
      }),
    });
    const actions = createGameActions(repository, { libraryRepository });

    const created = await actions.createGameFromPresetId(
      defaultPresetId,
      seedPresenters[0]!.id,
    );

    expect(created.game.title).toBe("12人狼人杀标准局");
    expect(created.game.players).toHaveLength(12);
    await expect(repository.get(created.game.id)).resolves.toEqual(created);
  });

  it("confirms the current draft into one official event without planning the next draft", async () => {
    const { actions, repository } = await createActions();
    const created = await actions.createGame();

    const withDraft = await actions.continueGame(created.game.id);
    expect(withDraft.draft).toMatchObject({ id: expect.stringMatching(/^draft_/) });
    expect(withDraft.events).toHaveLength(0);

    const confirmed = await actions.confirmDraft(created.game.id);

    expect(confirmed.draft).toBeNull();
    expect(getActiveEvents(confirmed.events)).toHaveLength(1);
    expect(confirmed.events[0]).toMatchObject({
      id: expect.stringMatching(/^event_1_/),
      index: 1,
      status: "active",
      createdFromDraftId: withDraft.draft?.id,
    });
    await expect(repository.get(created.game.id)).resolves.toMatchObject({
      game: { id: confirmed.game.id, updatedAt: confirmed.game.updatedAt },
      events: confirmed.events.map((event) => ({
        id: event.id,
        index: event.index,
        status: event.status,
        type: event.type,
      })),
      draft: null,
      generations: [],
    });

    const withNextDraft = await actions.continueGame(created.game.id);
    expect(withNextDraft.draft).toMatchObject({
      id: expect.stringMatching(/^draft_/),
      type: "role_assigned",
    });
  });

  it("does not overwrite an existing draft when continuing again", async () => {
    const { actions } = await createActions();
    const created = await actions.createGame();

    const firstContinue = await actions.continueGame(created.game.id);
    const secondContinue = await actions.continueGame(created.game.id);

    expect(secondContinue.draft).toEqual(firstContinue.draft);
  });

  it("supersedes active events after selected index and clears draft on rollback", async () => {
    const { actions, repository } = await createActions();
    const created = await actions.createGame();
    await actions.continueGame(created.game.id);
    await actions.confirmDraft(created.game.id);
    await actions.continueGame(created.game.id);
    const beforeRollback = await actions.confirmDraft(created.game.id);
    expect(getActiveEvents(beforeRollback.events).map((event) => event.index)).toEqual([
      1,
      2,
    ]);

    await actions.continueGame(created.game.id);
    const rolledBack = await actions.rollbackAfter(created.game.id, 1);

    expect(rolledBack.draft).toBeNull();
    expect(rolledBack.events.map((event) => [event.index, event.status])).toEqual([
      [1, "active"],
      [2, "superseded"],
    ]);
    await expect(repository.get(created.game.id)).resolves.toMatchObject({
      game: { id: rolledBack.game.id, updatedAt: rolledBack.game.updatedAt },
      events: rolledBack.events.map((event) => ({
        id: event.id,
        index: event.index,
        status: event.status,
        type: event.type,
      })),
      draft: null,
      generations: [],
    });
  });

  it("updates current draft payload before confirmation", async () => {
    const { actions } = await createActions();
    const created = await actions.createGame();

    for (let step = 0; step < created.game.players.length + 2; step += 1) {
      await actions.continueGame(created.game.id);
      await actions.confirmDraft(created.game.id);
    }

    const withWolfKillDraft = await actions.continueGame(created.game.id);
    const editedTarget = created.game.players[4]?.playerId;
    expect(editedTarget).toBeDefined();
    expect(withWolfKillDraft.draft).toMatchObject({
      type: "wolf_kill_selected",
    });
    expect(withWolfKillDraft.draft?.payload).not.toMatchObject({
      targetPlayerId: editedTarget,
    });

    const edited = await actions.editDraftPayload(created.game.id, {
      targetPlayerId: editedTarget,
    });

    expect(edited.draft).toMatchObject({
      type: "wolf_kill_selected",
      targetPlayerIds: [editedTarget],
      payload: { targetPlayerId: editedTarget },
    });

    const confirmed = await actions.confirmDraft(created.game.id);
    const wolfKillEvent = confirmed.events.find(
      (event) => event.type === "wolf_kill_selected",
    );

    expect(wolfKillEvent).toMatchObject({
      type: "wolf_kill_selected",
      targetPlayerIds: [editedTarget],
      payload: { targetPlayerId: editedTarget },
    });
  });

  it("plans speech drafts during continue and generates content during regenerate", async () => {
    const repository = createGameRepository(await createTempDir());
    const created = await createGameActions(repository).createGame();
    const actions = createGameActions(repository, {
      llmClient: new MockLlmClient([
        {
          text: "我这里先报信息，1 号查杀。",
          reasoning: "根据可见信息推进发言。",
        },
      ]),
    });

    const withSpeech = await continueUntilDraftType(
      actions,
      created.game.id,
      "day_speech_given",
    );

    expect(withSpeech.draft).toMatchObject({
      type: "day_speech_given",
      payload: { text: "我先给出自己的判断。" },
    });
    expect(withSpeech.generations).toHaveLength(0);

    const generated = await actions.regenerateDraft(created.game.id);

    expect(generated.draft).toMatchObject({
      type: "day_speech_given",
      payload: { text: "我这里先报信息，1 号查杀。" },
    });
    expect(generated.generations.at(-1)).toMatchObject({
      status: "success",
      purpose: "speech",
      parsedOutput: {
        text: "我这里先报信息，1 号查杀。",
        reasoning: "根据可见信息推进发言。",
      },
    });
  });

  it("keeps default speech draft and records failure when regeneration fails", async () => {
    const repository = createGameRepository(await createTempDir());
    const actions = createGameActions(repository, {
      llmClient: failingLlmClient(),
    });
    const created = await actions.createGame();

    const withSpeech = await continueUntilDraftType(
      actions,
      created.game.id,
      "day_speech_given",
    );

    expect(withSpeech.draft).toMatchObject({
      type: "day_speech_given",
      payload: { text: "我先给出自己的判断。" },
    });

    const regenerated = await actions.regenerateDraft(created.game.id);

    expect(regenerated.draft).toMatchObject({
      type: "day_speech_given",
      payload: { text: "我先给出自己的判断。" },
    });
    expect(regenerated.generations.at(-1)).toMatchObject({
      status: "failed",
      purpose: "speech",
      error: "model unavailable",
    });
  });

  it("regenerates the current speech draft without advancing", async () => {
    const repository = createGameRepository(await createTempDir());
    const created = await createGameActions(repository).createGame();
    const actions = createGameActions(repository, {
      llmClient: new MockLlmClient([
        {
          text: "重新生成后的发言。",
          reasoning: "根据当前可见信息重新组织发言。",
        },
      ]),
    });
    const withSpeech = await continueUntilDraftType(
      actions,
      created.game.id,
      "day_speech_given",
    );

    const regenerated = await actions.regenerateDraft(created.game.id);

    expect(regenerated.draft).toMatchObject({
      id: withSpeech.draft?.id,
      type: "day_speech_given",
      payload: { text: "重新生成后的发言。" },
    });
    expect(regenerated.events).toEqual(withSpeech.events);
    expect(regenerated.generations.at(-1)).toMatchObject({
      status: "success",
      parsedOutput: {
        text: "重新生成后的发言。",
        reasoning: "根据当前可见信息重新组织发言。",
      },
    });
  });

  it("keeps unsupported drafts unchanged when regenerating", async () => {
    const { actions } = await createActions();
    const created = await actions.createGame();
    const withRoleDraft = await actions.continueGame(created.game.id);

    const regenerated = await actions.regenerateDraft(created.game.id);

    expect(regenerated).toEqual(withRoleDraft);
  });

  it("generates wolf and seer action suggestions during regenerate", async () => {
    const repository = createGameRepository(await createTempDir());
    const wolfTarget = "placeholder";
    const actions = createGameActions(repository, {
      llmClient: new MockLlmClient([
        { targetPlayerId: wolfTarget },
        { targetPlayerId: wolfTarget },
      ]),
    });
    const created = await actions.createGame();
    const realWolfTarget = created.game.players[4]?.playerId;
    const realSeerTarget = created.game.players[0]?.playerId;
    expect(realWolfTarget).toBeDefined();
    expect(realSeerTarget).toBeDefined();
    const generatedActions = createGameActions(repository, {
      llmClient: new MockLlmClient([
        { targetPlayerId: realWolfTarget },
        { targetPlayerId: realSeerTarget },
      ]),
    });

    const withWolfKill = await continueUntilDraftType(
      generatedActions,
      created.game.id,
      "wolf_kill_selected",
    );
    expect(withWolfKill.draft).toMatchObject({
      type: "wolf_kill_selected",
    });

    const generatedWolfKill = await generatedActions.regenerateDraft(created.game.id);
    expect(generatedWolfKill.draft).toMatchObject({
      type: "wolf_kill_selected",
      payload: { targetPlayerId: realWolfTarget },
    });
    await generatedActions.confirmDraft(created.game.id);

    const withSeerCheck = await continueUntilDraftType(
      generatedActions,
      created.game.id,
      "seer_check_selected",
    );
    expect(withSeerCheck.draft).toMatchObject({
      type: "seer_check_selected",
    });

    const generatedSeerCheck = await generatedActions.regenerateDraft(created.game.id);
    expect(generatedSeerCheck.draft).toMatchObject({
      type: "seer_check_selected",
      payload: { targetPlayerId: realSeerTarget },
    });
    expect(generatedSeerCheck.generations.filter((record) => record.purpose === "action")).toHaveLength(2);
  });

  it("generates witch medicine action suggestions during regenerate", async () => {
    const repository = createGameRepository(await createTempDir());
    const actions = createGameActions(repository, {
      llmClient: new MockLlmClient([{ used: false, targetPlayerId: null }]),
    });
    const created = await actions.createGame();

    const withWitchAntidote = await continueUntilDraftType(
      actions,
      created.game.id,
      "witch_antidote_decided",
    );

    expect(withWitchAntidote.draft).toMatchObject({
      type: "witch_antidote_decided",
    });

    const generated = await actions.regenerateDraft(created.game.id);

    expect(generated.draft).toMatchObject({
      type: "witch_antidote_decided",
      payload: { used: false, targetPlayerId: null },
    });
    expect(generated.generations.at(-1)).toMatchObject({
      purpose: "action",
      status: "success",
    });
  });

  it("rejects invalid rollback indexes", async () => {
    const { actions } = await createActions();
    const created = await actions.createGame();

    await expect(actions.rollbackAfter(created.game.id, -1)).rejects.toThrow(
      "Invalid rollback index: -1",
    );
    await expect(actions.rollbackAfter(created.game.id, 1.5)).rejects.toThrow(
      "Invalid rollback index: 1.5",
    );
  });

  it("serializes overlapping draft edits and confirmations for one game", async () => {
    const repository = createDeferredSaveRepository();
    const actions = createGameActions(repository);
    const created = await actions.createGame();
    for (let step = 0; step < created.game.players.length + 2; step += 1) {
      await actions.continueGame(created.game.id);
      await actions.confirmDraft(created.game.id);
    }
    const withWolfKillDraft = await actions.continueGame(created.game.id);
    const editedTarget = created.game.players[4]?.playerId;
    expect(editedTarget).toBeDefined();
    expect(withWolfKillDraft.draft).toMatchObject({
      type: "wolf_kill_selected",
    });

    const editPromise = actions.editDraftPayload(created.game.id, {
      targetPlayerId: editedTarget,
    });
    await repository.waitForDeferredSave();
    const confirmPromise = actions.confirmDraft(created.game.id);

    repository.releaseDeferredSave();
    await editPromise;
    const confirmed = await confirmPromise;

    expect(confirmed.draft).toBeNull();
    expect(confirmed.events.at(-1)).toMatchObject({
      type: "wolf_kill_selected",
      targetPlayerIds: [editedTarget],
      payload: { targetPlayerId: editedTarget },
    });
    await expect(repository.get(created.game.id)).resolves.toEqual(confirmed);

    const withNextDraft = await actions.continueGame(created.game.id);
    expect(withNextDraft.draft).toMatchObject({
      type: "seer_check_selected",
    });
  });

  it("throws when the game does not exist", async () => {
    const { actions } = await createActions();
    const missingGameId = "game_missing" as GameId;

    await expect(actions.continueGame(missingGameId)).rejects.toThrow(
      "Game not found: game_missing",
    );
  });
});

async function continueUntilDraftType(
  actions: ReturnType<typeof createGameActions>,
  gameId: GameId,
  draftType: NonNullable<GameRecord["draft"]>["type"],
): Promise<GameRecord> {
  for (let step = 0; step < 40; step += 1) {
    const record = await actions.continueGame(gameId);
    if (record.draft?.type === draftType) {
      return record;
    }
    await actions.confirmDraft(gameId);
  }

  throw new Error(`Draft not reached: ${draftType}`);
}

function failingLlmClient(): LlmClient {
  return {
    async generateJson() {
      throw new Error("model unavailable");
    },
  };
}

function fakeLibraryRepository(
  overrides: Partial<LibraryRepository>,
): LibraryRepository {
  return {
    async getRoles() {
      return seedRoles;
    },
    async getCharacters() {
      return seedCharacters;
    },
    async getPresets() {
      return seedPresets;
    },
    async getPresenters() {
      return seedPresenters;
    },
    async getAll() {
      return { roles: seedRoles, characters: seedCharacters, presets: seedPresets, presenters: seedPresenters };
    },
    async loadAll() {
      return { roles: seedRoles, characters: seedCharacters, presets: seedPresets, presenters: seedPresenters };
    },
    async saveRoles() {},
    async saveCharacters() {},
    async savePresets() {},
    async savePresenters() {},
    async saveAll() {},
    async withLibraryLock(operation) {
      return operation();
    },
    ...overrides,
  };
}

function createDeferredSaveRepository(): GameRepository & {
  readonly waitForDeferredSave: () => Promise<void>;
  readonly releaseDeferredSave: () => void;
} {
  let record: GameRecord | null = null;
  let releaseDeferredSave: () => void = () => {};
  let deferredSaveStarted: () => void = () => {};
  let lockTail: Promise<void> = Promise.resolve();
  const deferredSaveStartedPromise = new Promise<void>((resolve) => {
    deferredSaveStarted = resolve;
  });
  const releaseDeferredSavePromise = new Promise<void>((resolve) => {
    releaseDeferredSave = resolve;
  });

  return {
    async get(gameId) {
      if (record?.game.id !== gameId) {
        return null;
      }

      return structuredClone(record);
    },

    async list() {
      return record ? [structuredClone(record)] : [];
    },

    async save(nextRecord) {
      if (
        nextRecord.draft?.type === "wolf_kill_selected" &&
        nextRecord.draft.payload.targetPlayerId ===
          record?.game.players[4]?.playerId
      ) {
        deferredSaveStarted();
        await releaseDeferredSavePromise;
      }

      record = structuredClone(nextRecord);
    },

    async delete(gameId) {
      if (record?.game.id === gameId) {
        record = null;
      }
    },

    async withGameLock(_gameId, operation) {
      const previous = lockTail;
      let releaseCurrent!: () => void;
      lockTail = new Promise<void>((resolve) => {
        releaseCurrent = resolve;
      });
      await previous;

      try {
        return await operation();
      } finally {
        releaseCurrent();
      }
    },

    waitForDeferredSave() {
      return deferredSaveStartedPromise;
    },

    releaseDeferredSave,
  };
}
