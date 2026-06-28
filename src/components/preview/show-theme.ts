import type { PlaybackItem } from "@/core/playback";
import type { StageLayout } from "./stage-layout";
import { mansionMurderTheme } from "./themes/mansion-murder";

export const DEFAULT_SHOW_THEME_ID = "mansion_murder";
export type ShowThemeId = typeof DEFAULT_SHOW_THEME_ID | string;

export type ShowThemeTokens = {
  readonly background: string;
  readonly panel: string;
  readonly text: string;
  readonly muted: string;
  readonly accent: string;
  readonly danger: string;
  readonly good: string;
  readonly wolf: string;
};

export type ThemeRenderInput = {
  readonly ctx: CanvasRenderingContext2D;
  readonly scene: PlaybackItem;
  readonly items: readonly PlaybackItem[];
  readonly layout: StageLayout;
  readonly timeMs: number;
  readonly sceneTimeMs: number;
  readonly enterProgress: number;
  readonly backgroundImages: PreviewBackgroundImages;
};

export type PreviewBackgroundImages = {
  readonly day: HTMLImageElement | null;
  readonly night: HTMLImageElement | null;
};

export type ShowThemeRenderers = {
  readonly renderBackground: (input: ThemeRenderInput) => void;
  readonly renderPlayerCard: (input: ThemeRenderInput) => void;
  readonly renderCenterStage: (input: ThemeRenderInput) => void;
  readonly renderSubtitle: (input: ThemeRenderInput) => void;
  readonly renderEffect: (input: ThemeRenderInput) => void;
};

export type ShowThemePack = {
  readonly id: ShowThemeId;
  readonly name: string;
  readonly tokens: ShowThemeTokens;
  readonly render: ShowThemeRenderers;
};

const themeRegistry: Record<string, ShowThemePack> = {
  [mansionMurderTheme.id]: mansionMurderTheme,
};

export function getShowTheme(themeId: string | null | undefined): ShowThemePack {
  return themeRegistry[themeId ?? ""] ?? themeRegistry[DEFAULT_SHOW_THEME_ID]!;
}

export function renderThemeFrame(
  ctx: CanvasRenderingContext2D,
  input: {
    readonly theme: ShowThemePack;
    readonly scene: PlaybackItem;
    readonly items: readonly PlaybackItem[];
    readonly layout: StageLayout;
    readonly timeMs: number;
    readonly backgroundImages?: PreviewBackgroundImages;
  },
): void {
  const sceneTimeMs = Math.max(0, input.timeMs - input.scene.startsAtMs);
  const enterProgress = Math.min(1, sceneTimeMs / 450);
  const renderInput: ThemeRenderInput = {
    ctx,
    scene: input.scene,
    items: input.items,
    layout: input.layout,
    timeMs: input.timeMs,
    sceneTimeMs,
    enterProgress,
    backgroundImages: input.backgroundImages ?? { day: null, night: null },
  };

  input.theme.render.renderBackground(renderInput);
  input.theme.render.renderPlayerCard(renderInput);
  input.theme.render.renderCenterStage(renderInput);
  input.theme.render.renderSubtitle(renderInput);
  input.theme.render.renderEffect(renderInput);
}
