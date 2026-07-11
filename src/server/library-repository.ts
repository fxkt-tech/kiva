import { randomUUID } from "node:crypto";
import {
  mkdir,
  readFile,
  rename,
  rm,
  rmdir,
  unlink,
  writeFile,
} from "node:fs/promises";
import { setTimeout } from "node:timers/promises";
import { join } from "node:path";
import {
  validateCharacterDefinitions,
  type CharacterDefinition,
} from "@/core/character-definition";
import { validateGamePresets, type GamePreset } from "@/core/game-preset";
import {
  validatePresenterDefinitions,
  type PresenterDefinition,
} from "@/core/presenter-definition";
import {
  validateRoleDefinitions,
  type RoleDefinition,
} from "@/core/role-definition";

export type LibraryRecord = {
  readonly roles: readonly RoleDefinition[];
  readonly characters: readonly CharacterDefinition[];
  readonly presets: readonly GamePreset[];
  readonly presenters: readonly PresenterDefinition[];
};

export type LibraryRepository = {
  readonly getRoles: () => Promise<readonly RoleDefinition[]>;
  readonly getCharacters: () => Promise<readonly CharacterDefinition[]>;
  readonly getPresets: () => Promise<readonly GamePreset[]>;
  readonly getPresenters: () => Promise<readonly PresenterDefinition[]>;
  readonly getAll: () => Promise<LibraryRecord>;
  readonly loadAll: () => Promise<LibraryRecord>;
  readonly saveRoles: (roles: readonly RoleDefinition[]) => Promise<void>;
  readonly saveCharacters: (
    characters: readonly CharacterDefinition[],
  ) => Promise<void>;
  readonly savePresets: (presets: readonly GamePreset[]) => Promise<void>;
  readonly savePresenters: (
    presenters: readonly PresenterDefinition[],
  ) => Promise<void>;
  readonly saveAll: (record: LibraryRecord) => Promise<void>;
  readonly withLibraryLock: <T>(operation: () => Promise<T>) => Promise<T>;
};

