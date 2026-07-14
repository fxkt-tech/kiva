import { mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { afterEach, describe, expect, it } from "vitest";
import { seedActors } from "@/seeds/actors";
import { seedLineups } from "@/seeds/lineups";
import { seedPresenters } from "@/seeds/presenters";
import { seedScripts } from "@/seeds/scripts";
import { createContentActions } from "../content-actions";
import { createContentCatalog } from "../content-catalog";
import { createGameRepository } from "../game-repository";

const tempDirs: string[] = [];

afterEach(async () => {
  await Promise.all(
    tempDirs.splice(0).map((dir) => rm(dir, { recursive: true, force: true })),
  );
});

describe("Content actions", () => {
  it("adds a complete new Actor at revision one", async () => {
    const { actions } = await createSeededActions();
    const source = seedActors.find((actor) => actor.id === "qiao_ke")!;
    const actor = {
      ...structuredClone(source),
      id: "new_observer",
      identity: { ...source.identity, name: "新观察者" },
      revision: 1,
    };

    await actions.saveActor(actor);

    expect(
      (await actions.getCatalog()).actors.find((item) => item.id === actor.id),
    ).toEqual(actor);
  });

  it("duplicates every Actor without changing the stable Qin Chuan identity", async () => {
    const { actions } = await createSeededActions();

    const copy = await actions.duplicateActor("qiao_ke");
    const qinCopy = await actions.duplicateActor("qin_chuan");

    expect(copy).toMatchObject({
      id: "qiao_ke_copy",
      identity: { name: "乔可 副本" },
      revision: 1,
    });
    expect(qinCopy).toMatchObject({
      id: "qin_chuan_copy",
      identity: { name: "秦川 副本" },
      revision: 1,
    });
    const catalog = await actions.getCatalog();
    expect(
      catalog.actors.filter(
        (actor) => actor.id === "qin_chuan" && actor.identity.name === "秦川",
      ),
    ).toHaveLength(1);
  });

  it("rejects stale Actor revisions instead of overwriting a concurrent edit", async () => {
    const { actions } = await createSeededActions();
    const source = seedActors.find((actor) => actor.id === "qiao_ke")!;
    const edited = {
      ...source,
      core: { ...source.core, drive: "第一次编辑后的驱动力。" },
      revision: 2,
    };

    await actions.saveActor(edited);
    await expect(actions.saveActor(edited)).rejects.toThrow(
      "revision conflict: expected 3, received 2",
    );
    const saved = (await actions.getCatalog()).actors.find(
      (actor) => actor.id === "qiao_ke",
    );
    expect(saved).toMatchObject({
      id: "qiao_ke",
      core: { drive: "第一次编辑后的驱动力。" },
      revision: 2,
    });
  });

  it("serializes concurrent catalog edits before checking revisions", async () => {
    const { actions } = await createSeededActions();
    const source = seedActors.find((actor) => actor.id === "qiao_ke")!;
    const edits = ["并发编辑甲。", "并发编辑乙。"].map((drive) => ({
      ...source,
      core: { ...source.core, drive },
      revision: 2,
    }));

    const results = await Promise.allSettled(edits.map(actions.saveActor));

    expect(results.filter((result) => result.status === "fulfilled")).toHaveLength(1);
    expect(results.filter((result) => result.status === "rejected")).toHaveLength(1);
    const saved = (await actions.getCatalog()).actors.find(
      (actor) => actor.id === source.id,
    );
    expect(saved?.revision).toBe(2);
    expect(edits.map((actor) => actor.core.drive)).toContain(saved?.core.drive);
  });

  it("duplicates a Script as a new selectable content object", async () => {
    const { actions } = await createSeededActions();

    const copy = await actions.duplicateScript(seedScripts[0]!.id);

    expect(copy).toMatchObject({
      id: `${seedScripts[0]!.id}_copy`,
      name: `${seedScripts[0]!.name} 副本`,
      enabled: true,
    });
    expect((await actions.getCatalog()).scripts.map((script) => script.id)).toContain(
      copy.id,
    );
  });

  it("creates a legal temporary game from twelve selected Actors without Qin Chuan", async () => {
    const { actions } = await createSeededActions();
    const replacement = seedActors.find((actor) => actor.id === "bai_qi")!;
    const lineup = {
      ...seedLineups[0]!,
      id: "temporary_without_qin",
      name: "无秦川测试局",
      seats: seedLineups[0]!.seats.map((seat) =>
        seat.actorId === "qin_chuan"
          ? { ...seat, actorId: replacement.id }
          : seat,
      ),
    };

    const record = await actions.createGameFromTemporaryLineup(
      lineup,
      seedPresenters[0]!.id,
      seedScripts[0]!.id,
    );

    expect(record.game.players).toHaveLength(12);
    expect(
      record.game.players.some(
        (player) => player.actor.sourceId === "qin_chuan",
      ),
    ).toBe(false);
    expect(record.game.players.map((player) => player.actor.sourceId)).toContain(
      "bai_qi",
    );
  });

  it("blocks game creation from a disabled persisted Lineup", async () => {
    const { actions } = await createSeededActions();
    const lineupId = seedLineups[0]!.id;

    await actions.setLineupEnabled(lineupId, false);

    await expect(
      actions.createGameFromLineup(
        lineupId,
        seedPresenters[0]!.id,
        seedScripts[0]!.id,
      ),
    ).rejects.toThrow(`Lineup not found or disabled: ${lineupId}`);
  });
});

async function createSeededActions() {
  const rootDir = await createTempDir();
  const catalog = createContentCatalog(rootDir);
  await catalog.save({
    actors: seedActors,
    lineups: seedLineups,
    presenters: seedPresenters,
    scripts: seedScripts,
  });
  return {
    actions: createContentActions({
      catalog,
      gameRepository: createGameRepository(rootDir),
    }),
  };
}

async function createTempDir(): Promise<string> {
  const dir = await mkdtemp(join(tmpdir(), "kiva-content-actions-"));
  tempDirs.push(dir);
  return dir;
}
