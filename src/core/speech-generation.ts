import { applyDraftPayloadEdit } from "./draft-edit";
import type { DraftEvent } from "./drafts";
import type { GameEvent } from "./events";
import type { Game } from "./game";
import type { EpisodeActorBrief } from "./episode-script";
import {
  createFailedGenerationRecord,
  createSuccessfulGenerationRecord,
  type GenerationRecord,
  type GenerationRequestSnapshot,
  type GenerationStageSnapshot,
} from "./generation-record";
import type { LlmClient, LlmTokenUsage } from "./llm";
import { isLlmSpeechDraft, type LlmSpeechDraft } from "./llm-task-specs";
import {
  selectedIntentEvidence,
  validatePlayerSpeechIntent,
} from "./player-intent";
import { buildPlayerLlmContext } from "./player-context";
import {
  buildSpeechIntentPrompt,
  buildSpeechPerformancePrompt,
  SPEECH_INTENT_PROMPT_VERSION,
  SPEECH_PERFORMANCE_PROMPT_VERSION,
  SPEECH_PROMPT_VERSION,
} from "./prompt-builders";
import { evaluateSpeech, type SpeechBudget } from "./speech-budget";
import {
  generateValidatedJson,
  ValidatedGenerationError,
  type ValidatedGenerationResult,
} from "./validated-generation";

export type GenerateSpeechDraftInput = {
  readonly game: Game;
  readonly events: readonly GameEvent[];
  readonly draft: DraftEvent;
  readonly llmClient: LlmClient;
  readonly generationId: string;
  readonly createdAt: string;
  readonly actorBrief?: EpisodeActorBrief | null;
};

export type GenerateSpeechDraftResult = {
  readonly draft: DraftEvent;
  readonly generation: GenerationRecord | null;
};

export async function generateSpeechDraft(
  input: GenerateSpeechDraftInput,
): Promise<GenerateSpeechDraftResult> {
  if (!isLlmSpeechDraft(input.draft)) {
    return { draft: input.draft, generation: null };
  }
  const draft = input.draft;
  const playerId = draft.payload.playerId;
  const context = buildPlayerLlmContext({
    game: input.game,
    events: input.events,
    viewerPlayerId: playerId,
  });
  const intentPrompt = buildSpeechIntentPrompt({
    context,
    draft,
    actorBrief: input.actorBrief,
  });
  const intentRequest = requestFor(intentPrompt);
  const stages: GenerationStageSnapshot[] = [];

  let intentResult: ValidatedGenerationResult<
    ReturnType<typeof validatePlayerSpeechIntent>
  >;
  try {
    intentResult = await generateValidatedJson({
      llmClient: input.llmClient,
      modelBinding: context.viewer.modelBinding,
      request: intentRequest,
      validate: (value) =>
        validatePlayerSpeechIntent({
          value,
          evidenceScope: intentPrompt.evidenceScope,
          publicSpeech: intentPrompt.publicSpeech,
          requireDisclosure: intentPrompt.requireDisclosure,
        }),
      repair: { outputContract: intentPrompt.outputContract },
    });
    stages.push(
      successfulStage(
        "decision",
        SPEECH_INTENT_PROMPT_VERSION,
        intentRequest,
        intentResult,
      ),
    );
  } catch (error) {
    const failure = generationFailure(error);
    stages.push(
      failedStage(
        "decision",
        SPEECH_INTENT_PROMPT_VERSION,
        intentRequest,
        context.viewer.modelBinding,
        error,
      ),
    );
    return {
      draft,
      generation: createFailedGenerationRecord({
        id: input.generationId,
        gameId: input.game.id,
        draftId: draft.id,
        playerId,
        purpose: "speech",
        promptVersion: SPEECH_PROMPT_VERSION,
        modelBinding: context.viewer.modelBinding,
        inputContextHash: contextHash(context),
        request: intentRequest,
        tokenUsage: failure.tokenUsage,
        rawOutput: failure.rawOutput,
        error,
        createdAt: input.createdAt,
        stages,
        attempts: failure.attempts,
      }),
    };
  }

  const evidence = selectedIntentEvidence({
    intent: intentResult.value,
    evidenceScope: intentPrompt.evidenceScope,
    publicSpeech: intentPrompt.publicSpeech,
  });
  const speechBudget = requiredSpeechBudget(intentPrompt.speechBudget);
  const performancePrompt = buildSpeechPerformancePrompt({
    context,
    draft,
    intent: intentResult.value,
    evidence,
    speechBudget,
    actorBrief: input.actorBrief,
  });
  const performanceRequest = requestFor(performancePrompt);
  try {
    const performanceResult = await generateValidatedJson({
      llmClient: input.llmClient,
      modelBinding: context.viewer.modelBinding,
      request: performanceRequest,
      validate: (value) => parsePerformanceText(value, speechBudget),
      repair: { outputContract: performancePrompt.outputContract },
    });
    stages.push(
      successfulStage(
        "performance",
        SPEECH_PERFORMANCE_PROMPT_VERSION,
        performanceRequest,
        performanceResult,
      ),
    );
    return {
      draft: applyDraftPayloadEdit(draft, { text: performanceResult.value }),
      generation: createSuccessfulGenerationRecord({
        id: input.generationId,
        gameId: input.game.id,
        draftId: draft.id,
        playerId,
        purpose: "speech",
        promptVersion: SPEECH_PROMPT_VERSION,
        modelBinding: context.viewer.modelBinding,
        inputContextHash: contextHash(context),
        request: performanceRequest,
        tokenUsage: mergeUsage(
          intentResult.tokenUsage,
          performanceResult.tokenUsage,
        ),
        rawOutput: performanceResult.output.rawText,
        parsedOutput: {
          intent: intentResult.value,
          text: performanceResult.value,
        },
        createdAt: input.createdAt,
        stages,
        attempts: performanceResult.attempts,
      }),
    };
  } catch (error) {
    const failure = generationFailure(error);
    stages.push(
      failedStage(
        "performance",
        SPEECH_PERFORMANCE_PROMPT_VERSION,
        performanceRequest,
        context.viewer.modelBinding,
        error,
      ),
    );
    return {
      draft,
      generation: createFailedGenerationRecord({
        id: input.generationId,
        gameId: input.game.id,
        draftId: draft.id,
        playerId,
        purpose: "speech",
        promptVersion: SPEECH_PROMPT_VERSION,
        modelBinding: context.viewer.modelBinding,
        inputContextHash: contextHash(context),
        request: performanceRequest,
        tokenUsage: mergeUsage(intentResult.tokenUsage, failure.tokenUsage),
        rawOutput: failure.rawOutput,
        error,
        createdAt: input.createdAt,
        stages,
        attempts: failure.attempts,
      }),
    };
  }
}

