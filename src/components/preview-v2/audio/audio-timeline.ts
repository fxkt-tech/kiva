import type { PlaybackItem } from "@/core/playback";
import { systemVoiceSourceForScene } from "@/components/preview/preview-audio";
import type { AudioCue } from "../composition/types";

export type AudioCueCandidate = Omit<AudioCue, "id">;
export type AudioCueResolver = (
  item: PlaybackItem,
) => AudioCueCandidate | null;

export function buildAudioTimeline(
  items: readonly PlaybackItem[],
  resolver: AudioCueResolver = systemVoiceCueForItem,
): readonly AudioCue[] {
  return items
    .flatMap((item) => {
      const candidate = resolver(item);
      return candidate
        ? [{
            ...candidate,
            id: [
              candidate.kind,
              item.index,
              candidate.startsAtMs,
            ].join(":"),
          }]
        : [];
    })
    .sort(
      (left, right) =>
        left.startsAtMs - right.startsAtMs || left.id.localeCompare(right.id),
    );
}

export function systemVoiceCueForItem(
  item: PlaybackItem,
): AudioCueCandidate | null {
  const src = systemVoiceSourceForScene(item);
  if (!src) {
    return null;
  }

  return {
    kind: "system-voice",
    src,
    startsAtMs: item.startsAtMs,
    durationMs: item.durationMs,
    trimStartMs: 0,
    volume: 1,
  };
}
