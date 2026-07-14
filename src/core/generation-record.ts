import type { ModelBindingSnapshot } from "./model-binding";
import type { LlmTokenUsage } from "./llm";
import type { DraftId, GameId, PlayerId } from "./types";
import { assertExactObjectKeys, isPlainObject } from "./model-binding";
import {
  ACTION_PROMPT_VERSION,
  SPEECH_INTENT_PROMPT_VERSION,
  SPEECH_PERFORMANCE_PROMPT_VERSION,
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
  readonly finishReason?: string | null;
  readonly rawOutput: string | null;
  readonly parsedOutput: Record<string, unknown> | null;
  readonly error: string | null;
};

export type GenerationStageSnapshot = {
  readonly stage: "decision" | "performance";
  readonly promptVersion: string;
  readonly provider: string;
  readonly model: string;
  readonly request: GenerationRequestSnapshot;
  readonly tokenUsage: LlmTokenUsage | null;
  readonly rawOutput: string | null;
  readonly parsedOutput: Record<string, unknown> | null;
  readonly error: string | null;
  readonly attempts?: readonly GenerationAttemptSnapshot[];
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
  readonly stages: readonly GenerationStageSnapshot[];
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
  readonly stages?: readonly GenerationStageSnapshot[];
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
  readonly stages?: readonly GenerationStageSnapshot[];
  readonly attempts?: readonly GenerationAttemptSnapshot[];
};

export function createSuccessfulGenerationRecord(
  input: SuccessfulGenerationRecordInput,
): GenerationRecord {
  const stages = input.stages ?? [stageFromSuccessfulInput(input)];
  assertStageSequence(stages, input.purpose, "success");
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
    stages,
    ...(input.attempts ? { attempts: input.attempts } : {}),
  };
}

