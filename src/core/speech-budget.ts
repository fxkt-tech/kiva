import type { DraftEvent } from "./drafts";

export const SPEECH_BUDGET_KEYS = [
  "wolf_strategy",
  "wolf_opinion",
  "day_first",
  "day_response",
  "last_words",
  "pk",
] as const;

export type SpeechBudgetKey = (typeof SPEECH_BUDGET_KEYS)[number];
export type SpeechBudgetTier = "normal" | "compressed" | "critical";

export type SpeechBudget = {
  readonly key: SpeechBudgetKey;
  readonly tier: SpeechBudgetTier;
  readonly targetMinCharacters: number;
  readonly targetMaxCharacters: number;
  readonly hardMaxCharacters: number;
};

export type SpeechEvaluation = {
  readonly characterCount: number;
  readonly estimatedDurationMs: number;
  readonly status: "short" | "target" | "long" | "over_limit";
  readonly withinHardLimit: boolean;
};

type BudgetedSpeechDraft = Extract<
  DraftEvent,
  {
    type:
      | "wolf_strategy_given"
      | "wolf_opinion_given"
      | "day_speech_given"
      | "last_words_given"
      | "pk_speech_given";
  }
>;

type BaseSpeechBudget = Omit<SpeechBudget, "key" | "tier">;

const ESTIMATED_CHARACTERS_PER_SECOND = 6.1;

const BASE_BUDGETS: Readonly<Record<SpeechBudgetKey, BaseSpeechBudget>> = {
  wolf_strategy: {
    targetMinCharacters: 110,
    targetMaxCharacters: 150,
    hardMaxCharacters: 180,
  },
  wolf_opinion: {
    targetMinCharacters: 70,
    targetMaxCharacters: 100,
    hardMaxCharacters: 120,
  },
  day_first: {
    targetMinCharacters: 100,
    targetMaxCharacters: 140,
    hardMaxCharacters: 170,
  },
  day_response: {
    targetMinCharacters: 140,
    targetMaxCharacters: 180,
    hardMaxCharacters: 220,
  },
  last_words: {
    targetMinCharacters: 160,
    targetMaxCharacters: 200,
    hardMaxCharacters: 240,
  },
  pk: {
    targetMinCharacters: 140,
    targetMaxCharacters: 180,
    hardMaxCharacters: 220,
  },
};

const TIER_MULTIPLIERS: Readonly<
  Record<SpeechBudgetTier, { readonly target: number; readonly hardMax: number }>
> = {
  normal: { target: 1, hardMax: 1 },
  compressed: { target: 0.85, hardMax: 0.9 },
  critical: { target: 0.7, hardMax: 0.8 },
};

export function speechBudgetForDraft(input: {
  readonly draft: BudgetedSpeechDraft;
  readonly hasPriorDaySpeech?: boolean;
  readonly tier?: SpeechBudgetTier;
}): SpeechBudget {
  return speechBudgetForKey(
    budgetKeyForDraft(input.draft, input.hasPriorDaySpeech ?? false),
    input.tier,
  );
}

export function speechBudgetForKey(
  key: SpeechBudgetKey,
  tier: SpeechBudgetTier = "normal",
): SpeechBudget {
  const base = BASE_BUDGETS[key];
  const multiplier = TIER_MULTIPLIERS[tier];
  const targetMinCharacters = scaled(base.targetMinCharacters, multiplier.target);
  const targetMaxCharacters = Math.max(
    targetMinCharacters,
    scaled(base.targetMaxCharacters, multiplier.target),
  );

  return {
    key,
    tier,
    targetMinCharacters,
    targetMaxCharacters,
    hardMaxCharacters: Math.max(
      targetMaxCharacters,
      scaled(base.hardMaxCharacters, multiplier.hardMax),
    ),
  };
}

export function spokenCharacterCount(text: string): number {
  return Array.from(text).filter((character) => !/\s/u.test(character)).length;
}

export function estimateSpeechDurationMs(characterCount: number): number {
  if (!Number.isFinite(characterCount) || characterCount <= 0) return 0;
  return Math.ceil((characterCount / ESTIMATED_CHARACTERS_PER_SECOND) * 1_000);
}

export function evaluateSpeech(
  text: string,
  budget: SpeechBudget,
): SpeechEvaluation {
  const characterCount = spokenCharacterCount(text);
  const withinHardLimit = characterCount <= budget.hardMaxCharacters;
  const status =
    !withinHardLimit
      ? "over_limit"
      : characterCount < budget.targetMinCharacters
        ? "short"
        : characterCount <= budget.targetMaxCharacters
          ? "target"
          : "long";

  return {
    characterCount,
    estimatedDurationMs: estimateSpeechDurationMs(characterCount),
    status,
    withinHardLimit,
  };
}

function budgetKeyForDraft(
  draft: BudgetedSpeechDraft,
  hasPriorDaySpeech: boolean,
): SpeechBudgetKey {
  switch (draft.type) {
    case "wolf_strategy_given":
      return "wolf_strategy";
    case "wolf_opinion_given":
      return "wolf_opinion";
    case "day_speech_given":
      return hasPriorDaySpeech ? "day_response" : "day_first";
    case "last_words_given":
      return "last_words";
    case "pk_speech_given":
      return "pk";
  }
}

function scaled(value: number, multiplier: number): number {
  return Math.max(1, Math.floor(value * multiplier));
}
