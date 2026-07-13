import { mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { seedCharacters } from "@/seeds/characters";
import { seedPresets } from "@/seeds/presets";
import { seedRoles } from "@/seeds/roles";
import { seedPresenters } from "@/seeds/presenters";
import { seedScripts } from "@/seeds/scripts";
import { createGameRepository } from "../game-repository";
import { createLibraryActions } from "../library-actions";
import {
  createLibraryRepository,
  type LibraryRecord,
  type LibraryRepository,
} from "../library-repository";

const tempDirs: string[] = [];
const NOW = "2026-06-27T12:34:56.000Z";
const defaultPresetId = "twelve_player_standard";
const werewolfRole = seedRoles.find((role) => role.id === "werewolf")!;
const seerRole = seedRoles.find((role) => role.id === "seer")!;

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
    presenters: seedPresenters,
    scripts: seedScripts,
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
    expect(library.presenters).toEqual(seedPresenters);
    expect(library.scripts).toEqual(seedScripts);
    expect(library.diagnostics.roles.werewolf).toMatchObject({
      valid: true,
      references: [defaultPresetId],
      messages: ["Built-in role contract locked"],
      promptPreview: expect.stringContaining(werewolfRole.systemPrompt),
    });
    expect(library.diagnostics.characters.qin_chuan).toMatchObject({
      valid: true,
      references: [defaultPresetId],
      promptPreview: expect.stringContaining(seedCharacters[0].systemPrompt),
    });
    expect(library.diagnostics.presets[defaultPresetId]).toMatchObject({
      valid: true,
      canCreateGame: true,
      messages: [],
      promptPreview: expect.stringContaining("模型："),
    });
    expect(library.diagnostics.presenters.wen_zhou).toMatchObject({
      valid: true,
      references: [],
      messages: [],
      promptPreview: expect.stringContaining("phase.night"),
    });
  });

  it("saves a role prompt while validating dependent presets as a complete library", async () => {
    const { actions, libraryRepository } = await createSeededActions();
    const updatedRole = {
      ...werewolfRole,
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
        ...werewolfRole,
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
      name: "更新后的 12 人预设",
      updatedAt: NOW,
    };

    await actions.savePreset(updatedPreset);

    const saved = await libraryRepository.loadAll();
    expect(
      saved.presets.find((preset) => preset.id === defaultPresetId),
    ).toMatchObject({
      name: "更新后的 12 人预设",
      updatedAt: NOW,
    });
    expect(saved.roles).toEqual(seedRoles);
    expect(saved.characters).toEqual(seedCharacters);
  });

  it("saves presenter identity, copy, and voice configuration", async () => {
    const { actions, libraryRepository } = await createSeededActions();
    const source = seedPresenters[0]!;
    const updatedPresenter = {
      ...source,
      name: "新守夜人",
      voiceProfile: {
        ...source.voiceProfile,
        voice: "zh-CN-XiaoyiNeural",
      },
      lines: {
        ...source.lines,
        "phase.night": {
          ...source.lines["phase.night"],
          template: "夜幕降临，所有玩家闭眼。",
        },
      },
      updatedAt: NOW,
    };

    await actions.savePresenter(updatedPresenter);

    const saved = await libraryRepository.loadAll();
    expect(saved.presenters.find((presenter) => presenter.id === source.id))
      .toMatchObject({
        name: "新守夜人",
        voiceProfile: { voice: "zh-CN-XiaoyiNeural" },
        lines: {
          "phase.night": {
            template: "夜幕降临，所有玩家闭眼。",
          },
        },
        updatedAt: NOW,
      });
  });

  it("duplicates a role with a fresh id and timestamps", async () => {
    const { actions, libraryRepository } = await createSeededActions();

    const copy = await actions.duplicateRole("seer");

    expect(copy).toMatchObject({
      id: "seer_copy",
      name: seerRole.name,
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

    const copy = await actions.duplicatePreset(defaultPresetId);

    expect(copy).toMatchObject({
      id: "twelve_player_standard_copy",
      name: seedPresets[0].name,
      createdAt: NOW,
      updatedAt: NOW,
    });
    expect(copy.roleIds).toEqual(seedPresets[0].roleIds);
    expect(copy.characterIds).toEqual(seedPresets[0].characterIds);
    await expect(libraryRepository.loadAll()).resolves.toMatchObject({
      presets: expect.arrayContaining([
        expect.objectContaining({ id: defaultPresetId }),
        expect.objectContaining({ id: "twelve_player_standard_copy" }),
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
      defaultPresetId,
      false,
    );

    expect(disabledPreset).toMatchObject({
      id: defaultPresetId,
      enabled: false,
      updatedAt: NOW,
    });
    const saved = await libraryRepository.loadAll();
    expect(
      saved.presets.find((preset) => preset.id === defaultPresetId),
    ).toMatchObject({
      enabled: false,
      updatedAt: NOW,
    });
    expect(saved.roles).toEqual(seedRoles);
    expect(saved.characters).toEqual(seedCharacters);
  });

  it("creates a twelve player game from a preset id", async () => {
    const { actions, gameRepository } = await createSeededActions();

    const record = await actions.createGameFromPreset(
      defaultPresetId,
      seedPresenters[0]!.id,
    );

    expect(record.game.title).toBe("12人狼人杀标准局");
    expect(record.game.players).toHaveLength(12);
    expect(record.game.presenter).toMatchObject({
      presenterSourceId: "wen_zhou",
      name: "闻舟",
    });
    expect(record.events).toEqual([]);
    await expect(gameRepository.get(record.game.id)).resolves.toEqual(record);
  });

  it("creates a game from a temporary preset without saving that preset", async () => {
    const { actions, libraryRepository, gameRepository } = await createSeededActions();
    const temporaryPreset = {
      ...seedPresets[0],
      id: "temporary_new_game",
      name: "临时随机局",
      seatAssignments: seedPresets[0].seatAssignments?.map((seat) =>
        seat.seatNo === 1
          ? { ...seat, characterId: "qiao_ke" }
          : seat.seatNo === 2
            ? { ...seat, characterId: "zhou_xu" }
            : seat,
      ) ?? null,
    };

    const record = await actions.createGameFromTemporaryPreset(
      temporaryPreset,
      seedPresenters[0]!.id,
    );

    expect(record.game.title).toBe("临时随机局");
    expect(record.game.players[0]?.name).toBe("乔可");
    expect(record.game.presenter).toMatchObject({
      presenterSourceId: "wen_zhou",
      name: "闻舟",
    });
    await expect(gameRepository.get(record.game.id)).resolves.toEqual(record);
    await expect(libraryRepository.loadAll()).resolves.toMatchObject({
      presets: seedPresets,
    });
  });

  it("rejects disabling a role while an enabled preset still references it", async () => {
    const { actions, libraryRepository } = await createSeededActions();

    await expect(actions.setRoleEnabled("werewolf", false)).rejects.toThrow(
      "Game preset twelve_player_standard roleIds[2] references disabled role: werewolf",
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
      "Game preset twelve_player_standard characterIds[2] references disabled character: qin_chuan",
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
        ...werewolfRole,
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

    const record = await actions.createGameFromPreset(
      defaultPresetId,
      seedPresenters[0]!.id,
    );

    expect(record.game.players).toHaveLength(12);
    expect(libraryRepository.lockCalls).toBe(1);
  });
});

function createRaceDetectingLibraryRepository(): LibraryRepository {
  let record = cloneLibraryRecord({
    roles: seedRoles,
    characters: seedCharacters,
    presets: seedPresets,
    presenters: seedPresenters,
    scripts: seedScripts,
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
    async getPresenters() {
      return (await loadAll()).presenters;
    },
    async getScripts() {
      return (await loadAll()).scripts;
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
    async savePresenters(presenters) {
      record = { ...record, presenters: [...presenters] };
    },
    async saveScripts(scripts) {
      record = { ...record, scripts: [...scripts] };
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
    presenters: seedPresenters,
    scripts: seedScripts,
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
    async getPresenters() {
      return cloneLibraryRecord(record).presenters;
    },
    async getScripts() {
      return cloneLibraryRecord(record).scripts;
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
    async savePresenters(presenters) {
      record = { ...record, presenters: [...presenters] };
    },
    async saveScripts(scripts) {
      record = { ...record, scripts: [...scripts] };
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