export function createFailedGenerationRecord(
  input: FailedGenerationRecordInput,
): GenerationRecord {
  const stages = input.stages ?? [stageFromFailedInput(input)];
  assertStageSequence(stages, input.purpose, "failed");
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
    stages,
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
      "stages",
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
      ? ["werewolf_speech_intent_v3", "werewolf_speech_performance_v1"]
      : ["werewolf_target_action_v3", "werewolf_optional_action_v3"];
  validateGenerationRequestSnapshot(value.request, allowedSchemaNames);
  if (!isLlmTokenUsage(value.tokenUsage)) {
    throw new Error("Generation record tokenUsage is invalid");
  }
  if (value.rawOutput !== null && typeof value.rawOutput !== "string") {
    throw new Error("Generation record rawOutput is invalid");
  }
  if (
    !Array.isArray(value.stages) ||
    value.stages.length === 0 ||
    value.stages.some((stage) => !isGenerationStageSnapshot(stage, purpose))
  ) {
    throw new Error("Generation record stages are invalid");
  }
  if (!hasValidStageSequence(value.stages, purpose, value.status)) {
    throw new Error("Generation record stage sequence is invalid");
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

function stageFromSuccessfulInput(
  input: SuccessfulGenerationRecordInput,
): GenerationStageSnapshot {
  return {
    stage: "decision",
    promptVersion: input.promptVersion,
    provider: input.modelBinding.provider,
    model: input.modelBinding.model,
    request: input.request,
    tokenUsage: input.tokenUsage ?? null,
    rawOutput: input.rawOutput,
    parsedOutput: input.parsedOutput,
    error: null,
    ...(input.attempts ? { attempts: input.attempts } : {}),
  };
}

function stageFromFailedInput(
  input: FailedGenerationRecordInput,
): GenerationStageSnapshot {
  return {
    stage: "decision",
    promptVersion: input.promptVersion,
    provider: input.modelBinding.provider,
    model: input.modelBinding.model,
    request: input.request,
    tokenUsage: input.tokenUsage ?? null,
    rawOutput: input.rawOutput,
    parsedOutput: null,
    error: errorMessage(input.error),
    ...(input.attempts ? { attempts: input.attempts } : {}),
  };
}

function isGenerationStageSnapshot(
  value: unknown,
  purpose: GenerationPurpose,
): value is GenerationStageSnapshot {
  if (!isPlainObject(value)) return false;
  try {
    assertExactObjectKeys(
      value,
      "Generation stage",
      [
        "stage",
        "promptVersion",
        "provider",
        "model",
        "request",
        "tokenUsage",
        "rawOutput",
        "parsedOutput",
        "error",
      ],
      ["attempts"],
    );
  } catch {
    return false;
  }
  if (
    (value.stage !== "decision" && value.stage !== "performance") ||
    !isNonBlankString(value.promptVersion) ||
    !isNonBlankString(value.provider) ||
    !isNonBlankString(value.model) ||
    !isLlmTokenUsage(value.tokenUsage) ||
    (value.rawOutput !== null && typeof value.rawOutput !== "string") ||
    (value.parsedOutput !== null && !isPlainObject(value.parsedOutput)) ||
    (value.error !== null && typeof value.error !== "string")
  ) {
    return false;
  }
  if (
    (value.error === null && value.parsedOutput === null) ||
    (value.error !== null && value.parsedOutput !== null)
  ) {
    return false;
  }
  const allowed =
    purpose === "action"
      ? ["werewolf_target_action_v3", "werewolf_optional_action_v3"]
      : value.stage === "decision"
        ? ["werewolf_speech_intent_v3"]
        : ["werewolf_speech_performance_v1"];
  try {
    validateGenerationRequestSnapshot(value.request, allowed);
  } catch {
    return false;
  }
  const expectedPromptVersion =
    purpose === "action"
      ? ACTION_PROMPT_VERSION
      : value.stage === "decision"
        ? SPEECH_INTENT_PROMPT_VERSION
        : SPEECH_PERFORMANCE_PROMPT_VERSION;
  if (value.promptVersion !== expectedPromptVersion) return false;
  return (
    value.attempts === undefined ||
    (Array.isArray(value.attempts) &&
      value.attempts.every((attempt) =>
        isGenerationAttemptSnapshot(attempt, allowed),
      ))
  );
}

function assertStageSequence(
  stages: readonly GenerationStageSnapshot[],
  purpose: GenerationPurpose,
  status: GenerationStatus,
): void {
  if (
    stages.some((stage) => !isGenerationStageSnapshot(stage, purpose)) ||
    !hasValidStageSequence(stages, purpose, status)
  ) {
    throw new Error("Generation stage sequence is invalid");
  }
}

function hasValidStageSequence(
  stages: readonly GenerationStageSnapshot[],
  purpose: GenerationPurpose,
  status: GenerationStatus,
): boolean {
  const succeeded = (stage: GenerationStageSnapshot) => stage.error === null;
  if (purpose === "action") {
    return (
      stages.length === 1 &&
      stages[0]?.stage === "decision" &&
      succeeded(stages[0]) === (status === "success")
    );
  }

  if (status === "success") {
    return (
      stages.length === 2 &&
      stages[0]?.stage === "decision" &&
      stages[1]?.stage === "performance" &&
      succeeded(stages[0]) &&
      succeeded(stages[1])
    );
  }

  return (
    (stages.length === 1 &&
      stages[0]?.stage === "decision" &&
      !succeeded(stages[0])) ||
    (stages.length === 2 &&
      stages[0]?.stage === "decision" &&
      stages[1]?.stage === "performance" &&
      succeeded(stages[0]) &&
      !succeeded(stages[1]))
  );
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
    assertExactObjectKeys(
      value,
      "Generation attempt",
      ["request", "tokenUsage", "rawOutput", "parsedOutput", "error"],
      ["finishReason"],
    );
    validateGenerationRequestSnapshot(value.request, allowedSchemaNames);
  } catch {
    return false;
  }
  return (
    isLlmTokenUsage(value.tokenUsage) &&
    (value.finishReason === undefined ||
      value.finishReason === null ||
      typeof value.finishReason === "string") &&
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
