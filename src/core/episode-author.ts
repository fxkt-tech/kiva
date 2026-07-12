import { randomUUID } from "node:crypto";
import type { Game } from "./game";
import type { LlmClient, LlmTokenUsage } from "./llm";
import {
  compileEpisodePlan,
  createEpisodeScriptSnapshot,
  type EpisodeAct,
  type EpisodeScriptSnapshot,
  type EpisodeSpeechBeat,
} from "./episode-script";
import { generateValidatedJson } from "./validated-generation";

export const EPISODE_AUTHOR_PROMPT_VERSION = "episode-author:v1";

export type AuthorEpisodeScriptResult = {
  readonly script: EpisodeScriptSnapshot;
  readonly tokenUsage: LlmTokenUsage | null;
  readonly repaired: boolean;
};

type EpisodeNarrative = {
  readonly title: string;
  readonly logline: string;
  readonly acts: readonly EpisodeAct[];
  readonly beats: readonly Omit<EpisodeSpeechBeat, "budget">[];
};

const MAX_BEATS_PER_AUTHOR_REQUEST = 12;

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
  const outlineRequest = {
    schemaName: "werewolf_episode_outline_v1",
    systemPrompt: [
      "你是狼人杀节目 Script Author。",
      "规则引擎已经确定了一条完整、合法的事件轨迹。你只负责让这条轨迹形成主题化因果故事，绝不能改变行动、票型、死亡、身份或胜方。",
      "本次只生成精炼的标题、logline 和三幕结构，不生成逐场 beat 或最终台词。只返回 JSON 对象。",
    ].join("\n"),
    messages: [
      {
        role: "user" as const,
        content: [
          `主题：${input.game.script.name}｜${input.game.script.theme}`,
          `共同背景：${input.game.script.background}`,
          `氛围：${input.game.script.atmosphere.join("、")}`,
          `计划胜方：${plan.plannedWinner}`,
          `计划天数：${plan.plannedDayCount}`,
          "",
          "角色与隐藏身份：",
          ...input.game.players.map(
            (player) =>
              `${player.playerId} | ${player.seatNo}号 ${player.name} | ${player.gameRole}`,
          ),
          "",
          "合法轨迹里程碑：",
          ...outlineSteps.map(
            (step) =>
              `STEP ${step.index} | ${step.slot.phase} | ${step.slot.type} | actor=${step.slot.actorPlayerId ?? "host"} | ${step.summary}`,
          ),
          "",
          "输出：title、logline、acts[{title,summary}]。acts 建议 3 项。",
        ].join("\n"),
      },
    ],
  };
  const outlineResult = await retryHeadersTimeout(() => generateValidatedJson({
    llmClient: input.llmClient,
    modelBinding,
    request: outlineRequest,
    validate: parseOutline,
    repair: {
      outputContract: [
        "title 和 logline 必须是非空字符串",
        "acts 必须是至少一项的 {title,summary} 数组",
      ],
    },
  }));
  const authoredBeats: Omit<EpisodeSpeechBeat, "budget">[] = [];
  let tokenUsage = outlineResult.tokenUsage;
  let repaired = Boolean(outlineResult.attempts);
  for (const batch of chunks(speechSteps, MAX_BEATS_PER_AUTHOR_REQUEST)) {
    const batchIndexes = batch.map((step) => step.index);
    const firstIndex = Math.max(1, batchIndexes[0]! - 3);
    const lastIndex = Math.min(
      plan.steps.length,
      batchIndexes.at(-1)! + 3,
    );
    const localTrace = plan.steps.slice(firstIndex - 1, lastIndex);
    const beatResult = await retryHeadersTimeout(() => generateValidatedJson({
      llmClient: input.llmClient,
      modelBinding,
      request: {
        schemaName: "werewolf_episode_beats_v1",
        systemPrompt: [
          "你是狼人杀节目 Script Author，正在分批编写逐场发言节拍。",
          "只能为本批 SPEECH_STEP 生成 beat，不得改变合法轨迹或泄露未来事件给当前演员。",
          "不要生成最终台词，只返回 JSON 对象。",
        ].join("\n"),
        messages: [
          {
            role: "user",
            content: [
              `剧名：${outlineResult.value.title}`,
              `Logline：${outlineResult.value.logline}`,
              `主题：${input.game.script.name}｜${input.game.script.theme}`,
              "",
              "本批附近的合法轨迹：",
              ...localTrace.map(
                (step) =>
                  `STEP ${step.index} | ${step.slot.phase} | ${step.slot.type} | actor=${step.slot.actorPlayerId ?? "host"}`,
              ),
              "",
              "本批必须生成：",
              ...batch.map(
                (step) =>
                  `SPEECH_STEP ${step.index} | ${step.slot.type} | actor=${step.slot.actorPlayerId}`,
              ),
              "",
              "输出 beats[{stepIndex,objective,stance,disclosure,themeHook}]。",
              "beats 必须且只能覆盖本批 SPEECH_STEP；disclosure 只能是 conceal、claim、not_applicable。",
              "themeHook 要形成可由后续公开事件验证的主题因果，但不得直接告诉演员未来会发生什么。",
            ].join("\n"),
          },
        ],
      },
      validate: (parsed) => parseBeats(parsed, batchIndexes),
      repair: {
        outputContract: [
          `beats 必须恰好覆盖 stepIndex：${batchIndexes.join(", ")}`,
          "每个 beat 包含 objective、stance、disclosure、themeHook",
        ],
      },
    }));
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
  return { title, logline, acts };
}

function parseBeats(
  parsed: Record<string, unknown>,
  expectedSpeechStepIndexes: readonly number[],
): readonly Omit<EpisodeSpeechBeat, "budget">[] {
  const beats = requiredArray(parsed.beats, "beats").map((value, index) => {
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
      disclosure: disclosure as EpisodeSpeechBeat["disclosure"],
      themeHook: requiredString(item.themeHook, `beats[${index}].themeHook`),
    };
  });
  const actual = beats.map((beat) => beat.stepIndex).sort((a, b) => a - b);
  const expected = [...expectedSpeechStepIndexes].sort((a, b) => a - b);
  if (JSON.stringify(actual) !== JSON.stringify(expected)) {
    throw new Error("beats must cover every speech step exactly once");
  }
  return beats;
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
