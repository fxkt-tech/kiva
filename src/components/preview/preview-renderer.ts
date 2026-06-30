import {
  playbackIndexAtMs,
  type PlaybackItem,
} from "@/core/playback";
import {
  DEFAULT_SHOW_THEME_ID,
  getShowTheme,
  type PreviewAvatarImages,
  type PreviewBackgroundImages,
  renderThemeFrame,
} from "./show-theme";
import { createSixPlayerStageLayout } from "./stage-layout";

export type PreviewRendererKind = "canvas2d" | "pixi";

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

export class Canvas2DPreviewRenderer implements PreviewRendererHandle {
  constructor(private readonly targetCanvas: HTMLCanvasElement) {}

  get canvas(): HTMLCanvasElement {
    return this.targetCanvas;
  }

  renderFrame(input: PreviewRenderFrameInput): void {
    drawPlaybackFrame(
      this.targetCanvas,
      input.items,
      input.timeMs,
      input.backgroundImages,
      input.avatarImages,
    );
  }

  destroy(): void {
    // The canvas is owned by React.
  }
}

export function drawPlaybackFrame(
  canvas: HTMLCanvasElement | null,
  items: readonly PlaybackItem[],
  timeMs: number,
  backgroundImages: PreviewBackgroundImages = { day: null, night: null },
  avatarImages: PreviewAvatarImages = {},
): void {
  if (!canvas) {
    return;
  }

  const context = canvas.getContext("2d");
  if (!context) {
    return;
  }

  const scene = items[playbackIndexAtMs(items, timeMs)];

  if (!scene) {
    context.fillStyle = "#050506";
    context.fillRect(0, 0, context.canvas.width, context.canvas.height);
    context.fillStyle = "#d4d4d8";
    context.font = "600 64px sans-serif";
    context.fillText("No playable scenes", 120, 520);
    return;
  }

  renderThemeFrame(context, {
    theme: getShowTheme(DEFAULT_SHOW_THEME_ID),
    scene,
    items,
    layout: createSixPlayerStageLayout(scene.players),
    timeMs,
    backgroundImages,
    avatarImages,
  });
}
