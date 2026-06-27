import { mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { seedCharacters } from "@/seeds/characters";
import { seedPresets } from "@/seeds/presets";
import { seedRoles } from "@/seeds/roles";
import { createGameRepository } from "../game-repository";
import { createLibraryActions } from "../library-actions";
import {
  createLibraryRepository,
  type LibraryRecord,
  type LibraryRepository,
} from "../library-repository";

const tempDirs: string[] = [];
const NOW = "2026-06-27T12:34:56.000Z";

beforeEach(() => {
  vi.useFakeTimers();
  vi.setSystemTime(new Date(NOW));
});

afterEach(async () => {
  vi.useRealTimers();
  await Promise.all(
    tempDirs.splice(0).map((dir) => rm(dir, { recursive: true, force: true })),
  );
});

async function createTempDir(): Promise<string> {
  const dir = await mkdtemp(join(tmpdir(), "kiva-library-actions-"));
  tempDirs.push(dir);
  return dir;
}

async function createSeededActions() {
  const rootDir = await createTempDir();
  const libraryRepository = createLibraryRepository(rootDir);
  const gameRepository = createGameRepository(rootDir);
  await libraryRepository.saveAll({
    roles: seedRoles,
    characters: seedCharacters,
    presets: seedPresets,
  });

  return {
    actions: createLibraryActions({ libraryRepository, gameRepository }),
    libraryRepository,
    gameRepository,
  };
}

describe("library actions", () => {
  it("returns the complete library with diagnostics", async () => {
    const { actions } = await createSeededActions();

    const library = await actions.getLibrary();

    expect(library.roles).toEqual(seedRoles);
    expect(library.characters).toEqual(seedCharacters);
    expect(library.presets).toEqual(seedPresets);
    expect(library.diagnostics.roles.werewolf).toMatchObject({
      valid: true,
      references: ["six_player_standard"],
      messages: ["Built-in role contract locked"],
      promptPreview: expect.stringContaining(seedRoles[0].systemPrompt),
    });
    expect(library.diagnostics.characters.qin_chuan).toMatchObject({
      valid: true,
      references: ["six_player_standard"],
      promptPreview: expect.stringContaining(seedCharacters[0].systemPrompt),
    });
    expect(library.diagnostics.presets.six_player_standard).toMatchObject({
      valid: true,
      canCreateGame: true,
      messages: [],
      promptPreview: expect.stringContaining("model:"),
    });
  });

  it("saves a role prompt while validating dependent presets as a complete library", async () => {
    const { actions, libraryRepository } = await createSeededActions();
    const updatedRole = {
      ...seedRoles[0],
      systemPrompt: "更新后的狼人系统提示词",
      updatedAt: NOW,
    };

    await actions.saveRole(updatedRole);

    const saved = await libraryRepository.loadAll();
    expect(saved.roles.find((role) => role.id === "werewolf")).toMatchObject({
      systemPrompt: "更新后的狼人系统提示词",
      updatedAt: NOW,
    });
    expect(saved.presets).toEqual(seedPresets);
  });

  it("rejects built-in role contract mismatches when saving a role", async () => {
    const { actions, libraryRepository } = await createSeededActions();

    await expect(
      actions.saveRole({
        ...seedRoles[0],
        team: "god",
        updatedAt: NOW,
      }),
    ).rejects.toThrow("Role werewolf does not match the current ruleset contract");

    await expect(libraryRepository.loadAll()).resolves.toMatchObject({
      roles: seedRoles,
    });
  });

  it("duplicates a character and then disables the copy", async () => {
    const { actions } = await createSeededActions();

    const copy = await actions.duplicateCharacter("qin_chuan");
    const disabledCopy = await actions.setCharacterEnabled(copy.id, false);

    expect(copy).toMatchObject({
      id: "qin_chuan_copy",
      name: seedCharacters[0].name,
      enabled: true,
      createdAt: NOW,
      updatedAt: NOW,
    });
    expect(disabledCopy).toMatchObject({
      id: "qin_chuan_copy",
      enabled: false,
      createdAt: NOW,
      updatedAt: NOW,
    });
    await expect(actions.getLibrary()).resolves.toMatchObject({
      diagnostics: {
        characters: {
          qin_chuan_copy: {
            valid: false,
            messages: ["Character is disabled"],
            references: [],
          },
        },
      },
    });
  });

  it("saves a character prompt while preserving dependent presets", async () => {
    const { actions, libraryRepository } = await createSeededActions();
    const updatedCharacter = {
      ...seedCharacters[0],
      systemPrompt: "更新后的角色卡系统提示词",
      updatedAt: NOW,
    };

    await actions.saveCharacter(updatedCharacter);

    const saved = await libraryRepository.loadAll();
    expect(
      saved.characters.find((character) => character.id === "qin_chuan"),
    ).toMatchObject({
      systemPrompt: "更新后的角色卡系统提示词",
      updatedAt: NOW,
    });
    expect(saved.presets).toEqual(seedPresets);
  });

  it("saves a preset after validating its role and character references", async () => {
    const { actions, libraryRepository } = await createSeededActions();
    const updatedPreset = {
      ...seedPresets[0],
      name: "更新后的 6 人预设",
      updatedAt: NOW,
    };

    await actions.savePreset(updatedPreset);

    const saved = await libraryRepository.loadAll();
    expect(
      saved.presets.find((preset) => preset.id === "six_player_standard"),
    ).toMatchObject({
      name: "更新后的 6 人预设",
      updatedAt: NOW,
    });
    expect(saved.roles).toEqual(seedRoles);
    expect(saved.characters).toEqual(seedCharacters);
  });

  it("duplicates a role with a fresh id and timestamps", async () => {
    const { actions, libraryRepository } = await createSeededActions();

    const copy = await actions.duplicateRole("seer");

    expect(copy).toMatchObject({
      id: "seer_copy",
      name: seedRoles[1].name,
      createdAt: NOW,
      updatedAt: NOW,
    });
    await expect(libraryRepository.loadAll()).resolves.toMatchObject({
      roles: expect.arrayContaining([
        expect.objectContaining({ id: "seer" }),
        expect.objectContaining({ id: "seer_copy" }),
      ]),
    });
  });

  it("duplicates a preset with a fresh id and timestamps", async () => {
    const { actions, libraryRepository } = await createSeededActions();

    const copy = await actions.duplicatePreset("six_player_standard");

    expect(copy).toMatchObject({
      id: "six_player_standard_copy",
      name: seedPresets[0].name,
      createdAt: NOW,
      updatedAt: NOW,
    });
    expect(copy.roleIds).toEqual(seedPresets[0].roleIds);
    expect(copy.characterIds).toEqual(seedPresets[0].characterIds);
    await expect(libraryRepository.loadAll()).resolves.toMatchObject({
      presets: expect.arrayContaining([
        expect.objectContaining({ id: "six_player_standard" }),
        expect.objectContaining({ id: "six_player_standard_copy" }),
      ]),
    });
  });

  it("uses an incrementing suffix when duplicate copy ids already exist", async () => {
    const { actions, libraryRepository } = await createSeededActions();

    const firstCopy = await actions.duplicateCharacter("qin_chuan");
    const secondCopy = await actions.duplicateCharacter("qin_chuan");

    expect(firstCopy.id).toBe("qin_chuan_copy");
    expect(secondCopy.id).toBe("qin_chuan_copy_2");
    await expect(libraryRepository.loadAll()).resolves.toMatchObject({
      characters: expect.arrayContaining([
        expect.objectContaining({ id: "qin_chuan" }),
        expect.objectContaining({ id: "qin_chuan_copy" }),
        expect.objectContaining({ id: "qin_chuan_copy_2" }),
      ]),
    });
  });

  it("disables a preset without changing its validated references", async () => {
    const { actions, libraryRepository } = await createSeededActions();

    const disabledPreset = await actions.setPresetEnabled(
      "six_player_standard",
      false,
    );

    expect(disabledPreset).toMatchObject({
      id: "six_player_standard",
      enabled: false,
      updatedAt: NOW,
    });
    const saved = await libraryRepository.loadAll();
    expect(
      saved.presets.find((preset) => preset.id === "six_player_standard"),
    ).toMatchObject({
      enabled: false,
      updatedAt: NOW,
    });
    expect(saved.roles).toEqual(seedRoles);
    expect(saved.characters).toEqual(seedCharacters);
  });

  it("creates a six player game from a preset id", async () => {
    const { actions, gameRepository } = await createSeededActions();

    const record = await actions.createGameFromPreset("six_player_standard");

    expect(record.game.title).toBe("6人狼人杀试运行");
    expect(record.game.players).toHaveLength(6);
    expect(record.events).toEqual([]);
    await expect(gameRepository.get(record.game.id)).resolves.toEqual(record);
  });

  it("rejects disabling a role while an enabled preset still references it", async () => {
    const { actions, libraryRepository } = await createSeededActions();

    await expect(actions.setRoleEnabled("werewolf", false)).rejects.toThrow(
      "Game preset six_player_standard roleIds[0] references disabled role: werewolf",
    );

    await expect(libraryRepository.loadAll()).resolves.toMatchObject({
      roles: seedRoles,
      presets: seedPresets,
    });
  });

  it("rejects disabling a character while an enabled preset still references it", async () => {
    const { actions, libraryRepository } = await createSeededActions();

    await expect(
      actions.setCharacterEnabled("qin_chuan", false),
    ).rejects.toThrow(
      "Game preset six_player_standard characterIds[0] references disabled character: qin_chuan",
    );

    await expect(libraryRepository.loadAll()).resolves.toMatchObject({
      characters: seedCharacters,
      presets: seedPresets,
    });
  });

  it("serializes concurrent library writes so both updates are preserved", async () => {
    const libraryRepository = createRaceDetectingLibraryRepository();
    const gameRepository = createGameRepository(await createTempDir());
    const actions = createLibraryActions({ libraryRepository, gameRepository });

    await Promise.all([
      actions.saveRole({
        ...seedRoles[0],
        systemPrompt: "并发更新后的狼人提示词",
        updatedAt: NOW,
      }),
      actions.saveCharacter({
        ...seedCharacters[0],
        systemPrompt: "并发更新后的角色卡提示词",
        updatedAt: NOW,
      }),
    ]);

    const saved = await libraryRepository.loadAll();
    expect(saved.roles.find((role) => role.id === "werewolf")).toMatchObject({
      systemPrompt: "并发更新后的狼人提示词",
    });
    expect(
      saved.characters.find((character) => character.id === "qin_chuan"),
    ).toMatchObject({
      systemPrompt: "并发更新后的角色卡提示词",
    });
  });

  it("reads library diagnostics under the library lock", async () => {
    const libraryRepository = createLockCountingLibraryRepository();
    const gameRepository = createGameRepository(await createTempDir());
    const actions = createLibraryActions({ libraryRepository, gameRepository });

    await actions.getLibrary();

    expect(libraryRepository.lockCalls).toBe(1);
  });

  it("creates games from presets under the library lock", async () => {
    const libraryRepository = createLockCountingLibraryRepository();
    const gameRepository = createGameRepository(await createTempDir());
    const actions = createLibraryActions({ libraryRepository, gameRepository });

    const record = await actions.createGameFromPreset("six_player_standard");

    expect(record.game.players).toHaveLength(6);
    expect(libraryRepository.lockCalls).toBe(1);
  });
});

function createRaceDetectingLibraryRepository(): LibraryRepository {
  let record = cloneLibraryRecord({
    roles: seedRoles,
    characters: seedCharacters,
    presets: seedPresets,
  });
  let lockTail: Promise<void> = Promise.resolve();
  let inLock = false;
  let hasUsedLock = false;
  let unlockedLoadAllCalls = 0;
  let releaseConcurrentLoads: (() => void) | null = null;
  const concurrentLoadsReady = new Promise<void>((resolve) => {
    releaseConcurrentLoads = resolve;
  });

  async function loadAll() {
    const snapshot = cloneLibraryRecord(record);
    if (!inLock && !hasUsedLock) {
      unlockedLoadAllCalls += 1;
      if (unlockedLoadAllCalls === 2) {
        releaseConcurrentLoads?.();
      }
      if (unlockedLoadAllCalls <= 2) {
        await concurrentLoadsReady;
      }
    }
    return snapshot;
  }

  return {
    async getRoles() {
      return (await loadAll()).roles;
    },
    async getCharacters() {
      return (await loadAll()).characters;
    },
    async getPresets() {
      return (await loadAll()).presets;
    },
    async getAll() {
      return loadAll();
    },
    loadAll,
    async saveRoles(roles) {
      record = { ...record, roles: [...roles] };
    },
    async saveCharacters(characters) {
      record = { ...record, characters: [...characters] };
    },
    async savePresets(presets) {
      record = { ...record, presets: [...presets] };
    },
    async saveAll(nextRecord) {
      record = cloneLibraryRecord(nextRecord);
    },
    async withLibraryLock(operation) {
      const previous = lockTail;
      let release: () => void = () => {};
      lockTail = new Promise<void>((resolve) => {
        release = resolve;
      });
      await previous;
      hasUsedLock = true;
      inLock = true;
      try {
        return await operation();
      } finally {
        inLock = false;
        release();
      }
    },
  };
}

function createLockCountingLibraryRepository(): LibraryRepository & {
  readonly lockCalls: number;
} {
  let lockCalls = 0;
  const repository = createMemoryLibraryRepository();

  return {
    ...repository,
    get lockCalls() {
      return lockCalls;
    },
    async withLibraryLock(operation) {
      lockCalls += 1;
      return operation();
    },
  };
}

function createMemoryLibraryRepository(): LibraryRepository {
  let record = cloneLibraryRecord({
    roles: seedRoles,
    characters: seedCharacters,
    presets: seedPresets,
  });

  return {
    async getRoles() {
      return cloneLibraryRecord(record).roles;
    },
    async getCharacters() {
      return cloneLibraryRecord(record).characters;
    },
    async getPresets() {
      return cloneLibraryRecord(record).presets;
    },
    async getAll() {
      return cloneLibraryRecord(record);
    },
    async loadAll() {
      return cloneLibraryRecord(record);
    },
    async saveRoles(roles) {
      record = { ...record, roles: [...roles] };
    },
    async saveCharacters(characters) {
      record = { ...record, characters: [...characters] };
    },
    async savePresets(presets) {
      record = { ...record, presets: [...presets] };
    },
    async saveAll(nextRecord) {
      record = cloneLibraryRecord(nextRecord);
    },
    async withLibraryLock(operation) {
      return operation();
    },
  };
}

function cloneLibraryRecord(record: LibraryRecord): LibraryRecord {
  return structuredClone(record) as LibraryRecord;
}
