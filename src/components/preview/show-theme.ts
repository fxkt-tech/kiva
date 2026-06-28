import type { PlaybackItem, PlaybackSceneKind } from "@/core/playback";
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
  readonly timeMs: number;
  readonly sceneTimeMs: number;
  readonly enterProgress: number;
};

export type ShowThemeRenderers = {
  readonly renderPhase: (input: ThemeRenderInput) => void;
  readonly renderAnnouncement: (input: ThemeRenderInput) => void;
  readonly renderSpeech: (input: ThemeRenderInput) => void;
  readonly renderVote: (input: ThemeRenderInput) => void;
  readonly renderResolution: (input: ThemeRenderInput) => void;
  readonly renderEnd: (input: ThemeRenderInput) => void;
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
    readonly timeMs: number;
  },
): void {
  const sceneTimeMs = Math.max(0, input.timeMs - input.scene.startsAtMs);
  const enterProgress = Math.min(1, sceneTimeMs / 450);
  const renderInput: ThemeRenderInput = {
    ctx,
    scene: input.scene,
    items: input.items,
    timeMs: input.timeMs,
    sceneTimeMs,
    enterProgress,
  };

  rendererForKind(input.theme, input.scene.kind)(renderInput);
}

function rendererForKind(
  theme: ShowThemePack,
  kind: PlaybackSceneKind,
): (input: ThemeRenderInput) => void {
  switch (kind) {
    case "phase":
      return theme.render.renderPhase;
    case "speech":
      return theme.render.renderSpeech;
    case "vote":
      return theme.render.renderVote;
    case "resolution":
      return theme.render.renderResolution;
    case "announcement":
      return theme.render.renderAnnouncement;
  }
}
