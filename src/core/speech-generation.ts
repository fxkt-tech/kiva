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
import { buildPlayerLlmContext } from "./player-context";
import { buildSpeechPrompt } from "./prompt-builders";

type SpeechDraft = Extract<
  DraftEvent,
  { type: "day_speech_given" | "last_words_given" | "pk_speech_given" }
>;

export type GenerateSpeechDraftInput = {
  readonly game: Game;
  readonly events: readonly GameEvent[];
  readonly draft: DraftEvent;
  readonly llmClient: LlmClient;
  readonly generationId: string;
  readonly createdAt: string;
};

export type GenerateSpeechDraftResult = {
  readonly draft: DraftEvent;
  readonly generation: GenerationRecord | null;
};

export async function generateSpeechDraft(
  input: GenerateSpeechDraftInput,
): Promise<GenerateSpeechDraftResult> {
  if (!isSpeechDraft(input.draft)) {
    return { draft: input.draft, generation: null };
  }

  const playerId = input.draft.payload.playerId;
  const context = buildPlayerLlmContext({
    game: input.game,
    events: input.events,
    viewerPlayerId: playerId,
  });
  const prompt = buildSpeechPrompt({ context, draft: input.draft });
  const request = {
    systemPrompt: prompt.systemPrompt,
    messages: prompt.messages,
    schemaName: prompt.schemaName,
  };

  try {
    const output = await input.llmClient.generateJson({
      modelBinding: context.viewer.modelBindingSnapshot,
      ...request,
    });
    const text = parseSpeechText(output.parsed);

    return {
      draft: applyDraftPayloadEdit(input.draft, { text }),
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
        rawOutput: output.rawText,
        parsedOutput: output.parsed,
        createdAt: input.createdAt,
      }),
    };
  } catch (error) {
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
        rawOutput: null,
        error,
        createdAt: input.createdAt,
      }),
    };
  }
}

function isSpeechDraft(draft: DraftEvent): draft is SpeechDraft {
  return (
    draft.type === "day_speech_given" ||
    draft.type === "last_words_given" ||
    draft.type === "pk_speech_given"
  );
}

function parseSpeechText(output: Record<string, unknown>): string {
  const text = output.text;
  if (typeof text !== "string" || text.trim().length === 0) {
    throw new Error("LLM speech output must include non-empty text");
  }

  return text.trim();
}

function contextHash(context: unknown): string {
  let hash = 0;
  const content = JSON.stringify(context);
  for (let index = 0; index < content.length; index += 1) {
    hash = (hash * 31 + content.charCodeAt(index)) >>> 0;
  }

  return hash.toString(16).padStart(8, "0");
}
