import type { ModelBindingSnapshot } from "./player";
import type { LlmTokenUsage } from "./llm";
import type { DraftId, GameId, PlayerId } from "./types";

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
  readonly request: GenerationRequestSnapshot | null;
  readonly tokenUsage: LlmTokenUsage | null;
  readonly rawOutput: string | null;
  readonly parsedOutput: Record<string, unknown> | null;
  readonly error: string | null;
  readonly createdAt: string;
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
  };
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
