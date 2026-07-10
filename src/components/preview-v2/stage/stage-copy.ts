import type { PlaybackItem, PlaybackScenePlayer } from "@/core/playback";

export const PRESENTER_NAME = "主理人";

export type TranscriptPresentation = {
  readonly speaker: {
    readonly kind: "player" | "presenter";
    readonly name: string;
    readonly seatNo: number | null;
    readonly avatar: string | null;
  };
  readonly content: string;
};

export function transcriptPresentationForScene(
  scene: PlaybackItem,
  activePlayer: PlaybackScenePlayer | null,
): TranscriptPresentation {
  const player = scene.kind === "speech" ? activePlayer : null;

  return {
    speaker: player
      ? {
          kind: "player",
          name: player.name,
          seatNo: player.seatNo,
          avatar: player.avatar,
        }
      : {
          kind: "presenter",
          name: PRESENTER_NAME,
          seatNo: null,
          avatar: null,
        },
    content: narrationTextForScene(scene),
  };
}

export function narrationTextForScene(scene: PlaybackItem): string {
  const narration = scene.text.trim();
  if (narration) {
    return narration;
  }

  if (scene.kind === "phase") {
    return phaseNarration(scene.phase);
  }

  const details = scene.details.map((detail) => detail.trim()).filter(Boolean);
  if (details.length > 0) {
    return details.join("；");
  }

  switch (scene.kind) {
    case "announcement":
      return "本环节信息已确认。";
    case "speech":
      return "该玩家本轮没有留下发言内容。";
    case "vote":
      return "本轮投票信息已记录。";
    case "resolution":
      return "本轮结算已经完成。";
  }
}

function phaseNarration(phase: PlaybackItem["phase"]): string {
  switch (phase) {
    case "setup":
      return "对局准备开始。";
    case "night":
      return "进入夜晚阶段，请所有玩家确认夜间行动。";
    case "day":
      return "天亮了，主理人将公布昨夜信息。";
    case "speech":
      return "进入依次发言阶段。";
    case "vote":
      return "进入本日放逐投票阶段。";
    case "pk":
      return "进入 PK 发言阶段。";
    case "last_words":
      return "进入遗言阶段。";
    case "ended":
      return "本局游戏已经结束。";
  }
}
