import React from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it, vi } from "vitest";
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

vi.mock("@/components/script/episode-generating-refresh", () => ({
  EpisodeGeneratingRefresh: () => null,
}));

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
    expect(html).toContain('aria-label="Enable auto-continue Script Author"');
    expect(html).toContain('aria-pressed="false"');
  });

  it("renders one reusable LLM details control per script-author request", () => {
    const request: EpisodeAuthorRequestRecord = {
      id: "episode_request_1",
      task: { kind: "story" },
      status: "success",
      promptVersion: "episode-author:v4",
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

  it("offers one guarded next step after a task completes", () => {
    const gameId = "ready-scripted-game" as GameId;
    const game = {
      ...createSeedGame({
        gameId,
        createdAt: "2026-07-14T00:00:00.000Z",
      }),
      runMode: "scripted" as const,
    };
    const workspace = {
      ...createEpisodeAuthorWorkspace({ game }),
      story: {
        title: "未明档案",
        logline: "完成故事主轴后等待群像分工。",
        acts: [{ title: "第一幕", summary: "建立冲突。" }],
      },
    };
    const record: GameRecord = {
      schemaVersion: GAME_RECORD_SCHEMA_VERSION,
      game,
      events: [],
      draft: null,
      generations: [],
      voiceArtifactsByEventId: {},
      episodeScript: {
        status: "ready",
        jobId: "ready_job",
        workspace,
        requests: [],
      },
    };

    const html = renderToStaticMarkup(
      React.createElement(EpisodeWorkspace, { record }),
    );

    expect(html).toContain("当前步骤已完成");
    expect(html).toContain("下一步：群像分工 · 0/1");
    expect(html).toContain("生成下一步");
    expect(html).not.toContain("animate-spin");
  });

  it("left-aligns Script Author text while generating", () => {
    const gameId = "left-aligned-generating-script" as GameId;
    const game = {
      ...createSeedGame({
        gameId,
        createdAt: "2026-07-14T00:00:00.000Z",
      }),
      runMode: "scripted" as const,
    };
    const record: GameRecord = {
      schemaVersion: GAME_RECORD_SCHEMA_VERSION,
      game,
      events: [],
      draft: null,
      generations: [],
      voiceArtifactsByEventId: {},
      episodeScript: {
        status: "generating",
        jobId: "left_aligned_job",
        startedAt: "2026-07-14T00:01:00.000Z",
        workspace: createEpisodeAuthorWorkspace({ game }),
        requests: [],
      },
    };

    const html = renderToStaticMarkup(
      React.createElement(EpisodeWorkspace, { record }),
    );

    expect(html).toContain('class="py-10 text-left"');
    expect(html).not.toContain('class="py-10 text-center"');
    expect(html).toContain('class="mx-auto h-8 w-8');
  });
});
