import { describe, expect, it } from "vitest";
import {
  calculateTokenCostCny,
  DEFAULT_TOKEN_PRICING_CNY,
} from "../token-usage";

describe("token usage pricing", () => {
  it("uses the default input and output prices per million tokens", () => {
    expect(DEFAULT_TOKEN_PRICING_CNY).toEqual({
      promptPerMillion: 3,
      completionPerMillion: 15,
    });
    expect(
      calculateTokenCostCny({
        promptTokens: 1_000_000,
        reasoningTokens: 0,
        completionTokens: 1_000_000,
        promptPerMillion: DEFAULT_TOKEN_PRICING_CNY.promptPerMillion,
        completionPerMillion: DEFAULT_TOKEN_PRICING_CNY.completionPerMillion,
      }),
    ).toBe(18);
  });

  it("prices reasoning tokens as input tokens", () => {
    expect(
      calculateTokenCostCny({
        promptTokens: 250_000,
        reasoningTokens: 250_000,
        completionTokens: 100_000,
        promptPerMillion: 8,
        completionPerMillion: 20,
      }),
    ).toBe(6);
  });

  it("treats invalid or negative values as zero", () => {
    expect(
      calculateTokenCostCny({
        promptTokens: -1,
        reasoningTokens: -2,
        completionTokens: Number.NaN,
        promptPerMillion: 6,
        completionPerMillion: 30,
      }),
    ).toBe(0);
  });
});
