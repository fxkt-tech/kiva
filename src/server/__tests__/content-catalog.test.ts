import { mkdtemp, readdir, readFile, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { afterEach, describe, expect, it } from "vitest";
import { RULE_ROLES } from "@/core/rule-role";
import { seedActors } from "@/seeds/actors";
import { seedLineups } from "@/seeds/lineups";
import { seedPresenters } from "@/seeds/presenters";
import { seedScripts } from "@/seeds/scripts";
import {
  createContentCatalog,
  type ContentCatalogData,
} from "../content-catalog";

const tempDirs: string[] = [];

afterEach(async () => {
  await Promise.all(
    tempDirs.splice(0).map((dir) => rm(dir, { recursive: true, force: true })),
  );
});

describe("Content Catalog", () => {
  it("atomically persists the four editable collections and exposes code Rule Roles", async () => {
    const rootDir = await createTempDir();
    const catalog = createContentCatalog(rootDir);

    await catalog.save(seedData());

    await expect(catalog.load()).resolves.toEqual({
      ruleRoles: RULE_ROLES,
      ...seedData(),
    });
    expect(
      (await readdir(rootDir)).filter((name) => name.endsWith(".json")).sort(),
    ).toEqual([
      "actors.json",
      "lineups.json",
      "presenters.json",
      "scripts.json",
    ]);
    for (const filename of [
      "actors.json",
      "lineups.json",
      "presenters.json",
      "scripts.json",
    ]) {
      const content = await readFile(join(rootDir, filename), "utf8");
      expect(content.endsWith("\n")).toBe(true);
      expect(content).toContain("\n  {");
    }
  });

  it("requires one stable Qin Chuan identity and rejects aliases", async () => {
    const catalog = createContentCatalog(await createTempDir());
    const alias = {
      ...structuredClone(seedActors.find((actor) => actor.id === "qiao_ke")!),
      id: "qin_alias",
      identity: {
        ...seedActors.find((actor) => actor.id === "qiao_ke")!.identity,
        name: "秦川",
      },
    };

    await expect(
      catalog.save({ ...seedData(), actors: [...seedActors, alias] }),
    ).rejects.toThrow('Stable Actor must remain exactly "秦川" / qin_chuan');
  });

  it("rejects an Actor pool that cannot supply a production 12-player cast", async () => {
    const catalog = createContentCatalog(await createTempDir());

    await expect(
      catalog.save({ ...seedData(), actors: seedActors.slice(0, 11) }),
    ).rejects.toThrow(
      "Actor pool is not production-ready: At least 12 enabled Actors are required",
    );
  });

  it("validates the complete catalog before replacing any file", async () => {
    const rootDir = await createTempDir();
    const catalog = createContentCatalog(rootDir);
    await catalog.save(seedData());
    const invalidLineup = {
      ...seedLineups[0]!,
      revision: 2,
      seats: seedLineups[0]!.seats.map((seat) =>
        seat.seatNo === 1 ? { ...seat, actorId: "missing_actor" } : seat,
      ),
    };

    await expect(
      catalog.save({ ...seedData(), lineups: [invalidLineup] }),
    ).rejects.toThrow("references unknown Actor: missing_actor");
    await expect(catalog.load()).resolves.toEqual({
      ruleRoles: RULE_ROLES,
      ...seedData(),
    });
  });
});

function seedData(): ContentCatalogData {
  return {
    actors: seedActors,
    lineups: seedLineups,
    presenters: seedPresenters,
    scripts: seedScripts,
  };
}

async function createTempDir(): Promise<string> {
  const dir = await mkdtemp(join(tmpdir(), "kiva-content-catalog-"));
  tempDirs.push(dir);
  return dir;
}
