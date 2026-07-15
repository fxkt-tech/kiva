import { describe, expect, it } from "vitest";
import {
  advanceEpisodeAuthor,
  authorEpisodeScript,
  createEpisodeAuthorWorkspace,
} from "../episode-author";
import { createSeedGame, type Game } from "../game";
import {
  LocalHeuristicLlmClient,
  LlmOutputParseError,
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
  validateEpisodeScriptState,
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
  it("advances exactly one semantic author task at a time", async () => {
    const requests: LlmGenerateJsonRequest[] = [];
    const local = new LocalHeuristicLlmClient();
    const llmClient: LlmClient = {
      async generateJson(request) {
        requests.push(request);
        return local.generateJson(request);
      },
    };

    const story = await advanceEpisodeAuthor({
      game,
      llmClient,
      createdAt: "2026-07-12T00:00:00.000Z",
    });

    expect(story.status).toBe("ready");
    expect(story.workspace.story).not.toBeNull();
    expect(story.workspace.ensemble).toBeNull();
    expect(story.requests.map((request) => request.task.kind)).toEqual([
      "story",
    ]);
    expect(requests).toHaveLength(1);

    const ensemble = await advanceEpisodeAuthor({
      game,
      llmClient,
      createdAt: "2026-07-12T00:01:00.000Z",
      workspace: story.workspace,
    });

    expect(ensemble.status).toBe("ready");
    expect(ensemble.workspace.ensemble).not.toBeNull();
    expect(ensemble.workspace.castDirections).toHaveLength(0);
    expect(ensemble.requests.map((request) => request.task.kind)).toEqual([
      "ensemble",
    ]);
    expect(requests).toHaveLength(2);
  });

  it("marks a successful checkpoint as a settled task with more work", async () => {
    const checkpoints: Array<{
      readonly semanticTaskComplete?: boolean;
      readonly hasNextTask?: boolean;
    }> = [];

    await advanceEpisodeAuthor({
      game,
      llmClient: new LocalHeuristicLlmClient(),
      createdAt: "2026-07-12T00:00:00.000Z",
      onCheckpoint: async (checkpoint) => {
        checkpoints.push(checkpoint);
      },
    });

    expect(checkpoints).toHaveLength(1);
    expect(checkpoints[0]).toMatchObject({
      semanticTaskComplete: true,
      hasNextTask: true,
    });
  });

  it("assembles a complete workspace without another provider call", async () => {
    const authored = await authorEpisodeScript({
      game,
      llmClient: new LocalHeuristicLlmClient(),
      createdAt: "2026-07-12T00:00:00.000Z",
    });
    let providerCalls = 0;

    const result = await advanceEpisodeAuthor({
      game,
      llmClient: {
        async generateJson(request) {
          providerCalls += 1;
          return new LocalHeuristicLlmClient().generateJson(request);
        },
      },
      createdAt: "2026-07-12T00:01:00.000Z",
      workspace: authored.workspace,
    });

    expect(result.status).toBe("complete");
    expect(providerCalls).toBe(0);
    expect(result.requests).toEqual([]);
    if (result.status !== "complete") return;
    expect(result.script.steps).toHaveLength(
      compileEpisodePlan(game).steps.length,
    );
  });

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

    expect(result.script.schemaVersion).toBe(3);
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
    expect(speechSteps.every((step) => step.speechBeat?.actorHook)).toBe(true);
    expect(speechSteps.every((step) => step.speechBeat?.arcMove)).toBe(true);

    expect(result.script.plannedWinner).toBe(plan.plannedWinner);
    expect(result.script.steps.map(compilerOwnedStepFields)).toEqual(
      plan.steps.map(compilerOwnedStepFields),
    );
    expect(() =>
      assertEpisodeScriptMatchesGame({ game, script: result.script }),
    ).not.toThrow();
    expect(result.requests[0]).toMatchObject({
      task: { kind: "story" },
      status: "success",
      promptVersion: "episode-author:v4",
      request: { schemaName: "werewolf_episode_story_v4" },
    });
    expect(result.requests).toEqual(expect.arrayContaining([
      expect.objectContaining({ task: { kind: "ensemble" } }),
      expect.objectContaining({ task: expect.objectContaining({ kind: "actor_arc" }) }),
      expect.objectContaining({ task: expect.objectContaining({ kind: "relationship" }) }),
      expect.objectContaining({ task: expect.objectContaining({ kind: "beats" }) }),
    ]));
    expect(
      result.requests
        .filter((request) => request.task.kind === "beats")
        .every(
        (request) =>
          request.task.kind === "beats" &&
          request.task.stepIndexes.length > 0 &&
          request.request.schemaName === "werewolf_episode_beats_v4",
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

  it("uses the dedicated Script Author binding instead of an actor model", async () => {
    const bindings: LlmGenerateJsonRequest["modelBinding"][] = [];
    const local = new LocalHeuristicLlmClient();
    const gameWithActorModel = {
      ...game,
      players: game.players.map((player, index) =>
        index === 0
          ? {
              ...player,
              actor: {
                ...player.actor,
                production: {
                  ...player.actor.production,
                  modelBinding: {
                    provider: "actor-provider",
                    model: "actor-model",
                    responseFormat: "json" as const,
                  },
                },
              },
            }
          : player,
      ),
    };

    await authorEpisodeScript({
      game: gameWithActorModel,
      llmClient: {
        async generateJson(request) {
          bindings.push(request.modelBinding);
          return local.generateJson(request);
        },
      },
      modelBinding: {
        provider: "author-provider",
        model: "author-model",
        responseFormat: "json",
      },
      createdAt: "2026-07-12T00:00:00.000Z",
    });

    expect(bindings.length).toBeGreaterThan(0);
    expect(bindings.every((binding) =>
      binding.provider === "author-provider" && binding.model === "author-model"
    )).toBe(true);
  });

  it("keeps global allocation compact, then authors one character and one local scene at a time", async () => {
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

    const ensemble = requests.find(
      (request) => request.schemaName === "werewolf_episode_ensemble_v4",
    );
    expect(ensemble).toBeDefined();
    const profileCards = prefixedJsonObjects(
      requestContent(ensemble!),
      "ACTOR_PROFILE",
    );
    expect(profileCards).toHaveLength(game.players.length);
    for (const player of game.players) {
      expect(profileCards).toContainEqual(
        expect.objectContaining({
          playerId: player.playerId,
          actor: expect.objectContaining({
            actorId: player.actor.sourceId,
            identity: expect.objectContaining({
              name: player.actor.identity.name,
            }),
          }),
          ruleRole: player.ruleRole,
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
    expect(ensemble?.systemPrompt).toContain("群像分工");
    expect(ensemble?.systemPrompt).toContain("不要展开长篇人物弧线");

    const actorArcRequests = requests.filter(
      (request) => request.schemaName === "werewolf_episode_actor_arc_v4",
    );
    expect(actorArcRequests).toHaveLength(game.players.length);
    expect(actorArcRequests.every((request) =>
      prefixedJsonObjects(requestContent(request), "ACTOR_PROFILE").length === 1
    )).toBe(true);

    const relationshipRequests = requests.filter(
      (request) => request.schemaName === "werewolf_episode_relationship_v4",
    );
    expect(relationshipRequests.length).toBeGreaterThan(0);
    for (const request of relationshipRequests) {
      const milestones = requestContent(request).match(
        /RELATIONSHIP_STEP\s+\d+/g,
      ) ?? [];
      expect(milestones.length).toBeGreaterThan(0);
      expect(milestones.length).toBeLessThanOrEqual(5);
    }

    const beatRequests = requests.filter(
      (request) => request.schemaName === "werewolf_episode_beats_v4",
    );
    expect(beatRequests.length).toBeGreaterThan(1);
    for (const request of beatRequests) {
      const content = requestContent(request);
      expect(content.match(/SPEECH_STEP\s+\d+/g)?.length ?? 0)
        .toBeLessThanOrEqual(5);
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
          actor: expect.objectContaining({
            actorId: player?.actor.sourceId,
          }),
          ruleRole: player?.ruleRole,
        });
        expect(direction).toMatchObject({ playerId: player?.playerId });
      }

      const priorMoves = prefixedJsonObjects(content, "PRIOR_MOVE");
      for (const actorId of new Set(priorMoves.map((move) => move.playerId))) {
        expect(priorMoves.filter((move) => move.playerId === actorId).length)
          .toBeLessThanOrEqual(1);
      }
    }
  });

  it("enforces the concise field contract before later tasks reuse author output", async () => {
    const local = new LocalHeuristicLlmClient();
    let storyAttempts = 0;
    const conciseClient: LlmClient = {
      async generateJson(request) {
        const result = await local.generateJson(request);
        if (
          request.schemaName === "werewolf_episode_story_v4" &&
          storyAttempts++ === 0
        ) {
          const parsed = { ...result.parsed, title: "长".repeat(81) };
          return { ...result, parsed, rawText: JSON.stringify(parsed) };
        }
        return result;
      },
    };

    const result = await authorEpisodeScript({
      game,
      llmClient: conciseClient,
      createdAt: "2026-07-12T00:00:00.000Z",
    });

    expect(storyAttempts).toBe(2);
    expect(Array.from(result.script.title).length).toBeLessThanOrEqual(80);
    expect(result.requests[0]?.attempts).toHaveLength(2);
    expect(
      result.requests[0]?.request.messages
        .map((message) => message.content)
        .join("\n"),
    ).toContain("80");
  });

  it("splits only a truncated beat task instead of repairing the incomplete JSON as a whole", async () => {
    const local = new LocalHeuristicLlmClient();
    const truncateLargeBeatOutput: LlmClient = {
      async generateJson(request) {
        const beatCount = request.messages
          .map((message) => message.content)
          .join("\n")
          .match(/SPEECH_STEP\s+\d+/g)?.length ?? 0;
        if (beatCount > 2) {
          throw new LlmOutputParseError(
            "LLM message content was not a valid JSON object: Unterminated string",
            '{"beats":[{"stepIndex":1,"objective":"truncated',
            { finishReason: "length", usage: null },
          );
        }
        return local.generateJson(request);
      },
    };

    const result = await authorEpisodeScript({
      game,
      llmClient: truncateLargeBeatOutput,
      createdAt: "2026-07-12T00:00:00.000Z",
    });

    expect(result.script.title).toEqual(expect.any(String));
    expect(result.requests).toEqual(expect.arrayContaining([
      expect.objectContaining({
        task: expect.objectContaining({ kind: "beats" }),
        status: "failed",
        finishReason: "length",
      }),
      expect.objectContaining({
        task: expect.objectContaining({ kind: "beats" }),
        status: "success",
      }),
    ]));
  });

  it("repairs dramaticWeight with the exact two-value contract", async () => {
    const local = new LocalHeuristicLlmClient();
    const ensembleRequests: LlmGenerateJsonRequest[] = [];
    let validEnsembleOutput: Awaited<
      ReturnType<LlmClient["generateJson"]>
    > | null = null;
    const contract =
      'dramaticWeight 只能是字符串 "primary" 或 "supporting"';
    const contractSensitiveClient: LlmClient = {
      async generateJson(request) {
        if (request.schemaName !== "werewolf_episode_ensemble_v4") {
          return local.generateJson(request);
        }

        ensembleRequests.push(request);
        if (validEnsembleOutput === null) {
          validEnsembleOutput = await local.generateJson(request);
          const castAssignments = validEnsembleOutput.parsed
            .castAssignments as readonly Record<string, unknown>[];
          const localizedWeights = ["功能演员", "主要演员", "核心主演"];
          const parsed = {
            ...validEnsembleOutput.parsed,
            castAssignments: castAssignments.map((assignment, index) => ({
              ...assignment,
              dramaticWeight: localizedWeights[index % localizedWeights.length],
            })),
          };
          return {
            ...validEnsembleOutput,
            parsed,
            rawText: JSON.stringify(parsed),
          };
        }

        if (requestContent(request).includes(contract)) {
          return validEnsembleOutput;
        }
        const castAssignments = validEnsembleOutput.parsed
          .castAssignments as readonly Record<string, unknown>[];
        const parsed = {
          ...validEnsembleOutput.parsed,
          castAssignments: castAssignments.map((assignment, index) => ({
            ...assignment,
            dramaticWeight: index % 3 + 1,
          })),
        };
        return {
          ...validEnsembleOutput,
          parsed,
          rawText: JSON.stringify(parsed),
        };
      },
    };

    const result = await authorEpisodeScript({
      game,
      llmClient: contractSensitiveClient,
      createdAt: "2026-07-12T00:00:00.000Z",
    });

    expect(ensembleRequests).toHaveLength(2);
    expect(ensembleRequests.every((request) =>
      requestContent(request).includes(contract)
    )).toBe(true);
    expect(
      result.requests.find((request) => request.task.kind === "ensemble")
        ?.attempts,
    ).toHaveLength(2);
  });

  it("includes the disclosure enum contract in initial beat requests", async () => {
    const local = new LocalHeuristicLlmClient();
    const beatRequests: LlmGenerateJsonRequest[] = [];
    const contract =
      "disclosure 只能是 conceal、claim、not_applicable";

    await authorEpisodeScript({
      game,
      llmClient: {
        async generateJson(request) {
          if (request.schemaName === "werewolf_episode_beats_v4") {
            beatRequests.push(request);
          }
          return local.generateJson(request);
        },
      },
      createdAt: "2026-07-12T00:00:00.000Z",
    });

    expect(beatRequests.length).toBeGreaterThan(0);
    expect(beatRequests.every((request) =>
      requestContent(request).includes(contract)
    )).toBe(true);
    expect(beatRequests.every((request) =>
      requestContent(request).includes("每个字符串字段最多 80 个字符")
    )).toBe(true);
  });

  it("shares exact story bounds with structural repair", async () => {
    const local = new LocalHeuristicLlmClient();
    const requests: LlmGenerateJsonRequest[] = [];
    const client: LlmClient = {
      async generateJson(request) {
        requests.push(request);
        if (
          request.schemaName === "werewolf_episode_story_v4" &&
          requests.length === 1
        ) {
          const valid = await local.generateJson(request);
          const acts = valid.parsed.acts as readonly Record<string, unknown>[];
          const parsed = { ...valid.parsed, acts: [...acts, ...acts, ...acts] };
          return { ...valid, parsed, rawText: JSON.stringify(parsed) };
        }
        return local.generateJson(request);
      },
    };

    await authorEpisodeScript({
      game,
      llmClient: client,
      createdAt: "2026-07-12T00:00:00.000Z",
    });

    const storyRequests = requests.filter(
      (request) => request.schemaName === "werewolf_episode_story_v4",
    );
    expect(storyRequests).toHaveLength(2);
    expect(storyRequests.every((request) => {
      const content = requestContent(request);
      return content.includes("acts 必须为 1 到 4 项") &&
        content.includes("每个字符串字段最多 80 个字符");
    })).toBe(true);
  });

  it("gives ensemble repair exact kinds and signature indexes", async () => {
    const local = new LocalHeuristicLlmClient();
    const ensembleRequests: LlmGenerateJsonRequest[] = [];
    let validEnsemble: Awaited<ReturnType<LlmClient["generateJson"]>> | null =
      null;
    const client: LlmClient = {
      async generateJson(request) {
        if (request.schemaName !== "werewolf_episode_ensemble_v4") {
          return local.generateJson(request);
        }
        ensembleRequests.push(request);
        if (validEnsemble === null) {
          validEnsemble = await local.generateJson(request);
          const assignments = validEnsemble.parsed
            .castAssignments as readonly Record<string, unknown>[];
          const parsed = {
            ...validEnsemble.parsed,
            castAssignments: assignments.map((assignment, index) =>
              index === 0
                ? { ...assignment, signatureStepIndex: -1 }
                : assignment
            ),
          };
          return { ...validEnsemble, parsed, rawText: JSON.stringify(parsed) };
        }
        return validEnsemble;
      },
    };

    await authorEpisodeScript({
      game,
      llmClient: client,
      createdAt: "2026-07-12T00:00:00.000Z",
    });

    expect(ensembleRequests).toHaveLength(2);
    expect(ensembleRequests.every((request) => {
      const content = requestContent(request);
      return content.includes(
        "kind 只能是 rivalry、alliance、contrast、trust_shift",
      ) && content.includes("signatureStepIndex 合法值");
    })).toBe(true);
  });

  it("rejects objectively incomplete ensemble output after repair", async () => {
    const story = {
      title: "待分配群像",
      logline: "合法轨迹仍需要完整群像分工。",
      acts: [{ title: "第一幕", summary: "先建立故事主轴。" }],
    };
    const invalidEnsemble = {
      castAssignments: [],
      relationshipSeeds: [],
    };

    await expect(
      authorEpisodeScript({
        game,
        llmClient: new MockLlmClient([
          story,
          invalidEnsemble,
          invalidEnsemble,
        ]),
        createdAt: "2026-07-12T00:00:00.000Z",
      }),
    ).rejects.toThrow("cover every player exactly once");
  });

  it("bounds story acts and relationship seeds so later task context stays finite", async () => {
    const oversizedStory = {
      title: "过长分幕",
      logline: "分幕数量必须有明确上限。",
      acts: Array.from({ length: 5 }, (_, index) => ({
        title: `第${index + 1}幕`,
        summary: "推进一段主轴。",
      })),
    };
    await expect(
      authorEpisodeScript({
        game,
        llmClient: new MockLlmClient([oversizedStory, oversizedStory]),
        createdAt: "2026-07-12T00:00:00.000Z",
      }),
    ).rejects.toThrow("between 1 and 4");

    const local = new LocalHeuristicLlmClient();
    let validEnsemble: Record<string, unknown> | null = null;
    const oversizedRelationships: LlmClient = {
      async generateJson(request) {
        const result = await local.generateJson(request);
        if (request.schemaName !== "werewolf_episode_ensemble_v4") {
          return result;
        }
        validEnsemble ??= result.parsed;
        const relationshipSeeds = [
          ...validEnsemble.relationshipSeeds as Record<string, unknown>[],
          {
            playerIds: [
              game.players[0]!.playerId,
              game.players[2]!.playerId,
            ],
            kind: "contrast",
          },
        ];
        const parsed = { ...validEnsemble, relationshipSeeds };
        return { ...result, parsed, rawText: JSON.stringify(parsed) };
      },
    };
    await expect(
      authorEpisodeScript({
        game,
        llmClient: oversizedRelationships,
        createdAt: "2026-07-12T00:00:00.000Z",
      }),
    ).rejects.toThrow("between 1 and 6");
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
          ? {
              ...player,
              actor: {
                ...player.actor,
                core: {
                  ...player.actor.core,
                  stableCore: `${player.actor.core.stableCore}，但压力下会主动冒险`,
                },
              },
            }
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
      actorHook: speechStep.speechBeat?.actorHook,
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

  it("rejects v2 Script Author intermediate state instead of migrating it", () => {
    expect(() =>
      validateEpisodeScriptState({
        status: "generating",
        jobId: "old_job",
        startedAt: "2026-07-12T00:00:00.000Z",
        workspace: createEpisodeAuthorWorkspace({ game }),
        requests: [{
          id: "old_request",
          kind: "outline",
          stepIndexes: [],
          status: "success",
          promptVersion: "episode-author:v2",
        }],
      }, "scripted"),
    ).toThrow("Episode author request 1");
  });

  it("round-trips only the exact ready Script Author state", () => {
    const ready = {
      status: "ready",
      jobId: "ready_job",
      workspace: createEpisodeAuthorWorkspace({ game }),
      requests: [],
    } as const;

    expect(validateEpisodeScriptState(ready, "scripted")).toEqual(ready);
    expect(() =>
      validateEpisodeScriptState({ ...ready, unexpected: true }, "scripted")
    ).toThrow("unknown: unexpected");
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
    dramaticFunction: `${player.actor.identity.name}负责推动一条独立判断线`,
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
          performanceMove: "根据可见事实给出明确判断",
          disclosure: step.speechBeat.disclosure,
          themeHook: "让当前选择成为后续可验证的主题因果",
          actorHook: "用演员稳定的人物方法组织表达",
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
