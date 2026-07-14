import { randomUUID } from "node:crypto";
import type { Game } from "./game";
import type { GenerationRequestSnapshot } from "./generation-record";
import type { LlmClient, LlmTokenUsage } from "./llm";
import {
  assertEpisodeDramaturgy,
  compileEpisodePlan,
  createEpisodeScriptSnapshot,
  EPISODE_AUTHOR_PROMPT_VERSION,
  episodeActorProfile,
  episodePerformanceOpportunities,
  MAX_EPISODE_RELATIONSHIP_SEEDS,
  MAX_EPISODE_STORY_ACTS,
  validateEpisodeAuthorWorkspace,
  type CompiledEpisodePlan,
  type EpisodeAct,
  type EpisodeAuthorRequestRecord,
  type EpisodeAuthorTask,
  type EpisodeAuthorWorkspace,
  type EpisodeCastAssignment,
  type EpisodeCastDirection,
  type EpisodeActorProfile,
  type EpisodeEnsembleMap,
  type EpisodeRelationshipDirection,
  type EpisodeRelationshipKind,
  type EpisodeRelationshipSeed,
  type EpisodeScriptSnapshot,
  type EpisodeSpeechBeat,
  type EpisodeStorySpine,
} from "./episode-script";
import type { ModelBindingSnapshot } from "./model-binding";
import type { PlayerId } from "./types";
import {
  generateValidatedJson,
  ValidatedGenerationError,
  type ValidatedGenerationResult,
} from "./validated-generation";

export const DEFAULT_EPISODE_AUTHOR_MODEL_BINDING = {
  provider: "volcengine",
  model: "doubao-seed-2-1-turbo-260628",
  responseFormat: "json",
} as const satisfies ModelBindingSnapshot;

export type AuthorEpisodeScriptResult = {
  readonly script: EpisodeScriptSnapshot;
  readonly workspace: EpisodeAuthorWorkspace;
  readonly requests: readonly EpisodeAuthorRequestRecord[];
  readonly tokenUsage: LlmTokenUsage | null;
  readonly repaired: boolean;
};

export type EpisodeAuthorCheckpoint = {
  readonly workspace: EpisodeAuthorWorkspace;
  readonly request: EpisodeAuthorRequestRecord;
};

export class EpisodeAuthoringError extends Error {
  readonly requests: readonly EpisodeAuthorRequestRecord[];
  readonly workspace: EpisodeAuthorWorkspace;

  constructor(input: {
    readonly error: unknown;
    readonly requests: readonly EpisodeAuthorRequestRecord[];
    readonly workspace: EpisodeAuthorWorkspace;
  }) {
    super(input.error instanceof Error ? input.error.message : String(input.error), {
      cause: input.error,
    });
    this.name = "EpisodeAuthoringError";
    this.requests = structuredClone(input.requests);
    this.workspace = structuredClone(input.workspace);
  }
}

type RequestOutcome<Value> =
  | {
      readonly ok: true;
      readonly result: ValidatedGenerationResult<Value>;
      readonly record: EpisodeAuthorRequestRecord;
    }
  | {
      readonly ok: false;
      readonly error: unknown;
      readonly finishReason: string | null;
      readonly record: EpisodeAuthorRequestRecord;
    };

const MAX_BEATS_PER_AGENT_SCENE = 5;
const MAX_PRIOR_MOVES_PER_ACTOR = 1;
const MAX_AUTHOR_FIELD_CHARACTERS = 80;
const CONCISE_AUTHOR_OUTPUT_INSTRUCTION =
  `每个字符串字段最多 ${MAX_AUTHOR_FIELD_CHARACTERS} 个字符，使用完整短句，不要扩写。`;

export function createEpisodeAuthorWorkspace(input: {
  readonly game: Game;
  readonly modelBinding?: ModelBindingSnapshot;
}): EpisodeAuthorWorkspace {
  const plan = compileEpisodePlan(input.game);
  return {
    inputHash: plan.inputHash,
    modelBinding: structuredClone(
      input.modelBinding ?? DEFAULT_EPISODE_AUTHOR_MODEL_BINDING,
    ),
    planStepCount: plan.steps.length,
    speechStepCount: plan.steps.filter((step) => step.speechBeat !== null).length,
    story: null,
    ensemble: null,
    castDirections: [],
    relationships: [],
    beats: [],
  };
}

export function episodeAuthorProgress(workspace: EpisodeAuthorWorkspace): {
  readonly label: string;
  readonly completed: number;
  readonly total: number;
} {
  if (!workspace.story) return { label: "故事主轴", completed: 0, total: 1 };
  if (!workspace.ensemble) return { label: "群像分工", completed: 0, total: 1 };
  if (workspace.castDirections.length < workspace.ensemble.castAssignments.length) {
    return {
      label: "角色弧线",
      completed: workspace.castDirections.length,
      total: workspace.ensemble.castAssignments.length,
    };
  }
  if (workspace.relationships.length < workspace.ensemble.relationshipSeeds.length) {
    return {
      label: "关系弧线",
      completed: workspace.relationships.length,
      total: workspace.ensemble.relationshipSeeds.length,
    };
  }
  return {
    label: workspace.beats.length < workspace.speechStepCount ? "场景节拍" : "最终校验",
    completed: workspace.beats.length,
    total: workspace.speechStepCount,
  };
}

