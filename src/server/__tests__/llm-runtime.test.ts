import { describe, expect, it } from "vitest";
import { LocalHeuristicLlmClient, OpenAICompatibleLlmClient } from "@/core/llm";
import { createRuntimeLlmClient } from "../llm-runtime";

describe("LLM runtime factory", () => {
  it("uses a local heuristic client when OpenAI-compatible env is absent", async () => {
    const client = createRuntimeLlmClient({ env: {} });

    expect(client).toBeInstanceOf(LocalHeuristicLlmClient);
    await expect(
      client?.generateJson({
        modelBinding: {
          provider: "mock",
          model: "mock-model",
          responseFormat: "json",
        },
        systemPrompt: "test",
        messages: [{ role: "user", content: "请把 PlayerIntent 写成自然台词" }],
        schemaName: "werewolf_speech_performance_v1",
      }),
    ).resolves.toMatchObject({
      parsed: { text: expect.stringContaining("我先") },
    });
  });

  it("selects a legal action target without relying on internal draft enums", async () => {
    const client = createRuntimeLlmClient({ env: {} });

    await expect(
      client?.generateJson({
        modelBinding: {
          provider: "mock",
          model: "mock-model",
          responseFormat: "json",
        },
        systemPrompt: "test",
        messages: [
          {
            role: "user",
            content: [
              "【合法候选】",
              "- p8 | 8 号 | 陈墨",
              "【输出】",
              '只输出 {"targetPlayerId":"合法候选中的 playerId","decisionSummary":"简短决策摘要"}。',
            ].join("\n"),
          },
        ],
        schemaName: "werewolf_target_action_v3",
      }),
    ).resolves.toMatchObject({
      parsed: {
        targetPlayerId: "p8",
        decisionSummary: expect.any(String),
      },
    });
  });

  it("creates OpenAI-compatible client when base URL and API key are present", () => {
    const client = createRuntimeLlmClient({
      env: {
        OPENAI_COMPATIBLE_BASE_URL: "https://api.example.test/v1",
        OPENAI_COMPATIBLE_API_KEY: "secret",
      },
      fetch: async () => new Response("{}"),
    });

    expect(client).toBeInstanceOf(OpenAICompatibleLlmClient);
  });

});
