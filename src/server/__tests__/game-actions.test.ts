import { describe, expect, it } from "vitest";
import { RULE_ROLES } from "@/core/rule-role";
import { LocalHeuristicLlmClient, type LlmClient } from "@/core/llm";
import { seedActors } from "@/seeds/actors";
import { seedLineups } from "@/seeds/lineups";
import { seedPresenters } from "@/seeds/presenters";
import { seedScripts } from "@/seeds/scripts";
import { createGameActions } from "../game-actions";
import type { ContentCatalog, ContentCatalogData } from "../content-catalog";
import type { GameRecord, GameRepository } from "../game-repository";
import type { GameId } from "@/core/types";

describe("game actions with Content Catalog", () => {
  it("creates a schema-v2 game from the selected Lineup", async () => {
    const repository = memoryGameRepository();
    const actions = createGameActions(repository, {
      contentCatalog: memoryCatalog(),
    });
    const record = await actions.createGameFromLineupId(
      seedLineups[0]!.id,
      seedPresenters[0]!.id,
      seedScripts[0]!.id,
      "game",
    );
    expect(record.schemaVersion).toBe(2);
    expect(record.game.players.map((player) => player.actor.sourceId)).toEqual(
      seedLineups[0]!.seats.map((seat) => seat.actorId),
    );
    expect(await repository.get(record.game.id)).toEqual(record);
  });

  it("continues a new game without reading legacy library objects", async () => {
    const repository = memoryGameRepository();
    const actions = createGameActions(repository, {
      contentCatalog: memoryCatalog(),
    });
    const created = await actions.createGame();
    const continued = await actions.continueGame(created.game.id);
    expect(continued.draft?.type).toBe("role_assigned");
    expect(continued.draft?.payload).toMatchObject({
      role: continued.game.players[0]!.ruleRole.id,
    });
  });

  it("runs the segmented Script Author against the actual cast", async () => {
    const repository = memoryGameRepository();
    const replacement = seedActors.find((actor) => actor.id === "bai_qi")!;
    const lineup = {
      ...seedLineups[0]!,
      id: "script_cast_without_qin",
      seats: seedLineups[0]!.seats.map((seat) =>
        seat.actorId === "qin_chuan"
          ? { ...seat, actorId: replacement.id }
          : seat,
      ),
    };
    const actions = createGameActions(repository, {
      contentCatalog: memoryCatalog({ lineups: [lineup] }),
      llmClient: new LocalHeuristicLlmClient(),
    });
    const created = await actions.createGameFromLineupId(
      lineup.id,
      seedPresenters[0]!.id,
      seedScripts[0]!.id,
      "scripted",
    );
    const generated = await actions.generateEpisodeScript(created.game.id);
    expect(generated.episodeScript?.status).toBe("review");
    if (generated.episodeScript?.status !== "review") return;
    expect(generated.episodeScript.candidate.castDirections).toHaveLength(12);
    expect(generated.episodeScript.requests.length).toBeGreaterThan(3);
    expect(
      generated.episodeScript.requests.every(
        (request) => request.promptVersion === "episode-author:v4",
      ),
    ).toBe(true);
    const authorInput = JSON.stringify(
      generated.episodeScript.requests.map((request) => request.request.messages),
    );
    expect(authorInput).toContain("bai_qi");
    expect(authorInput).not.toContain("qin_chuan");
  }, 20_000);

  it("runs one guarded Script Author task per background job", async () => {
    const repository = memoryGameRepository();
    const local = new LocalHeuristicLlmClient();
    let providerCalls = 0;
    const llmClient: LlmClient = {
      async generateJson(request) {
        providerCalls += 1;
        return local.generateJson(request);
      },
    };
    const actions = createGameActions(repository, {
      contentCatalog: memoryCatalog(),
      llmClient,
    });
    const created = await actions.createGameFromLineupId(
      seedLineups[0]!.id,
      seedPresenters[0]!.id,
      seedScripts[0]!.id,
      "scripted",
    );

    const storyJobId = await actions.startEpisodeScriptGeneration(
      created.game.id,
      null,
    );
    expect(storyJobId).toEqual(expect.any(String));
    expect(
      await actions.startEpisodeScriptGeneration(created.game.id, null),
    ).toBeNull();
    if (!storyJobId) return;

    const story = await actions.runEpisodeScriptGeneration(
      created.game.id,
      storyJobId,
    );
    expect(story.episodeScript?.status).toBe("ready");
    if (story.episodeScript?.status !== "ready") return;
    expect(story.episodeScript.workspace.story).not.toBeNull();
    expect(story.episodeScript.workspace.ensemble).toBeNull();
    expect(providerCalls).toBe(1);

    expect(
      await actions.startEpisodeScriptGeneration(
        created.game.id,
        "stale_job",
      ),
    ).toBeNull();
    const ensembleJobId = await actions.startEpisodeScriptGeneration(
      created.game.id,
      story.episodeScript.jobId,
    );
    expect(ensembleJobId).toEqual(expect.any(String));
    expect(
      await actions.startEpisodeScriptGeneration(
        created.game.id,
        story.episodeScript.jobId,
      ),
    ).toBeNull();
    if (!ensembleJobId) return;

    await actions.runEpisodeScriptGeneration(created.game.id, storyJobId);
    expect(providerCalls).toBe(1);

    const ensemble = await actions.runEpisodeScriptGeneration(
      created.game.id,
      ensembleJobId,
    );
    expect(ensemble.episodeScript?.status).toBe("ready");
    if (ensemble.episodeScript?.status !== "ready") return;
    expect(ensemble.episodeScript.workspace.ensemble).not.toBeNull();
    expect(ensemble.episodeScript.workspace.castDirections).toHaveLength(0);
    expect(
      ensemble.episodeScript.requests.map((request) => request.task.kind),
    ).toEqual(["story", "ensemble"]);
    expect(providerCalls).toBe(2);
  });
});

function memoryCatalog(
  patch: Partial<ContentCatalogData> = {},
): ContentCatalog {
  let data: ContentCatalogData = {
    actors: patch.actors ?? seedActors,
    lineups: patch.lineups ?? seedLineups,
    presenters: patch.presenters ?? seedPresenters,
    scripts: patch.scripts ?? seedScripts,
  };
  return {
    async load() {
      return { ruleRoles: RULE_ROLES, ...structuredClone(data) };
    },
    async save(next) {
      data = structuredClone(next);
    },
    async update(change) {
      const next = change({ ruleRoles: RULE_ROLES, ...structuredClone(data) });
      data = structuredClone(next.data);
      return next.result;
    },
  };
}

function memoryGameRepository(): GameRepository {
  const records = new Map<GameId, GameRecord>();
  return {
    async get(id) { return structuredClone(records.get(id) ?? null); },
    async list() { return structuredClone([...records.values()]); },
    async save(record) { records.set(record.game.id, structuredClone(record)); },
    async delete(id) { records.delete(id); },
    voicePath() { return "/tmp/voice.mp3"; },
    async voiceTempPath() { return "/tmp/voice.tmp"; },
    async publishVoice() { return "voice.mp3"; },
    async withGameLock(_id, operation) { return operation(); },
  };
}
