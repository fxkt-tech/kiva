import { readFile } from "node:fs/promises";
import { join } from "node:path";
import { notFound } from "next/navigation";

const allowedFiles = new Set([
  "action_seer_check.mp3",
  "action_seer_result.mp3",
  "action_witch_antidote.mp3",
  "action_witch_death_info.mp3",
  "action_witch_poison.mp3",
  "action_wolf_kill.mp3",
  "announcement_death.mp3",
  "announcement_safe_night.mp3",
  "game_end_good_win.mp3",
  "game_end_wolves_win.mp3",
  "phase_day_start.mp3",
  "phase_game_ended.mp3",
  "phase_last_words_start.mp3",
  "phase_night_start.mp3",
  "phase_pk_start.mp3",
  "phase_setup_start.mp3",
  "phase_speech_start.mp3",
  "phase_vote_start.mp3",
  "player_1_last_words_prompt.mp3",
  "player_1_pk_speech_prompt.mp3",
  "player_1_speech_prompt.mp3",
  "player_2_last_words_prompt.mp3",
  "player_2_pk_speech_prompt.mp3",
  "player_2_speech_prompt.mp3",
  "player_3_last_words_prompt.mp3",
  "player_3_pk_speech_prompt.mp3",
  "player_3_speech_prompt.mp3",
  "player_4_last_words_prompt.mp3",
  "player_4_pk_speech_prompt.mp3",
  "player_4_speech_prompt.mp3",
  "player_5_last_words_prompt.mp3",
  "player_5_pk_speech_prompt.mp3",
  "player_5_speech_prompt.mp3",
  "player_6_last_words_prompt.mp3",
  "player_6_pk_speech_prompt.mp3",
  "player_6_speech_prompt.mp3",
  "resolution_night.mp3",
  "resolution_pk.mp3",
  "resolution_vote.mp3",
  "role_card.mp3",
  "unknown_player.mp3",
  "vote_abstain.mp3",
  "vote_exile.mp3",
]);

type SystemVoiceAssetRouteProps = {
  readonly params: Promise<{
    readonly file: string;
  }>;
};

export async function GET(
  _request: Request,
  { params }: SystemVoiceAssetRouteProps,
) {
  const { file } = await params;
  if (!allowedFiles.has(file)) {
    notFound();
  }

  const dataDir = process.env.KIVA_DATA_DIR ?? "kivdb";
  const content = await readFile(
    join(dataDir, "assets", "voice", "system", file),
  );

  return new Response(content, {
    headers: {
      "cache-control": "public, max-age=31536000, immutable",
      "content-type": "audio/mpeg",
    },
  });
}