function parsePerformanceText(
  output: Record<string, unknown>,
  budget: SpeechBudget,
): string {
  const text = output.text;
  if (typeof text !== "string" || text.trim().length === 0) {
    throw new Error("Speech performance must include non-empty text");
  }
  const normalized = text.trim();
  const evaluation = evaluateSpeech(normalized, budget);
  if (!evaluation.withinHardLimit) {
    throw new Error(
      `Speech text exceeds hard limit: ${evaluation.characterCount} > ${budget.hardMaxCharacters}`,
    );
  }
  return normalized;
}

function requestFor(prompt: {
  readonly systemPrompt: string;
  readonly messages: GenerationRequestSnapshot["messages"];
  readonly schemaName: string;
}): GenerationRequestSnapshot {
  return {
    systemPrompt: prompt.systemPrompt,
    messages: prompt.messages,
    schemaName: prompt.schemaName,
  };
}

function successfulStage<Value>(
  stage: GenerationStageSnapshot["stage"],
  promptVersion: string,
  request: GenerationRequestSnapshot,
  result: ValidatedGenerationResult<Value>,
): GenerationStageSnapshot {
  return {
    stage,
    promptVersion,
    provider: result.output.provider,
    model: result.output.model,
    request,
    tokenUsage: result.tokenUsage,
    rawOutput: result.output.rawText,
    parsedOutput: result.output.parsed,
    error: null,
    ...(result.attempts ? { attempts: result.attempts } : {}),
  };
}

function failedStage(
  stage: GenerationStageSnapshot["stage"],
  promptVersion: string,
  request: GenerationRequestSnapshot,
  modelBinding: { readonly provider: string; readonly model: string },
  error: unknown,
): GenerationStageSnapshot {
  const failure = generationFailure(error);
  return {
    stage,
    promptVersion,
    provider: modelBinding.provider,
    model: modelBinding.model,
    request,
    tokenUsage: failure.tokenUsage,
    rawOutput: failure.rawOutput,
    parsedOutput: null,
    error: error instanceof Error ? error.message : String(error),
    ...(failure.attempts ? { attempts: failure.attempts } : {}),
  };
}

function generationFailure(error: unknown) {
  const failure = error instanceof ValidatedGenerationError ? error : null;
  return {
    tokenUsage: failure?.tokenUsage ?? null,
    rawOutput: failure?.rawOutput ?? null,
    attempts: failure?.attempts,
  };
}

function requiredSpeechBudget(value: SpeechBudget | undefined): SpeechBudget {
  if (!value) throw new Error("Speech prompt must include a budget");
  return value;
}

function mergeUsage(
  left: LlmTokenUsage | null,
  right: LlmTokenUsage | null,
): LlmTokenUsage | null {
  if (!left) return right;
  if (!right) return left;
  const sum = (a: number | null | undefined, b: number | null | undefined) =>
    a === null || a === undefined || b === null || b === undefined ? null : a + b;
  return {
    promptTokens: sum(left.promptTokens, right.promptTokens),
    completionTokens: sum(left.completionTokens, right.completionTokens),
    totalTokens: sum(left.totalTokens, right.totalTokens),
    cachedPromptTokens: sum(left.cachedPromptTokens, right.cachedPromptTokens),
    reasoningTokens: sum(left.reasoningTokens, right.reasoningTokens),
  };
}

function contextHash(context: unknown): string {
  let hash = 0;
  const content = JSON.stringify(context);
  for (let index = 0; index < content.length; index += 1) {
    hash = (hash * 31 + content.charCodeAt(index)) >>> 0;
  }
  return hash.toString(16).padStart(8, "0");
}