export async function authorEpisodeScript(input: {
  readonly game: Game;
  readonly llmClient: LlmClient;
  readonly createdAt: string;
  readonly modelBinding?: ModelBindingSnapshot;
  readonly workspace?: EpisodeAuthorWorkspace;
  readonly onCheckpoint?: (
    checkpoint: EpisodeAuthorCheckpoint,
  ) => Promise<void>;
}): Promise<AuthorEpisodeScriptResult> {
  const plan = compileEpisodePlan(input.game);
  const profiles = input.game.players.map(episodeActorProfile);
  const opportunities = episodePerformanceOpportunities(input.game, plan);
  assertPerformanceOpportunities(profiles, opportunities);

  let workspace = input.workspace
    ? validateEpisodeAuthorWorkspace(input.workspace)
    : createEpisodeAuthorWorkspace({
        game: input.game,
        modelBinding: input.modelBinding,
      });
  assertWorkspaceMatchesPlan(workspace, plan);
  const requests: EpisodeAuthorRequestRecord[] = [];

  const checkpoint = async (
    nextWorkspace: EpisodeAuthorWorkspace,
    request: EpisodeAuthorRequestRecord,
  ): Promise<void> => {
    workspace = validateEpisodeAuthorWorkspace(nextWorkspace);
    requests.push(request);
    await input.onCheckpoint?.({ workspace, request });
  };

  const fail = (error: unknown): never => {
    throw new EpisodeAuthoringError({ error, requests, workspace });
  };

  const completeTask = async <Value>(taskInput: {
    readonly task: EpisodeAuthorTask;
    readonly request: GenerationRequestSnapshot;
    readonly validate: (parsed: Record<string, unknown>) => Value;
    readonly outputContract: readonly string[];
    readonly apply: (value: Value) => EpisodeAuthorWorkspace;
  }): Promise<void> => {
    const outcome = await performAgentRequest({
      ...taskInput,
      llmClient: input.llmClient,
      modelBinding: workspace.modelBinding,
      createdAt: input.createdAt,
    });
    if (outcome.ok) {
      await checkpoint(taskInput.apply(outcome.result.value), outcome.record);
      return;
    }
    await checkpoint(workspace, outcome.record);
    if (outcome.finishReason === "length") {
      const conciseOutcome = await performAgentRequest({
        ...taskInput,
        request: buildConciseRetryRequest(taskInput.request),
        llmClient: input.llmClient,
        modelBinding: workspace.modelBinding,
        createdAt: input.createdAt,
      });
      if (conciseOutcome.ok) {
        await checkpoint(
          taskInput.apply(conciseOutcome.result.value),
          conciseOutcome.record,
        );
        return;
      }
      await checkpoint(workspace, conciseOutcome.record);
      fail(conciseOutcome.error);
    }
    fail(outcome.error);
  };

  const completeBeatTask = async (
    task: Extract<EpisodeAuthorTask, { readonly kind: "beats" }>,
  ): Promise<void> => {
    const request = buildBeatRequest({
      game: input.game,
      plan,
      workspace,
      profiles,
      stepIndexes: task.stepIndexes,
    });
    const outcome = await performAgentRequest({
      task,
      request,
      llmClient: input.llmClient,
      modelBinding: workspace.modelBinding,
      createdAt: input.createdAt,
      validate: (parsed) => parseBeats(parsed, task.stepIndexes),
      outputContract: [
        `beats 必须恰好覆盖 stepIndex：${task.stepIndexes.join(", ")}`,
        "每个 beat 包含非空 objective、performanceMove、themeHook、actorHook、arcMove",
        "disclosure 只能是 conceal、claim、not_applicable；relationshipMove 必须是非空字符串或 null",
      ],
    });
    if (outcome.ok) {
      const existing = new Map(
        workspace.beats.map((beat) => [beat.stepIndex, beat]),
      );
      for (const beat of outcome.result.value) existing.set(beat.stepIndex, beat);
      await checkpoint(
        {
          ...workspace,
          beats: [...existing.values()].sort((left, right) =>
            left.stepIndex - right.stepIndex
          ),
        },
        outcome.record,
      );
      return;
    }

    await checkpoint(workspace, outcome.record);
    if (outcome.finishReason === "length" && task.stepIndexes.length > 1) {
      const midpoint = Math.ceil(task.stepIndexes.length / 2);
      await completeBeatTask({
        kind: "beats",
        stepIndexes: task.stepIndexes.slice(0, midpoint),
      });
      await completeBeatTask({
        kind: "beats",
        stepIndexes: task.stepIndexes.slice(midpoint),
      });
      return;
    }
    fail(outcome.error);
  };

  while (true) {
    const task = nextEpisodeAuthorTask({ game: input.game, plan, workspace });
    if (!task) break;

    switch (task.kind) {
      case "story": {
        const request = buildStoryRequest(input.game, plan);
        await completeTask({
          task,
          request,
          validate: parseStory,
          outputContract: [
            "title 和 logline 必须是非空字符串",
            "acts 必须是至少一项的 {title,summary} 数组",
            "只写全局故事主轴，不写角色弧线、关系细节或逐场台词",
          ],
          apply: (story) => ({ ...workspace, story }),
        });
        break;
      }
      case "ensemble": {
        const story = requiredWorkspaceValue(workspace.story, "story");
        const request = buildEnsembleRequest({
          game: input.game,
          plan,
          story,
          profiles,
          opportunities,
        });
        await completeTask({
          task,
          request,
          validate: (parsed) =>
            parseEnsemble(parsed, input.game, opportunities),
          outputContract: [
            `castAssignments 必须恰好覆盖：${profiles.map((profile) => profile.playerId).join(", ")}`,
            "每项只包含 playerId、dramaticWeight、dramaticFunction、signatureStepIndex",
            `relationshipSeeds 必须为 1 到 ${MAX_EPISODE_RELATIONSHIP_SEEDS} 项；每项只包含两个不同 playerId 和约定 kind`,
            "不要展开 baseline、pressure、change、payoff 或关系长文",
          ],
          apply: (ensemble) => ({ ...workspace, ensemble }),
        });
        break;
      }
      case "actor_arc": {
        const story = requiredWorkspaceValue(workspace.story, "story");
        const ensemble = requiredWorkspaceValue(workspace.ensemble, "ensemble");
        const profile = requiredProfile(profiles, task.playerId);
        const assignment = requiredAssignment(ensemble, task.playerId);
        const request = buildActorArcRequest({
          game: input.game,
          plan,
          story,
          profile,
          assignment,
          relationshipSeeds: ensemble.relationshipSeeds.filter((seed) =>
            seed.playerIds.includes(task.playerId)
          ),
        });
        await completeTask({
          task,
          request,
          validate: (parsed) => parseActorDirection(parsed, assignment),
          outputContract: [
            `playerId 必须为 ${task.playerId}`,
            "baseline、pressure、change、payoff、signatureDescription 必须是非空字符串",
            "只展开当前一名角色；保持人物核心，不写最终台词",
          ],
          apply: (direction) => ({
            ...workspace,
            castDirections: [...workspace.castDirections, direction],
          }),
        });
        break;
      }
      case "relationship": {
        const story = requiredWorkspaceValue(workspace.story, "story");
        const ensemble = requiredWorkspaceValue(workspace.ensemble, "ensemble");
        const seed = requiredRelationshipSeed(ensemble, task.playerIds);
        const request = buildRelationshipRequest({
          plan,
          story,
          seed,
          profiles: task.playerIds.map((playerId) =>
            requiredProfile(profiles, playerId)
          ),
          directions: task.playerIds.map((playerId) =>
            requiredDirection(workspace.castDirections, playerId)
          ),
        });
        await completeTask({
          task,
          request,
          validate: (parsed) => parseRelationship(parsed, seed),
          outputContract: [
            `playerIds 必须为 ${task.playerIds.join(", ")}`,
            "setup、development、payoff 必须是非空字符串",
            "关系只来自本局公开互动，不得虚构赛前历史或游戏事实",
          ],
          apply: (relationship) => ({
            ...workspace,
            relationships: [...workspace.relationships, relationship],
          }),
        });
        break;
      }
      case "beats":
        await completeBeatTask(task);
        break;
    }
  }

  const story = requiredWorkspaceValue(workspace.story, "story");
  const ensemble = requiredWorkspaceValue(workspace.ensemble, "ensemble");
  assertWorkspaceComplete({ game: input.game, plan, workspace, ensemble });
  const script = createEpisodeScriptSnapshot({
    id: `episode_${randomUUID()}`,
    game: input.game,
    plan,
    ...story,
    castDirections: workspace.castDirections,
    relationships: workspace.relationships,
    beats: workspace.beats,
    createdAt: input.createdAt,
    provider: workspace.modelBinding.provider,
    model: workspace.modelBinding.model,
  });

  return {
    script,
    workspace,
    requests,
    tokenUsage: requests.reduce<LlmTokenUsage | null>(
      (usage, request) => mergeUsage(usage, request.tokenUsage),
      null,
    ),
    repaired: requests.some((request) => Boolean(request.attempts)),
  };
}

