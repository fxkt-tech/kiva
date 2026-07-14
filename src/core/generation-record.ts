import type { ModelBindingSnapshot } from "./player";
import type { LlmTokenUsage } from "./llm";
import type { DraftId, GameId, PlayerId } from "./types";
import { assertExactObjectKeys, isPlainObject } from "./model-binding";
import {
  ACTION_PROMPT_VERSION,
  SPEECH_PROMPT_VERSION,
} from "./prompt-builders";

export type GenerationPurpose = "speech" | "action";
export type GenerationStatus = "success" | "failed";
export type GenerationRequestSnapshot = {
  readonly schemaName: string;
  readonly systemPrompt: string;
  readonly messages: readonly {
    readonly role: "system" | "user" | "assistant";
    readonly content: string;
  }[];
};

export type GenerationAttemptSnapshot = {
  readonly request: GenerationRequestSnapshot;
  readonly tokenUsage: LlmTokenUsage | null;
  readonly rawOutput: string | null;
  readonly parsedOutput: Record<string, unknown> | null;
  readonly error: string | null;
};

export type GenerationRecord = {
  readonly id: string;
  readonly gameId: GameId;
  readonly draftId: DraftId;
  readonly playerId: PlayerId;
  readonly purpose: GenerationPurpose;
  readonly status: GenerationStatus;
  readonly promptVersion: string;
  readonly provider: string;
  readonly model: string;
  readonly inputContextHash: string;
  readonly request: GenerationRequestSnapshot;
  readonly tokenUsage: LlmTokenUsage | null;
  readonly rawOutput: string | null;
  readonly parsedOutput: Record<string, unknown> | null;
  readonly error: string | null;
  readonly createdAt: string;
  readonly attempts?: readonly GenerationAttemptSnapshot[];
};

export type SuccessfulGenerationRecordInput = {
  readonly id: string;
  readonly gameId: GameId;
  readonly draftId: DraftId;
  readonly playerId: PlayerId;
  readonly purpose: GenerationPurpose;
  readonly promptVersion: string;
  readonly modelBinding: ModelBindingSnapshot;
  readonly inputContextHash: string;
  readonly request: GenerationRequestSnapshot;
  readonly tokenUsage?: LlmTokenUsage | null;
  readonly rawOutput: string;
  readonly parsedOutput: Record<string, unknown>;
  readonly createdAt: string;
  readonly attempts?: readonly GenerationAttemptSnapshot[];
};

export type FailedGenerationRecordInput = {
  readonly id: string;
  readonly gameId: GameId;
  readonly draftId: DraftId;
  readonly playerId: PlayerId;
  readonly purpose: GenerationPurpose;
  readonly promptVersion: string;
  readonly modelBinding: ModelBindingSnapshot;
  readonly inputContextHash: string;
  readonly request: GenerationRequestSnapshot;
  readonly tokenUsage?: LlmTokenUsage | null;
  readonly rawOutput: string | null;
  readonly error: unknown;
  readonly createdAt: string;
  readonly attempts?: readonly GenerationAttemptSnapshot[];
};

export function createSuccessfulGenerationRecord(
  input: SuccessfulGenerationRecordInput,
): GenerationRecord {
  return {
    id: input.id,
    gameId: input.gameId,
    draftId: input.draftId,
    playerId: input.playerId,
    purpose: input.purpose,
    status: "success",
    promptVersion: input.promptVersion,
    provider: input.modelBinding.provider,
    model: input.modelBinding.model,
    inputContextHash: input.inputContextHash,
    request: input.request,
    tokenUsage: input.tokenUsage ?? null,
    rawOutput: input.rawOutput,
    parsedOutput: input.parsedOutput,
    error: null,
    createdAt: input.createdAt,
    ...(input.attempts ? { attempts: input.attempts } : {}),
  };
}

export function createFailedGenerationRecord(
  input: FailedGenerationRecordInput,
): GenerationRecord {
  return {
    id: input.id,
    gameId: input.gameId,
    draftId: input.draftId,
    playerId: input.playerId,
    purpose: input.purpose,
    status: "failed",
    promptVersion: input.promptVersion,
    provider: input.modelBinding.provider,
    model: input.modelBinding.model,
    inputContextHash: input.inputContextHash,
    request: input.request,
    tokenUsage: input.tokenUsage ?? null,
    rawOutput: input.rawOutput,
    parsedOutput: null,
    error: errorMessage(input.error),
    createdAt: input.createdAt,
    ...(input.attempts ? { attempts: input.attempts } : {}),
  };
}

