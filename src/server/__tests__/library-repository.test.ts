import { mkdir, mkdtemp, readdir, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { afterEach, describe, expect, it } from "vitest";
import type { CharacterDefinition } from "@/core/character-definition";
import type { GamePreset } from "@/core/game-preset";
import type { RoleDefinition } from "@/core/role-definition";
import { seedCharacters } from "@/seeds/characters";
import { seedPresets } from "@/seeds/presets";
import { seedRoles } from "@/seeds/roles";
import { createLibraryRepository } from "../library-repository";

const tempDirs: string[] = [];

afterEach(async () => {
  await Promise.all(
    tempDirs.splice(0).map((dir) => rm(dir, { recursive: true, force: true })),
  );
});

async function createTempDir(): Promise<string> {
  const dir = await mkdtemp(join(tmpdir(), "kiva-library-repository-"));
  tempDirs.push(dir);
  return dir;
}

describe("library repository", () => {
  it("returns empty arrays when library files are missing", async () => {
    const repository = createLibraryRepository(await createTempDir());

    await expect(repository.getRoles()).resolves.toEqual([]);
    await expect(repository.getCharacters()).resolves.toEqual([]);
    await expect(repository.getPresets()).resolves.toEqual([]);
    await expect(repository.getAll()).resolves.toEqual({
      roles: [],
      characters: [],
      presets: [],
    });
  });

  it("returns seed data after saveAll", async () => {
    const repository = createLibraryRepository(await createTempDir());

    await repository.saveAll({
      roles: seedRoles,
      characters: seedCharacters,
      presets: seedPresets,
    });

    await expect(repository.getAll()).resolves.toEqual({
      roles: seedRoles,
      characters: seedCharacters,
      presets: seedPresets,
    });
    await expect(repository.loadAll()).resolves.toEqual({
      roles: seedRoles,
      characters: seedCharacters,
      presets: seedPresets,
    });
  });

  it("writes pretty JSON with trailing newlines", async () => {
    const rootDir = await createTempDir();
    const repository = createLibraryRepository(rootDir);

    await repository.saveAll({
      roles: seedRoles,
      characters: seedCharacters,
      presets: seedPresets,
    });

    for (const filename of ["roles.json", "characters.json", "presets.json"]) {
      const content = await readFile(join(rootDir, filename), "utf8");
      expect(content).toContain('\n  {');
      expect(content.endsWith("\n")).toBe(true);
    }
  });

  it("validates roles, characters, and presets before saving each file", async () => {
    const repository = createLibraryRepository(await createTempDir());

    await expect(
      repository.saveRoles({} as unknown as readonly RoleDefinition[]),
    ).rejects.toThrow("Role definitions must be an array");
    await expect(
      repository.saveCharacters({} as unknown as readonly CharacterDefinition[]),
    ).rejects.toThrow("Character definitions must be an array");
    await expect(
      repository.savePresets({} as unknown as readonly GamePreset[]),
    ).rejects.toThrow("Game presets must be an array");
  });

  it("rejects preset references that are missing while reading or saving", async () => {
    const rootDir = await createTempDir();
    const repository = createLibraryRepository(rootDir);

    await writeFile(
      join(rootDir, "presets.json"),
      `${JSON.stringify(seedPresets, null, 2)}\n`,
      "utf8",
    );

    await expect(repository.getPresets()).rejects.toThrow(
      "references unknown role: werewolf",
    );
    await expect(repository.savePresets(seedPresets)).rejects.toThrow(
      "references unknown role: werewolf",
    );
  });

  it("saves presets after seed roles and characters exist", async () => {
    const repository = createLibraryRepository(await createTempDir());

    await repository.saveRoles(seedRoles);
    await repository.saveCharacters(seedCharacters);
    await repository.savePresets(seedPresets);

    await expect(repository.getPresets()).resolves.toEqual(seedPresets);
  });

  it("does not leave temporary files after atomic writes", async () => {
    const rootDir = await createTempDir();
    const repository = createLibraryRepository(rootDir);

    await repository.saveAll({
      roles: seedRoles,
      characters: seedCharacters,
      presets: seedPresets,
    });

    const filenames = await readdir(rootDir);
    expect([...filenames].sort()).toEqual([
      "characters.json",
      "presets.json",
      "roles.json",
    ]);
  });

  it("cleans temporary files after a failed single-file write", async () => {
    const rootDir = await createTempDir();
    const repository = createLibraryRepository(rootDir);

    await mkdir(join(rootDir, "roles.json"));

    await expect(repository.saveRoles(seedRoles)).rejects.toThrow();

    const filenames = await readdir(rootDir);
    expect(filenames).toEqual(["roles.json"]);
  });

  it("replaces stale directory entries during saveAll", async () => {
    const rootDir = await createTempDir();
    const repository = createLibraryRepository(rootDir);

    await mkdir(join(rootDir, "characters.json"));

    await repository.saveAll({
      roles: seedRoles,
      characters: seedCharacters,
      presets: seedPresets,
    });

    await expect(repository.getAll()).resolves.toEqual({
      roles: seedRoles,
      characters: seedCharacters,
      presets: seedPresets,
    });
    const filenames = await readdir(rootDir);
    expect(filenames.some((filename) => filename.endsWith(".tmp"))).toBe(false);
    expect(filenames.some((filename) => filename.endsWith(".bak"))).toBe(false);
  });
});
