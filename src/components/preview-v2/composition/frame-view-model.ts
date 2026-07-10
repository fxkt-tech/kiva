import {
  playbackIndexAtMs,
  type PlaybackItem,
} from "@/core/playback";
import { createShotFrame } from "@/components/preview/shot-engine/director";
import type { ShotFrame } from "@/components/preview/shot-engine/types";
import type { CompositionAssets } from "./types";

export type HtmlFrameViewModel = {
  readonly shot: ShotFrame;
  readonly assets: CompositionAssets;
};

export function createHtmlFrameViewModel(input: {
  readonly gameTitle: string;
  readonly items: readonly PlaybackItem[];
  readonly timeMs: number;
  readonly assets: CompositionAssets;
}): HtmlFrameViewModel | null {
  const scene = input.items[playbackIndexAtMs(input.items, input.timeMs)];
  if (!scene) {
    return null;
  }

  return {
    shot: createShotFrame({
      gameTitle: input.gameTitle,
      scene,
      items: input.items,
      timeMs: input.timeMs,
      backgroundImages: { day: null, night: null },
      avatarImages: {},
    }),
    assets: input.assets,
  };
}
