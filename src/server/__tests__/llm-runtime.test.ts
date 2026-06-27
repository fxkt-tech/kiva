import { describe, expect, it } from "vitest";
import { OpenAICompatibleLlmClient } from "@/core/llm";
import { createRuntimeLlmClient } from "../llm-runtime";

describe("LLM runtime factory", () => {
  it("returns undefined when OpenAI-compatible env is absent", () => {
    expect(createRuntimeLlmClient({ env: {} })).toBeUndefined();
    expect(
      createRuntimeLlmClient({
        env: { OPENAI_COMPATIBLE_BASE_URL: "https://api.example.test/v1" },
      }),
    ).toBeUndefined();
    expect(
      createRuntimeLlmClient({
        env: { OPENAI_COMPATIBLE_API_KEY: "secret" },
      }),
    ).toBeUndefined();
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
