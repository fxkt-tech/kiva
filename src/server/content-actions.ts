import type { ActorDefinition } from "@/core/actor-definition";
import type { GameRunMode } from "@/core/game-run-mode";
import type { GameScriptDefinition } from "@/core/game-script";
import type { Lineup } from "@/core/lineup";
import type { PresenterDefinition } from "@/core/presenter-definition";
import type { GameRecord, GameRepository } from "./game-repository";
import { createGameActions } from "./game-actions";
import type {
  ContentCatalog,
  ContentCatalogData,
  ContentCatalogSnapshot,
} from "./content-catalog";

export type ContentActions = ReturnType<typeof createContentActions>;

export function createContentActions(input: {
  readonly catalog: ContentCatalog;
  readonly gameRepository: GameRepository;
}) {
  const gameActions = createGameActions(input.gameRepository, {
    contentCatalog: input.catalog,
  });

  async function mutate<T>(
    change: (catalog: ContentCatalogSnapshot) => {
      readonly data: ContentCatalogData;
      readonly result: T;
    },
  ): Promise<T> {
    return input.catalog.update(change);
  }

  return {
    async getCatalog(): Promise<ContentCatalogSnapshot> {
      return input.catalog.load();
    },

    async saveActor(actor: ActorDefinition): Promise<ActorDefinition> {
      return mutate((catalog) => {
        assertNextRevision(catalog.actors, actor, "Actor");
        return {
          data: mutableData(catalog, {
            actors: upsertById(catalog.actors, actor),
          }),
          result: actor,
        };
      });
    },

    async duplicateActor(id: string): Promise<ActorDefinition> {
      return mutate((catalog) => {
        const source = requireById(catalog.actors, id, "Actor");
        const copy: ActorDefinition = {
          ...structuredClone(source),
          id: nextCopyId(id, catalog.actors.map((actor) => actor.id)),
          identity: {
            ...source.identity,
            name: `${source.identity.name} 副本`,
          },
          revision: 1,
        };
        return {
          data: mutableData(catalog, { actors: [...catalog.actors, copy] }),
          result: copy,
        };
      });
    },

    async setActorEnabled(
      id: string,
      enabled: boolean,
    ): Promise<ActorDefinition> {
      return mutate((catalog) => {
        const source = requireById(catalog.actors, id, "Actor");
        const actor = { ...source, enabled, revision: source.revision + 1 };
        return {
          data: mutableData(catalog, {
            actors: replaceById(catalog.actors, actor),
          }),
          result: actor,
        };
      });
    },

    async saveLineup(lineup: Lineup): Promise<Lineup> {
      return mutate((catalog) => {
        assertNextRevision(catalog.lineups, lineup, "Lineup");
        return {
          data: mutableData(catalog, {
            lineups: upsertById(catalog.lineups, lineup),
          }),
          result: lineup,
        };
      });
    },

    async duplicateLineup(id: string): Promise<Lineup> {
      return mutate((catalog) => {
        const source = requireById(catalog.lineups, id, "Lineup");
        const copy: Lineup = {
          ...structuredClone(source),
          id: nextCopyId(id, catalog.lineups.map((lineup) => lineup.id)),
          name: `${source.name} 副本`,
          revision: 1,
        };
        return {
          data: mutableData(catalog, { lineups: [...catalog.lineups, copy] }),
          result: copy,
        };
      });
    },

    async setLineupEnabled(id: string, enabled: boolean): Promise<Lineup> {
      return mutate((catalog) => {
        const source = requireById(catalog.lineups, id, "Lineup");
        const lineup = { ...source, enabled, revision: source.revision + 1 };
        return {
          data: mutableData(catalog, {
            lineups: replaceById(catalog.lineups, lineup),
          }),
          result: lineup,
        };
      });
    },

    async savePresenter(
      presenter: PresenterDefinition,
    ): Promise<PresenterDefinition> {
      return mutate((catalog) => ({
        data: mutableData(catalog, {
          presenters: upsertById(catalog.presenters, presenter),
        }),
        result: presenter,
      }));
    },

    async saveScript(script: GameScriptDefinition): Promise<GameScriptDefinition> {
      return mutate((catalog) => ({
        data: mutableData(catalog, {
          scripts: upsertById(catalog.scripts, script),
        }),
        result: script,
      }));
    },

    async duplicateScript(id: string): Promise<GameScriptDefinition> {
      return mutate((catalog) => {
        const source = requireById(catalog.scripts, id, "Script");
        const now = new Date().toISOString();
        const copy: GameScriptDefinition = {
          ...structuredClone(source),
          id: nextCopyId(id, catalog.scripts.map((script) => script.id)),
          name: `${source.name} 副本`,
          createdAt: now,
          updatedAt: now,
        };
        return {
          data: mutableData(catalog, { scripts: [...catalog.scripts, copy] }),
          result: copy,
        };
      });
    },

    async createGameFromLineup(
      lineupId: string,
      presenterId: string,
      scriptId?: string,
      runMode: GameRunMode = "game",
    ): Promise<GameRecord> {
      return gameActions.createGameFromLineupId(
        lineupId,
        presenterId,
        scriptId,
        runMode,
      );
    },

    async createGameFromTemporaryLineup(
      lineup: Lineup,
      presenterId: string,
      scriptId?: string,
      runMode: GameRunMode = "game",
    ): Promise<GameRecord> {
      const catalog = await input.catalog.load();
      return gameActions.createGameFromLineupRecord(
        lineup,
        catalog,
        presenterId,
        scriptId,
        runMode,
      );
    },
  };
}

function mutableData(
  catalog: ContentCatalogSnapshot,
  patch: Partial<ContentCatalogData>,
): ContentCatalogData {
  return {
    actors: patch.actors ?? catalog.actors,
    lineups: patch.lineups ?? catalog.lineups,
    presenters: patch.presenters ?? catalog.presenters,
    scripts: patch.scripts ?? catalog.scripts,
  };
}

function upsertById<T extends { readonly id: string }>(
  values: readonly T[],
  next: T,
): readonly T[] {
  return values.some((value) => value.id === next.id)
    ? replaceById(values, next)
    : [...values, next];
}

function replaceById<T extends { readonly id: string }>(
  values: readonly T[],
  next: T,
): readonly T[] {
  return values.map((value) => (value.id === next.id ? next : value));
}

function requireById<T extends { readonly id: string }>(
  values: readonly T[],
  id: string,
  label: string,
): T {
  const value = values.find((candidate) => candidate.id === id);
  if (!value) throw new Error(`${label} not found: ${id}`);
  return value;
}

function nextCopyId(id: string, existingIds: readonly string[]): string {
  const existing = new Set(existingIds);
  for (let index = 1; ; index += 1) {
    const suffix = index === 1 ? "copy" : `copy_${index}`;
    const candidate = `${id}_${suffix}`;
    if (!existing.has(candidate)) return candidate;
  }
}

function assertNextRevision<T extends { readonly id: string; readonly revision: number }>(
  values: readonly T[],
  next: T,
  label: string,
): void {
  const current = values.find((value) => value.id === next.id);
  const expected = current ? current.revision + 1 : 1;
  if (next.revision !== expected) {
    throw new Error(
      `${label} ${next.id} revision conflict: expected ${expected}, received ${next.revision}`,
    );
  }
}
