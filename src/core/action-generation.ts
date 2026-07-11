import { applyDraftPayloadEdit, type DraftPayloadEdit } from "./draft-edit";
import type { DraftEvent } from "./drafts";
import type { GameEvent } from "./events";
import type { Game } from "./game";
import {
  createFailedGenerationRecord,
  createSuccessfulGenerationRecord,
  type GenerationRecord,
} from "./generation-record";
import {
  canActorUseActionOptions,
  legalActionOptions,
  type LegalActionOptions,
  type LlmActionDraft,
} from "./llm-action-options";
import type { LlmClient } from "./llm";
import { isLlmActionDraft } from "./llm-task-specs";
import { buildPlayerLlmContext } from "./player-context";
import { buildActionPrompt, type LlmPromptMode } from "./prompt-builders";
import type { PlayerId } from "./types";
import {
  generateValidatedJson,
  ValidatedGenerationError,
} from "./validated-generation";

export type GenerateActionDraftInput = {
  readonly game: Game;
  readonly events: readonly GameEvent[];
  readonly draft: DraftEvent;
  readonly llmClient: LlmClient;
  readonly generationId: string;
  readonly createdAt: string;
  readonly promptMode?: LlmPromptMode;
};

export type GenerateActionDraftResult = {
  readonly draft: DraftEvent;
  readonly generation: GenerationRecord | null;
};