function nextEpisodeAuthorTask(input: {
  readonly game: Game;
  readonly plan: CompiledEpisodePlan;
  readonly workspace: EpisodeAuthorWorkspace;
}): EpisodeAuthorTask | null {
  const { game, plan, workspace } = input;
  if (!workspace.story) return { kind: "story" };
  if (!workspace.ensemble) return { kind: "ensemble" };

  const completedPlayers = new Set(
    workspace.castDirections.map((direction) => direction.playerId),
  );
  const nextPlayer = game.players.find(
    (player) => !completedPlayers.has(player.playerId),
  );
  if (nextPlayer) return { kind: "actor_arc", playerId: nextPlayer.playerId };

  const completedPairs = new Set(
    workspace.relationships.map((relationship) => pairKey(relationship.playerIds)),
  );
  const nextRelationship = workspace.ensemble.relationshipSeeds.find(
    (seed) => !completedPairs.has(pairKey(seed.playerIds)),
  );
  if (nextRelationship) {
    return {
      kind: "relationship",
      playerIds: [...nextRelationship.playerIds],
    };
  }

  const completedBeats = new Set(workspace.beats.map((beat) => beat.stepIndex));
  const remaining = plan.steps.filter(
    (step) => step.speechBeat !== null && !completedBeats.has(step.index),
  );
  const first = remaining[0];
  if (!first) return null;
  const stepIndexes: number[] = [];
  for (const step of remaining) {
    if (
      stepIndexes.length >= MAX_BEATS_PER_AGENT_SCENE ||
      step.slot.phase !== first.slot.phase
    ) {
      break;
    }
    stepIndexes.push(step.index);
  }
  return { kind: "beats", stepIndexes };
}

function buildStoryRequest(
  game: Game,
  plan: CompiledEpisodePlan,
): GenerationRequestSnapshot {
  const milestones = episodeMilestones(plan);
  return {
    schemaName: "werewolf_episode_story_v4",
    systemPrompt: [
      "你是狼人杀节目的 Script Author Agent，当前只负责故事主轴。",
      "合法事件轨迹已经确定；不得改变行动、票型、死亡、身份、预算或胜方。",
      "本轮不处理角色弧线、关系细节或逐场台词。只返回 JSON 对象。",
      CONCISE_AUTHOR_OUTPUT_INSTRUCTION,
    ].join("\n"),
    messages: [
      {
        role: "user",
        content: [
          `提示词版本：${EPISODE_AUTHOR_PROMPT_VERSION}`,
          `主题：${game.script.name}｜${game.script.theme}`,
          `共同背景：${game.script.background}`,
          `氛围：${game.script.atmosphere.join("、")}`,
          `计划胜方：${plan.plannedWinner}`,
          `计划天数：${plan.plannedDayCount}`,
          "",
          "合法轨迹里程碑：",
          ...milestones,
          "",
          `输出 title、logline、acts[{title,summary}]。acts 必须为 1 到 ${MAX_EPISODE_STORY_ACTS} 项，建议 3 项。`,
          "只建立全局因果和节奏，不写任何玩家的完整弧线、关系长文或最终台词。",
        ].join("\n"),
      },
    ],
  };
}

