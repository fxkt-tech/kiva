import React from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it, vi } from "vitest";
import { EpisodeAuthorRequestWorkspace } from "@/components/script/episode-author-request-workspace";
import { createEpisodeAuthorWorkspace } from "@/core/episode-author";
import type { EpisodeAuthorRequestRecord } from "@/core/episode-script";
import { createSeedGame } from "@/core/game";
import type { GameId } from "@/core/types";
import {
  GAME_RECORD_SCHEMA_VERSION,
  type GameRecord,
} from "@/server/game-repository";
import {
  EpisodeAuthorGeneratingStatus,
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

  it("renders one selectable request list with an inline details panel", () => {
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
      React.createElement(EpisodeAuthorRequestWorkspace, { requests: [request] }),
    );

    expect(html).toContain("LLM requests");
    expect(html).toContain(">LLM Details<");
    expect(html).toContain("1 · 120 tokens");
    expect(html).not.toContain("1 requests · 120 tokens");
    expect(html).toContain("openai-compatible/author-model");
    expect(html).toContain('aria-pressed="true"');
    expect(html).toContain("Author the outline.");
    expect(html).toContain("Token usage");
    expect(html).not.toContain("<dialog");
    expect(html).toContain('aria-label="Collapse all LLM detail sections"');
    expect(html.match(/<details open=""/g)).toHaveLength(4);
    expect(html.match(/<summary/g)).toHaveLength(4);
    const requestRow = html.match(
      /<button[^>]*aria-label="Story spine request"[\s\S]*?<\/button>/,
    )?.[0];
    expect(requestRow).toBeDefined();
    expect(requestRow!.indexOf("rounded-full")).toBeLessThan(
      requestRow!.indexOf(">Story spine<"),
    );
    expect(requestRow).not.toContain("openai-compatible");
    expect(requestRow).not.toContain("120");

    const newerRequest: EpisodeAuthorRequestRecord = {
      ...request,
      id: "episode_request_2",
      task: { kind: "ensemble" },
      request: {
        ...request.request,
        systemPrompt: "Author the ensemble.",
      },
      createdAt: "2026-07-14T00:01:00.000Z",
    };
    const reversedHtml = renderToStaticMarkup(
      React.createElement(EpisodeAuthorRequestWorkspace, {
        requests: [request, newerRequest],
      }),
    );
    expect(reversedHtml.indexOf("Ensemble map")).toBeLessThan(
      reversedHtml.indexOf("Story spine"),
    );
    expect(reversedHtml).toContain("Author the ensemble.");
    expect(reversedHtml).not.toContain("Author the outline.");
  });

  it("keeps the LLM request panel visible before the first request", () => {
    const html = renderToStaticMarkup(
      React.createElement(EpisodeAuthorRequestWorkspace, { requests: [] }),
    );

    expect(html).toContain("LLM requests");
    expect(html).toContain(">LLM Details<");
    expect(html).toContain("0 · 0 tokens");
    expect(html).toContain("开始生成后，请求记录会按执行顺序出现在这里");
    expect(html).toContain("选择一条请求后，这里会显示 Prompt、Token 和模型输出");
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
    expect(html).toContain('aria-label="Script Author current status"');
    expect(html).toContain("当前状态");
    expect(html).toContain("READY");
    expect(html).toContain("下一步：群像分工 · 0/1");
    expect(html).toContain('role="progressbar"');
    expect(html).toContain("故事主轴");
    expect(html).toContain("群像分工");
    expect(html).toContain("角色弧线");
    expect(html).toContain("关系弧线");
    expect(html).toContain("场景节拍");
    expect(html).not.toContain("已保留请求");
    expect(html).toMatch(/故事主轴<\/dt><dd[^>]*>1 \/ 1/);
    expect(html).toMatch(/群像分工<\/dt><dd[^>]*>0 \/ 1/);
    expect(html).toMatch(/关系弧线<\/dt><dd[^>]*>0 \/ —/);
    expect(html).toContain("生成下一步");
    expect(html).toMatch(
      /aria-label="Script Author current status"[\s\S]*自动生成下一步[\s\S]*生成下一步[\s\S]*<\/section>/,
    );
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

    expect(html).toContain("当前状态");
    expect(html).toContain("GENERATING");
    expect(html).toContain('role="progressbar"');
    expect(html).toContain('aria-valuenow="0"');
    expect(html).toContain("故事主轴");
    expect(html).toContain("群像分工");
    expect(html).toContain("角色弧线");
    expect(html).toContain("关系弧线");
    expect(html).toContain("场景节拍");
    expect(html).not.toContain("已保留请求");
    expect(html).toMatch(/故事主轴<\/dt><dd[^>]*>0 \/ 1/);
    expect(html).toMatch(/群像分工<\/dt><dd[^>]*>0 \/ 1/);
    expect(html).toMatch(/关系弧线<\/dt><dd[^>]*>0 \/ —/);
    expect(html).toContain("重试当前阶段");
    expect(html).not.toContain("重新开始");
    expect(html).toContain('class="text-left"');
    expect(html).not.toContain('class="text-center"');
  });

  it("shows ready once the current job request is persisted", () => {
    const gameId = "settled-generating-script" as GameId;
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
        logline: "故事主轴已经完成。",
        acts: [{ title: "第一幕", summary: "建立冲突。" }],
      },
    };
    const request: EpisodeAuthorRequestRecord = {
      id: "settled_story_request",
      task: { kind: "story" },
      status: "success",
      promptVersion: "episode-author:v4",
      provider: "openai-compatible",
      model: "author-model",
      request: {
        schemaName: "werewolf_episode_story_v4",
        systemPrompt: "Author the story.",
        messages: [{ role: "user", content: "Game context" }],
      },
      tokenUsage: null,
      finishReason: "stop",
      rawOutput: '{"title":"未明档案"}',
      parsedOutput: { title: "未明档案" },
      error: null,
      createdAt: "2026-07-14T00:01:01.000Z",
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
        jobId: "settled_job",
        startedAt: "2026-07-14T00:01:00.000Z",
        workspace,
        requests: [request],
      },
    };

    const html = renderToStaticMarkup(
      React.createElement(EpisodeWorkspace, { record }),
    );

    expect(html).toContain("READY");
    expect(html).toContain("生成下一步");
    expect(html).not.toContain("GENERATING");
    expect(html).not.toContain("重试当前阶段");
  });
});
