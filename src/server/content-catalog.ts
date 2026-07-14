import { randomUUID } from "node:crypto";
import {
  mkdir,
  readFile,
  rename,
  rm,
  unlink,
  writeFile,
} from "node:fs/promises";
import { join } from "node:path";
import {
  assertActorLibraryInvariants,
  validateActorDefinitions,
  type ActorDefinition,
} from "@/core/actor-definition";
import { diagnoseActorPool } from "@/core/actor-diagnostics";
import {
  validateGameScriptDefinitions,
  type GameScriptDefinition,
} from "@/core/game-script";
import { validateLineups, type Lineup } from "@/core/lineup";
import {
  validatePresenterDefinitions,
  type PresenterDefinition,
} from "@/core/presenter-definition";
import { RULE_ROLES, type RuleRole } from "@/core/rule-role";
import { acquireDirectoryLock } from "./directory-lock";

export type ContentCatalogData = {
  readonly actors: readonly ActorDefinition[];
  readonly lineups: readonly Lineup[];
  readonly presenters: readonly PresenterDefinition[];
  readonly scripts: readonly GameScriptDefinition[];
};

export type ContentCatalogSnapshot = ContentCatalogData & {
  readonly ruleRoles: readonly RuleRole[];
};

export type ContentCatalog = {
  readonly load: () => Promise<ContentCatalogSnapshot>;
  readonly save: (data: ContentCatalogData) => Promise<void>;
  readonly update: <T>(
    change: (catalog: ContentCatalogSnapshot) => {
      readonly data: ContentCatalogData;
      readonly result: T;
    },
  ) => Promise<T>;
};

export function createContentCatalog(rootDir = "kivdb"): ContentCatalog {
  const paths = {
    actors: join(rootDir, "actors.json"),
    lineups: join(rootDir, "lineups.json"),
    presenters: join(rootDir, "presenters.json"),
    scripts: join(rootDir, "scripts.json"),
  } as const;
  const lockPath = join(rootDir, "locks", "content-catalog.lock");

  async function loadUnlocked(): Promise<ContentCatalogSnapshot> {
    const actors = validatedActors(await readJson(paths.actors));
    const lineups = validateLineups(await readJson(paths.lineups), actors);
    const presenters = validatedPresenters(
      await readJson(paths.presenters),
    );
    const scripts = validatedScripts(await readJson(paths.scripts));
    return {
      ruleRoles: structuredClone(RULE_ROLES),
      actors,
      lineups,
      presenters,
      scripts,
    };
  }

  async function saveUnlocked(data: ContentCatalogData): Promise<void> {
    const actors = validatedActors(data.actors);
    const validated: ContentCatalogData = {
      actors,
      lineups: validateLineups(data.lineups, actors),
      presenters: validatedPresenters(data.presenters),
      scripts: validatedScripts(data.scripts),
    };
    await writeJsonBatch(
      Object.entries(paths).map(([key, path]) => ({
        path,
        data: validated[key as keyof ContentCatalogData],
      })),
      rootDir,
    );
  }

  async function underLock<T>(operation: () => Promise<T>): Promise<T> {
    const release = await acquireDirectoryLock(lockPath);
    try {
      return await operation();
    } finally {
      await release();
    }
  }

  return {
    load() {
      return underLock(loadUnlocked);
    },
    async save(data) {
      await underLock(() => saveUnlocked(data));
    },
    update(change) {
      return underLock(async () => {
        const next = change(await loadUnlocked());
        await saveUnlocked(next.data);
        return next.result;
      });
    },
  };
}

function validatedActors(value: unknown): readonly ActorDefinition[] {
  const actors = validateActorDefinitions(value);
  assertActorLibraryInvariants(actors);
  const issues = diagnoseActorPool(actors).hardIssues;
  if (issues.length > 0) {
    throw new Error(`Actor pool is not production-ready: ${issues.join("; ")}`);
  }
  return actors;
}

function validatedPresenters(value: unknown): readonly PresenterDefinition[] {
  const presenters = validatePresenterDefinitions(value);
  const enabled = presenters.filter((presenter) => presenter.enabled);
  if (enabled.length !== 1) {
    throw new Error(
      `Content Catalog requires exactly one enabled Presenter; found ${enabled.length}`,
    );
  }
  return presenters;
}

function validatedScripts(value: unknown): readonly GameScriptDefinition[] {
  const scripts = validateGameScriptDefinitions(value);
  if (!scripts.some((script) => script.enabled)) {
    throw new Error("Content Catalog requires at least one enabled Script");
  }
  return scripts;
}

async function readJson(path: string): Promise<unknown> {
  try {
    return JSON.parse(await readFile(path, "utf8"));
  } catch (error) {
    if (isNodeError(error) && error.code === "ENOENT") return [];
    throw error;
  }
}

async function writeJsonBatch(
  entries: readonly { readonly path: string; readonly data: unknown }[],
  rootDir: string,
): Promise<void> {
  await mkdir(rootDir, { recursive: true });
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
      if (entry.replaced) await unlinkIfExists(entry.path);
      if (entry.hadOriginal) await moveIfExists(entry.backupPath, entry.path);
    }
    throw error;
  }
  await Promise.all(staged.map((entry) => rm(entry.backupPath, { force: true })));
}

async function moveIfExists(from: string, to: string): Promise<boolean> {
  try {
    await rename(from, to);
    return true;
  } catch (error) {
    if (isNodeError(error) && error.code === "ENOENT") return false;
    throw error;
  }
}

async function unlinkIfExists(path: string): Promise<void> {
  try {
    await unlink(path);
  } catch (error) {
    if (isNodeError(error) && error.code === "ENOENT") return;
    throw error;
  }
}

function isNodeError(error: unknown): error is NodeJS.ErrnoException {
  return error instanceof Error && "code" in error;
}
