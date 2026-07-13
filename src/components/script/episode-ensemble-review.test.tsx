import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { createSeedGame } from "@/core/game";
import type { EpisodeScriptSnapshot } from "@/core/episode-script";
import type { GameId } from "@/core/types";
import { EpisodeEnsembleReview } from "./episode-ensemble-review";

const game = createSeedGame({
  gameId: "ensemble_review" as GameId,
  createdAt: "2026-07-13T00:00:00.000Z",
});
const [first, second] = game.players;

describe("EpisodeEnsembleReview", () => {
  it("renders read-only cast arcs, signature moments, and relationships", () => {
    const html = renderToStaticMarkup(
      <EpisodeEnsembleReview
        script={scriptWith({
          schemaVersion: 2,
          castDirections: [
            {
              playerId: first!.playerId,
              dramaticWeight: "primary",
              dramaticFunction: "把谨慎观察推进成公开追问",
              baseline: "先记录矛盾，不急于站队",
              pressure: "沉默会让关键证词失去复核机会",
              change: "保留谨慎，但开始承担明确判断",
              payoff: "用一次可验证的质疑锁定终局选择",
              signatureMoment: {
                stepIndex: 18,
                description: "第一次公开指出档案版本冲突",
              },
            },
            {
              playerId: second!.playerId,
              dramaticWeight: "supporting",
              dramaticFunction: "用直接表达迫使讨论脱离模糊区",
              baseline: "快速给出直觉判断",
              pressure: "一次误判让直觉受到公开挑战",
              change: "仍然直接，但主动补上验证标准",
              payoff: "在关键投票前修正自己的第一印象",
              signatureMoment: {
                stepIndex: 26,
                description: "承认误判并给出新的验证路线",
              },
            },
          ],
          relationships: [
            {
              playerIds: [first!.playerId, second!.playerId],
              kind: "contrast",
              setup: "谨慎记录与直接判断第一次碰撞",
              development: "两人开始互相要求给出验证标准",
              payoff: "不同方法在最终选择中形成互补",
            },
          ],
        })}
        players={game.players}
      />,
    );

    expect(html).toContain(first!.name);
    expect(html).toContain(second!.name);
    expect(html).toContain("主要角色");
    expect(html).toContain("支持角色");
    expect(html).toContain("把谨慎观察推进成公开追问");
    expect(html).toContain("标志时刻 #18");
    expect(html).toContain("谨慎记录与直接判断第一次碰撞");
    expect(html).toContain("反差");
    expect(html).not.toContain("<input");
    expect(html).not.toContain("<textarea");
    expect(html).not.toContain("<button");
  });

  it("shows a neutral compatibility note for schema v1", () => {
    const html = renderToStaticMarkup(
      <EpisodeEnsembleReview
        script={scriptWith({
          schemaVersion: 1,
          castDirections: [],
          relationships: [],
        })}
        players={game.players}
      />,
    );

    expect(html).toContain("兼容载入的旧版剧本");
    expect(html).not.toContain("主要角色");
  });
});

function scriptWith(
  fields: Pick<
    EpisodeScriptSnapshot,
    "schemaVersion" | "castDirections" | "relationships"
  >,
): EpisodeScriptSnapshot {
  return {
    ...fields,
    id: "episode_review",
    gameId: game.id,
    compilerVersion: "episode-compiler:v1",
    inputHash: "episode_test",
    title: "测试剧本",
    logline: "测试群像审核",
    plannedWinner: "good",
    plannedDayCount: 1,
    targetDurationMs: 1_000,
    acts: [{ title: "第一幕", summary: "测试" }],
    steps: [],
    createdAt: "2026-07-13T00:00:00.000Z",
    author: { provider: "test", model: "test" },
  };
}
