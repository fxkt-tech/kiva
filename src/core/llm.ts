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
  readonly finishReason: string | null;
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
  readonly finishReason: string | null;
  readonly usage: LlmTokenUsage | null;

  constructor(
    message: string,
    rawText: string,
    options: ErrorOptions & {
      readonly finishReason?: string | null;
      readonly usage?: LlmTokenUsage | null;
    } = {},
  ) {
    super(message, options);
    this.name = "LlmOutputParseError";
    this.rawText = rawText;
    this.finishReason = options.finishReason ?? null;
    this.usage = options.usage ?? null;
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
      finishReason: "stop",
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
      finishReason: "stop",
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
    const finishReason = optionalString(body.choices?.[0]?.finish_reason);
    const usage = parseOpenAICompatibleUsage(body.usage);
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
        { cause: error, finishReason, usage },
      );
    }

    return {
      provider: request.modelBinding.provider,
      model: request.modelBinding.model,
      rawText,
      parsed,
      usage,
      finishReason,
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
    readonly finish_reason?: unknown;
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

function optionalString(value: unknown): string | null {
  return typeof value === "string" && value.trim().length > 0
    ? value.trim()
    : null;
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

  const episodeOutline = {
    title: "未明档案：被改写的终局",
    logline: "众人在一份持续被改写的档案中追索矛盾，最终让每次选择都成为结局的证词。",
    acts: [
      { title: "雨夜开卷", summary: "身份与第一处异常被同时封入档案。" },
      { title: "证词交锋", summary: "公开判断和暗中行动不断改写彼此的可信度。" },
      { title: "终局归档", summary: "幸存者用最后的选择确认档案的真实版本。" },
    ],
  };

  if (request.schemaName === "werewolf_episode_story_v3") {
    return episodeOutline;
  }

  if (request.schemaName === "werewolf_episode_ensemble_v3") {
    const profiles = prefixedJsonObjects(userContent, "CHARACTER_PROFILE");
    const relationshipKinds = [
      "rivalry",
      "alliance",
      "contrast",
      "trust_shift",
    ] as const;
    const castAssignments = profiles.map((profile, index) => {
      const playerId = localString(profile.playerId, `p${index + 1}`);
      const name = localString(profile.name, playerId);
      const persona = localCharacterTrait(profile);
      const opportunities = Array.isArray(profile.performanceStepIndexes)
        ? profile.performanceStepIndexes.filter(
            (value): value is number => Number.isInteger(value),
          )
        : [];
      const signatureStep =
        opportunities[index % Math.max(1, opportunities.length)] ?? 1;
      return {
        playerId,
        dramaticWeight: index < Math.max(2, Math.ceil(profiles.length / 3))
          ? "primary"
          : "supporting",
        dramaticFunction: `${name}以${persona}成为第${index + 1}条判断线的推动者`,
        signatureStepIndex: signatureStep,
      };
    });
    const relationshipSeeds = profiles.flatMap((profile, index) => {
      if (index % 2 !== 0 || index + 1 >= profiles.length) return [];
      const leftId = localString(profile.playerId, `p${index + 1}`);
      const rightId = localString(
        profiles[index + 1]?.playerId,
        `p${index + 2}`,
      );
      const kind = relationshipKinds[(index / 2) % relationshipKinds.length]!;
      return [
        {
          playerIds: [leftId, rightId],
          kind,
        },
      ];
    });
    return { castAssignments, relationshipSeeds };
  }

  if (request.schemaName === "werewolf_episode_character_v3") {
    const profile = prefixedJsonObjects(userContent, "CHARACTER_PROFILE")[0] ?? {};
    const assignment = prefixedJsonObjects(userContent, "CAST_ASSIGNMENT")[0] ?? {};
    const playerId = localString(assignment.playerId, localString(profile.playerId, "unknown"));
    const name = localString(profile.name, playerId);
    const persona = localCharacterTrait(profile);
    const signatureStepIndex = Number.isInteger(assignment.signatureStepIndex)
      ? Number(assignment.signatureStepIndex)
      : 1;
    return {
      playerId,
      baseline: `${name}先按${persona}观察局面并建立自己的判断标准`,
      pressure: "公开冲突迫使其在坚持原有方法与承认盲点之间作出选择",
      change: "保留核心判断方式，同时学会把不确定性转化为可验证的下一步",
      payoff: "在关键节点用一次清晰选择兑现此前建立的判断标准",
      signatureDescription: `${name}在真实可用的第 ${signatureStepIndex} 步让个人判断方式成为局面转折点`,
    };
  }

  if (request.schemaName === "werewolf_episode_relationship_v3") {
    const seed = prefixedJsonObjects(userContent, "RELATIONSHIP_SEED")[0] ?? {};
    return {
      playerIds: Array.isArray(seed.playerIds) ? seed.playerIds : [],
      setup: "两人的判断方法在一次公开选择中首次形成可见差异",
      development: "后续公开发言让差异升级为互相检验或有限协作",
      payoff: "最终选择回应此前累积的分歧，并完成一次可信的关系变化",
    };
  }

  if (request.schemaName === "werewolf_episode_beats_v3") {
    const actorContexts = new Map(
      prefixedJsonObjects(userContent, "ACTOR_CONTEXT").map((context) => {
        const profile = localObject(context.profile);
        return [localString(profile?.playerId, "unknown"), context] as const;
      }),
    );
    const speechSteps = [...userContent.matchAll(
      /SPEECH_STEP\s+(\d+)[^\n]*?\|\s*actor=([^\s|]+)/g,
    )].map((match) => ({
      stepIndex: Number(match[1]),
      playerId: match[2]!,
    }));
    return {
      beats: speechSteps.map(({ stepIndex, playerId }) => {
        const context = actorContexts.get(playerId);
        const profile = localObject(context?.profile);
        const direction = localObject(context?.direction);
        const relationships = Array.isArray(context?.relationships)
          ? context.relationships
          : [];
        const name = localString(profile?.name, playerId);
        const trait = profile
          ? localCharacterTrait(profile)
          : "自己的稳定判断方式";
        const relationship = localObject(relationships[0]);
        return {
          stepIndex,
          objective: "让当前立场推动一条可由后续公开事件验证的冲突线。",
          stance: "明确选择一项当前判断，并指出它与前序证词的差异。",
          disclosure: "conceal",
          themeHook: "把本场选择写成对档案版本的争夺，并由后续公开结果完成验证。",
          characterHook: `${name}以${trait}组织本场表达，不机械重复口头禅`,
          arcMove: localString(
            direction?.change,
            "在压力下保留人物核心，同时显露一个可继续发展的次要侧面",
          ),
          relationshipMove: relationship
            ? `只在当前公开互动中推进与${localRelationshipPartner(relationship, playerId)}的${localString(relationship.kind, "关系变化")}`
            : null,
        };
      }),
    };
  }

  if (request.schemaName === "werewolf_speech_v2") {
    return {
      ...(userContent.includes('"disclosure"')
        ? { disclosure: "conceal" }
        : {}),
      text: "我先根据目前能看到的信息给出自己的判断。",
      decisionSummary: "当前信息有限，先给出可继续验证的方向。",
    };
  }

  if (
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

  return {};
}

function prefixedJsonObjects(
  content: string,
  prefix: string,
): readonly Record<string, unknown>[] {
  return content
    .split("\n")
    .filter((line) => line.startsWith(`${prefix} `))
    .flatMap((line) => {
      try {
        const parsed = JSON.parse(line.slice(prefix.length + 1)) as unknown;
        return parsed && typeof parsed === "object" && !Array.isArray(parsed)
          ? [parsed as Record<string, unknown>]
          : [];
      } catch {
        return [];
      }
    });
}

function localObject(value: unknown): Record<string, unknown> | null {
  return value && typeof value === "object" && !Array.isArray(value)
    ? (value as Record<string, unknown>)
    : null;
}

function localString(value: unknown, fallback: string): string {
  return typeof value === "string" && value.trim().length > 0
    ? value.trim()
    : fallback;
}

function localCharacterTrait(profile: Record<string, unknown>): string {
  return localString(
    profile.persona,
    localString(
      profile.reasoningStyle,
      localString(profile.speakingStyle, "自然克制的真人表达"),
    ),
  );
}

function localRelationshipPartner(
  relationship: Record<string, unknown>,
  playerId: string,
): string {
  const playerIds = Array.isArray(relationship.playerIds)
    ? relationship.playerIds.filter(
        (value): value is string => typeof value === "string",
      )
    : [];
  return playerIds.find((candidate) => candidate !== playerId) ?? "另一位玩家";
}
