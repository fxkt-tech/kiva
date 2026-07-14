import React from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { createSeedGame } from "@/core/game";
import type { GameId } from "@/core/types";
import {
  GAME_RECORD_SCHEMA_VERSION,
  type GameRecord,
} from "@/server/game-repository";
import { EpisodeWorkspace } from "./page";

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
});
