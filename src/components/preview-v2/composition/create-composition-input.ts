import type { GameId } from "@/core/types";
import type { PlaybackItem } from "@/core/playback";
import { buildAudioTimeline } from "../audio/audio-timeline";
import {
  VIDEO_COMPOSITION_SCHEMA_VERSION,
  type VideoCompositionInput,
} from "./types";
import { DEFAULT_COMPOSITION_ASSETS } from "./video-spec";

export function createCompositionInput(input: {
  readonly gameId: GameId | string;
  readonly gameTitle: string;
  readonly items: readonly PlaybackItem[];
}): VideoCompositionInput {
  const avatarUrls = Object.fromEntries(
    input.items
      .flatMap((item) => item.players)
      .flatMap((player) =>
        player.avatar ? [[player.avatar, player.avatar] as const] : [],
      ),
  );

  return {
    schemaVersion: VIDEO_COMPOSITION_SCHEMA_VERSION,
    gameId: input.gameId,
    gameTitle: input.gameTitle,
    items: input.items,
    assets: {
      ...DEFAULT_COMPOSITION_ASSETS,
      avatarUrls,
    },
    audioCues: buildAudioTimeline(input.items),
  };
}
