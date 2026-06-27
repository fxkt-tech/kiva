import type { DraftEvent } from "./drafts";
import type { LlmMessage } from "./llm";
import type { PlayerLlmContext } from "./player-context";

export const SPEECH_PROMPT_VERSION = "speech:v1";

export type SpeechPromptInput = {
  readonly context: PlayerLlmContext;
  readonly draft: Extract<
    DraftEvent,
    { type: "day_speech_given" | "last_words_given" | "pk_speech_given" }
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
          "玩家名单：",
          ...input.context.roster.map((player) => rosterLine(player)),
          "",
          "你可见的事件时间线：",
          ...input.context.timeline.map(
            (item) => `#${item.index} ${item.title}：${item.text}`,
          ),
          "",
          "当前要生成的发言：",
          `draft=${input.draft.type}`,
          `phase=${input.draft.phase}`,
          `day=${input.draft.payload.dayNumber}`,
          `round=${"round" in input.draft.payload ? input.draft.payload.round : 0}`,
          "",
          "要求：",
          "- 只根据以上可见信息分析。",
          "- 发言要像真实玩家，不要解释你是 AI。",
          "- 不要编造未发生的事件。",
          '- 必须输出 JSON 对象，且只包含 "text" 和 "reasoning" 字段。',
          '- reasoning 是给主理人看的简短决策依据，只能引用以上可见信息。',
        ].join("\n"),
      },
    ],
  };
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
    case "villager":
      return "平民";
    default:
      return role;
  }
}

function factionLabel(faction: string): string {
  return faction === "wolves" ? "狼人阵营" : "好人阵营";
}
