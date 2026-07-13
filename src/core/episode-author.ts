import { randomUUID } from "node:crypto";
import type { Game } from "./game";
import type { LlmClient, LlmTokenUsage } from "./llm";
import {
  assertEpisodeDramaturgy,
  compileEpisodePlan,
  createEpisodeScriptSnapshot,
  episodeCharacterProfile,
  episodePerformanceOpportunities,
  type CompiledEpisodePlan,
  type EpisodeAct,
  type EpisodeCastDirection,
  type EpisodeCharacterProfile,
  type EpisodeRelationshipDirection,
  type EpisodeRelationshipKind,
  type EpisodeScriptSnapshot,
  type EpisodeSpeechBeat,
} from "./episode-script";
import type { PlayerId } from "./types";
import { generateValidatedJson } from "./validated-generation";

export const EPISODE_AUTHOR_PROMPT_VERSION = "episode-author:v2";

export type AuthorEpisodeScriptResult = {
  readonly script: EpisodeScriptSnapshot;
  readonly tokenUsage: LlmTokenUsage | null;
  readonly repaired: boolean;
};

type EpisodeNarrative = {
  readonly title: string;
  readonly logline: string;
  readonly acts: readonly EpisodeAct[];
  readonly castDirections: readonly EpisodeCastDirection[];
  readonly relationships: readonly EpisodeRelationshipDirection[];
  readonly beats: readonly Omit<EpisodeSpeechBeat, "budget">[];
};

const MAX_BEATS_PER_AUTHOR_REQUEST = 12;
const MAX_PRIOR_MOVES_PER_ACTOR = 2;

