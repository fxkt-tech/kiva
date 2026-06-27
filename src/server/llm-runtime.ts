import { OpenAICompatibleLlmClient, type LlmClient } from "@/core/llm";

type RuntimeEnv = Partial<
  Pick<
    NodeJS.ProcessEnv,
    "OPENAI_COMPATIBLE_BASE_URL" | "OPENAI_COMPATIBLE_API_KEY"
  >
>;

export type CreateRuntimeLlmClientInput = {
  readonly env?: RuntimeEnv;
  readonly fetch?: typeof fetch;
};

export function createRuntimeLlmClient(
  input: CreateRuntimeLlmClientInput = {},
): LlmClient | undefined {
  const env = input.env ?? process.env;
  const baseUrl = env.OPENAI_COMPATIBLE_BASE_URL?.trim();
  const apiKey = env.OPENAI_COMPATIBLE_API_KEY?.trim();

  if (!baseUrl || !apiKey) {
    return undefined;
  }

  return new OpenAICompatibleLlmClient({
    baseUrl,
    apiKey,
    fetch: input.fetch,
  });
}
