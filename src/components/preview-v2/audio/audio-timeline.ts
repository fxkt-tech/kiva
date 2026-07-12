import type { PlaybackItem } from "@/core/playback";
import type { AudioCue } from "../composition/types";
import { VOICE_TIMING } from "@/core/playback";

export function buildAudioTimeline(
  items: readonly PlaybackItem[],
  gameId?: string,
): readonly AudioCue[] {
  const system = items
    .flatMap((item) => {
      if (item.presenterVoiceClips?.length && item.presenterSourceId) {
        let offsetMs = 0;
        return item.presenterVoiceClips.map<AudioCue>((clip, index) => {
          if (index > 0) offsetMs += VOICE_TIMING.presenterClipGapMs;
          const cue: AudioCue = {
            id: `presenter-voice:${item.index}:${index}:${clip.clipId}`,
            kind: "system-voice",
            src: `/api/presenters/${encodeURIComponent(item.presenterSourceId!)}/voice/${encodeURIComponent(clip.file)}`,
            startsAtMs: item.startsAtMs + offsetMs,
            durationMs: clip.durationMs,
            trimStartMs: 0,
            volume: 1,
          };
          offsetMs += clip.durationMs;
          return cue;
        });
      }
      return [];
    })
    .sort(
      (left, right) =>
        left.startsAtMs - right.startsAtMs || left.id.localeCompare(right.id),
    );
  const players = items.flatMap<AudioCue>((item) =>
    item.playerVoice && gameId
      ? [{
          id: `player-voice:${item.playerVoice.eventId}`,
          kind: "player-voice",
          src: `/api/games/${encodeURIComponent(gameId)}/voice/${encodeURIComponent(item.playerVoice.eventId)}`,
          startsAtMs: item.startsAtMs + item.playerVoice.startsAtOffsetMs,
          durationMs: item.playerVoice.durationMs,
          trimStartMs: 0,
          volume: 1,
        }]
      : [],
  );
  return [...system, ...players].sort(
    (left, right) => left.startsAtMs - right.startsAtMs || left.id.localeCompare(right.id),
  );
}
