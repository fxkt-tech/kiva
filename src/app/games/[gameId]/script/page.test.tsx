import React from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { createEpisodeAuthorWorkspace } from "@/core/episode-author";
import { createSeedGame } from "@/core/game";
import type { GameId } from "@/core/types";
import type { EpisodeAuthorRequestRecord } from "@/core/episode-script";
import {
  GAME_RECORD_SCHEMA_VERSION,
  type GameRecord,
} from "@/server/game-repository";
import {
  EpisodeAuthorGeneratingStatus,
  EpisodeAuthorRequests,
  EpisodeWorkspace,
} from "./page";

describe("ScriptPreparationPage", () => {
  it("offers manual generation while a newly created scripted game is idle", () => {
    const gameId = "idle-scripted-game" as GameId;
    const record: GameRecord = {
      schemaVersion: GAME_RECORD_SCHEMA_VERSION,
      game: {
        ...createSeedGame({
          gameId,
          createdAt: "2026-07-14T00:00:00.000Z",
        }),
        runMode: "scripted",
      },
      events: [],
      draft: null,
      generations: [],
      voiceArtifactsByEventId: {},
      episodeScript: { status: "idle" },
    };

    const html = renderToStaticMarkup(
      React.createElement(EpisodeWorkspace, { record }),
    );

    expect(html).toContain("开始生成剧本");
    expect(html).toContain("系统会先用当前角色与规则模拟完整终局");
    expect(html).not.toContain("正在生成并模拟整局剧本");
  });

  it("renders one reusable LLM details control per script-author request", () => {
    const request: EpisodeAuthorRequestRecord = {
      id: "episode_request_1",
      task: { kind: "story" },
      status: "success",
      promptVersion: "episode-author:v3",
      provider: "openai-compatible",
      model: "author-model",
      request: {
        schemaName: "werewolf_episode_story_v3",
        systemPrompt: "Author the outline.",
        messages: [{ role: "user", content: "Game context" }],
      },
      tokenUsage: {
        promptTokens: 100,
        completionTokens: 20,
        totalTokens: 120,
      },
      finishReason: "stop",
      rawOutput: '{"title":"未明档案"}',
      parsedOutput: { title: "未明档案" },
      error: null,
      createdAt: "2026-07-14T00:00:00.000Z",
    };

    const html = renderToStaticMarkup(
      React.createElement(EpisodeAuthorRequests, { requests: [request] }),
    );

    expect(html).toContain("LLM requests");
    expect(html).toContain("1 requests · 120 tokens");
    expect(html).toContain("openai-compatible/author-model");
    expect(html).toContain('aria-label="Story spine LLM details"');
    expect(html).toContain("Author the outline.");
    expect(html).toContain("Token usage");
  });

  it("shows the persisted Script Author Agent phase while generating", () => {
    const gameId = "generating-scripted-game" as GameId;
    const game = {
      ...createSeedGame({
        gameId,
        createdAt: "2026-07-14T00:00:00.000Z",
      }),
      runMode: "scripted" as const,
    };
    const html = renderToStaticMarkup(
      React.createElement(EpisodeAuthorGeneratingStatus, {
        workspace: createEpisodeAuthorWorkspace({ game }),
      }),
    );

    expect(html).toContain("Script Author Agent");
    expect(html).toContain("故事主轴 · 0/1");
  });
});
