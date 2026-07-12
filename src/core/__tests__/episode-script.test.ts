import { describe, expect, it } from "vitest";
import { createSeedGame } from "../game";
import { authorEpisodeScript } from "../episode-author";
import { LocalHeuristicLlmClient } from "../llm";
import type { LlmClient } from "../llm";
import {
  compileEpisodePlan,
  createEpisodeScriptSnapshot,
  episodeInputHash,
  planNextEpisodeDraft,
} from "../episode-script";
import type { DraftId, GameId } from "../types";

const game = createSeedGame({
  gameId: "episode_game" as GameId,
  createdAt: "2026-07-12T00:00:00.000Z",
});

describe("episode script", () => {
  it("authors narrative beats for every compiled speech step", async () => {
    const result = await authorEpisodeScript({
      game,
      llmClient: new LocalHeuristicLlmClient(),
      createdAt: "2026-07-12T00:00:00.000Z",
    });
    const speechSteps = result.script.steps.filter(
      (step) => step.speechBeat !== null,
    );

    expect(result.script.title).toContain("未明档案");
    expect(result.script.acts).toHaveLength(3);
    expect(speechSteps.length).toBeGreaterThan(0);
    expect(speechSteps.every((step) => step.speechBeat?.themeHook)).toBe(true);
  });

  it("keeps each author request below the provider timeout-sized beat batch", async () => {
    const local = new LocalHeuristicLlmClient();
    const timeoutOnLargeRequest: LlmClient = {
      async generateJson(request) {
        const beatCount = request.messages
          .map((message) => message.content)
          .join("\n")
          .match(/SPEECH_STEP\s+\d+/g)?.length ?? 0;
        if (beatCount > 12) {
          throw new Error(
            "LLM request failed before response: POST https://ark.example/chat/completions: fetch failed; cause: Headers Timeout Error",
          );
        }
        return local.generateJson(request);
      },
    };

    await expect(
      authorEpisodeScript({
        game,
        llmClient: timeoutOnLargeRequest,
        createdAt: "2026-07-12T00:00:00.000Z",
      }),
    ).resolves.toMatchObject({ script: { title: expect.any(String) } });
  });

  it("compiles a deterministic legal trace to game end", () => {
    const first = compileEpisodePlan(game);
    const second = compileEpisodePlan(game);

    expect(first.steps).toEqual(second.steps);
    expect(first.steps.at(-1)?.slot.type).toBe("game_ended");
    expect(first.simulatedEvents.at(-1)?.type).toBe("game_ended");
    expect(first.inputHash).toBe(episodeInputHash(game));
    expect(first.targetDurationMs).toBeGreaterThan(0);
  });

  it("binds the next runtime draft to the approved first step", () => {
    const plan = compileEpisodePlan(game);
    const script = createEpisodeScriptSnapshot({
      id: "episode_1",
      game,
      plan,
      title: "未明档案：第一卷",
      logline: "一份被篡改的档案迫使众人互相审视。",
      acts: [{ title: "开卷", summary: "身份被封入档案。" }],
      createdAt: "2026-07-12T00:00:00.000Z",
      provider: "test",
      model: "test",
    });

    const result = planNextEpisodeDraft({
      game,
      events: [],
      script,
      draftId: "runtime_draft" as DraftId,
      createdAt: "2026-07-12T00:01:00.000Z",
    });

    expect(result.draft?.type).toBe(script.steps[0]?.slot.type);
    expect(result.draft?.payload).toEqual(script.steps[0]?.plannedPayload);
    expect(result.actorBrief).toBeNull();
  });

  it("rejects a script when the game input changes", () => {
    const plan = compileEpisodePlan(game);
    const script = createEpisodeScriptSnapshot({
      id: "episode_2",
      game,
      plan,
      title: "未明档案：第二卷",
      logline: "第二份存根暴露了新的冲突。",
      acts: [],
      createdAt: "2026-07-12T00:00:00.000Z",
      provider: "test",
      model: "test",
    });

    expect(() =>
      planNextEpisodeDraft({
        game: { ...game, script: { ...game.script, theme: "changed" } },
        events: [],
        script,
        draftId: "runtime_draft" as DraftId,
        createdAt: "2026-07-12T00:01:00.000Z",
      }),
    ).toThrow("no longer matches game input");
  });
});
