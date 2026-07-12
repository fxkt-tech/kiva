"use client";

import { useState } from "react";
import {
  calculateTokenCostCny,
  DEFAULT_TOKEN_PRICING_CNY,
} from "@/core/token-usage";

export function TokenCostCalculator({
  promptTokens,
  reasoningTokens,
  completionTokens,
}: {
  readonly promptTokens: number;
  readonly reasoningTokens: number;
  readonly completionTokens: number;
}) {
  const [promptPrice, setPromptPrice] = useState(
    String(DEFAULT_TOKEN_PRICING_CNY.promptPerMillion),
  );
  const [completionPrice, setCompletionPrice] = useState(
    String(DEFAULT_TOKEN_PRICING_CNY.completionPerMillion),
  );
  const cost = calculateTokenCostCny({
    promptTokens,
    reasoningTokens,
    completionTokens,
    promptPerMillion: numericPrice(promptPrice),
    completionPerMillion: numericPrice(completionPrice),
  });

  return (
    <div className="mt-3 rounded-md border border-interactive-border bg-background/65 p-3">
      <div className="grid gap-3 sm:grid-cols-2">
        <PriceInput
          label="输入价格"
          value={promptPrice}
          onChange={setPromptPrice}
        />
        <PriceInput
          label="输出价格"
          value={completionPrice}
          onChange={setCompletionPrice}
        />
      </div>
      <div className="mt-3 flex items-end justify-between gap-4 border-t border-border pt-3">
        <div>
          <p className="text-[11px] text-subtle">
            输入按 Prompt + Reasoning，输出按 Completion
          </p>
          <p className="mt-0.5 text-xs text-muted">单位：元 / 百万 tokens</p>
        </div>
        <div className="text-right">
          <p className="text-[11px] uppercase tracking-[0.12em] text-subtle">
            预估价格
          </p>
          <output className="font-mono text-lg font-semibold tabular-nums text-foreground">
            ¥{formatCost(cost)}
          </output>
        </div>
      </div>
    </div>
  );
}

function PriceInput({
  label,
  value,
  onChange,
}: {
  readonly label: string;
  readonly value: string;
  readonly onChange: (value: string) => void;
}) {
  return (
    <label className="text-xs text-muted">
      <span className="font-medium text-subtle">{label}</span>
      <div className="mt-1 flex items-center overflow-hidden rounded border border-interactive-border bg-background">
        <span className="border-r border-border px-2 text-subtle">¥</span>
        <input
          aria-label={`${label}（元/百万 tokens）`}
          className="min-w-0 flex-1 bg-transparent px-2 py-1.5 font-mono text-sm tabular-nums text-foreground outline-none"
          min="0"
          step="0.01"
          type="number"
          value={value}
          onChange={(event) => onChange(event.target.value)}
        />
      </div>
    </label>
  );
}

function numericPrice(value: string): number {
  const parsed = Number(value);
  return Number.isFinite(parsed) && parsed >= 0 ? parsed : 0;
}

function formatCost(value: number): string {
  return new Intl.NumberFormat("zh-CN", {
    minimumFractionDigits: value < 0.01 ? 6 : 4,
    maximumFractionDigits: value < 0.01 ? 6 : 4,
  }).format(value);
}
