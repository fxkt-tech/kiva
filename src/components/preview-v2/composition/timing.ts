import { playbackTotalDurationMs, type PlaybackItem } from "@/core/playback";

export function millisecondsToFrame(
  milliseconds: number,
  fps: number,
): number {
  return Math.max(0, Math.floor((milliseconds * fps) / 1000));
}

export function frameToMilliseconds(frame: number, fps: number): number {
  return (Math.max(0, frame) * 1000) / fps;
}

export function sceneStartFrame(milliseconds: number, fps: number): number {
  return Math.max(0, Math.ceil((milliseconds * fps) / 1000));
}

export function compositionDurationInFrames(
  items: readonly PlaybackItem[],
  fps: number,
): number {
  return Math.max(1, Math.ceil((playbackTotalDurationMs(items) * fps) / 1000));
}
