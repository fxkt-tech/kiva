import type { GameId } from "@/core/types";
import type { PlaybackItem } from "@/core/playback";
import {
  legacyGameScriptSnapshot,
  type GameScriptSnapshot,
} from "@/core/game-script";
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
  readonly script?: GameScriptSnapshot;
}): VideoCompositionInput {
  const script = input.script ?? legacyGameScriptSnapshot();
  const avatarUrls = Object.fromEntries(
    [
      ...input.items.flatMap((item) => item.players.map((player) => player.avatar)),
      ...input.items.map((item) => item.presenterAvatar),
    ].flatMap((avatar) =>
      avatar ? [[avatar, avatar] as const] : [],
    ),
  );

  return {
    schemaVersion: VIDEO_COMPOSITION_SCHEMA_VERSION,
    gameId: input.gameId,
    gameTitle: input.gameTitle,
    script,
    items: input.items,
    assets: {
      ...DEFAULT_COMPOSITION_ASSETS,
      dayBackgroundUrl: script.presentation.dayBackground,
      nightBackgroundUrl: script.presentation.nightBackground,
      avatarUrls,
    },
    audioCues: buildAudioTimeline(input.items, input.gameId),
  };
}