export async function generateActionDraft(
  input: GenerateActionDraftInput,
): Promise<GenerateActionDraftResult> {
  if (!isLlmActionDraft(input.draft)) {
    return { draft: input.draft, generation: null };
  }

  const draft = input.draft;
  const playerId = actionActorId(draft);
  const context = buildPlayerLlmContext({
    game: input.game,
    events: input.events,
    viewerPlayerId: playerId,
  });
  const options = legalActionOptions({
    game: input.game,
    events: input.events,
    draft,
  });
  const prompt = buildActionPrompt({
    context,
    draft,
    options,
  }, input.promptMode);
  const request = {
    systemPrompt: prompt.systemPrompt,
    messages: prompt.messages,
    schemaName: prompt.schemaName,
  };

  if (
    !canActorUseActionOptions({
      game: input.game,
      events: input.events,
      draft,
    })
  ) {
    return {
      draft: input.draft,
      generation: createFailedGenerationRecord({
        id: input.generationId,
        gameId: input.game.id,
        draftId: draft.id,
        playerId,
        purpose: "action",
        promptVersion: prompt.promptVersion,
        modelBinding: context.viewer.modelBindingSnapshot,
        inputContextHash: contextHash(context),
        request,
        rawOutput: null,
        error: new Error(`Player cannot perform ${draft.type}`),
        createdAt: input.createdAt,
      }),
    };
  }

  try {
    const result = await generateValidatedJson({
      llmClient: input.llmClient,
      modelBinding: context.viewer.modelBindingSnapshot,
      request,
      validate: (parsed) =>
        parseAndValidateActionEdit(draft, parsed, options),
      repair: {
        outputContract: actionRepairContract(draft, options),
        legalTargetPlayerIds: options.targetPlayerIds,
      },
    });

    return {
      draft: applyDraftPayloadEdit(draft, result.value),
      generation: createSuccessfulGenerationRecord({
        id: input.generationId,
        gameId: input.game.id,
        draftId: draft.id,
        playerId,
        purpose: "action",
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
      draft,
      generation: createFailedGenerationRecord({
        id: input.generationId,
        gameId: input.game.id,
        draftId: draft.id,
        playerId,
        purpose: "action",
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

function actionRepairContract(
  draft: LlmActionDraft,
  options: LegalActionOptions,
): readonly string[] {
  if (
    draft.type === "witch_antidote_decided" ||
    draft.type === "witch_poison_decided"
  ) {
    return [
      "used 必须是布尔值",
      "used=false 时 targetPlayerId 必须为 null",
      options.canUse === false
        ? "本次规则禁止使用，used 必须为 false"
        : "used=true 时 targetPlayerId 必须来自合法候选",
      "decisionSummary 应是最多两句的简短字符串",
    ];
  }

  return [
    options.allowNoTarget
      ? "targetPlayerId 必须是合法候选之一或 null"
      : "targetPlayerId 必须是合法候选之一，不能是 null",
    "decisionSummary 应是最多两句的简短字符串",
  ];
}

function actionActorId(draft: LlmActionDraft): PlayerId {
  switch (draft.type) {
    case "vote_cast":
      return draft.payload.voterPlayerId;
    default:
      if (!draft.actorPlayerId) {
        throw new Error(`Missing actorPlayerId for ${draft.type}`);
      }
      return draft.actorPlayerId;
  }
}

function parseAndValidateActionEdit(
  draft: LlmActionDraft,
  output: Record<string, unknown>,
  options: LegalActionOptions,
): DraftPayloadEdit {
  switch (draft.type) {
    case "seer_check_selected":
    case "wolf_vote_cast":
    case "guard_protect_selected":
    case "hunter_shot_decided":
      return targetEdit(draft.type, output, options.targetPlayerIds);
    case "vote_cast":
      return voteEdit(output, options);
    case "witch_antidote_decided":
    case "witch_poison_decided":
      return witchEdit(draft.type, output, options.targetPlayerIds);
  }
}

function targetEdit(
  draftType: LlmActionDraft["type"],
  output: Record<string, unknown>,
  legalTargets: readonly PlayerId[],
): DraftPayloadEdit {
  const targetPlayerId = output.targetPlayerId;
  if (typeof targetPlayerId !== "string" || !legalTargets.includes(targetPlayerId as PlayerId)) {
    throw new Error(`Illegal targetPlayerId for ${draftType}`);
  }

  return { targetPlayerId: targetPlayerId as PlayerId };
}

function voteEdit(
  output: Record<string, unknown>,
  options: LegalActionOptions,
): DraftPayloadEdit {
  if (!("targetPlayerId" in output)) {
    throw new Error("targetPlayerId is required for vote_cast");
  }

  const targetPlayerId = output.targetPlayerId;
  if (targetPlayerId === null) {
    if (!options.allowNoTarget) {
      throw new Error("Abstain is not allowed for vote_cast");
    }
    return { targetPlayerId: null };
  }

  if (
    typeof targetPlayerId !== "string" ||
    !options.targetPlayerIds.includes(targetPlayerId as PlayerId)
  ) {
    throw new Error("Illegal targetPlayerId for vote_cast");
  }

  return { targetPlayerId: targetPlayerId as PlayerId };
}

function witchEdit(
  draftType: "witch_antidote_decided" | "witch_poison_decided",
  output: Record<string, unknown>,
  legalTargets: readonly PlayerId[],
): DraftPayloadEdit {
  if (typeof output.used !== "boolean") {
    throw new Error(`used is required for ${draftType}`);
  }

  const used = output.used;
  if (!used) {
    if (output.targetPlayerId !== null) {
      throw new Error(`targetPlayerId must be null when used=false for ${draftType}`);
    }
    return { used: false, targetPlayerId: null };
  }

  const targetPlayerId = output.targetPlayerId;
  if (typeof targetPlayerId !== "string" || !legalTargets.includes(targetPlayerId as PlayerId)) {
    throw new Error(`Illegal targetPlayerId for ${draftType}`);
  }

  return { used: true, targetPlayerId: targetPlayerId as PlayerId };
}

function contextHash(context: unknown): string {
  let hash = 0;
  const content = JSON.stringify(context);
  for (let index = 0; index < content.length; index += 1) {
    hash = (hash * 31 + content.charCodeAt(index)) >>> 0;
  }

  return hash.toString(16).padStart(8, "0");
}