function buildEnsembleRequest(input: {
  readonly game: Game;
  readonly plan: CompiledEpisodePlan;
  readonly story: EpisodeStorySpine;
  readonly profiles: readonly EpisodeActorProfile[];
  readonly opportunities: ReadonlyMap<PlayerId, readonly number[]>;
}): GenerationRequestSnapshot {
  return {
    schemaName: "werewolf_episode_ensemble_v4",
    systemPrompt: [
      "你是狼人杀节目的 Script Author Agent，当前只负责群像分工。",
      "比较全体演员，分配主次功能、真实标志节点和本局关系配对。",
      "保持输出紧凑，不要展开长篇人物弧线、关系过程或最终台词。只返回 JSON 对象。",
      CONCISE_AUTHOR_OUTPUT_INSTRUCTION,
    ].join("\n"),
    messages: [
      {
        role: "user",
        content: [
          `提示词版本：${EPISODE_AUTHOR_PROMPT_VERSION}`,
          `STORY ${JSON.stringify(input.story)}`,
          `计划胜方：${input.plan.plannedWinner}`,
          "",
          "演员精简卡与可用标志节点（每行 JSON）：",
          ...input.profiles.map((profile) =>
            actorProfileLine(
              profile,
              input.opportunities.get(profile.playerId) ?? [],
            )
          ),
          "",
          "输出 castAssignments[{playerId,dramaticWeight,dramaticFunction,signatureStepIndex}]。",
          "输出 relationshipSeeds[{playerIds,kind}]，kind 为 rivalry/alliance/contrast/trust_shift。",
          `castAssignments 恰好覆盖所有演员；relationshipSeeds 必须为 1 到 ${MAX_EPISODE_RELATIONSHIP_SEEDS} 项，同一无向玩家对不得重复。`,
          "不要输出 baseline、pressure、change、payoff、setup、development 或关系 payoff。",
        ].join("\n"),
      },
    ],
  };
}

function buildActorArcRequest(input: {
  readonly game: Game;
  readonly plan: CompiledEpisodePlan;
  readonly story: EpisodeStorySpine;
  readonly profile: EpisodeActorProfile;
  readonly assignment: EpisodeCastAssignment;
  readonly relationshipSeeds: readonly EpisodeRelationshipSeed[];
}): GenerationRequestSnapshot {
  const signatureStep = input.plan.steps[input.assignment.signatureStepIndex - 1];
  return {
    schemaName: "werewolf_episode_actor_arc_v4",
    systemPrompt: [
      "你是狼人杀节目的 Script Author Agent，当前只展开一名演员的本局弧线。",
      "稳定人物核心必须保留；变化来自合法轨迹中的压力、失误、适应和兑现。",
      "不得改变游戏事实，不得写最终台词。只返回 JSON 对象。",
      CONCISE_AUTHOR_OUTPUT_INSTRUCTION,
    ].join("\n"),
    messages: [
      {
        role: "user",
        content: [
          `提示词版本：${EPISODE_AUTHOR_PROMPT_VERSION}`,
          `STORY ${JSON.stringify(input.story)}`,
          `ACTOR_PROFILE ${JSON.stringify(input.profile)}`,
          `CAST_ASSIGNMENT ${JSON.stringify(input.assignment)}`,
          `RELATED_SEEDS ${JSON.stringify(input.relationshipSeeds)}`,
          `SIGNATURE_STEP ${input.assignment.signatureStepIndex} | ${signatureStep?.summary ?? "合法表演节点"}`,
          `主题背景：${input.game.script.name}｜${input.game.script.theme}`,
          "",
          "输出 {playerId,baseline,pressure,change,payoff,signatureDescription}。",
          "只处理当前演员；change 必须保留人物核心，所有描述基于本局合法轨迹。",
        ].join("\n"),
      },
    ],
  };
}

function buildRelationshipRequest(input: {
  readonly plan: CompiledEpisodePlan;
  readonly story: EpisodeStorySpine;
  readonly seed: EpisodeRelationshipSeed;
  readonly profiles: readonly EpisodeActorProfile[];
  readonly directions: readonly EpisodeCastDirection[];
}): GenerationRequestSnapshot {
  return {
    schemaName: "werewolf_episode_relationship_v4",
    systemPrompt: [
      "你是狼人杀节目的 Script Author Agent，当前只展开一对本局关系。",
      "关系必须从公开互动中形成，不得虚构赛前历史、私下交易或游戏证据。",
      "不得写最终台词。只返回 JSON 对象。",
      CONCISE_AUTHOR_OUTPUT_INSTRUCTION,
    ].join("\n"),
    messages: [
      {
        role: "user",
        content: [
          `提示词版本：${EPISODE_AUTHOR_PROMPT_VERSION}`,
          `STORY ${JSON.stringify(input.story)}`,
          `RELATIONSHIP_SEED ${JSON.stringify(input.seed)}`,
          ...input.profiles.map((profile) =>
            `ACTOR_PROFILE ${JSON.stringify(profile)}`
          ),
          ...input.directions.map((direction) =>
            `ACTOR_DIRECTION ${JSON.stringify(direction)}`
          ),
          "",
          "本局关系可用节点：",
          ...relationshipMilestoneLines(
            input.plan,
            input.seed.playerIds,
            input.directions,
          ),
          "",
          "输出 {playerIds,setup,development,payoff}。只处理当前关系。",
        ].join("\n"),
      },
    ],
  };
}

