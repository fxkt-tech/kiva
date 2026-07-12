import type { ModelBindingSnapshot } from "./player";

export type LlmMessage = {
  readonly role: "system" | "user" | "assistant";
  readonly content: string;
};

export type LlmGenerateJsonRequest = {
  readonly modelBinding: ModelBindingSnapshot;
  readonly systemPrompt: string;
  readonly messages: readonly LlmMessage[];
  readonly schemaName: string;
};

export type LlmGenerateJsonResult = {
  readonly provider: string;
  readonly model: string;
  readonly rawText: string;
  readonly parsed: Record<string, unknown>;
  readonly usage: LlmTokenUsage | null;
};

export type LlmTokenUsage = {
  readonly promptTokens: number | null;
  readonly completionTokens: number | null;
  readonly totalTokens: number | null;
  readonly cachedPromptTokens?: number | null;
  readonly reasoningTokens?: number | null;
};

export type LlmClient = {
  readonly generateJson: (
    request: LlmGenerateJsonRequest,
  ) => Promise<LlmGenerateJsonResult>;
};

export class LlmOutputParseError extends Error {
  readonly rawText: string;

  constructor(message: string, rawText: string, options?: ErrorOptions) {
    super(message, options);
    this.name = "LlmOutputParseError";
    this.rawText = rawText;
  }
}

export class MockLlmClient implements LlmClient {
  private readonly queuedOutputs: Record<string, unknown>[];
  private nextIndex = 0;

  constructor(outputs: readonly Record<string, unknown>[] = []) {
    this.queuedOutputs = [...outputs];
  }

  async generateJson(
    request: LlmGenerateJsonRequest,
  ): Promise<LlmGenerateJsonResult> {
    const parsed = this.queuedOutputs[this.nextIndex] ?? {};
    this.nextIndex += 1;

    return {
      provider: request.modelBinding.provider,
      model: request.modelBinding.model,
      rawText: JSON.stringify(parsed),
      parsed,
      usage: null,
    };
  }
}

export class LocalHeuristicLlmClient implements LlmClient {
  async generateJson(
    request: LlmGenerateJsonRequest,
  ): Promise<LlmGenerateJsonResult> {
    const parsed = localOutputForRequest(request);

    return {
      provider: request.modelBinding.provider,
      model: request.modelBinding.model,
      rawText: JSON.stringify(parsed),
      parsed,
      usage: null,
    };
  }
}

export type OpenAICompatibleLlmClientInput = {
  readonly baseUrl: string;
  readonly apiKey: string;
  readonly fetch?: typeof fetch;
};

export class OpenAICompatibleLlmClient implements LlmClient {
  private readonly baseUrl: string;
  private readonly apiKey: string;
  private readonly fetchImpl: typeof fetch;

  constructor(input: OpenAICompatibleLlmClientInput) {
    this.baseUrl = input.baseUrl.replace(/\/+$/, "");
    this.apiKey = input.apiKey;
    this.fetchImpl = input.fetch ?? fetch;
  }

  async generateJson(
    request: LlmGenerateJsonRequest,
  ): Promise<LlmGenerateJsonResult> {
    const endpoint = `${this.baseUrl}/chat/completions`;
    const response = await fetchOpenAICompatibleJson(
      this.fetchImpl,
      endpoint,
      {
        method: "POST",
        headers: {
          authorization: `Bearer ${this.apiKey}`,
          "content-type": "application/json",
        },
        body: JSON.stringify({
          model: request.modelBinding.model,
          response_format: { type: "json_object" },
          messages: [
            { role: "system", content: request.systemPrompt },
            ...request.messages,
          ],
        }),
      },
    );

    const responseText = await response.text();
    if (!response.ok) {
      throw new Error(
        `LLM request failed: POST ${endpoint}: ${response.status} ${response.statusText}${responseText ? `: ${truncate(responseText, 500)}` : ""}`,
      );
    }

    const body = parseOpenAICompatibleResponse(responseText, endpoint);
    const rawText = body.choices?.[0]?.message?.content;
    if (typeof rawText !== "string") {
      throw new Error("LLM response did not include message content");
    }

    let parsed: Record<string, unknown>;
    try {
      parsed = parseLlmJsonObject(rawText);
    } catch (error) {
      throw new LlmOutputParseError(
        `LLM message content was not a valid JSON object: ${errorWithCauseMessage(error)}`,
        rawText,
        { cause: error },
      );
    }

    return {
      provider: request.modelBinding.provider,
      model: request.modelBinding.model,
      rawText,
      parsed,
      usage: parseOpenAICompatibleUsage(body.usage),
    };
  }
}

async function fetchOpenAICompatibleJson(
  fetchImpl: typeof fetch,
  endpoint: string,
  init: RequestInit,
): Promise<Response> {
  try {
    return await fetchImpl(endpoint, init);
  } catch (error) {
    throw new Error(
      `LLM request failed before response: ${init.method ?? "GET"} ${endpoint}: ${errorWithCauseMessage(error)}`,
    );
  }
}

function parseOpenAICompatibleResponse(
  responseText: string,
  endpoint: string,
): OpenAICompatibleResponse {
  try {
    return JSON.parse(responseText) as OpenAICompatibleResponse;
  } catch (error) {
    throw new Error(
      `LLM response was not valid JSON: POST ${endpoint}: ${errorWithCauseMessage(error)}${responseText ? `: ${truncate(responseText, 500)}` : ""}`,
    );
  }
}

function errorWithCauseMessage(error: unknown): string {
  if (!(error instanceof Error)) {
    return String(error);
  }

  const cause = error.cause;
  if (cause === undefined) {
    return error.message;
  }

  return `${error.message}; cause: ${cause instanceof Error ? cause.message : String(cause)}`;
}

