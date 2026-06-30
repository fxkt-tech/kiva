import type { PlaybackItem } from "@/core/playback";

export type PreviewBackgroundImages = {
  readonly day: HTMLImageElement | null;
  readonly night: HTMLImageElement | null;
};

export type PreviewAvatarImages = Readonly<Record<string, HTMLImageElement | null>>;

export type PreviewRenderFrameInput = {
  readonly items: readonly PlaybackItem[];
  readonly timeMs: number;
  readonly backgroundImages: PreviewBackgroundImages;
  readonly avatarImages: PreviewAvatarImages;
};

export type PreviewRendererHandle = {
  readonly canvas: HTMLCanvasElement | null;
  renderFrame(input: PreviewRenderFrameInput): void;
  destroy(): void;
};
