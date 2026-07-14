import { describe, expect, it } from "vitest";
import {
  LocalHeuristicLlmClient,
  LlmOutputParseError,
  MockLlmClient,
  OpenAICompatibleLlmClient,
  parseLlmJsonObject,
  type LlmGenerateJsonRequest,
} from "../llm";

const request = {
  modelBinding: {
    provider: "mock",
    model: "mock-model",
    responseFormat: "json",
  },
  systemPrompt: "You are a test model.",
  messages: [{ role: "user", content: "Return JSON." }],
  schemaName: "test_schema",
} satisfies LlmGenerateJsonRequest;

describe("LLM client boundary", () => {
  it("returns queued JSON from the mock client", async () => {
    const client = new MockLlmClient([{ text: "hello" }, { target: "p1" }]);

    await expect(client.generateJson(request)).resolves.toEqual({
      provider: "mock",
      model: "mock-model",
      rawText: '{"text":"hello"}',
      parsed: { text: "hello" },
      usage: null,
      finishReason: "stop",
    });
    await expect(client.generateJson(request)).resolves.toMatchObject({
      parsed: { target: "p1" },
    });
  });

  it("returns a conceal disclosure for local Prompt v2 special-role speech", async () => {
    const client = new LocalHeuristicLlmClient();

    await expect(
      client.generateJson({
        ...request,
        schemaName: "werewolf_speech_v2",
        messages: [
          {
            role: "user",
            content:
              '只输出 {"disclosure":"conceal 或 claim","text":"...","decisionSummary":"..."}',
          },
        ],
      }),
    ).resolves.toMatchObject({
      parsed: {
        disclosure: "conceal",
        text: expect.any(String),
        decisionSummary: expect.any(String),
      },
    });
  });

  it("parses OpenAI-compatible token usage", async () => {
    const client = new OpenAICompatibleLlmClient({
      baseUrl: "https://llm.example.test/v1",
      apiKey: "secret",
      fetch: async () =>
        new Response(
          JSON.stringify({
            choices: [{ message: { content: '{"text":"from api"}' } }],
            usage: {
              prompt_tokens: 120,
              completion_tokens: 30,
              total_tokens: 150,
              prompt_tokens_details: { cached_tokens: 24 },
              completion_tokens_details: { reasoning_tokens: 8 },
            },
          }),
          { status: 200, headers: { "content-type": "application/json" } },
        ),
    });

    await expect(client.generateJson(request)).resolves.toMatchObject({
      usage: {
        promptTokens: 120,
        completionTokens: 30,
        totalTokens: 150,
        cachedPromptTokens: 24,
        reasoningTokens: 8,
      },
    });
  });

  it("preserves finish reason and usage when assistant JSON is truncated", async () => {
    const client = new OpenAICompatibleLlmClient({
      baseUrl: "https://llm.example.test/v1",
      apiKey: "secret",
      fetch: async () =>
        new Response(
          JSON.stringify({
            choices: [
              {
                finish_reason: "length",
                message: { content: '{"text":"not finished' },
              },
            ],
            usage: {
              prompt_tokens: 100,
              completion_tokens: 4096,
              total_tokens: 4196,
            },
          }),
          { status: 200, headers: { "content-type": "application/json" } },
        ),
    });

    const error = await client.generateJson(request).catch((caught) => caught);

    expect(error).toBeInstanceOf(LlmOutputParseError);
    expect(error).toMatchObject({
      rawText: '{"text":"not finished',
      finishReason: "length",
      usage: {
        promptTokens: 100,
        completionTokens: 4096,
        totalTokens: 4196,
      },
    });
  });

  it("parses plain and fenced JSON objects", () => {
    expect(parseLlmJsonObject('{"text":"hello"}')).toEqual({ text: "hello" });
    expect(parseLlmJsonObject('```json\n{"text":"hello"}\n```')).toEqual({
      text: "hello",
    });
  });

  it("rejects non-object JSON", () => {
    expect(() => parseLlmJsonObject('"hello"')).toThrow(
      "LLM output must be a JSON object",
    );
    expect(() => parseLlmJsonObject("[1,2,3]")).toThrow(
      "LLM output must be a JSON object",
    );
  });

  it("sends OpenAI-compatible chat completion payloads through injected fetch", async () => {
    const calls: unknown[] = [];
    const client = new OpenAICompatibleLlmClient({
      baseUrl: "https://llm.example.test/v1",
      apiKey: "secret",
      fetch: async (url, init) => {
        calls.push({ url, init });
        return new Response(
          JSON.stringify({
            choices: [{ message: { content: '{"text":"from api"}' } }],
          }),
          { status: 200, headers: { "content-type": "application/json" } },
        );
      },
    });

    await expect(client.generateJson(request)).resolves.toMatchObject({
      provider: "mock",
      model: "mock-model",
      parsed: { text: "from api" },
    });
    expect(calls).toHaveLength(1);
    expect(calls[0]).toMatchObject({
      url: "https://llm.example.test/v1/chat/completions",
      init: {
        method: "POST",
        headers: {
          authorization: "Bearer secret",
          "content-type": "application/json",
        },
      },
    });
    expect(JSON.parse(String((calls[0] as { init: RequestInit }).init.body))).toEqual({
      model: "mock-model",
      response_format: { type: "json_object" },
      messages: [
        { role: "system", content: "You are a test model." },
        { role: "user", content: "Return JSON." },
      ],
    });
  });

  it("includes the endpoint and transport cause when fetch fails", async () => {
    const client = new OpenAICompatibleLlmClient({
      baseUrl: "https://llm.example.test/v1",
      apiKey: "secret",
      fetch: async () => {
        throw new TypeError("fetch failed", {
          cause: new Error("connect ECONNREFUSED 127.0.0.1:11434"),
        });
      },
    });

    await expect(client.generateJson(request)).rejects.toThrow(
      "LLM request failed before response: POST https://llm.example.test/v1/chat/completions: fetch failed; cause: connect ECONNREFUSED 127.0.0.1:11434",
    );
  });
});