export async function authorEpisodeScript(input: {
  readonly game: Game;
  readonly llmClient: LlmClient;
  readonly createdAt: string;
}): Promise<AuthorEpisodeScriptResult> {
  const plan = compileEpisodePlan(input.game);
  const speechSteps = plan.steps.filter((step) => step.speechBeat !== null);
  const outlineSteps = plan.steps.filter((step) =>
    [
      "phase_started",
      "death_announced",
      "exile_resolved",
      "hunter_shot_decided",
      "game_ended",
    ].includes(step.slot.type),
  );
  const modelBinding = input.game.players[0]?.modelBindingSnapshot;
  if (!modelBinding) throw new Error("Episode author model binding is missing");

  const profiles = input.game.players.map(episodeCharacterProfile);
  const opportunities = episodePerformanceOpportunities(input.game, plan);
  assertPerformanceOpportunities(profiles, opportunities);
  const outlineResult = await retryHeadersTimeout(() =>
    generateValidatedJson({
      llmClient: input.llmClient,
      modelBinding,
      request: {
        schemaName: "werewolf_episode_outline_v2",
        systemPrompt: [
          "你是狼人杀节目的全局 Script Author。",
          "规则引擎已经确定完整、合法的事件轨迹。你负责群像设计和主题化因果，绝不能改变行动、票型、死亡、身份、预算或胜方。",
          "让人物鲜明来自稳定性格在压力下的选择、失误、适应与兑现，不要把人设写成重复口头禅，也不要突然替换人物核心。",
          "你可以设计仅在本局互动中逐步形成的竞争、联盟、反差与信任变化；不得虚构开局前关系、私下交易或任何游戏事实，戏剧方向也不能作为身份或可信度证据。",
          "只写标题、logline、幕结构、群像方向和关系方向，不写最终台词。只返回 JSON 对象。",
        ].join("\n"),
        messages: [
          {
            role: "user",
            content: [
              `提示词版本：${EPISODE_AUTHOR_PROMPT_VERSION}`,
              `主题：${input.game.script.name}｜${input.game.script.theme}`,
              `共同背景：${input.game.script.background}`,
              `氛围：${input.game.script.atmosphere.join("、")}`,
              `计划胜方：${plan.plannedWinner}`,
              `计划天数：${plan.plannedDayCount}`,
              "",
              "角色资料与真实可用的标志性时刻（每行 JSON）：",
              ...profiles.map((profile) =>
                characterProfileLine(
                  profile,
                  opportunities.get(profile.playerId) ?? [],
                ),
              ),
              "",
              "合法轨迹里程碑：",
              ...outlineSteps.map(
                (step) =>
                  `STEP ${step.index} | ${step.slot.phase} | ${step.slot.type} | actor=${step.slot.actorPlayerId ?? "host"} | ${step.summary}`,
              ),
              "",
              "输出字段：",
              "- title、logline、acts[{title,summary}]（acts 建议 3 项）",
              "- castDirections：每位玩家恰好一项，字段为 playerId、dramaticWeight(primary/supporting)、dramaticFunction、baseline、pressure、change、payoff、signatureMoment{stepIndex,description}",
              "- relationships：字段为 playerIds[恰好两个不同玩家]、kind(rivalry/alliance/contrast/trust_shift)、setup、development、payoff",
              "每个 signatureMoment.stepIndex 必须取自该玩家 CHARACTER_PROFILE 的 performanceStepIndexes。",
              "主次可以不均，但每个人都要有清晰功能与标志性时刻；变化必须保留人物核心，并由压力与公开互动逐步挣得。",
              "relationships 只描述本局将如何铺垫、发展和兑现，不得把它写成已知事实或提前透露给演员。",
            ].join("\n"),
          },
        ],
      },
      validate: (parsed) => parseOutline(parsed, input.game, plan),
      repair: {
        outputContract: [
          "title 和 logline 必须是非空字符串，acts 必须是至少一项的 {title,summary} 数组",
          `castDirections 必须恰好覆盖这些 playerId 且不重复：${profiles.map((profile) => profile.playerId).join(", ")}`,
          ...profiles.map(
            (profile) =>
              `${profile.playerId} 的 signatureMoment.stepIndex 只能是：${(opportunities.get(profile.playerId) ?? []).join(", ")}`,
          ),
          "relationship 的 playerIds 必须是两个不同的已知玩家，同一无向玩家对只能出现一次",
          "dramaticWeight 和 kind 只能使用约定枚举，所有说明字段都必须非空",
        ],
      },
    }),
  );

  const authoredBeats: Omit<EpisodeSpeechBeat, "budget">[] = [];
  let tokenUsage = outlineResult.tokenUsage;
  let repaired = Boolean(outlineResult.attempts);
  const profileByPlayerId = new Map(
    profiles.map((profile) => [profile.playerId, profile]),
  );
  const directionByPlayerId = new Map(
    outlineResult.value.castDirections.map((direction) => [
      direction.playerId,
      direction,
    ]),
  );

  for (const batch of chunks(speechSteps, MAX_BEATS_PER_AUTHOR_REQUEST)) {
    const batchIndexes = batch.map((step) => step.index);
    const actorPlayerIds = uniquePlayerIds(
      batch.map((step) => step.slot.actorPlayerId),
    );
    const firstIndex = Math.max(1, batchIndexes[0]! - 3);
    const lastIndex = Math.min(plan.steps.length, batchIndexes.at(-1)! + 3);
    const localTrace = plan.steps.slice(firstIndex - 1, lastIndex);
    const priorMoves = priorMovesForActors(
      authoredBeats,
      plan,
      actorPlayerIds,
    );
    const beatResult = await retryHeadersTimeout(() =>
      generateValidatedJson({
        llmClient: input.llmClient,
        modelBinding,
        request: {
          schemaName: "werewolf_episode_beats_v2",
          systemPrompt: [
            "你是狼人杀节目 Script Author，正在分批编写人物驱动的逐场发言节拍。",
            "只能为本批 SPEECH_STEP 生成方向，不得修改合法轨迹、planned payload 或编译器预算，也不得向演员泄露未来事件。",
            "人物核心保持稳定；让压力暴露次要侧面、造成失误或促成有根据的适应。不要生成最终台词，只返回 JSON 对象。",
          ].join("\n"),
          messages: [
            {
              role: "user",
              content: [
                `提示词版本：${EPISODE_AUTHOR_PROMPT_VERSION}`,
                `剧名：${outlineResult.value.title}`,
                `Logline：${outlineResult.value.logline}`,
                `主题：${input.game.script.name}｜${input.game.script.theme}`,
                "",
                "本批演员资料、全局方向与相关关系（每行 JSON）：",
                ...actorPlayerIds.map((playerId) =>
                  actorContextLine({
                    profile: requiredMapValue(
                      profileByPlayerId,
                      playerId,
                      "character profile",
                    ),
                    direction: requiredMapValue(
                      directionByPlayerId,
                      playerId,
                      "cast direction",
                    ),
                    relationships: outlineResult.value.relationships.filter(
                      (relationship) =>
                        relationship.playerIds.includes(playerId),
                    ),
                  }),
                ),
                ...(priorMoves.length > 0
                  ? [
                      "",
                      `这些是同一演员此前最多 ${MAX_PRIOR_MOVES_PER_ACTOR} 个已写方向，用于延续而非重置弧线：`,
                      ...priorMoves.map(
                        (move) => `PRIOR_MOVE ${JSON.stringify(move)}`,
                      ),
                    ]
                  : []),
                "",
                "本批附近的合法轨迹（只用于作者规划；不得在 beat 中把未来写成演员已知事实）：",
                ...localTrace.map(
                  (step) =>
                    `STEP ${step.index} | ${step.slot.phase} | ${step.slot.type} | actor=${step.slot.actorPlayerId ?? "host"} | ${step.summary}`,
                ),
                "",
                "本批必须生成：",
                ...batch.map(
                  (step) =>
                    `SPEECH_STEP ${step.index} | ${step.slot.type} | actor=${step.slot.actorPlayerId} | budget=${JSON.stringify(step.speechBeat?.budget ?? null)}`,
                ),
                "",
                "输出 beats[{stepIndex,objective,stance,disclosure,themeHook,characterHook,arcMove,relationshipMove}]。",
                "beats 必须且只能覆盖本批 SPEECH_STEP；disclosure 只能是 conceal、claim、not_applicable；relationshipMove 没有适用关系时为 null。",
                "characterHook 说明本场如何由演员稳定性格与表达/判断倾向驱动；arcMove 说明本场是铺垫、受压、适应还是兑现；relationshipMove 只描述当前互动推进。",
                "所有方向都必须由演员届时可见事实执行；不得写最终台词、未来答案、隐藏身份提示或把关系方向当证据。",
              ].join("\n"),
            },
          ],
        },
        validate: (parsed) => parseBeats(parsed, batchIndexes),
        repair: {
          outputContract: [
            `beats 必须恰好覆盖 stepIndex：${batchIndexes.join(", ")}`,
            "每个 beat 包含非空 objective、stance、themeHook、characterHook、arcMove",
            "disclosure 只能是 conceal、claim、not_applicable；relationshipMove 必须是非空字符串或 null",
          ],
        },
      }),
    );
    authoredBeats.push(...beatResult.value);
    tokenUsage = mergeUsage(tokenUsage, beatResult.tokenUsage);
    repaired ||= Boolean(beatResult.attempts);
  }

  return {
    script: createEpisodeScriptSnapshot({
      id: `episode_${randomUUID()}`,
      game: input.game,
      plan,
      ...outlineResult.value,
      beats: authoredBeats,
      createdAt: input.createdAt,
      provider: outlineResult.output.provider,
      model: outlineResult.output.model,
    }),
    tokenUsage,
    repaired,
  };
}

