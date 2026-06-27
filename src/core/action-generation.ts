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
import type { PlayerContextRosterEntry } from "./player-context";
import {
  baseViewerSystemPrompts,
  firstNonEmpty,
  uniqueNonEmptyPrompts,
} from "./prompt-builders";
import { mechanicForDraftType } from "./role-mechanics";
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
  const mechanicKey = mechanicForDraftType(input.draft.type);
  const request = {
    systemPrompt: [
      ...uniqueNonEmptyPrompts([
        ...baseViewerSystemPrompts(context.viewer),
        firstNonEmpty(context.viewer.roleActionPromptSnapshot ?? ""),
      ]),
      "你只能根据可见信息给出狼人杀行动建议。",
      "必须输出 JSON 对象，不要输出 Markdown。",
    ].join("\n"),
    messages: [
      {
        role: "user" as const,
        content: [
          `draft=${input.draft.type}`,
          `mechanic=${mechanicKey}`,
          `你是：${context.viewer.seatNo} 号 ${context.viewer.name}`,
          `你的身份：${context.viewer.roleName}`,
          "玩家名单：",
          ...context.roster.map((player) => rosterPromptLine(player)),
          "可选目标：",
          ...legalTargetIds.map((targetPlayerId) => `playerId=${targetPlayerId}`),
          "可见事件：",
          ...context.timeline.flatMap((item) => timelinePromptLines(item)),
          '输出字段：targetPlayerId、reasoning，可选 used。弃票用 {"targetPlayerId":null,"reasoning":"简短说明原因"}。',
          "reasoning 是给主理人看的简短决策依据，只能引用以上可见信息。",
        ].join("\n"),
      },
    ],
    schemaName: "werewolf_action_v1",
  };

  if (
    !canActorPerformActionDraft(input.game, input.events, input.draft, context.viewer)
  ) {
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
        request,
        rawOutput: null,
        error: new Error(`Player cannot perform ${input.draft.type}`),
        createdAt: input.createdAt,
      }),
    };
  }

  try {
    const output = await input.llmClient.generateJson({
      modelBinding: context.viewer.modelBindingSnapshot,
      ...request,
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
        purpose: "action",
        promptVersion: ACTION_PROMPT_VERSION,
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

function rosterPromptLine(player: PlayerContextRosterEntry): string {
  return [
    `${player.seatNo} ${player.name}`,
    `playerId=${player.playerId}`,
    player.role ? `role=${player.role}` : null,
    player.faction ? `faction=${player.faction}` : null,
  ]
    .filter((part): part is string => part !== null)
    .join(" ");
}

function timelinePromptLines(item: {
  readonly index: number;
  readonly title: string;
  readonly text: string;
  readonly details?: readonly string[];
}): string[] {
  return [
    `#${item.index} ${item.title}：${item.text}`,
    ...(item.details ?? []).map((detail) => `  - ${detail}`),
  ];
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
      return legalAntidoteTargets(events);
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

function canActorPerformActionDraft(
  game: Game,
  events: readonly GameEvent[],
  draft: ActionDraft,
  viewer: {
    readonly playerId: PlayerId;
    readonly role: Game["players"][number]["gameRole"];
    readonly mechanicKey: Game["players"][number]["mechanicKey"];
  },
): boolean {
  switch (draft.type) {
    case "wolf_kill_selected":
      return viewer.role === "werewolf" && viewer.mechanicKey === "wolf_kill";
    case "seer_check_selected":
      return viewer.role === "seer" && viewer.mechanicKey === "seer_check";
    case "witch_antidote_decided":
    case "witch_poison_decided":
      return viewer.role === "witch" && viewer.mechanicKey === "witch_medicine";
    case "vote_cast":
      return deriveGameState(game.players, events).alivePlayerIds.includes(
        viewer.playerId,
      );
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
      return witchEdit(draft.type, output, legalAntidoteTargets(events));
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

function legalAntidoteTargets(events: readonly GameEvent[]): readonly PlayerId[] {
  const wolfKill = [...events]
    .reverse()
    .find((event): event is Extract<GameEvent, { type: "wolf_kill_selected" }> =>
      event.type === "wolf_kill_selected",
    );

  return wolfKill ? [wolfKill.payload.targetPlayerId] : [];
}

function contextHash(context: unknown): string {
  let hash = 0;
  const content = JSON.stringify(context);
  for (let index = 0; index < content.length; index += 1) {
    hash = (hash * 31 + content.charCodeAt(index)) >>> 0;
  }

  return hash.toString(16).padStart(8, "0");
}
