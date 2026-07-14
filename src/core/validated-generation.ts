import type {
  GenerationAttemptSnapshot,
  GenerationRequestSnapshot,
} from "./generation-record";
import {
  LlmOutputParseError,
  type LlmClient,
  type LlmGenerateJsonResult,
  type LlmTokenUsage,
} from "./llm";
import type { ModelBindingSnapshot } from "./player";
import type { PlayerId } from "./types";

export type ValidatedGenerationResult<Value> = {
  readonly value: Value;
  readonly output: LlmGenerateJsonResult;
  readonly tokenUsage: LlmTokenUsage | null;
  readonly attempts?: readonly GenerationAttemptSnapshot[];
};

export class ValidatedGenerationError extends Error {
  readonly attempts: readonly GenerationAttemptSnapshot[];
  readonly rawOutput: string | null;
  readonly tokenUsage: LlmTokenUsage | null;
  readonly finishReason: string | null;

  constructor(input: {
    readonly error: unknown;
    readonly attempts: readonly GenerationAttemptSnapshot[];
    readonly rawOutput: string | null;
    readonly tokenUsage: LlmTokenUsage | null;
    readonly finishReason?: string | null;
  }) {
    super(errorMessage(input.error), { cause: input.error });
    this.name = "ValidatedGenerationError";
    this.attempts = input.attempts;
    this.rawOutput = input.rawOutput;
    this.tokenUsage = input.tokenUsage;
    this.finishReason = input.finishReason ?? null;
  }
}

export async function generateValidatedJson<Value>(input: {
  readonly llmClient: LlmClient;
  readonly modelBinding: ModelBindingSnapshot;
  readonly request: GenerationRequestSnapshot;
  readonly validate: (parsed: Record<string, unknown>) => Value;
  readonly repair: {
    readonly outputContract: readonly string[];
    readonly legalTargetPlayerIds?: readonly PlayerId[];
  };
}): Promise<ValidatedGenerationResult<Value>> {
  let firstOutput: LlmGenerateJsonResult;
  try {
    firstOutput = await input.llmClient.generateJson({
      modelBinding: input.modelBinding,
      ...input.request,
    });
  } catch (error) {
    if (!(error instanceof LlmOutputParseError)) throw error;
    if (error.finishReason === "length") {
      throw new ValidatedGenerationError({
        error,
        attempts: [attemptFromError(input.request, error)],
        rawOutput: error.rawText,
        tokenUsage: error.usage,
        finishReason: error.finishReason,
      });
    }
    return repairAfterInvalidAttempt({
      ...input,
      firstAttempt: attemptFromError(input.request, error),
      firstRawOutput: error.rawText,
      firstUsage: error.usage,
      firstError: error,
    });
  }

  try {
    return {
      value: input.validate(firstOutput.parsed),
      output: firstOutput,
      tokenUsage: firstOutput.usage,
    };
  } catch (error) {
    return repairAfterInvalidAttempt({
      ...input,
      firstAttempt: attemptFromOutput(input.request, firstOutput, error),
      firstRawOutput: firstOutput.rawText,
      firstUsage: firstOutput.usage,
      firstError: error,
    });
  }
}