export function createLibraryRepository(rootDir = "kivdb"): LibraryRepository {
  const rolesPath = join(rootDir, "roles.json");
  const charactersPath = join(rootDir, "characters.json");
  const presetsPath = join(rootDir, "presets.json");
  const presentersPath = join(rootDir, "presenters.json");
  const locksDir = join(rootDir, "locks");
  const libraryLockPath = join(locksDir, "library.lock");

  async function ensureRootDir(): Promise<void> {
    await mkdir(rootDir, { recursive: true });
  }

  async function ensureLocksDir(): Promise<void> {
    await mkdir(locksDir, { recursive: true });
  }

  async function readJson(path: string): Promise<unknown[]> {
    try {
      return JSON.parse(await readFile(path, "utf8")) as unknown[];
    } catch (error) {
      if (isNodeError(error) && error.code === "ENOENT") {
        return [];
      }

      throw error;
    }
  }

  async function readRoles(): Promise<readonly RoleDefinition[]> {
    return validateRoleDefinitions(await readJson(rolesPath));
  }

  async function readCharacters(): Promise<readonly CharacterDefinition[]> {
    return validateCharacterDefinitions(await readJson(charactersPath));
  }

  async function readPresenters(): Promise<readonly PresenterDefinition[]> {
    return validatePresenterDefinitions(await readJson(presentersPath));
  }

  async function readPresets(
    libraries: {
      readonly roles: readonly RoleDefinition[];
      readonly characters: readonly CharacterDefinition[];
    },
  ): Promise<readonly GamePreset[]> {
    return validateGamePresets(await readJson(presetsPath), libraries);
  }

  async function writeJson(path: string, data: unknown): Promise<void> {
    await ensureRootDir();
    const tempPath = `${path}.${randomUUID()}.tmp`;
    try {
      await writeFile(tempPath, `${JSON.stringify(data, null, 2)}\n`, "utf8");
      await rename(tempPath, path);
    } catch (error) {
      await unlinkIfExists(tempPath);
      throw error;
    }
  }

  async function writeJsonBatch(
    entries: readonly { readonly path: string; readonly data: unknown }[],
  ): Promise<void> {
    await ensureRootDir();

    const staged = entries.map((entry) => ({
      ...entry,
      tempPath: `${entry.path}.${randomUUID()}.tmp`,
      backupPath: `${entry.path}.${randomUUID()}.bak`,
      hadOriginal: false,
      replaced: false,
    }));

    try {
      for (const entry of staged) {
        await writeFile(
          entry.tempPath,
          `${JSON.stringify(entry.data, null, 2)}\n`,
          "utf8",
        );
      }

      for (const entry of staged) {
        entry.hadOriginal = await moveIfExists(entry.path, entry.backupPath);
        await rename(entry.tempPath, entry.path);
        entry.replaced = true;
      }
    } catch (error) {
      for (const entry of staged) {
        await unlinkIfExists(entry.tempPath);

        if (entry.replaced) {
          await unlinkIfExists(entry.path);
        }

        if (entry.hadOriginal) {
          await moveIfExists(entry.backupPath, entry.path);
        }
      }

      throw error;
    }

    await Promise.all(staged.map((entry) => removeIfExists(entry.backupPath)));
  }

  return {
    async getRoles() {
      return readRoles();
    },

    async getCharacters() {
      return readCharacters();
    },

    async getPresets() {
      const roles = await readRoles();
      const characters = await readCharacters();
      return readPresets({ roles, characters });
    },

    async getPresenters() {
      return readPresenters();
    },

    async getAll() {
      const roles = await readRoles();
      const characters = await readCharacters();
      const presets = await readPresets({ roles, characters });
      const presenters = await readPresenters();
      return { roles, characters, presets, presenters };
    },

    async loadAll() {
      const roles = await readRoles();
      const characters = await readCharacters();
      const presets = await readPresets({ roles, characters });
      const presenters = await readPresenters();
      return { roles, characters, presets, presenters };
    },

    async saveRoles(roles) {
      const validatedRoles = validateRoleDefinitions(roles);
      await writeJson(rolesPath, validatedRoles);
    },

    async saveCharacters(characters) {
      const validatedCharacters = validateCharacterDefinitions(characters);
      await writeJson(charactersPath, validatedCharacters);
    },

    async savePresets(presets) {
      const roles = await readRoles();
      const characters = await readCharacters();
      const validatedPresets = validateGamePresets(presets, { roles, characters });
      await writeJson(presetsPath, validatedPresets);
    },

    async savePresenters(presenters) {
      await writeJson(presentersPath, validatePresenterDefinitions(presenters));
    },

    async saveAll(record) {
      const roles = validateRoleDefinitions(record.roles);
      const characters = validateCharacterDefinitions(record.characters);
      const presets = validateGamePresets(record.presets, { roles, characters });
      const presenters = validatePresenterDefinitions(record.presenters);

      await writeJsonBatch([
        { path: rolesPath, data: roles },
        { path: charactersPath, data: characters },
        { path: presetsPath, data: presets },
        { path: presentersPath, data: presenters },
      ]);
    },

    async withLibraryLock(operation) {
      const release = await acquireLock(libraryLockPath, ensureLocksDir);
      try {
        return await operation();
      } finally {
        await release();
      }
    },
  };
}

function isNodeError(error: unknown): error is NodeJS.ErrnoException {
  return error instanceof Error && "code" in error;
}

async function moveIfExists(from: string, to: string): Promise<boolean> {
  try {
    await rename(from, to);
    return true;
  } catch (error) {
    if (isNodeError(error) && error.code === "ENOENT") {
      return false;
    }

    throw error;
  }
}

async function unlinkIfExists(path: string): Promise<void> {
  try {
    await unlink(path);
  } catch (error) {
    if (isNodeError(error) && error.code === "ENOENT") {
      return;
    }

    throw error;
  }
}

async function removeIfExists(path: string): Promise<void> {
  await rm(path, { force: true, recursive: true });
}

async function acquireLock(
  path: string,
  ensureParentDir: () => Promise<void>,
): Promise<() => Promise<void>> {
  await ensureParentDir();

  while (true) {
    try {
      await mkdir(path);
      return () => rmdir(path);
    } catch (error) {
      if (isNodeError(error) && error.code === "EEXIST") {
        await setTimeout(10);
        continue;
      }

      throw error;
    }
  }
}
