import {
  diagnoseCharacter,
  diagnosePreset,
  diagnoseRole,
  type LibraryDiagnostic,
} from "@/core/library-diagnostics";
import type { CharacterDefinition } from "@/core/character-definition";
import type { GamePreset } from "@/core/game-preset";
import type { RoleDefinition } from "@/core/role-definition";
import { createGameActions } from "./game-actions";
import type { GameRecord, GameRepository } from "./game-repository";
import type { LibraryRecord, LibraryRepository } from "./library-repository";

export type LibraryDiagnostics = {
  readonly roles: Readonly<Record<string, LibraryDiagnostic>>;
  readonly characters: Readonly<Record<string, LibraryDiagnostic>>;
  readonly presets: Readonly<Record<string, LibraryDiagnostic>>;
};

export type LibraryActionsRecord = LibraryRecord & {
  readonly diagnostics: LibraryDiagnostics;
};

export type CreateLibraryActionsOptions = {
  readonly libraryRepository: LibraryRepository;
  readonly gameRepository: GameRepository;
};

export type LibraryActions = ReturnType<typeof createLibraryActions>;

export function createLibraryActions({
  libraryRepository,
  gameRepository,
}: CreateLibraryActionsOptions) {
  const gameActions = createGameActions(gameRepository, { libraryRepository });

  async function loadLibrary(): Promise<LibraryRecord> {
    return libraryRepository.loadAll();
  }

  async function saveLibrary(record: LibraryRecord): Promise<void> {
    await libraryRepository.saveAll(record);
  }

  return {
    async getLibrary(): Promise<LibraryActionsRecord> {
      return libraryRepository.withLibraryLock(async () => {
        const library = await loadLibrary();
        return {
          ...library,
          diagnostics: diagnosticsForLibrary(library),
        };
      });
    },

    async saveRole(role: RoleDefinition): Promise<RoleDefinition> {
      return libraryRepository.withLibraryLock(async () => {
        const library = await loadLibrary();
        const roles = upsertById(library.roles, role);
        await saveLibrary({ ...library, roles });
        return role;
      });
    },

    async saveCharacter(
      character: CharacterDefinition,
    ): Promise<CharacterDefinition> {
      return libraryRepository.withLibraryLock(async () => {
        const library = await loadLibrary();
        const characters = upsertById(library.characters, character);
        await saveLibrary({ ...library, characters });
        return character;
      });
    },

    async savePreset(preset: GamePreset): Promise<GamePreset> {
      return libraryRepository.withLibraryLock(async () => {
        const library = await loadLibrary();
        const presets = upsertById(library.presets, preset);
        await saveLibrary({ ...library, presets });
        return preset;
      });
    },

    async duplicateRole(id: string): Promise<RoleDefinition> {
      return libraryRepository.withLibraryLock(async () => {
        const library = await loadLibrary();
        const source = requireById(library.roles, id, "Role");
        const timestamp = now();
        const copy: RoleDefinition = {
          ...source,
          id: nextCopyId(
            id,
            library.roles.map((role) => role.id),
          ),
          createdAt: timestamp,
          updatedAt: timestamp,
        };
        await saveLibrary({ ...library, roles: [...library.roles, copy] });
        return copy;
      });
    },

    async duplicateCharacter(id: string): Promise<CharacterDefinition> {
      return libraryRepository.withLibraryLock(async () => {
        const library = await loadLibrary();
        const source = requireById(library.characters, id, "Character");
        const timestamp = now();
        const copy: CharacterDefinition = {
          ...source,
          id: nextCopyId(
            id,
            library.characters.map((character) => character.id),
          ),
          createdAt: timestamp,
          updatedAt: timestamp,
        };
        await saveLibrary({
          ...library,
          characters: [...library.characters, copy],
        });
        return copy;
      });
    },

    async duplicatePreset(id: string): Promise<GamePreset> {
      return libraryRepository.withLibraryLock(async () => {
        const library = await loadLibrary();
        const source = requireById(library.presets, id, "Game preset");
        const timestamp = now();
        const copy: GamePreset = {
          ...source,
          id: nextCopyId(
            id,
            library.presets.map((preset) => preset.id),
          ),
          createdAt: timestamp,
          updatedAt: timestamp,
        };
        await saveLibrary({ ...library, presets: [...library.presets, copy] });
        return copy;
      });
    },

    async setRoleEnabled(
      id: string,
      enabled: boolean,
    ): Promise<RoleDefinition> {
      return libraryRepository.withLibraryLock(async () => {
        const library = await loadLibrary();
        const role = requireById(library.roles, id, "Role");
        const updatedRole = { ...role, enabled, updatedAt: now() };
        const roles = replaceById(library.roles, updatedRole);
        await saveLibrary({ ...library, roles });
        return updatedRole;
      });
    },

    async setCharacterEnabled(
      id: string,
      enabled: boolean,
    ): Promise<CharacterDefinition> {
      return libraryRepository.withLibraryLock(async () => {
        const library = await loadLibrary();
        const character = requireById(library.characters, id, "Character");
        const updatedCharacter = { ...character, enabled, updatedAt: now() };
        const characters = replaceById(library.characters, updatedCharacter);
        await saveLibrary({ ...library, characters });
        return updatedCharacter;
      });
    },

    async setPresetEnabled(id: string, enabled: boolean): Promise<GamePreset> {
      return libraryRepository.withLibraryLock(async () => {
        const library = await loadLibrary();
        const preset = requireById(library.presets, id, "Game preset");
        const updatedPreset = { ...preset, enabled, updatedAt: now() };
        const presets = replaceById(library.presets, updatedPreset);
        await saveLibrary({ ...library, presets });
        return updatedPreset;
      });
    },

    async createGameFromPreset(presetId: string): Promise<GameRecord> {
      return libraryRepository.withLibraryLock(async () =>
        gameActions.createGameFromPresetId(presetId),
      );
    },

    async createGameFromTemporaryPreset(preset: GamePreset): Promise<GameRecord> {
      return libraryRepository.withLibraryLock(async () => {
        const library = await loadLibrary();
        return gameActions.createGameFromPresetRecord(preset, library);
      });
    },
  };
}

