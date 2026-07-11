import type { LlmMessage } from "./llm";
import type { LegalActionOptions } from "./llm-action-options";
import type { LlmActionDraft, LlmSpeechDraft } from "./llm-task-specs";
import type { PlayerLlmContext } from "./player-context";
import { mechanicForDraftType } from "./role-mechanics";
import type { Ruleset } from "./types";

export const LEGACY_SPEECH_PROMPT_VERSION = "speech:v1";
export const LEGACY_ACTION_PROMPT_VERSION = "action:v1";

export type LegacyBuiltPrompt = {
  readonly promptVersion: string;
  readonly schemaName: string;
  readonly systemPrompt: string;
  readonly messages: readonly LlmMessage[];
};

export function buildSpeechPromptV1(input: {
  readonly context: PlayerLlmContext;
  readonly draft: LlmSpeechDraft;
}): LegacyBuiltPrompt {
  return {
    promptVersion: LEGACY_SPEECH_PROMPT_VERSION,
    schemaName: "werewolf_speech_v1",
    systemPrompt: [
      ...legacyBaseViewerSystemPrompts(input.context),
      "你正在参与一局狼人杀内容创作。",
      "你只能依据用户消息中列出的可见信息发言。",
      "禁止引用、暗示或利用未出现在可见信息中的上帝视角事实。",
      '必须输出 JSON 对象，格式为 {"text":"你的发言","reasoning":"简短说明你为什么这样发言"}，不要输出 Markdown。',
    ].join("\n"),
    messages: [
      {
        role: "user",
        content: [
          `游戏：${input.context.gameTitle}`,
          `你是：${input.context.viewer.seatNo} 号 ${input.context.viewer.name}`,
          `你的身份：${input.context.viewer.roleName}（${factionLabel(input.context.viewer.faction)}）`,
          `人设：${input.context.viewer.persona}`,
          `发言风格：${input.context.viewer.speakingStyle}`,
          `推理风格：${input.context.viewer.reasoningStyle}`,
          "",
          ...legacyRulesetPromptLines(input.context.ruleset),
          "",
          "玩家名单：",
          ...input.context.roster.map(legacyRosterLine),
          "",
          "你可见的事件时间线：",
          ...input.context.timeline.map((item) => `- ${item.title}：${item.text}`),
          "",
          "当前要生成的发言：",
          `draft=${input.draft.type}`,
          `phase=${input.draft.phase}`,
          `day=${input.draft.payload.dayNumber}`,
          `round=${"round" in input.draft.payload ? input.draft.payload.round : 0}`,
          "",
          "要求：",
          ...(input.draft.type === "wolf_strategy_given"
            ? ["- 你是首夜战术制定者，先提出狼队整体打法、身份伪装与当夜行动方向。"]
            : input.draft.type === "wolf_opinion_given"
              ? ["- 结合已经发布的狼队战术与意见，提出你的分析和建议，不要直接声称已完成投票。"]
              : []),
          "- 只根据以上可见信息分析。",
          "- 发言要像真实玩家，不要解释你是 AI。",
          "- 不要编造未发生的事件。",
          '- 必须输出 JSON 对象，且只包含 "text" 和 "reasoning" 字段。',
          "- reasoning 是玩家的简短思考过程，只能引用以上可见信息。",
        ].join("\n"),
      },
    ],
  };
}

export function buildActionPromptV1(input: {
  readonly context: PlayerLlmContext;
  readonly draft: LlmActionDraft;
  readonly options: LegalActionOptions;
}): LegacyBuiltPrompt {
  const roleActionPrompt = input.context.viewer.roleActionPromptSnapshot?.trim();
  return {
    promptVersion: LEGACY_ACTION_PROMPT_VERSION,
    schemaName: "werewolf_action_v1",
    systemPrompt: [
      ...legacyBaseViewerSystemPrompts(input.context),
      ...(roleActionPrompt ? [roleActionPrompt] : []),
      "你只能根据可见信息给出狼人杀行动建议。",
      "必须输出 JSON 对象，不要输出 Markdown。",
    ].join("\n"),
    messages: [
      {
        role: "user",
        content: [
          `draft=${input.draft.type}`,
          `mechanic=${mechanicForDraftType(input.draft.type)}`,
          `你是：${input.context.viewer.seatNo} 号 ${input.context.viewer.name}`,
          `你的身份：${input.context.viewer.roleName}`,
          ...legacyRulesetPromptLines(input.context.ruleset),
          "玩家名单：",
          ...input.context.roster.map(legacyActionRosterLine),
          "可选目标：",
          ...input.options.targetPlayerIds.map(
            (targetPlayerId) => `playerId=${targetPlayerId}`,
          ),
          "可见事件：",
          ...input.context.timeline.flatMap((item) => [
            `- ${item.title}：${item.text}`,
            ...(item.details ?? []).map((detail) => `  - ${detail}`),
          ]),
          legacyActionOutputInstruction(input.draft.type),
          "reasoning 是玩家的简短思考过程，只能引用以上可见信息。",
        ].join("\n"),
      },
    ],
  };
}

