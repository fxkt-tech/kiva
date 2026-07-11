import {
  LocalHeuristicLlmClient,
  OpenAICompatibleLlmClient,
  type LlmClient,
} from "@/core/llm";
import type { LlmPromptMode } from "@/core/prompt-builders";

type RuntimeEnv = Partial<
  Pick<
    NodeJS.ProcessEnv,
    | "OPENAI_COMPATIBLE_BASE_URL"
    | "OPENAI_COMPATIBLE_API_KEY"
    | "KIVA_LLM_PROMPT_VERSION"
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
    return new LocalHeuristicLlmClient();
  }

  return new OpenAICompatibleLlmClient({
    baseUrl,
    apiKey,
    fetch: input.fetch,
  });
}

export function resolveRuntimePromptMode(
  input: Pick<CreateRuntimeLlmClientInput, "env"> = {},
): LlmPromptMode {
  const value = (input.env ?? process.env).KIVA_LLM_PROMPT_VERSION?.trim();
  if (!value || value === "v2") return "v2";
  if (value === "v1") return "v1";
  throw new Error(
    `Unsupported KIVA_LLM_PROMPT_VERSION: ${value}; expected v1 or v2`,
  );
}
