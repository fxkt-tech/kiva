import type { PlaybackItem } from "@/core/playback";

const systemVoiceBasePath = "/kivdb-assets/voice/system";

const titleVoiceFiles: ReadonlyMap<string, string> = new Map([
  ["身份牌", "role_card.mp3"],
  ["遗言阶段", "phase_last_words_start.mp3"],
  ["发言阶段", "phase_speech_start.mp3"],
  ["放逐投票", "phase_vote_start.mp3"],
  ["PK 阶段", "phase_pk_start.mp3"],
  ["准备阶段", "phase_setup_start.mp3"],
  ["游戏结束", "phase_game_ended.mp3"],
  ["狼人刀人", "action_wolf_kill.mp3"],
  ["预言家查验", "action_seer_check.mp3"],
  ["查验结果", "action_seer_result.mp3"],
  ["女巫死亡信息", "action_witch_death_info.mp3"],
  ["女巫解药", "action_witch_antidote.mp3"],
  ["女巫毒药", "action_witch_poison.mp3"],
  ["夜间结算", "resolution_night.mp3"],
  ["投票结算", "resolution_vote.mp3"],
  ["PK 结算", "resolution_pk.mp3"],
]);

export function systemVoiceSourceForScene(scene: PlaybackItem): string | null {
  const file = systemVoiceFileForScene(scene);
  return file ? `${systemVoiceBasePath}/${file}` : null;
}

function systemVoiceFileForScene(scene: PlaybackItem): string | null {
  const playerPromptFile = playerPromptVoiceFileForScene(scene);
  if (playerPromptFile) {
    return playerPromptFile;
  }

  if (/^第 \d+ 夜开始$/.test(scene.title)) {
    return "phase_night_start.mp3";
  }

  if (/^第 \d+ 天开始$/.test(scene.title)) {
    return "phase_day_start.mp3";
  }

  if (scene.title.startsWith("游戏结束：")) {
    return scene.title.includes("狼人")
      ? "game_end_wolves_win.mp3"
      : "game_end_good_win.mp3";
  }

  if (scene.title === "昨夜死讯") {
    return scene.text.includes("平安夜")
      ? "announcement_safe_night.mp3"
      : "announcement_death.mp3";
  }

  return titleVoiceFiles.get(scene.title) ?? null;
}

function playerPromptVoiceFileForScene(scene: PlaybackItem): string | null {
  if (scene.kind !== "speech") {
    return null;
  }

  const speaker = scene.players.find((player) => player.highlighted);
  if (!speaker || speaker.seatNo < 1 || speaker.seatNo > 6) {
    return null;
  }

  if (scene.title.includes("PK 发言")) {
    return `player_${speaker.seatNo}_pk_speech_prompt.mp3`;
  }

  if (scene.title.includes("遗言")) {
    return `player_${speaker.seatNo}_last_words_prompt.mp3`;
  }

  if (scene.title.includes("发言")) {
    return `player_${speaker.seatNo}_speech_prompt.mp3`;
  }

  return null;
}
