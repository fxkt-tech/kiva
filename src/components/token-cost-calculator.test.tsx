import React from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { TokenCostCalculator } from "./token-cost-calculator";

describe("TokenCostCalculator", () => {
  it("renders editable default prices and the calculated cost", () => {
    const html = renderToStaticMarkup(
      React.createElement(TokenCostCalculator, {
        promptTokens: 1_000_000,
        reasoningTokens: 500_000,
        completionTokens: 1_000_000,
      }),
    );

    expect(html).toContain("输入价格");
    expect(html).toContain('value="6"');
    expect(html).toContain("输出价格");
    expect(html).toContain('value="30"');
    expect(html).toContain("¥39.0000");
  });
});