function buildBeatRequest(input: {
  readonly game: Game;
  readonly plan: CompiledEpisodePlan;
  readonly workspace: EpisodeAuthorWorkspace;
  readonly profiles: readonly EpisodeActorProfile[];
  readonly stepIndexes: readonly number[];
}): GenerationRequestSnapshot {
  const batch = input.stepIndexes.map((index) => {
    const step = input.plan.steps[index - 1];
    if (!step?.speechBeat) throw new Error(`Missing speech step: ${index}`);
    return step;
  });
  const actorPlayerIds = uniquePlayerIds(
    batch.map((step) => step.slot.actorPlayerId),
  );
  const firstIndex = Math.max(1, input.stepIndexes[0]! - 2);
  const lastIndex = Math.min(
    input.plan.steps.length,
    input.stepIndexes.at(-1)! + 2,
  );
  const localTrace = input.plan.steps.slice(firstIndex - 1, lastIndex);
  const priorMoves = priorMovesForActors(
    input.workspace.beats,
    input.plan,
    actorPlayerIds,
  );
  return {
    schemaName: "werewolf_episode_beats_v4",
    systemPrompt: [
      "你是狼人杀节目的 Script Author Agent，当前只编写一个局部场景的发言节拍。",
      "不得修改合法轨迹、planned payload 或预算，不得向演员泄露未来事件。",
      "不要生成最终台词。只返回 JSON 对象。",
      CONCISE_AUTHOR_OUTPUT_INSTRUCTION,
    ].join("\n"),
    messages: [
      {
        role: "user",
        content: [
          `提示词版本：${EPISODE_AUTHOR_PROMPT_VERSION}`,
          `STORY ${JSON.stringify(requiredWorkspaceValue(input.workspace.story, "story"))}`,
          `主题：${input.game.script.name}｜${input.game.script.theme}`,
          "",
          "当前场景演员上下文（每行 JSON）：",
          ...actorPlayerIds.map((playerId) =>
            actorContextLine({
              profile: requiredProfile(input.profiles, playerId),
              direction: requiredDirection(
                input.workspace.castDirections,
                playerId,
              ),
              relationships: input.workspace.relationships.filter(
                (relationship) => relationship.playerIds.includes(playerId),
              ),
            })
          ),
          ...(priorMoves.length > 0
            ? [
                "",
                "当前演员最近一个已写方向：",
                ...priorMoves.map((move) =>
                  `PRIOR_MOVE ${JSON.stringify(move)}`
                ),
              ]
            : []),
          "",
          "局部合法轨迹：",
          ...localTrace.map(
            (step) =>
              `STEP ${step.index} | ${step.slot.phase} | ${step.slot.type} | actor=${step.slot.actorPlayerId ?? "host"} | ${step.summary}`,
          ),
          "",
          "本场必须生成：",
          ...batch.map(
            (step) =>
              `SPEECH_STEP ${step.index} | ${step.slot.type} | actor=${step.slot.actorPlayerId} | budget=${JSON.stringify(step.speechBeat?.budget ?? null)}`,
          ),
          "",
          "输出 beats[{stepIndex,objective,performanceMove,disclosure,themeHook,actorHook,arcMove,relationshipMove}]。",
          "beats 必须且只能覆盖本场 SPEECH_STEP；relationshipMove 无适用关系时为 null。",
          "所有方向必须由演员届时可见事实执行，不得写未来答案或隐藏身份提示。",
        ].join("\n"),
      },
    ],
  };
}

function buildConciseRetryRequest(
  request: GenerationRequestSnapshot,
): GenerationRequestSnapshot {
  return {
    ...request,
    systemPrompt: [
      request.systemPrompt,
      "上次响应达到输出上限。本次从原始任务重新生成，不要复述上次内容。",
      CONCISE_AUTHOR_OUTPUT_INSTRUCTION,
    ].join("\n"),
  };
}

async function performAgentRequest<Value>(input: {
  readonly task: EpisodeAuthorTask;
  readonly request: GenerationRequestSnapshot;
  readonly llmClient: LlmClient;
  readonly modelBinding: ModelBindingSnapshot;
  readonly createdAt: string;
  readonly validate: (parsed: Record<string, unknown>) => Value;
  readonly outputContract: readonly string[];
}): Promise<RequestOutcome<Value>> {
  try {
    const result = await retryHeadersTimeout(() =>
      generateValidatedJson({
        llmClient: input.llmClient,
        modelBinding: input.modelBinding,
        request: input.request,
        validate: input.validate,
        repair: { outputContract: input.outputContract },
      })
    );
    return {
      ok: true,
      result,
      record: {
        id: `episode_request_${randomUUID()}`,
        task: structuredClone(input.task),
        status: "success",
        promptVersion: EPISODE_AUTHOR_PROMPT_VERSION,
        provider: result.output.provider,
        model: result.output.model,
        request: input.request,
        tokenUsage: result.tokenUsage,
        finishReason: result.output.finishReason,
        rawOutput: result.output.rawText,
        parsedOutput: result.output.parsed,
        error: null,
        createdAt: input.createdAt,
        ...(result.attempts ? { attempts: result.attempts } : {}),
      },
    };
  } catch (error) {
    const failure = error instanceof ValidatedGenerationError ? error : null;
    return {
      ok: false,
      error,
      finishReason: failure?.finishReason ?? null,
      record: {
        id: `episode_request_${randomUUID()}`,
        task: structuredClone(input.task),
        status: "failed",
        promptVersion: EPISODE_AUTHOR_PROMPT_VERSION,
        provider: input.modelBinding.provider,
        model: input.modelBinding.model,
        request: input.request,
        tokenUsage: failure?.tokenUsage ?? null,
        finishReason: failure?.finishReason ?? null,
        rawOutput: failure?.rawOutput ?? null,
        parsedOutput: null,
        error: error instanceof Error ? error.message : String(error),
        createdAt: input.createdAt,
        ...(failure?.attempts ? { attempts: failure.attempts } : {}),
      },
    };
  }
}