function diagnosticsForLibrary(library: LibraryRecord): LibraryDiagnostics {
  return {
    roles: Object.fromEntries(
      library.roles.map((role) => [
        role.id,
        diagnoseRole({ ...library, role }),
      ]),
    ),
    characters: Object.fromEntries(
      library.characters.map((character) => [
        character.id,
        diagnoseCharacter({ ...library, character }),
      ]),
    ),
    presets: Object.fromEntries(
      library.presets.map((preset) => [
        preset.id,
        diagnosePreset({ ...library, preset }),
      ]),
    ),
  };
}

function upsertById<T extends { readonly id: string }>(
  items: readonly T[],
  nextItem: T,
): readonly T[] {
  return items.some((item) => item.id === nextItem.id)
    ? replaceById(items, nextItem)
    : [...items, nextItem];
}

function replaceById<T extends { readonly id: string }>(
  items: readonly T[],
  nextItem: T,
): readonly T[] {
  return items.map((item) => (item.id === nextItem.id ? nextItem : item));
}

function requireById<T extends { readonly id: string }>(
  items: readonly T[],
  id: string,
  label: string,
): T {
  const item = items.find((candidate) => candidate.id === id);
  if (!item) {
    throw new Error(`${label} not found: ${id}`);
  }

  return item;
}

function nextCopyId(id: string, existingIds: readonly string[]): string {
  const existing = new Set(existingIds);
  const baseId = `${id}_copy`;
  if (!existing.has(baseId)) {
    return baseId;
  }

  for (let suffix = 2; ; suffix += 1) {
    const candidate = `${baseId}_${suffix}`;
    if (!existing.has(candidate)) {
      return candidate;
    }
  }
}

function now(): string {
  return new Date().toISOString();
}
