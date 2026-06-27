import { applyDraftPayloadEdit, type DraftPayloadEdit } from "./draft-edit";
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
import {
  getLegalNightTargets,
} from "./rules";
import { deriveGameState } from "./state";
import type { PlayerId } from "./types";

type ActionDraft = Extract<
  DraftEvent,
  | { type: "seer_check_selected" }
  | { type: "wolf_kill_selected" }
  | { type: "vote_cast" }
  | { type: "witch_antidote_decided" }
  | { type: "witch_poison_decided" }
>;

const ACTION_PROMPT_VERSION = "action:v1";

export type GenerateActionDraftInput = {
  readonly game: Game;
  readonly events: readonly GameEvent[];
  readonly draft: DraftEvent;
  readonly llmClient: LlmClient;
  readonly generationId: string;
  readonly createdAt: string;
};

export type GenerateActionDraftResult = {
  readonly draft: DraftEvent;
  readonly generation: GenerationRecord | null;
};

export async function generateActionDraft(
  input: GenerateActionDraftInput,
): Promise<GenerateActionDraftResult> {
  if (!isActionDraft(input.draft)) {
    return { draft: input.draft, generation: null };
  }

  const playerId = actionActorId(input.draft);
  const context = buildPlayerLlmContext({
    game: input.game,
    events: input.events,
    viewerPlayerId: playerId,
  });
  const legalTargetIds = legalTargetIdsForDraft(input.game, input.events, input.draft);

  try {
    const output = await input.llmClient.generateJson({
      modelBinding: context.viewer.modelBindingSnapshot,
      systemPrompt: [
        context.viewer.systemPrompt,
        "你只能根据可见信息给出狼人杀行动建议。",
        "必须输出 JSON 对象，不要输出 Markdown。",
      ].join("\n"),
      messages: [
        {
          role: "user",
          content: [
            `draft=${input.draft.type}`,
            "玩家名单：",
            ...context.roster.map(
              (player) =>
                `${player.seatNo} ${player.name} playerId=${player.playerId}`,
            ),
            "可选目标：",
            ...legalTargetIds.map((targetPlayerId) => `playerId=${targetPlayerId}`),
            "可见事件：",
            ...context.timeline.map((item) => `#${item.index} ${item.title}`),
            '输出字段：targetPlayerId，可选 used。弃票用 {"targetPlayerId":null}。',
          ].join("\n"),
        },
      ],
      schemaName: "werewolf_action_v1",
    });
    const edit = parseAndValidateActionEdit(input.game, input.events, input.draft, output.parsed);

    return {
      draft: applyDraftPayloadEdit(input.draft, edit),
      generation: createSuccessfulGenerationRecord({
        id: input.generationId,
        gameId: input.game.id,
        draftId: input.draft.id,
        playerId,
        purpose: "action",
        promptVersion: ACTION_PROMPT_VERSION,
        modelBinding: context.viewer.modelBindingSnapshot,
        inputContextHash: contextHash(context),
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
        purpose: "action",
        promptVersion: ACTION_PROMPT_VERSION,
        modelBinding: context.viewer.modelBindingSnapshot,
        inputContextHash: contextHash(context),
        rawOutput: null,
        error,
        createdAt: input.createdAt,
      }),
    };
  }
}

function legalTargetIdsForDraft(
  game: Game,
  events: readonly GameEvent[],
  draft: ActionDraft,
): readonly PlayerId[] {
  switch (draft.type) {
    case "seer_check_selected":
      return legalNightTargets(game, events, "seer_check", draft.actorPlayerId);
    case "wolf_kill_selected":
      return legalNightTargets(game, events, "wolf_kill", draft.actorPlayerId);
    case "witch_antidote_decided":
      return legalNightTargets(game, events, "wolf_kill", draft.actorPlayerId);
    case "witch_poison_decided":
      return legalNightTargets(game, events, "witch_poison", draft.actorPlayerId);
    case "vote_cast":
      return deriveGameState(game.players, events).alivePlayerIds.filter(
        (playerId) => playerId !== draft.payload.voterPlayerId,
      );
  }
}

function isActionDraft(draft: DraftEvent): draft is ActionDraft {
  return (
    draft.type === "seer_check_selected" ||
    draft.type === "wolf_kill_selected" ||
    draft.type === "vote_cast" ||
    draft.type === "witch_antidote_decided" ||
    draft.type === "witch_poison_decided"
  );
}

function actionActorId(draft: ActionDraft): PlayerId {
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
  game: Game,
  events: readonly GameEvent[],
  draft: ActionDraft,
  output: Record<string, unknown>,
): DraftPayloadEdit {
  switch (draft.type) {
    case "seer_check_selected":
      return targetEdit(draft.type, output, legalNightTargets(game, events, "seer_check", draft.actorPlayerId));
    case "wolf_kill_selected":
      return targetEdit(draft.type, output, legalNightTargets(game, events, "wolf_kill", draft.actorPlayerId));
    case "vote_cast":
      return voteEdit(game, events, draft, output);
    case "witch_antidote_decided":
      return witchEdit(draft.type, output, legalNightTargets(game, events, "wolf_kill", draft.actorPlayerId));
    case "witch_poison_decided":
      return witchEdit(draft.type, output, legalNightTargets(game, events, "witch_poison", draft.actorPlayerId));
  }
}

function targetEdit(
  draftType: ActionDraft["type"],
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
  game: Game,
  events: readonly GameEvent[],
  draft: Extract<ActionDraft, { type: "vote_cast" }>,
  output: Record<string, unknown>,
): DraftPayloadEdit {
  const targetPlayerId = output.targetPlayerId ?? null;
  if (targetPlayerId === null) {
    return { targetPlayerId: null };
  }

  const state = deriveGameState(game.players, events);
  const legalTargets = state.alivePlayerIds.filter(
    (playerId) => playerId !== draft.payload.voterPlayerId,
  );
  if (typeof targetPlayerId !== "string" || !legalTargets.includes(targetPlayerId as PlayerId)) {
    throw new Error("Illegal targetPlayerId for vote_cast");
  }

  return { targetPlayerId: targetPlayerId as PlayerId };
}

function witchEdit(
  draftType: "witch_antidote_decided" | "witch_poison_decided",
  output: Record<string, unknown>,
  legalTargets: readonly PlayerId[],
): DraftPayloadEdit {
  const used = output.used === true;
  if (!used) {
    return { used: false, targetPlayerId: null };
  }

  const targetPlayerId = output.targetPlayerId;
  if (typeof targetPlayerId !== "string" || !legalTargets.includes(targetPlayerId as PlayerId)) {
    throw new Error(`Illegal targetPlayerId for ${draftType}`);
  }

  return { used: true, targetPlayerId: targetPlayerId as PlayerId };
}

function legalNightTargets(
  game: Game,
  events: readonly GameEvent[],
  action: "wolf_kill" | "seer_check" | "witch_poison",
  actorPlayerId: PlayerId | undefined,
): readonly PlayerId[] {
  const state = deriveGameState(game.players, events);
  return getLegalNightTargets(action, game.players, state.alivePlayerIds, actorPlayerId);
}

function contextHash(context: unknown): string {
  let hash = 0;
  const content = JSON.stringify(context);
  for (let index = 0; index < content.length; index += 1) {
    hash = (hash * 31 + content.charCodeAt(index)) >>> 0;
  }

  return hash.toString(16).padStart(8, "0");
}
