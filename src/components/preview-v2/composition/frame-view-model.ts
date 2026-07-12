import {
  playbackIndexAtMs,
  type PlaybackItem,
} from "@/core/playback";
import { createShotFrame } from "@/components/preview/shot-engine/director";
import type { ShotFrame } from "@/components/preview/shot-engine/types";
import type { CompositionAssets } from "./types";
import {
  legacyGameScriptSnapshot,
  type GameScriptSnapshot,
} from "@/core/game-script";

export type HtmlFrameViewModel = {
  readonly shot: ShotFrame;
  readonly assets: CompositionAssets;
  readonly script: GameScriptSnapshot;
};

export function createHtmlFrameViewModel(input: {
  readonly gameTitle: string;
  readonly items: readonly PlaybackItem[];
  readonly timeMs: number;
  readonly assets: CompositionAssets;
  readonly script?: GameScriptSnapshot;
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
    }),
    assets: input.assets,
    script: input.script ?? legacyGameScriptSnapshot(),
  };
}
