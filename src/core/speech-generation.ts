import { applyDraftPayloadEdit } from "./draft-edit";
import type { DraftEvent } from "./drafts";
import type { GameEvent } from "./events";
import type { Game } from "./game";
import {
  createFailedGenerationRecord,
  createSuccessfulGenerationRecord,
  type GenerationRecord,
} from "./generation-record";
import type { LlmClient } from "./llm";
import {
  isLlmSpeechDraft,
  type LlmSpeechDraft,
} from "./llm-task-specs";
import { buildPlayerLlmContext } from "./player-context";
import {
  buildSpeechPrompt,
  SPEECH_PROMPT_VERSION,
  type LlmPromptMode,
} from "./prompt-builders";
import {
  generateValidatedJson,
  ValidatedGenerationError,
} from "./validated-generation";

export type GenerateSpeechDraftInput = {
  readonly game: Game;
  readonly events: readonly GameEvent[];
  readonly draft: DraftEvent;
  readonly llmClient: LlmClient;
  readonly generationId: string;
  readonly createdAt: string;
  readonly promptMode?: LlmPromptMode;
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
  const prompt = buildSpeechPrompt(
    { context, draft },
    input.promptMode,
  );
  const request = {
    systemPrompt: prompt.systemPrompt,
    messages: prompt.messages,
    schemaName: prompt.schemaName,
  };
  const requireDisclosure =
    prompt.promptVersion === SPEECH_PROMPT_VERSION &&
    requiresDisclosure(draft, context.viewer.role);

  try {
    const result = await generateValidatedJson({
      llmClient: input.llmClient,
      modelBinding: context.viewer.modelBindingSnapshot,
      request,
      validate: (parsed) =>
        parseAndValidateSpeechText(parsed, requireDisclosure),
      repair: {
        outputContract: speechRepairContract(requireDisclosure),
      },
    });

    return {
      draft: applyDraftPayloadEdit(input.draft, { text: result.value }),
      generation: createSuccessfulGenerationRecord({
        id: input.generationId,
        gameId: input.game.id,
        draftId: input.draft.id,
        playerId,
        purpose: "speech",
        promptVersion: prompt.promptVersion,
        modelBinding: context.viewer.modelBindingSnapshot,
        inputContextHash: contextHash(context),
        request,
        tokenUsage: result.tokenUsage,
        rawOutput: result.output.rawText,
        parsedOutput: result.output.parsed,
        attempts: result.attempts,
        createdAt: input.createdAt,
      }),
    };
  } catch (error) {
    const failure =
      error instanceof ValidatedGenerationError ? error : null;
    return {
      draft: input.draft,
      generation: createFailedGenerationRecord({
        id: input.generationId,
        gameId: input.game.id,
        draftId: input.draft.id,
        playerId,
        purpose: "speech",
        promptVersion: prompt.promptVersion,
        modelBinding: context.viewer.modelBindingSnapshot,
        inputContextHash: contextHash(context),
        request,
        tokenUsage: failure?.tokenUsage,
        rawOutput: failure?.rawOutput ?? null,
        error,
        createdAt: input.createdAt,
        attempts: failure?.attempts,
      }),
    };
  }
}

function parseAndValidateSpeechText(
  output: Record<string, unknown>,
  requireDisclosure: boolean,
): string {
  const text = output.text;
  if (typeof text !== "string" || text.trim().length === 0) {
    throw new Error("LLM speech output must include non-empty text");
  }

  if (
    requireDisclosure &&
    output.disclosure !== "conceal" &&
    output.disclosure !== "claim"
  ) {
    throw new Error(
      "Public special-role speech must include disclosure=conceal or claim",
    );
  }

  return text.trim();
}

function speechRepairContract(requireDisclosure: boolean): readonly string[] {
  return [
    "text 必须是非空字符串",
    "decisionSummary 应是最多两句的简短字符串",
    ...(requireDisclosure
      ? ["disclosure 必须是 conceal 或 claim"]
      : []),
    "只返回 JSON 对象",
  ];
}

function isPublicSpeechDraft(draft: LlmSpeechDraft): boolean {
  return (
    draft.type !== "wolf_strategy_given" &&
    draft.type !== "wolf_opinion_given"
  );
}

function requiresDisclosure(
  draft: LlmSpeechDraft,
  role: Game["players"][number]["gameRole"],
): boolean {
  return (
    isPublicSpeechDraft(draft) &&
    role !== "werewolf" &&
    role !== "villager"
  );
}

function contextHash(context: unknown): string {
  let hash = 0;
  const content = JSON.stringify(context);
  for (let index = 0; index < content.length; index += 1) {
    hash = (hash * 31 + content.charCodeAt(index)) >>> 0;
  }

  return hash.toString(16).padStart(8, "0");
}
