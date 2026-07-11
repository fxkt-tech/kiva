import type { DraftEvent } from "./drafts";
import type { LlmMessage } from "./llm";
import type { PlayerLlmContext } from "./player-context";
import type { Ruleset } from "./types";

export const SPEECH_PROMPT_VERSION = "speech:v1";

export type SpeechPromptInput = {
  readonly context: PlayerLlmContext;
  readonly draft: Extract<
    DraftEvent,
    { type: "day_speech_given" | "last_words_given" | "pk_speech_given" | "wolf_strategy_given" | "wolf_opinion_given" }
  >;
};

export type BuiltPrompt = {
  readonly promptVersion: string;
  readonly schemaName: string;
  readonly systemPrompt: string;
  readonly messages: readonly LlmMessage[];
};

export function buildSpeechPrompt(input: SpeechPromptInput): BuiltPrompt {
  return {
    promptVersion: SPEECH_PROMPT_VERSION,
    schemaName: "werewolf_speech_v1",
    systemPrompt: [
      ...baseViewerSystemPrompts(input.context.viewer),
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
          ...rulesetPromptLines(input.context.ruleset),
          "",
          "玩家名单：",
          ...input.context.roster.map((player) => rosterLine(player)),
          "",
          "你可见的事件时间线：",
          ...input.context.timeline.map(
            (item) => `- ${item.title}：${item.text}`,
          ),
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

export function rulesetPromptLines(ruleset: Ruleset): readonly string[] {
  return [
    "本局规则：",
    `- 玩家数：${ruleset.playerCount}`,
    `- 角色配置：狼人 ${ruleset.roleCounts.werewolf}、预言家 ${ruleset.roleCounts.seer}、女巫 ${ruleset.roleCounts.witch}、猎人 ${ruleset.roleCounts.hunter}、守卫 ${ruleset.roleCounts.guard}、平民 ${ruleset.roleCounts.villager}`,
    "- 本局没有白痴、骑士、狼王、警长或其他未列出的身份。",
    "- 守卫每晚守护一名存活玩家，可以自守，不能连续两晚守同一人；守中狼刀则该玩家不死亡。",
    "- 守卫和女巫解药同救同一人时，该玩家仍然不死亡。",
    "- 猎人被狼人刀死或被投票放逐时可以开枪；被女巫毒死不能开枪。",
    `- 胜利条件：${winConditionLabel(ruleset.winCondition)}`,
    `- 女巫首夜自救：${ruleset.witchFirstNightSelfSave ? "允许" : "不允许"}`,
    `- 女巫同夜使用解药和毒药：${ruleset.witchAllowSameNightAntidoteAndPoison ? "允许" : "不允许"}`,
    `- 投票公开：${voteRevealLabel(ruleset.voteReveal)}`,
    `- 死亡身份公开：${deadRoleRevealLabel(ruleset.deadRoleReveal)}`,
    `- PK 投票范围：${pkVotersLabel(ruleset.pkVoters)}`,
    `- 弃票：${ruleset.allowAbstainVote ? "允许" : "不允许"}`,
  ];
}

export function baseViewerSystemPrompts(
  viewer: Pick<
    PlayerLlmContext["viewer"],
    | "characterSystemPromptSnapshot"
    | "roleSystemPromptSnapshot"
    | "systemPrompt"
  >,
): string[] {
  return uniqueNonEmptyPrompts([
    firstNonEmpty(viewer.characterSystemPromptSnapshot, viewer.systemPrompt),
    viewer.roleSystemPromptSnapshot,
  ]);
}

export function firstNonEmpty(...values: readonly string[]): string {
  return values.map((value) => value.trim()).find((value) => value.length > 0) ?? "";
}

export function uniqueNonEmptyPrompts(values: readonly string[]): string[] {
  const seen = new Set<string>();
  const prompts: string[] = [];

  for (const value of values) {
    const prompt = value.trim();
    if (prompt.length === 0 || seen.has(prompt)) {
      continue;
    }

    seen.add(prompt);
    prompts.push(prompt);
  }

  return prompts;
}

function rosterLine(player: PlayerLlmContext["roster"][number]): string {
  const self = player.isSelf ? "（你）" : "";
  if (player.role && player.faction) {
    return `${player.seatNo} 号 ${player.name}${self}：${roleLabel(player.role)}（${factionLabel(player.faction)}）`;
  }

  return `${player.seatNo} 号 ${player.name}${self}`;
}

function roleLabel(role: string): string {
  switch (role) {
    case "werewolf":
      return "狼人";
    case "seer":
      return "预言家";
    case "witch":
      return "女巫";
    case "hunter":
      return "猎人";
    case "guard":
      return "守卫";
    case "villager":
      return "平民";
    default:
      return role;
  }
}

function factionLabel(faction: string): string {
  return faction === "wolves" ? "狼人阵营" : "好人阵营";
}

function winConditionLabel(winCondition: Ruleset["winCondition"]): string {
  switch (winCondition) {
    case "slaughter_side":
      return "屠边";
    case "slaughter_all":
      return "屠城";
  }
}

function voteRevealLabel(voteReveal: Ruleset["voteReveal"]): string {
  switch (voteReveal) {
    case "after_all_votes":
      return "所有人投票后统一公开";
    case "immediate":
      return "投票后立即公开";
  }
}

function deadRoleRevealLabel(deadRoleReveal: Ruleset["deadRoleReveal"]): string {
  switch (deadRoleReveal) {
    case "endgame":
      return "游戏结束后公开";
    case "on_death":
      return "死亡时公开";
  }
}

function pkVotersLabel(pkVoters: Ruleset["pkVoters"]): string {
  switch (pkVoters) {
    case "non_pk_only":
      return "仅非 PK 玩家";
    case "all_living_non_self":
      return "所有存活且不能投自己";
  }
}
