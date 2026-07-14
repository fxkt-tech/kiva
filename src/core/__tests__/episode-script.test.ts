import { describe, expect, it } from "vitest";
import { authorEpisodeScript } from "../episode-author";
import { createSeedGame, type Game } from "../game";
import {
  LocalHeuristicLlmClient,
  MockLlmClient,
  type LlmClient,
  type LlmGenerateJsonRequest,
} from "../llm";
import {
  actorBriefForStep,
  assertEpisodeScriptMatchesGame,
  compileEpisodePlan,
  createEpisodeScriptSnapshot,
  episodeInputHash,
  episodePerformanceOpportunities,
  planNextEpisodeDraft,
  validateEpisodeScriptSnapshot,
  type CompiledEpisodePlan,
  type EpisodeCastDirection,
  type EpisodeRelationshipDirection,
  type EpisodeSpeechBeat,
} from "../episode-script";
import type { DraftId, GameId } from "../types";

const game = createSeedGame({
  gameId: "episode_game" as GameId,
  createdAt: "2026-07-12T00:00:00.000Z",
});

describe("episode script", () => {
  it("authors ensemble direction and character progression for every speech", async () => {
    const result = await authorEpisodeScript({
      game,
      llmClient: new LocalHeuristicLlmClient(),
      createdAt: "2026-07-12T00:00:00.000Z",
    });
    const plan = compileEpisodePlan(game);
    const speechSteps = result.script.steps.filter(
      (step) => step.speechBeat !== null,
    );

    expect(result.script.schemaVersion).toBe(2);
    expect(result.script.title).toContain("未明档案");
    expect(result.script.acts).toHaveLength(3);
    expect(result.script.castDirections).toHaveLength(game.players.length);
    expect(result.script.castDirections.map((direction) => direction.playerId))
      .toEqual(game.players.map((player) => player.playerId));
    expect(result.script.castDirections.some(
      (direction) => direction.dramaticWeight === "primary",
    )).toBe(true);
    expect(result.script.castDirections.some(
      (direction) => direction.dramaticWeight === "supporting",
    )).toBe(true);
    expect(result.script.relationships.length).toBeGreaterThan(0);
    expect(speechSteps.length).toBeGreaterThan(0);
    expect(speechSteps.every((step) => step.speechBeat?.themeHook)).toBe(true);
    expect(speechSteps.every((step) => step.speechBeat?.characterHook)).toBe(true);
    expect(speechSteps.every((step) => step.speechBeat?.arcMove)).toBe(true);

    expect(result.script.plannedWinner).toBe(plan.plannedWinner);
    expect(result.script.steps.map(compilerOwnedStepFields)).toEqual(
      plan.steps.map(compilerOwnedStepFields),
    );
    expect(() =>
      assertEpisodeScriptMatchesGame({ game, script: result.script }),
    ).not.toThrow();
    expect(result.requests[0]).toMatchObject({
      kind: "outline",
      status: "success",
      promptVersion: "episode-author:v2",
      request: { schemaName: "werewolf_episode_outline_v2" },
    });
    expect(result.requests.slice(1)).not.toHaveLength(0);
    expect(
      result.requests.slice(1).every(
        (request) =>
          request.kind === "beats" &&
          request.stepIndexes.length > 0 &&
          request.request.schemaName === "werewolf_episode_beats_v2",
      ),
    ).toBe(true);
  });

  it("records and aggregates provider token usage for every author request", async () => {
    const local = new LocalHeuristicLlmClient();
    const usageClient: LlmClient = {
      async generateJson(request) {
        const result = await local.generateJson(request);
        return {
          ...result,
          usage: {
            promptTokens: 100,
            completionTokens: 20,
            totalTokens: 120,
            cachedPromptTokens: 10,
            reasoningTokens: 5,
          },
        };
      },
    };

    const result = await authorEpisodeScript({
      game,
      llmClient: usageClient,
      createdAt: "2026-07-12T00:00:00.000Z",
    });

    expect(result.requests.every((request) => request.tokenUsage?.totalTokens === 120))
      .toBe(true);
    expect(result.tokenUsage).toMatchObject({
      promptTokens: result.requests.length * 100,
      completionTokens: result.requests.length * 20,
      totalTokens: result.requests.length * 120,
      cachedPromptTokens: result.requests.length * 10,
      reasoningTokens: result.requests.length * 5,
    });
  });

  it("sends every concise character profile to the outline and scoped actor context to beat batches", async () => {
    const requests: LlmGenerateJsonRequest[] = [];
    const local = new LocalHeuristicLlmClient();
    const capturingClient: LlmClient = {
      async generateJson(request) {
        requests.push(request);
        return local.generateJson(request);
      },
    };

    await authorEpisodeScript({
      game,
      llmClient: capturingClient,
      createdAt: "2026-07-12T00:00:00.000Z",
    });

    const outline = requests.find(
      (request) => request.schemaName === "werewolf_episode_outline_v2",
    );
    expect(outline).toBeDefined();
    const profileCards = prefixedJsonObjects(
      requestContent(outline!),
      "CHARACTER_PROFILE",
    );
    expect(profileCards).toHaveLength(game.players.length);
    for (const player of game.players) {
      expect(profileCards).toContainEqual(
        expect.objectContaining({
          playerId: player.playerId,
          name: player.name,
          gameRole: player.gameRole,
          persona: player.persona,
          speakingStyle: player.speakingStyle,
          reasoningStyle: player.reasoningStyle,
          performanceStepIndexes: expect.any(Array),
        }),
      );
      const profile = profileCards.find(
        (candidate) => candidate.playerId === player.playerId,
      );
      expect(profile?.performanceStepIndexes).toEqual(
        expect.arrayContaining([expect.any(Number)]),
      );
    }
    expect(outline?.systemPrompt).toContain("人物鲜明");
    expect(outline?.systemPrompt).toContain("不得虚构开局前关系");
    expect(outline?.systemPrompt).toContain("不写最终台词");

    const beatRequests = requests.filter(
      (request) => request.schemaName === "werewolf_episode_beats_v2",
    );
    expect(beatRequests.length).toBeGreaterThan(1);
    for (const request of beatRequests) {
      const content = requestContent(request);
      const speechActors = [...content.matchAll(
        /SPEECH_STEP\s+\d+[^\n]*?\|\s*actor=([^\s|]+)/g,
      )].map((match) => match[1]);
      const actorContexts = prefixedJsonObjects(content, "ACTOR_CONTEXT");
      expect(new Set(actorContexts.map((context) =>
        objectValue(context.profile)?.playerId,
      ))).toEqual(new Set(speechActors));
      for (const context of actorContexts) {
        const profile = objectValue(context.profile);
        const direction = objectValue(context.direction);
        const player = game.players.find(
          (candidate) => candidate.playerId === profile?.playerId,
        );
        expect(profile).toMatchObject({
          persona: player?.persona,
          speakingStyle: player?.speakingStyle,
          reasoningStyle: player?.reasoningStyle,
        });
        expect(direction).toMatchObject({ playerId: player?.playerId });
      }

      const priorMoves = prefixedJsonObjects(content, "PRIOR_MOVE");
      for (const actorId of new Set(priorMoves.map((move) => move.playerId))) {
        expect(priorMoves.filter((move) => move.playerId === actorId).length)
          .toBeLessThanOrEqual(2);
      }
    }
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

  it("rejects objectively incomplete ensemble output after repair", async () => {
    const invalidOutline = {
      title: "无效群像",
      logline: "遗漏角色的群像不能进入导演审核。",
      acts: [{ title: "第一幕", summary: "仍然遗漏角色。" }],
      castDirections: [],
      relationships: [],
    };

    await expect(
      authorEpisodeScript({
        game,
        llmClient: new MockLlmClient([invalidOutline, invalidOutline]),
        createdAt: "2026-07-12T00:00:00.000Z",
      }),
    ).rejects.toThrow("cover every player exactly once");
  });

  it("compiles a deterministic character-aware legal trace to game end", () => {
    const first = compileEpisodePlan(game);
    const second = compileEpisodePlan(game);

    expect(first.steps).toEqual(second.steps);
    expect(first.steps.at(-1)?.slot.type).toBe("game_ended");
    expect(first.simulatedEvents.at(-1)?.type).toBe("game_ended");
    expect(first.inputHash).toBe(episodeInputHash(game));
    expect(first.targetDurationMs).toBeGreaterThan(0);
    expect(episodeInputHash(game)).toBe(episodeInputHash(structuredClone(game)));

    const changedGame = {
      ...game,
      players: game.players.map((player, index) =>
        index === 0
          ? { ...player, persona: `${player.persona}，但压力下会主动冒险` }
          : player,
      ),
    };
    expect(episodeInputHash(changedGame)).not.toBe(episodeInputHash(game));
  });

  it("binds the next runtime draft to the approved first step", () => {
    const plan = compileEpisodePlan(game);
    const script = createEpisodeScriptSnapshot(
      validSnapshotInput(game, plan, "episode_1"),
    );

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

  it("projects only the current speech move into the runtime actor brief", () => {
    const plan = compileEpisodePlan(game);
    const input = validSnapshotInput(game, plan, "episode_current_brief");
    const script = createEpisodeScriptSnapshot({
      ...input,
      castDirections: input.castDirections.map((direction, index) =>
        index === 0
          ? { ...direction, payoff: "未来群像兑现不应进入当前演员提示" }
          : direction,
      ),
    });
    const speechStep = script.steps.find((step) => step.speechBeat)!;
    const brief = actorBriefForStep(script, speechStep.index);

    expect(brief).toMatchObject({
      stepIndex: speechStep.index,
      characterHook: speechStep.speechBeat?.characterHook,
      arcMove: speechStep.speechBeat?.arcMove,
      relationshipMove: speechStep.speechBeat?.relationshipMove,
    });
    expect(JSON.stringify(brief)).not.toContain(script.logline);
    expect(JSON.stringify(brief)).not.toContain(script.plannedWinner);
    expect(JSON.stringify(brief)).not.toContain(
      "未来群像兑现不应进入当前演员提示",
    );
  });

  it("rejects missing cast, invalid signature steps, and invalid relationships", () => {
    const plan = compileEpisodePlan(game);
    const input = validSnapshotInput(game, plan, "episode_invalid");

    expect(() =>
      createEpisodeScriptSnapshot({
        ...input,
        castDirections: input.castDirections.slice(1),
      }),
    ).toThrow("cover every player exactly once");

    expect(() =>
      createEpisodeScriptSnapshot({
        ...input,
        castDirections: [
          input.castDirections[0]!,
          input.castDirections[0]!,
          ...input.castDirections.slice(2),
        ],
      }),
    ).toThrow("cover every player exactly once");

    expect(() =>
      createEpisodeScriptSnapshot({
        ...input,
        castDirections: input.castDirections.map((direction, index) =>
          index === 0
            ? {
                ...direction,
                signatureMoment: {
                  ...direction.signatureMoment,
                  stepIndex: plan.steps.length + 100,
                },
              }
            : direction,
        ),
      }),
    ).toThrow("invalid signature step");

    expect(() =>
      createEpisodeScriptSnapshot({
        ...input,
        relationships: [
          {
            playerIds: [
              game.players[0]!.playerId,
              "unknown_player" as Game["players"][number]["playerId"],
            ],
            kind: "rivalry",
            setup: "建立分歧",
            development: "公开升级",
            payoff: "选择兑现",
          },
        ],
      }),
    ).toThrow("references invalid players");

    const pair: EpisodeRelationshipDirection = {
      playerIds: [
        game.players[0]!.playerId,
        game.players[1]!.playerId,
      ],
      kind: "contrast",
      setup: "建立反差",
      development: "公开检验",
      payoff: "形成互补",
    };
    expect(() =>
      createEpisodeScriptSnapshot({
        ...input,
        relationships: [
          pair,
          { ...pair, playerIds: [pair.playerIds[1], pair.playerIds[0]] },
        ],
      }),
    ).toThrow("Duplicate episode relationship");
  });

  it("rejects schema-v1 snapshots", () => {
    const plan = compileEpisodePlan(game);
    const current = createEpisodeScriptSnapshot(
      validSnapshotInput(game, plan, "episode_current_source"),
    );

    expect(() =>
      validateEpisodeScriptSnapshot({ ...current, schemaVersion: 1 }),
    ).toThrow("Unsupported episode script schema: 1");
  });

  it("rejects a v2 script when character or script input changes", () => {
    const plan = compileEpisodePlan(game);
    const script = createEpisodeScriptSnapshot(
      validSnapshotInput(game, plan, "episode_2"),
    );

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

function validSnapshotInput(
  sourceGame: Game,
  plan: CompiledEpisodePlan,
  id: string,
) {
  return {
    id,
    game: sourceGame,
    plan,
    title: "未明档案：第一卷",
    logline: "一份被篡改的档案迫使众人互相审视。",
    acts: [{ title: "开卷", summary: "身份被封入档案。" }],
    castDirections: validCastDirections(sourceGame, plan),
    relationships: [] as readonly EpisodeRelationshipDirection[],
    beats: validBeats(plan),
    createdAt: "2026-07-12T00:00:00.000Z",
    provider: "test",
    model: "test",
  };
}

function validCastDirections(
  sourceGame: Game,
  plan: CompiledEpisodePlan,
): readonly EpisodeCastDirection[] {
  const opportunities = episodePerformanceOpportunities(sourceGame, plan);
  return sourceGame.players.map((player, index) => ({
    playerId: player.playerId,
    dramaticWeight: index < 3 ? "primary" : "supporting",
    dramaticFunction: `${player.name}负责推动一条独立判断线`,
    baseline: "按稳定的人物方法观察局面",
    pressure: "公开冲突放大其方法的盲点",
    change: "保留核心但学会修正一次判断",
    payoff: "用关键选择兑现修正后的判断",
    signatureMoment: {
      stepIndex: opportunities.get(player.playerId)?.[0] ?? -1,
      description: "在真实事件节点留下可识别的个人选择",
    },
  }));
}

function validBeats(
  plan: CompiledEpisodePlan,
): readonly Omit<EpisodeSpeechBeat, "budget">[] {
  return plan.steps.flatMap((step) =>
    step.speechBeat
      ? [{
          stepIndex: step.index,
          objective: "推动当前可见冲突",
          stance: "根据可见事实给出明确判断",
          disclosure: step.speechBeat.disclosure,
          themeHook: "让当前选择成为后续可验证的主题因果",
          characterHook: "用演员稳定的人物方法组织表达",
          arcMove: "在压力下推进一个有根据的次要侧面",
          relationshipMove: null,
        }]
      : [],
  );
}

function compilerOwnedStepFields(step: CompiledEpisodePlan["steps"][number]) {
  return {
    index: step.index,
    slot: step.slot,
    plannedPayload: step.plannedPayload,
    budget: step.speechBeat?.budget ?? null,
  };
}

function requestContent(request: LlmGenerateJsonRequest): string {
  return request.messages.map((message) => message.content).join("\n");
}

function prefixedJsonObjects(
  content: string,
  prefix: string,
): readonly Record<string, unknown>[] {
  return content
    .split("\n")
    .filter((line) => line.startsWith(`${prefix} `))
    .map((line) => JSON.parse(line.slice(prefix.length + 1)) as Record<string, unknown>);
}

function objectValue(value: unknown): Record<string, unknown> | null {
  return value && typeof value === "object" && !Array.isArray(value)
    ? value as Record<string, unknown>
    : null;
}
