import { randomUUID } from "node:crypto";
import { mkdir, readFile, rename, writeFile } from "node:fs/promises";
import { join } from "node:path";
import {
  validateCharacterDefinitions,
  type CharacterDefinition,
} from "@/core/character-definition";
import { validateGamePresets, type GamePreset } from "@/core/game-preset";
import {
  validateRoleDefinitions,
  type RoleDefinition,
} from "@/core/role-definition";

export type LibraryRecord = {
  readonly roles: readonly RoleDefinition[];
  readonly characters: readonly CharacterDefinition[];
  readonly presets: readonly GamePreset[];
};

export type LibraryRepository = {
  readonly getRoles: () => Promise<readonly RoleDefinition[]>;
  readonly getCharacters: () => Promise<readonly CharacterDefinition[]>;
  readonly getPresets: () => Promise<readonly GamePreset[]>;
  readonly getAll: () => Promise<LibraryRecord>;
  readonly saveRoles: (roles: readonly RoleDefinition[]) => Promise<void>;
  readonly saveCharacters: (
    characters: readonly CharacterDefinition[],
  ) => Promise<void>;
  readonly savePresets: (presets: readonly GamePreset[]) => Promise<void>;
  readonly saveAll: (record: LibraryRecord) => Promise<void>;
};

export function createLibraryRepository(rootDir = ".kiva-data"): LibraryRepository {
  const rolesPath = join(rootDir, "roles.json");
  const charactersPath = join(rootDir, "characters.json");
  const presetsPath = join(rootDir, "presets.json");

  async function ensureRootDir(): Promise<void> {
    await mkdir(rootDir, { recursive: true });
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
    await writeFile(tempPath, `${JSON.stringify(data, null, 2)}\n`, "utf8");
    await rename(tempPath, path);
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

    async getAll() {
      const roles = await readRoles();
      const characters = await readCharacters();
      const presets = await readPresets({ roles, characters });
      return { roles, characters, presets };
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

    async saveAll(record) {
      const roles = validateRoleDefinitions(record.roles);
      const characters = validateCharacterDefinitions(record.characters);
      const presets = validateGamePresets(record.presets, { roles, characters });

      await writeJson(rolesPath, roles);
      await writeJson(charactersPath, characters);
      await writeJson(presetsPath, presets);
    },
  };
}

function isNodeError(error: unknown): error is NodeJS.ErrnoException {
  return error instanceof Error && "code" in error;
}