async function repairAfterInvalidAttempt<Value>(input: {
  readonly llmClient: LlmClient;
  readonly modelBinding: ModelBindingSnapshot;
  readonly request: GenerationRequestSnapshot;
  readonly validate: (parsed: Record<string, unknown>) => Value;
  readonly repair: {
    readonly outputContract: readonly string[];
    readonly legalTargetPlayerIds?: readonly PlayerId[];
  };
  readonly firstAttempt: GenerationAttemptSnapshot;
  readonly firstRawOutput: string | null;
  readonly firstUsage: LlmTokenUsage | null;
  readonly firstError: unknown;
}): Promise<ValidatedGenerationResult<Value>> {
  const repairRequest = buildRepairRequest({
    schemaName: input.request.schemaName,
    invalidOutput: input.firstRawOutput,
    error: input.firstError,
    outputContract: input.repair.outputContract,
    legalTargetPlayerIds: input.repair.legalTargetPlayerIds,
  });

  let repairedOutput: LlmGenerateJsonResult;
  try {
    repairedOutput = await input.llmClient.generateJson({
      modelBinding: input.modelBinding,
      ...repairRequest,
    });
  } catch (error) {
    const rawOutput =
      error instanceof LlmOutputParseError ? error.rawText : null;
    const attempts = [
      input.firstAttempt,
      attemptFromError(repairRequest, error),
    ];
    throw new ValidatedGenerationError({
      error,
      attempts,
      rawOutput,
      tokenUsage: mergeTokenUsage(
        input.firstUsage,
        error instanceof LlmOutputParseError ? error.usage : null,
      ),
      finishReason:
        error instanceof LlmOutputParseError ? error.finishReason : null,
    });
  }

  try {
    const value = input.validate(repairedOutput.parsed);
    return {
      value,
      output: repairedOutput,
      tokenUsage: mergeTokenUsage(input.firstUsage, repairedOutput.usage),
      attempts: [
        input.firstAttempt,
        attemptFromOutput(repairRequest, repairedOutput, null),
      ],
    };
  } catch (error) {
    const attempts = [
      input.firstAttempt,
      attemptFromOutput(repairRequest, repairedOutput, error),
    ];
    throw new ValidatedGenerationError({
      error,
      attempts,
      rawOutput: repairedOutput.rawText,
      tokenUsage: mergeTokenUsage(input.firstUsage, repairedOutput.usage),
      finishReason: repairedOutput.finishReason,
    });
  }
}

function buildRepairRequest(input: {
  readonly schemaName: string;
  readonly invalidOutput: string | null;
  readonly error: unknown;
  readonly outputContract: readonly string[];
  readonly legalTargetPlayerIds?: readonly PlayerId[];
}): GenerationRequestSnapshot {
  const legalTargets = input.legalTargetPlayerIds?.length
    ? [
        "合法 targetPlayerId：",
        ...input.legalTargetPlayerIds.map((playerId) => `- ${playerId}`),
      ]
    : [];

  return {
    schemaName: input.schemaName,
    systemPrompt: [
      "你只负责修复一个无效的 JSON 输出。",
      "不要重新分析游戏，不要添加解释，只返回符合契约的 JSON 对象。",
    ].join("\n"),
    messages: [
      {
        role: "user",
        content: [
          "上次输出：",
          input.invalidOutput ?? "（模型没有返回可记录的文本）",
          "",
          `错误：${errorMessage(input.error)}`,
          "",
          "必须满足：",
          ...input.outputContract.map((line) => `- ${line}`),
          ...legalTargets,
        ].join("\n"),
      },
    ],
  };
}

function attemptFromOutput(
  request: GenerationRequestSnapshot,
  output: LlmGenerateJsonResult,
  error: unknown | null,
): GenerationAttemptSnapshot {
  return {
    request,
    tokenUsage: output.usage,
    finishReason: output.finishReason,
    rawOutput: output.rawText,
    parsedOutput: output.parsed,
    error: error === null ? null : errorMessage(error),
  };
}

function attemptFromError(
  request: GenerationRequestSnapshot,
  error: unknown,
): GenerationAttemptSnapshot {
  return {
    request,
    tokenUsage: error instanceof LlmOutputParseError ? error.usage : null,
    finishReason:
      error instanceof LlmOutputParseError ? error.finishReason : null,
    rawOutput: error instanceof LlmOutputParseError ? error.rawText : null,
    parsedOutput: null,
    error: errorMessage(error),
  };
}

function mergeTokenUsage(
  first: LlmTokenUsage | null,
  second: LlmTokenUsage | null,
): LlmTokenUsage | null {
  if (!first) return second;
  if (!second) return first;
  return {
    promptTokens: addCounts(first.promptTokens, second.promptTokens),
    completionTokens: addCounts(
      first.completionTokens,
      second.completionTokens,
    ),
    totalTokens: addCounts(first.totalTokens, second.totalTokens),
    cachedPromptTokens: addCounts(
      first.cachedPromptTokens,
      second.cachedPromptTokens,
    ),
    reasoningTokens: addCounts(first.reasoningTokens, second.reasoningTokens),
  };
}

function addCounts(
  first: number | null | undefined,
  second: number | null | undefined,
): number | null {
  return first === null || first === undefined || second === null || second === undefined
    ? null
    : first + second;
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
