import type { PlaybackItem } from "@/core/playback";
import type { StageLayout } from "./stage-layout";
import { createShotFrame } from "./shot-engine/director";
import type { SeatTrackSlot } from "./shot-engine/shot-layout";
import type { ShotFrame } from "./shot-engine/types";
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

export type SkinRenderInput = {
  readonly ctx: CanvasRenderingContext2D;
  readonly frame: ShotFrame;
  readonly skin: PreviewThemeSkin;
};

export type PreviewBackgroundImages = {
  readonly day: HTMLImageElement | null;
  readonly night: HTMLImageElement | null;
};

export type PreviewAvatarImages = Readonly<Record<string, HTMLImageElement | null>>;

export type SkinSeatInput = SkinRenderInput & {
  readonly slot: SeatTrackSlot;
};

export type PreviewThemeDrawers = {
  readonly background: (input: SkinRenderInput) => void;
  readonly topBar: (input: SkinRenderInput) => void;
  readonly seat: (input: SkinSeatInput) => void;
  readonly speechShot: (input: SkinRenderInput) => void;
  readonly voteShot: (input: SkinRenderInput) => void;
  readonly phaseShot: (input: SkinRenderInput) => void;
  readonly announcementShot: (input: SkinRenderInput) => void;
  readonly resolutionShot: (input: SkinRenderInput) => void;
  readonly subtitle: (input: SkinRenderInput) => void;
  readonly effects: (input: SkinRenderInput) => void;
};

export type PreviewThemeSkin = {
  readonly id: ShowThemeId;
  readonly name: string;
  readonly tokens: ShowThemeTokens;
  readonly draw: PreviewThemeDrawers;
};

export type ShowThemePack = PreviewThemeSkin;

const themeRegistry: Record<string, PreviewThemeSkin> = {
  [mansionMurderTheme.id]: mansionMurderTheme,
};

export function getShowTheme(themeId: string | null | undefined): PreviewThemeSkin {
  return themeRegistry[themeId ?? ""] ?? themeRegistry[DEFAULT_SHOW_THEME_ID]!;
}

export function renderThemeFrame(
  ctx: CanvasRenderingContext2D,
  input: {
    readonly theme: ShowThemePack;
    readonly scene: PlaybackItem;
    readonly items: readonly PlaybackItem[];
    readonly layout?: StageLayout;
    readonly timeMs: number;
    readonly backgroundImages?: PreviewBackgroundImages;
    readonly avatarImages?: PreviewAvatarImages;
  },
): void {
  const frame = createShotFrame({
    scene: input.scene,
    items: input.items,
    timeMs: input.timeMs,
    backgroundImages: input.backgroundImages ?? { day: null, night: null },
    avatarImages: input.avatarImages ?? {},
  });
  const renderInput: SkinRenderInput = {
    ctx,
    frame,
    skin: input.theme,
  };

  input.theme.draw.background(renderInput);
  input.theme.draw.topBar(renderInput);
  frame.layout.seatSlots.forEach((slot) => {
    input.theme.draw.seat({ ...renderInput, slot });
  });

  switch (input.scene.kind) {
    case "speech":
      input.theme.draw.speechShot(renderInput);
      break;
    case "vote":
      input.theme.draw.voteShot(renderInput);
      break;
    case "resolution":
      input.theme.draw.resolutionShot(renderInput);
      break;
    case "phase":
      input.theme.draw.phaseShot(renderInput);
      break;
    case "announcement":
      input.theme.draw.announcementShot(renderInput);
      break;
  }

  input.theme.draw.subtitle(renderInput);
  input.theme.draw.effects(renderInput);
}
