import type { PlaybackItem, PlaybackScenePlayer } from "@/core/playback";
import type { PreviewAvatarImages, PreviewBackgroundImages } from "../preview-renderer";
import type { ShotLayout } from "./shot-layout";
import type { SubtitleCue } from "./subtitles";

export type ShotClock = {
  readonly absoluteMs: number;
  readonly sceneMs: number;
  readonly durationMs: number;
  readonly progress: number;
  readonly enterProgress: number;
  readonly exitProgress: number;
};

export type RenderablePlayer = PlaybackScenePlayer & {
  readonly emphasis: "active" | "highlighted" | "normal";
};

export type ShotFrame = {
  readonly gameTitle: string;
  readonly scene: PlaybackItem;
  readonly items: readonly PlaybackItem[];
  readonly layout: ShotLayout;
  readonly clock: ShotClock;
  readonly players: readonly RenderablePlayer[];
  readonly activePlayer: RenderablePlayer | null;
  readonly highlightedPlayers: readonly RenderablePlayer[];
  readonly subtitle: SubtitleCue | null;
  readonly backgroundImages: PreviewBackgroundImages;
  readonly avatarImages: PreviewAvatarImages;
};