function parseStory(parsed: Record<string, unknown>): EpisodeStorySpine {
  const rawActs = requiredArray(parsed.acts, "acts");
  if (rawActs.length === 0 || rawActs.length > MAX_EPISODE_STORY_ACTS) {
    throw new Error(
      `acts must contain between 1 and ${MAX_EPISODE_STORY_ACTS} items`,
    );
  }
  const acts = rawActs.map((value, index) => {
    const item = requiredObject(value, `acts[${index}]`);
    return {
      title: requiredString(item.title, `acts[${index}].title`),
      summary: requiredString(item.summary, `acts[${index}].summary`),
    };
  });
  return {
    title: requiredString(parsed.title, "title"),
    logline: requiredString(parsed.logline, "logline"),
    acts,
  };
}

function parseEnsemble(
  parsed: Record<string, unknown>,
  game: Game,
  opportunities: ReadonlyMap<PlayerId, readonly number[]>,
): EpisodeEnsembleMap {
  const castAssignments = requiredArray(
    parsed.castAssignments,
    "castAssignments",
  ).map((value, index): EpisodeCastAssignment => {
    const item = requiredObject(value, `castAssignments[${index}]`);
    const weight = item.dramaticWeight;
    if (weight !== "primary" && weight !== "supporting") {
      throw new Error(`castAssignments[${index}].dramaticWeight is invalid`);
    }
    if (!Number.isInteger(item.signatureStepIndex)) {
      throw new Error(`castAssignments[${index}].signatureStepIndex is invalid`);
    }
    return {
      playerId: requiredString(
        item.playerId,
        `castAssignments[${index}].playerId`,
      ) as PlayerId,
      dramaticWeight: weight,
      dramaticFunction: requiredString(
        item.dramaticFunction,
        `castAssignments[${index}].dramaticFunction`,
      ),
      signatureStepIndex: item.signatureStepIndex as number,
    };
  });
  const expectedIds = game.players.map((player) => player.playerId).sort();
  const actualIds = castAssignments
    .map((assignment) => assignment.playerId)
    .sort();
  if (JSON.stringify(actualIds) !== JSON.stringify(expectedIds)) {
    throw new Error("castAssignments must cover every player exactly once");
  }
  for (const assignment of castAssignments) {
    if (!(opportunities.get(assignment.playerId) ?? []).includes(
      assignment.signatureStepIndex,
    )) {
      throw new Error(
        `castAssignment ${assignment.playerId} has an invalid signature step`,
      );
    }
  }

  const knownPlayers = new Set(expectedIds);
  const seenPairs = new Set<string>();
  const rawRelationshipSeeds = requiredArray(
    parsed.relationshipSeeds,
    "relationshipSeeds",
  );
  if (
    rawRelationshipSeeds.length === 0 ||
    rawRelationshipSeeds.length > MAX_EPISODE_RELATIONSHIP_SEEDS
  ) {
    throw new Error(
      `relationshipSeeds must contain between 1 and ${MAX_EPISODE_RELATIONSHIP_SEEDS} items`,
    );
  }
  const relationshipSeeds = rawRelationshipSeeds.map(
    (value, index): EpisodeRelationshipSeed => {
      const item = requiredObject(value, `relationshipSeeds[${index}]`);
      const playerIds = requiredArray(
        item.playerIds,
        `relationshipSeeds[${index}].playerIds`,
      );
      if (playerIds.length !== 2) {
        throw new Error(`relationshipSeeds[${index}].playerIds is invalid`);
      }
      const pair: readonly [PlayerId, PlayerId] = [
        requiredString(
          playerIds[0],
          `relationshipSeeds[${index}].playerIds[0]`,
        ) as PlayerId,
        requiredString(
          playerIds[1],
          `relationshipSeeds[${index}].playerIds[1]`,
        ) as PlayerId,
      ];
      if (
        pair[0] === pair[1] ||
        !knownPlayers.has(pair[0]) ||
        !knownPlayers.has(pair[1])
      ) {
        throw new Error(
          `relationshipSeeds[${index}] references invalid players`,
        );
      }
      const key = pairKey(pair);
      if (seenPairs.has(key)) {
        throw new Error(`Duplicate relationship seed: ${key}`);
      }
      seenPairs.add(key);
      if (!isRelationshipKind(item.kind)) {
        throw new Error(`relationshipSeeds[${index}].kind is invalid`);
      }
      return { playerIds: pair, kind: item.kind };
    },
  );
  return { castAssignments, relationshipSeeds };
}

function parseActorDirection(
  parsed: Record<string, unknown>,
  assignment: EpisodeCastAssignment,
): EpisodeCastDirection {
  const playerId = requiredString(parsed.playerId, "playerId") as PlayerId;
  if (playerId !== assignment.playerId) {
    throw new Error(`playerId must be ${assignment.playerId}`);
  }
  return {
    playerId,
    dramaticWeight: assignment.dramaticWeight,
    dramaticFunction: assignment.dramaticFunction,
    baseline: requiredString(parsed.baseline, "baseline"),
    pressure: requiredString(parsed.pressure, "pressure"),
    change: requiredString(parsed.change, "change"),
    payoff: requiredString(parsed.payoff, "payoff"),
    signatureMoment: {
      stepIndex: assignment.signatureStepIndex,
      description: requiredString(
        parsed.signatureDescription,
        "signatureDescription",
      ),
    },
  };
}