function parseOutline(
  parsed: Record<string, unknown>,
  game: Game,
  plan: CompiledEpisodePlan,
): Omit<EpisodeNarrative, "beats"> {
  const title = requiredString(parsed.title, "title");
  const logline = requiredString(parsed.logline, "logline");
  const acts = requiredArray(parsed.acts, "acts").map((value, index) => {
    const item = requiredObject(value, `acts[${index}]`);
    return {
      title: requiredString(item.title, `acts[${index}].title`),
      summary: requiredString(item.summary, `acts[${index}].summary`),
    };
  });
  if (acts.length === 0) throw new Error("acts must not be empty");

  const castDirections = requiredArray(
    parsed.castDirections,
    "castDirections",
  ).map((value, index): EpisodeCastDirection => {
    const item = requiredObject(value, `castDirections[${index}]`);
    const dramaticWeight = item.dramaticWeight;
    if (dramaticWeight !== "primary" && dramaticWeight !== "supporting") {
      throw new Error(`castDirections[${index}].dramaticWeight is invalid`);
    }
    const signatureMoment = requiredObject(
      item.signatureMoment,
      `castDirections[${index}].signatureMoment`,
    );
    if (!Number.isInteger(signatureMoment.stepIndex)) {
      throw new Error(
        `castDirections[${index}].signatureMoment.stepIndex must be an integer`,
      );
    }
    return {
      playerId: requiredString(
        item.playerId,
        `castDirections[${index}].playerId`,
      ) as PlayerId,
      dramaticWeight,
      dramaticFunction: requiredString(
        item.dramaticFunction,
        `castDirections[${index}].dramaticFunction`,
      ),
      baseline: requiredString(
        item.baseline,
        `castDirections[${index}].baseline`,
      ),
      pressure: requiredString(
        item.pressure,
        `castDirections[${index}].pressure`,
      ),
      change: requiredString(
        item.change,
        `castDirections[${index}].change`,
      ),
      payoff: requiredString(
        item.payoff,
        `castDirections[${index}].payoff`,
      ),
      signatureMoment: {
        stepIndex: signatureMoment.stepIndex as number,
        description: requiredString(
          signatureMoment.description,
          `castDirections[${index}].signatureMoment.description`,
        ),
      },
    };
  });
  const relationships = requiredArray(
    parsed.relationships,
    "relationships",
  ).map((value, index): EpisodeRelationshipDirection => {
    const item = requiredObject(value, `relationships[${index}]`);
    const playerIds = requiredArray(
      item.playerIds,
      `relationships[${index}].playerIds`,
    );
    if (playerIds.length !== 2) {
      throw new Error(
        `relationships[${index}].playerIds must contain two players`,
      );
    }
    const kind = item.kind;
    if (!isRelationshipKind(kind)) {
      throw new Error(`relationships[${index}].kind is invalid`);
    }
    return {
      playerIds: [
        requiredString(
          playerIds[0],
          `relationships[${index}].playerIds[0]`,
        ) as PlayerId,
        requiredString(
          playerIds[1],
          `relationships[${index}].playerIds[1]`,
        ) as PlayerId,
      ],
      kind,
      setup: requiredString(item.setup, `relationships[${index}].setup`),
      development: requiredString(
        item.development,
        `relationships[${index}].development`,
      ),
      payoff: requiredString(item.payoff, `relationships[${index}].payoff`),
    };
  });

  assertEpisodeDramaturgy({ game, plan, castDirections, relationships });
  return { title, logline, acts, castDirections, relationships };
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
        stance: requiredString(item.stance, `beats[${index}].stance`),
        disclosure,
        themeHook: requiredString(item.themeHook, `beats[${index}].themeHook`),
        characterHook: requiredString(
          item.characterHook,
          `beats[${index}].characterHook`,
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

function characterProfileLine(
  profile: EpisodeCharacterProfile,
  performanceStepIndexes: readonly number[],
): string {
  return `CHARACTER_PROFILE ${JSON.stringify({
    ...profile,
    performanceStepIndexes,
  })}`;
}

function actorContextLine(input: {
  readonly profile: EpisodeCharacterProfile;
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
  readonly characterHook: string | null;
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
        characterHook: beat.characterHook,
        arcMove: beat.arcMove,
        relationshipMove: beat.relationshipMove,
      })),
  );
}

function assertPerformanceOpportunities(
  profiles: readonly EpisodeCharacterProfile[],
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

function uniquePlayerIds(
  values: readonly (PlayerId | null)[],
): readonly PlayerId[] {
  return [...new Set(values.filter((value): value is PlayerId => value !== null))];
}

function requiredMapValue<Key, Value>(
  values: ReadonlyMap<Key, Value>,
  key: Key,
  label: string,
): Value {
  const value = values.get(key);
  if (!value) throw new Error(`Missing ${label}: ${String(key)}`);
  return value;
}

function isRelationshipKind(value: unknown): value is EpisodeRelationshipKind {
  return (
    value === "rivalry" ||
    value === "alliance" ||
    value === "contrast" ||
    value === "trust_shift"
  );
}

function chunks<Value>(
  values: readonly Value[],
  size: number,
): readonly (readonly Value[])[] {
  const result: Value[][] = [];
  for (let index = 0; index < values.length; index += size) {
    result.push(values.slice(index, index + size));
  }
  return result;
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
  return value.trim();
}

function nullableString(value: unknown, label: string): string | null {
  if (value === null) return null;
  return requiredString(value, label);
}