function legacyActionOutputInstruction(type: LlmActionDraft["type"]): string {
  switch (type) {
    case "witch_antidote_decided":
      return [
        "输出字段：used、targetPlayerId、reasoning。",
        'used 是必填布尔值。使用解药输出 {"used":true,"targetPlayerId":"从可选目标中选择的 playerId","reasoning":"简短说明原因"}。',
        '不使用解药输出 {"used":false,"targetPlayerId":null,"reasoning":"简短说明原因"}。',
      ].join("\n");
    case "witch_poison_decided":
      return [
        "输出字段：used、targetPlayerId、reasoning。",
        'used 是必填布尔值。使用毒药输出 {"used":true,"targetPlayerId":"从可选目标中选择的 playerId","reasoning":"简短说明原因"}。',
        '不使用毒药输出 {"used":false,"targetPlayerId":null,"reasoning":"简短说明原因"}。',
      ].join("\n");
    case "vote_cast":
      return '输出字段：targetPlayerId、reasoning。弃票用 {"targetPlayerId":null,"reasoning":"简短说明原因"}。';
    case "seer_check_selected":
    case "wolf_vote_cast":
    case "guard_protect_selected":
    case "hunter_shot_decided":
      return "输出字段：targetPlayerId、reasoning。必须从可选目标中选择一个 playerId。";
  }
}

function legacyBaseViewerSystemPrompts(
  context: PlayerLlmContext,
): readonly string[] {
  return uniqueNonEmpty([
    firstNonEmpty(
      context.viewer.characterSystemPromptSnapshot,
      context.viewer.systemPrompt,
    ),
    context.viewer.roleSystemPromptSnapshot,
  ]);
}

function legacyRulesetPromptLines(ruleset: Ruleset): readonly string[] {
  return [
    "本局规则：",
    `- 玩家数：${ruleset.playerCount}`,
    `- 角色配置：狼人 ${ruleset.roleCounts.werewolf}、预言家 ${ruleset.roleCounts.seer}、女巫 ${ruleset.roleCounts.witch}、猎人 ${ruleset.roleCounts.hunter}、守卫 ${ruleset.roleCounts.guard}、平民 ${ruleset.roleCounts.villager}`,
    "- 本局没有白痴、骑士、狼王、警长或其他未列出的身份。",
    "- 守卫每晚守护一名存活玩家，可以自守，不能连续两晚守同一人；守中狼刀则该玩家不死亡。",
    "- 守卫和女巫解药同救同一人时，该玩家仍然不死亡。",
    "- 猎人被狼人刀死或被投票放逐时可以开枪；被女巫毒死不能开枪。",
    `- 胜利条件：${ruleset.winCondition === "slaughter_side" ? "屠边" : "屠城"}`,
    `- 女巫首夜自救：${ruleset.witchFirstNightSelfSave ? "允许" : "不允许"}`,
    `- 女巫同夜使用解药和毒药：${ruleset.witchAllowSameNightAntidoteAndPoison ? "允许" : "不允许"}`,
    `- 投票公开：${ruleset.voteReveal === "immediate" ? "投票后立即公开" : "所有人投票后统一公开"}`,
    `- 死亡身份公开：${ruleset.deadRoleReveal === "on_death" ? "死亡时公开" : "游戏结束后公开"}`,
    `- PK 投票范围：${ruleset.pkVoters === "non_pk_only" ? "仅非 PK 玩家" : "所有存活且不能投自己"}`,
    `- 弃票：${ruleset.allowAbstainVote ? "允许" : "不允许"}`,
  ];
}

function legacyRosterLine(
  player: PlayerLlmContext["roster"][number],
): string {
  const self = player.isSelf ? "（你）" : "";
  if (player.role && player.faction) {
    return `${player.seatNo} 号 ${player.name}${self}：${roleLabel(player.role)}（${factionLabel(player.faction)}）`;
  }
  return `${player.seatNo} 号 ${player.name}${self}`;
}

function legacyActionRosterLine(
  player: PlayerLlmContext["roster"][number],
): string {
  return [
    `${player.seatNo} ${player.name}`,
    `playerId=${player.playerId}`,
    player.role ? `role=${player.role}` : null,
    player.faction ? `faction=${player.faction}` : null,
  ]
    .filter((part): part is string => part !== null)
    .join(" ");
}

function roleLabel(role: string): string {
  const labels: Record<string, string> = {
    werewolf: "狼人",
    seer: "预言家",
    witch: "女巫",
    hunter: "猎人",
    guard: "守卫",
    villager: "平民",
  };
  return labels[role] ?? role;
}

function factionLabel(faction: string): string {
  return faction === "wolves" ? "狼人阵营" : "好人阵营";
}

function firstNonEmpty(...values: readonly string[]): string {
  return values.map((value) => value.trim()).find(Boolean) ?? "";
}

function uniqueNonEmpty(values: readonly string[]): readonly string[] {
  return [...new Set(values.map((value) => value.trim()).filter(Boolean))];
}