function parseRelationship(
  parsed: Record<string, unknown>,
  seed: EpisodeRelationshipSeed,
): EpisodeRelationshipDirection {
  const playerIds = requiredArray(parsed.playerIds, "playerIds");
  if (
    playerIds.length !== 2 ||
    pairKey(playerIds.map((value, index) =>
      requiredString(value, `playerIds[${index}]`) as PlayerId
    )) !== pairKey(seed.playerIds)
  ) {
    throw new Error(`playerIds must be ${seed.playerIds.join(", ")}`);
  }
  return {
    playerIds: [...seed.playerIds],
    kind: seed.kind,
    setup: requiredString(parsed.setup, "setup"),
    development: requiredString(parsed.development, "development"),
    payoff: requiredString(parsed.payoff, "payoff"),
  };
}

function parseBeats(
  parsed: Record<string, unknown>,
  expectedSpeechStepIndexes: readonly number[],
): readonly Omit<EpisodeSpeechBeat, "budget">[] {
  const beats = requiredArray(parsed.beats, "beats").map(
    (value, index): Omit<EpisodeSpeechBeat, "budget"> => {
      const item = requiredObject(value, `beats[${index}]`);
      const stepIndex = item.stepIndex;
      if (!Number.isInteger(stepIndex)) {
        throw new Error(`beats[${index}].stepIndex must be an integer`);
      }
      const disclosure = item.disclosure;
      if (
        disclosure !== "conceal" &&
        disclosure !== "claim" &&
        disclosure !== "not_applicable"
      ) {
        throw new Error(`beats[${index}].disclosure is invalid`);
      }
      return {
        stepIndex: stepIndex as number,
        objective: requiredString(item.objective, `beats[${index}].objective`),
        performanceMove: requiredString(item.performanceMove, `beats[${index}].performanceMove`),
        disclosure,
        themeHook: requiredString(item.themeHook, `beats[${index}].themeHook`),
        actorHook: requiredString(
          item.actorHook,
          `beats[${index}].actorHook`,
        ),
        arcMove: requiredString(item.arcMove, `beats[${index}].arcMove`),
        relationshipMove: nullableString(
          item.relationshipMove,
          `beats[${index}].relationshipMove`,
        ),
      };
    },
  );
  const actual = beats.map((beat) => beat.stepIndex).sort((a, b) => a - b);
  const expected = [...expectedSpeechStepIndexes].sort((a, b) => a - b);
  if (JSON.stringify(actual) !== JSON.stringify(expected)) {
    throw new Error("beats must cover every speech step exactly once");
  }
  return beats;
}

function assertWorkspaceMatchesPlan(
  workspace: EpisodeAuthorWorkspace,
  plan: CompiledEpisodePlan,
): void {
  const speechStepCount = plan.steps.filter((step) => step.speechBeat).length;
  if (
    workspace.inputHash !== plan.inputHash ||
    workspace.planStepCount !== plan.steps.length ||
    workspace.speechStepCount !== speechStepCount
  ) {
    throw new Error("Episode author workspace no longer matches the compiled plan");
  }
}

function assertWorkspaceComplete(input: {
  readonly game: Game;
  readonly plan: CompiledEpisodePlan;
  readonly workspace: EpisodeAuthorWorkspace;
  readonly ensemble: EpisodeEnsembleMap;
}): void {
  if (
    input.workspace.castDirections.length !== input.game.players.length ||
    input.workspace.relationships.length !== input.ensemble.relationshipSeeds.length ||
    input.workspace.beats.length !== input.workspace.speechStepCount
  ) {
    throw new Error("Episode author workspace is incomplete");
  }
  assertEpisodeDramaturgy({
    game: input.game,
    plan: input.plan,
    castDirections: input.workspace.castDirections,
    relationships: input.workspace.relationships,
  });
}

function episodeMilestones(plan: CompiledEpisodePlan): readonly string[] {
  return plan.steps
    .filter((step) =>
      [
        "phase_started",
        "death_announced",
        "exile_resolved",
        "hunter_shot_decided",
        "game_ended",
      ].includes(step.slot.type)
    )
    .map(
      (step) =>
        `STEP ${step.index} | ${step.slot.phase} | ${step.slot.type} | ${step.summary}`,
    );
}

function actorProfileLine(
  profile: EpisodeActorProfile,
  performanceStepIndexes: readonly number[],
): string {
  return `ACTOR_PROFILE ${JSON.stringify({
    ...profile,
    performanceStepIndexes,
  })}`;
}

function actorContextLine(input: {
  readonly profile: EpisodeActorProfile;
  readonly direction: EpisodeCastDirection;
  readonly relationships: readonly EpisodeRelationshipDirection[];
}): string {
  return `ACTOR_CONTEXT ${JSON.stringify(input)}`;
}

function priorMovesForActors(
  authoredBeats: readonly Omit<EpisodeSpeechBeat, "budget">[],
  plan: CompiledEpisodePlan,
  actorPlayerIds: readonly PlayerId[],
): readonly {
  readonly playerId: PlayerId;
  readonly stepIndex: number;
  readonly actorHook: string | null;
  readonly arcMove: string | null;
  readonly relationshipMove: string | null;
}[] {
  return actorPlayerIds.flatMap((playerId) =>
    authoredBeats
      .filter(
        (beat) =>
          plan.steps[beat.stepIndex - 1]?.slot.actorPlayerId === playerId,
      )
      .slice(-MAX_PRIOR_MOVES_PER_ACTOR)
      .map((beat) => ({
        playerId,
        stepIndex: beat.stepIndex,
        actorHook: beat.actorHook,
        arcMove: beat.arcMove,
        relationshipMove: beat.relationshipMove,
      })),
  );
}

