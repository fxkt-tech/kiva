import type { GenerationRecord } from "./generation-record";
import type { LlmTokenUsage } from "./llm";

export type TokenUsageSummary = {
  readonly promptTokens: number;
  readonly completionTokens: number;
  readonly totalTokens: number;
  readonly cachedPromptTokens: number;
  readonly reasoningTokens: number;
  readonly recordedGenerations: number;
  readonly unrecordedGenerations: number;
};

export const DEFAULT_TOKEN_PRICING_CNY = {
  promptPerMillion: 6,
  completionPerMillion: 30,
} as const;

export function calculateTokenCostCny(input: {
  readonly promptTokens: number;
  readonly reasoningTokens: number;
  readonly completionTokens: number;
  readonly promptPerMillion: number;
  readonly completionPerMillion: number;
}): number {
  return (
    ((nonNegativeNumber(input.promptTokens) +
      nonNegativeNumber(input.reasoningTokens)) *
      nonNegativeNumber(input.promptPerMillion) +
      nonNegativeNumber(input.completionTokens) *
        nonNegativeNumber(input.completionPerMillion)) /
    1_000_000
  );
}

export function summarizeGenerationTokenUsage(
  generations: readonly GenerationRecord[],
): TokenUsageSummary {
  return generations.reduce<TokenUsageSummary>(
    (summary, generation) => {
      const usage = generation.tokenUsage;
      if (!usage) {
        return {
          ...summary,
          unrecordedGenerations: summary.unrecordedGenerations + 1,
        };
      }

      return {
        promptTokens: summary.promptTokens + tokenCount(usage.promptTokens),
        completionTokens:
          summary.completionTokens + tokenCount(usage.completionTokens),
        totalTokens: summary.totalTokens + totalTokenCount(usage),
        cachedPromptTokens:
          summary.cachedPromptTokens + tokenCount(usage.cachedPromptTokens),
        reasoningTokens:
          summary.reasoningTokens + tokenCount(usage.reasoningTokens),
        recordedGenerations: summary.recordedGenerations + 1,
        unrecordedGenerations: summary.unrecordedGenerations,
      };
    },
    {
      promptTokens: 0,
      completionTokens: 0,
      totalTokens: 0,
      cachedPromptTokens: 0,
      reasoningTokens: 0,
      recordedGenerations: 0,
      unrecordedGenerations: 0,
    },
  );
}

export function tokenCount(value: number | null | undefined): number {
  return typeof value === "number" && Number.isFinite(value) ? value : 0;
}

function nonNegativeNumber(value: number): number {
  return Number.isFinite(value) && value >= 0 ? value : 0;
}

function totalTokenCount(usage: LlmTokenUsage): number {
  if (typeof usage.totalTokens === "number" && Number.isFinite(usage.totalTokens)) {
    return usage.totalTokens;
  }

  return tokenCount(usage.promptTokens) + tokenCount(usage.completionTokens);
}
