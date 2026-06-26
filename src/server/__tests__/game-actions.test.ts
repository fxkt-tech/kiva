import { mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { afterEach, describe, expect, it } from "vitest";
import { getActiveEvents } from "@/core/event-log";
import type { GameId } from "@/core/types";
import { createGameActions } from "../game-actions";
import {
  createGameRepository,
  type GameRecord,
  type GameRepository,
} from "../game-repository";

const tempDirs: string[] = [];

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
      game: { id: expect.stringMatching(/^game_/) },
      events: [],
      draft: null,
    });
    await expect(repository.get(created.game.id)).resolves.toEqual(created);
    await expect(actions.getGame(created.game.id)).resolves.toEqual(created);
  });

  it("continues a game, confirms the draft, appends one official event, and clears draft", async () => {
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
    await expect(repository.get(created.game.id)).resolves.toEqual(confirmed);
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
    await expect(repository.get(created.game.id)).resolves.toEqual(rolledBack);
  });

  it("updates current draft display before confirmation", async () => {
    const { actions } = await createActions();
    const created = await actions.createGame();
    await actions.continueGame(created.game.id);

    const edited = await actions.editDraftDisplay(created.game.id, {
      title: "  Custom title  ",
      text: "  Custom text  ",
    });

    expect(edited.draft?.display).toEqual({
      title: "Custom title",
      text: "Custom text",
    });

    const confirmed = await actions.confirmDraft(created.game.id);
    expect(confirmed.events[0]?.display).toEqual({
      title: "Custom title",
      text: "Custom text",
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
    await actions.continueGame(created.game.id);

    const editPromise = actions.editDraftDisplay(created.game.id, {
      title: "Edited",
      text: "Edited text",
    });
    await repository.waitForDeferredSave();
    const confirmPromise = actions.confirmDraft(created.game.id);

    repository.releaseDeferredSave();
    await editPromise;
    const confirmed = await confirmPromise;

    expect(confirmed.draft).toBeNull();
    expect(confirmed.events).toHaveLength(1);
    expect(confirmed.events[0]?.display).toEqual({
      title: "Edited",
      text: "Edited text",
    });
    await expect(repository.get(created.game.id)).resolves.toEqual(confirmed);
  });

  it("throws when the game does not exist", async () => {
    const { actions } = await createActions();
    const missingGameId = "game_missing" as GameId;

    await expect(actions.continueGame(missingGameId)).rejects.toThrow(
      "Game not found: game_missing",
    );
  });
});

function createDeferredSaveRepository(): GameRepository & {
  readonly waitForDeferredSave: () => Promise<void>;
  readonly releaseDeferredSave: () => void;
} {
  let record: GameRecord | null = null;
  let releaseDeferredSave: () => void = () => {};
  let deferredSaveStarted: () => void = () => {};
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
      if (nextRecord.draft?.display?.title === "Edited") {
        deferredSaveStarted();
        await releaseDeferredSavePromise;
      }

      record = structuredClone(nextRecord);
    },

    waitForDeferredSave() {
      return deferredSaveStartedPromise;
    },

    releaseDeferredSave,
  };
}