function relationshipMilestoneLines(
  plan: CompiledEpisodePlan,
  playerIds: readonly [PlayerId, PlayerId],
  directions: readonly EpisodeCastDirection[],
): readonly string[] {
  const pair = new Set(playerIds);
  const actorStepIndexes = plan.steps
    .filter(
      (step) =>
        step.slot.actorPlayerId !== null && pair.has(step.slot.actorPlayerId),
    )
    .map((step) => step.index);
  const indexes = [
    ...new Set(
      [
        actorStepIndexes[0],
        actorStepIndexes[Math.floor(actorStepIndexes.length / 2)],
        actorStepIndexes.at(-1),
        ...directions.map((direction) => direction.signatureMoment.stepIndex),
      ].filter((value): value is number => value !== undefined),
    ),
  ].sort((left, right) => left - right);

  return indexes.map((index) => {
    const step = plan.steps[index - 1];
    if (!step) throw new Error(`Missing relationship milestone step: ${index}`);
    return [
      `RELATIONSHIP_STEP ${index}`,
      step.slot.phase,
      step.slot.type,
      `actor=${step.slot.actorPlayerId ?? "host"}`,
      step.summary,
    ].join(" | ");
  });
}

function assertPerformanceOpportunities(
  profiles: readonly EpisodeActorProfile[],
  opportunities: ReadonlyMap<PlayerId, readonly number[]>,
): void {
  const missing = profiles.filter(
    (profile) => (opportunities.get(profile.playerId) ?? []).length === 0,
  );
  if (missing.length > 0) {
    throw new Error(
      `Episode plan has no performance opportunity for: ${missing.map((profile) => profile.playerId).join(", ")}`,
    );
  }
}

function requiredProfile(
  profiles: readonly EpisodeActorProfile[],
  playerId: PlayerId,
): EpisodeActorProfile {
  const profile = profiles.find((candidate) => candidate.playerId === playerId);
  if (!profile) throw new Error(`Missing Actor profile: ${playerId}`);
  return profile;
}

function requiredAssignment(
  ensemble: EpisodeEnsembleMap,
  playerId: PlayerId,
): EpisodeCastAssignment {
  const assignment = ensemble.castAssignments.find(
    (candidate) => candidate.playerId === playerId,
  );
  if (!assignment) throw new Error(`Missing cast assignment: ${playerId}`);
  return assignment;
}

function requiredRelationshipSeed(
  ensemble: EpisodeEnsembleMap,
  playerIds: readonly [PlayerId, PlayerId],
): EpisodeRelationshipSeed {
  const seed = ensemble.relationshipSeeds.find(
    (candidate) => pairKey(candidate.playerIds) === pairKey(playerIds),
  );
  if (!seed) throw new Error(`Missing relationship seed: ${playerIds.join(":")}`);
  return seed;
}

function requiredDirection(
  directions: readonly EpisodeCastDirection[],
  playerId: PlayerId,
): EpisodeCastDirection {
  const direction = directions.find((candidate) => candidate.playerId === playerId);
  if (!direction) throw new Error(`Missing cast direction: ${playerId}`);
  return direction;
}

function requiredWorkspaceValue<Value>(
  value: Value | null,
  label: string,
): Value {
  if (value === null) throw new Error(`Missing episode author ${label}`);
  return value;
}

function uniquePlayerIds(
  values: readonly (PlayerId | null)[],
): readonly PlayerId[] {
  return [...new Set(values.filter((value): value is PlayerId => value !== null))];
}

function pairKey(playerIds: readonly PlayerId[]): string {
  return [...playerIds].sort().join(":");
}

function isRelationshipKind(value: unknown): value is EpisodeRelationshipKind {
  return (
    value === "rivalry" ||
    value === "alliance" ||
    value === "contrast" ||
    value === "trust_shift"
  );
}

function mergeUsage(
  left: LlmTokenUsage | null,
  right: LlmTokenUsage | null,
): LlmTokenUsage | null {
  if (!left) return right;
  if (!right) return left;
  return {
    promptTokens: add(left.promptTokens, right.promptTokens),
    completionTokens: add(left.completionTokens, right.completionTokens),
    totalTokens: add(left.totalTokens, right.totalTokens),
    cachedPromptTokens: add(left.cachedPromptTokens, right.cachedPromptTokens),
    reasoningTokens: add(left.reasoningTokens, right.reasoningTokens),
  };
}

function add(
  left: number | null | undefined,
  right: number | null | undefined,
): number | null {
  return left === null || left === undefined
    ? right ?? null
    : right === null || right === undefined
      ? left
      : left + right;
}

async function retryHeadersTimeout<Value>(
  operation: () => Promise<Value>,
): Promise<Value> {
  try {
    return await operation();
  } catch (error) {
    if (!isHeadersTimeout(error)) throw error;
    return operation();
  }
}

function isHeadersTimeout(error: unknown): boolean {
  return error instanceof Error && /Headers Timeout Error/i.test(error.message);
}

function requiredArray(value: unknown, label: string): readonly unknown[] {
  if (!Array.isArray(value)) throw new Error(`${label} must be an array`);
  return value;
}

function requiredObject(
  value: unknown,
  label: string,
): Record<string, unknown> {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    throw new Error(`${label} must be an object`);
  }
  return value as Record<string, unknown>;
}

function requiredString(value: unknown, label: string): string {
  if (typeof value !== "string" || value.trim().length === 0) {
    throw new Error(`${label} must be a non-empty string`);
  }
  const trimmed = value.trim();
  if (Array.from(trimmed).length > MAX_AUTHOR_FIELD_CHARACTERS) {
    throw new Error(
      `${label} must not exceed ${MAX_AUTHOR_FIELD_CHARACTERS} characters`,
    );
  }
  return trimmed;
}

function nullableString(value: unknown, label: string): string | null {
  if (value === null) return null;
  return requiredString(value, label);
}
