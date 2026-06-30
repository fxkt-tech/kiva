import type { TextStyleOptions } from "pixi.js";

export const PIXI_PREVIEW_FONT_FAMILY =
  "DIN Condensed, Bahnschrift, Arial Narrow, Impact, sans-serif";

export type PixiPreviewTheme = {
  readonly colors: {
    readonly background: number;
    readonly surface: number;
    readonly panel: number;
    readonly panelAlt: number;
    readonly text: number;
    readonly muted: number;
    readonly accent: number;
    readonly brass: number;
    readonly danger: number;
    readonly good: number;
    readonly wolf: number;
    readonly black: number;
    readonly white: number;
  };
  readonly alpha: {
    readonly panel: number;
    readonly panelSoft: number;
    readonly stroke: number;
    readonly dead: number;
  };
  readonly layout: {
    readonly width: number;
    readonly height: number;
    readonly topBarHeight: number;
    readonly outerMargin: number;
    readonly columnGap: number;
    readonly sideColumnRatio: number;
    readonly centerColumnRatio: number;
    readonly subtitleHeight: number;
    readonly subtitleTopRatio: number;
    readonly seatCardHeight: number;
    readonly radius: number;
  };
  readonly typography: {
    readonly brand: TextStyleOptions;
    readonly title: TextStyleOptions;
    readonly meta: TextStyleOptions;
    readonly playerName: TextStyleOptions;
    readonly body: TextStyleOptions;
    readonly subtitle: TextStyleOptions;
  };
};

export const mansionPixiTheme: PixiPreviewTheme = {
  colors: {
    background: 0x030506,
    surface: 0x071012,
    panel: 0x070909,
    panelAlt: 0x0d1010,
    text: 0xeee7da,
    muted: 0x9a9488,
    accent: 0x86b4bd,
    brass: 0xb8a064,
    danger: 0xa95a56,
    good: 0x88ab96,
    wolf: 0xb75b58,
    black: 0x000000,
    white: 0xffffff,
  },
  alpha: {
    panel: 0.70,
    panelSoft: 0.50,
    stroke: 0.18,
    dead: 0.42,
  },
  layout: {
    width: 1920,
    height: 1080,
    topBarHeight: 88,
    outerMargin: 24,
    columnGap: 24,
    sideColumnRatio: 0.25,
    centerColumnRatio: 0.50,
    subtitleHeight: 176,
    subtitleTopRatio: 0.74,
    seatCardHeight: 126,
    radius: 12,
  },
  typography: {
    brand: {
      fill: 0xc2ad72,
      fontFamily: PIXI_PREVIEW_FONT_FAMILY,
      fontSize: 22,
      fontWeight: "900",
      lineHeight: 30,
      letterSpacing: 0,
    },
    title: {
      fill: 0xece7da,
      fontFamily: PIXI_PREVIEW_FONT_FAMILY,
      fontSize: 32,
      fontWeight: "900",
      lineHeight: 42,
      letterSpacing: 0,
    },
    meta: {
      fill: 0x9a9488,
      fontFamily: PIXI_PREVIEW_FONT_FAMILY,
      fontSize: 16,
      fontWeight: "700",
      lineHeight: 24,
      letterSpacing: 0,
    },
    playerName: {
      fill: 0xece7da,
      fontFamily: PIXI_PREVIEW_FONT_FAMILY,
      fontSize: 24,
      fontWeight: "900",
      letterSpacing: 0,
    },
    body: {
      fill: 0xd9d2c4,
      fontFamily: PIXI_PREVIEW_FONT_FAMILY,
      fontSize: 16,
      fontWeight: "700",
      letterSpacing: 0,
    },
    subtitle: {
      fill: 0xece7da,
      fontFamily: PIXI_PREVIEW_FONT_FAMILY,
      fontSize: 28,
      fontWeight: "700",
      letterSpacing: 0,
      lineHeight: 39,
      wordWrap: true,
      wordWrapWidth: 860,
    },
  },
};
