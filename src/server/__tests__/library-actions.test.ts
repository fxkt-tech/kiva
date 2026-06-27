import { mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { seedCharacters } from "@/seeds/characters";
import { seedPresets } from "@/seeds/presets";
import { seedRoles } from "@/seeds/roles";
import { createGameRepository } from "../game-repository";
import { createLibraryActions } from "../library-actions";
import { createLibraryRepository } from "../library-repository";

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
});