function truncate(value: string, maxLength: number): string {
  return value.length <= maxLength ? value : `${value.slice(0, maxLength)}...`;
}

type OpenAICompatibleResponse = {
  readonly choices?: readonly {
    readonly message?: {
      readonly content?: unknown;
    };
  }[];
  readonly usage?: unknown;
};

function parseOpenAICompatibleUsage(usage: unknown): LlmTokenUsage | null {
  if (usage === null || typeof usage !== "object") {
    return null;
  }

  const record = usage as Record<string, unknown>;
  const promptDetails = objectRecord(record.prompt_tokens_details);
  const completionDetails = objectRecord(record.completion_tokens_details);

  return {
    promptTokens: optionalTokenCount(record.prompt_tokens),
    completionTokens: optionalTokenCount(record.completion_tokens),
    totalTokens: optionalTokenCount(record.total_tokens),
    cachedPromptTokens: optionalTokenCount(promptDetails?.cached_tokens),
    reasoningTokens: optionalTokenCount(completionDetails?.reasoning_tokens),
  };
}

function objectRecord(value: unknown): Record<string, unknown> | null {
  return value !== null && typeof value === "object"
    ? (value as Record<string, unknown>)
    : null;
}

function optionalTokenCount(value: unknown): number | null {
  return Number.isFinite(value) ? Number(value) : null;
}

export function parseLlmJsonObject(rawText: string): Record<string, unknown> {
  const parsed = JSON.parse(stripJsonFence(rawText));

  if (
    parsed === null ||
    typeof parsed !== "object" ||
    Array.isArray(parsed)
  ) {
    throw new Error("LLM output must be a JSON object");
  }

  return parsed as Record<string, unknown>;
}

function stripJsonFence(rawText: string): string {
  const trimmed = rawText.trim();
  const fenced = /^```(?:json)?\s*([\s\S]*?)\s*```$/i.exec(trimmed);
  return fenced ? fenced[1].trim() : trimmed;
}

function localOutputForRequest(
  request: LlmGenerateJsonRequest,
): Record<string, unknown> {
  const userContent = request.messages
    .filter((message) => message.role === "user")
    .map((message) => message.content)
    .join("\n");

  if (
    request.schemaName === "werewolf_episode_narrative_v1" ||
    request.schemaName === "werewolf_episode_outline_v1" ||
    request.schemaName === "werewolf_episode_beats_v1"
  ) {
    const stepIndexes = [...userContent.matchAll(/SPEECH_STEP\s+(\d+)/g)].map(
      (match) => Number(match[1]),
    );
    const outline = {
      title: "未明档案：被改写的终局",
      logline: "众人在一份持续被改写的档案中追索矛盾，最终让每次选择都成为结局的证词。",
      acts: [
        { title: "雨夜开卷", summary: "身份与第一处异常被同时封入档案。" },
        { title: "证词交锋", summary: "公开判断和暗中行动不断改写彼此的可信度。" },
        { title: "终局归档", summary: "幸存者用最后的选择确认档案的真实版本。" },
      ],
    };
    const beats = stepIndexes.map((stepIndex) => ({
      stepIndex,
      objective: "让当前立场推动一条可在后续事件中验证的冲突线。",
      stance: "明确选择一项当前判断，并指出它与前序证词的矛盾。",
      disclosure: "conceal",
      themeHook: "把本场选择写成对档案版本的争夺，并让后续投票或行动完成验证。",
    }));
    if (request.schemaName === "werewolf_episode_outline_v1") return outline;
    if (request.schemaName === "werewolf_episode_beats_v1") return { beats };
    return {
      ...outline,
      beats,
    };
  }

  if (
    request.schemaName === "werewolf_speech_v1" ||
    request.schemaName === "werewolf_speech_v2"
  ) {
    return {
      ...(request.schemaName === "werewolf_speech_v2" &&
      userContent.includes('"disclosure"')
        ? { disclosure: "conceal" }
        : {}),
      text: "我先根据目前能看到的信息给出自己的判断。",
      decisionSummary: "当前信息有限，先给出可继续验证的方向。",
    };
  }

  if (
    request.schemaName === "werewolf_action_v1" ||
    request.schemaName === "werewolf_target_action_v2" ||
    request.schemaName === "werewolf_optional_action_v2"
  ) {
    if (request.schemaName === "werewolf_optional_action_v2") {
      return {
        used: false,
        targetPlayerId: null,
        decisionSummary: "当前不消耗一次性资源。",
      };
    }

    if (request.schemaName === "werewolf_target_action_v2") {
      if (userContent.includes("弃票时输出")) {
        return {
          targetPlayerId: null,
          decisionSummary: "当前选择合法弃票。",
        };
      }

      const targetPlayerId = /【合法候选】[\s\S]*?-\s+([^\s|]+)\s+\|/.exec(
        userContent,
      )?.[1];
      return {
        targetPlayerId: targetPlayerId ?? null,
        decisionSummary: targetPlayerId
          ? "从本次合法候选中选择一个目标。"
          : "当前没有合法候选。",
      };
    }

    if (
      userContent.includes("witch_antidote_decided") ||
      userContent.includes("witch_poison_decided")
    ) {
      return { used: false, targetPlayerId: null };
    }

    if (userContent.includes("vote_cast")) {
      return { targetPlayerId: null };
    }

    const targetPlayerId = /可选目标：[\s\S]*?playerId=([^\s]+)/.exec(
      userContent,
    )?.[1];
    return { targetPlayerId: targetPlayerId ?? null };
  }

  return {};
}