export function validateGenerationRecord(
  value: unknown,
  expectedGameId?: GameId,
): GenerationRecord {
  if (!isPlainObject(value)) {
    throw new Error("Generation record must be an object");
  }
  assertExactObjectKeys(
    value,
    "Generation record",
    [
      "id",
      "gameId",
      "draftId",
      "playerId",
      "purpose",
      "status",
      "promptVersion",
      "provider",
      "model",
      "inputContextHash",
      "request",
      "tokenUsage",
      "rawOutput",
      "parsedOutput",
      "error",
      "createdAt",
    ],
    ["attempts"],
  );
  for (const field of [
    "id",
    "gameId",
    "draftId",
    "playerId",
    "provider",
    "model",
    "inputContextHash",
    "createdAt",
  ] as const) {
    if (!isNonBlankString(value[field])) {
      throw new Error(`Generation record ${field} must be set`);
    }
  }
  if (expectedGameId !== undefined && value.gameId !== expectedGameId) {
    throw new Error("Generation record belongs to another game");
  }
  if (value.purpose !== "speech" && value.purpose !== "action") {
    throw new Error("Generation record purpose is invalid");
  }
  const purpose = value.purpose;
  if (value.status !== "success" && value.status !== "failed") {
    throw new Error("Generation record status is invalid");
  }
  const expectedPromptVersion =
    purpose === "speech" ? SPEECH_PROMPT_VERSION : ACTION_PROMPT_VERSION;
  if (value.promptVersion !== expectedPromptVersion) {
    throw new Error(`Unsupported generation prompt version: ${String(value.promptVersion)}`);
  }
  const allowedSchemaNames =
    purpose === "speech"
      ? ["werewolf_speech_v2"]
      : ["werewolf_target_action_v2", "werewolf_optional_action_v2"];
  validateGenerationRequestSnapshot(value.request, allowedSchemaNames);
  if (!isLlmTokenUsage(value.tokenUsage)) {
    throw new Error("Generation record tokenUsage is invalid");
  }
  if (value.rawOutput !== null && typeof value.rawOutput !== "string") {
    throw new Error("Generation record rawOutput is invalid");
  }
  if (
    value.parsedOutput !== null &&
    !isPlainObject(value.parsedOutput)
  ) {
    throw new Error("Generation record parsedOutput is invalid");
  }
  if (value.error !== null && typeof value.error !== "string") {
    throw new Error("Generation record error is invalid");
  }
  if (
    (value.status === "success" &&
      (value.parsedOutput === null || value.error !== null)) ||
    (value.status === "failed" &&
      (value.parsedOutput !== null || typeof value.error !== "string"))
  ) {
    throw new Error("Generation record status payload is inconsistent");
  }
  if (
    value.attempts !== undefined &&
    (!Array.isArray(value.attempts) ||
      value.attempts.some(
        (attempt) =>
          !isGenerationAttemptSnapshot(attempt, allowedSchemaNames),
      ))
  ) {
    throw new Error("Generation record attempts are invalid");
  }

  return structuredClone(value) as GenerationRecord;
}

export function validateGenerationRequestSnapshot(
  value: unknown,
  allowedSchemaNames?: readonly string[],
): GenerationRequestSnapshot {
  if (!isPlainObject(value)) {
    throw new Error("Generation request snapshot is invalid");
  }
  try {
    assertExactObjectKeys(value, "Generation request snapshot", [
      "schemaName",
      "systemPrompt",
      "messages",
    ]);
  } catch {
    throw new Error("Generation request snapshot is invalid");
  }
  if (
    !isNonBlankString(value.schemaName) ||
    typeof value.systemPrompt !== "string" ||
    !Array.isArray(value.messages) ||
    value.messages.some(
      (message) =>
        !isPlainObject(message) ||
        (message.role !== "system" &&
          message.role !== "user" &&
          message.role !== "assistant") ||
        typeof message.content !== "string" ||
        Object.keys(message).some(
          (key) => key !== "role" && key !== "content",
        ),
    )
  ) {
    throw new Error("Generation request snapshot is invalid");
  }
  if (allowedSchemaNames && !allowedSchemaNames.includes(value.schemaName)) {
    throw new Error(`Unsupported generation request schema: ${value.schemaName}`);
  }
  return structuredClone(value) as GenerationRequestSnapshot;
}

export function isGenerationAttemptSnapshot(
  value: unknown,
  allowedSchemaNames?: readonly string[],
): value is GenerationAttemptSnapshot {
  if (!isPlainObject(value)) return false;
  try {
    assertExactObjectKeys(value, "Generation attempt", [
      "request",
      "tokenUsage",
      "rawOutput",
      "parsedOutput",
      "error",
    ]);
    validateGenerationRequestSnapshot(value.request, allowedSchemaNames);
  } catch {
    return false;
  }
  return (
    isLlmTokenUsage(value.tokenUsage) &&
    (value.rawOutput === null || typeof value.rawOutput === "string") &&
    (value.parsedOutput === null || isPlainObject(value.parsedOutput)) &&
    (value.error === null || typeof value.error === "string")
  );
}

export function isLlmTokenUsage(value: unknown): value is LlmTokenUsage | null {
  if (value === null) return true;
  if (!isPlainObject(value)) return false;
  try {
    assertExactObjectKeys(
      value,
      "Token usage",
      ["promptTokens", "completionTokens", "totalTokens"],
      ["cachedPromptTokens", "reasoningTokens"],
    );
  } catch {
    return false;
  }
  return [
    value.promptTokens,
    value.completionTokens,
    value.totalTokens,
    value.cachedPromptTokens,
    value.reasoningTokens,
  ].every(
    (count, index) =>
      (index >= 3 && count === undefined) ||
      count === null ||
      (typeof count === "number" && Number.isFinite(count) && count >= 0),
  );
}

function isNonBlankString(value: unknown): value is string {
  return typeof value === "string" && value.trim().length > 0;
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
