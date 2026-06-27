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
          temperature: 0.7,
          maxTokens: 500,
          responseFormat: "json",
        },
        systemPrompt: "test",
        messages: [{ role: "user", content: "draft=day_speech_given" }],
        schemaName: "werewolf_speech_v1",
      }),
    ).resolves.toMatchObject({
      parsed: { text: expect.stringContaining("我先") },
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
