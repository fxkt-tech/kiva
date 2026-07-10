export const VIDEO_SPEC = {
  width: 1920,
  height: 1080,
  fps: 30,
  codec: "h264",
  audioCodec: "aac",
  pixelFormat: "yuv420p",
  compositionId: "KivaPlaybackV2",
} as const;

export const DEFAULT_COMPOSITION_ASSETS = {
  fontUrl: "/kivdb-assets/preview/noto-sans-sc-900.ttf",
  dayBackgroundUrl: "/kivdb-assets/preview/day-background.png",
  nightBackgroundUrl: "/kivdb-assets/preview/night-background.png",
} as const;
