import type { PlaybackItem } from "@/core/playback";

const systemVoiceBasePath = "/kivdb-assets/voice/system";

export function systemVoiceSourceForScene(scene: PlaybackItem): string | null {
  const file = scene.presenterCue.voiceFile;
  return file ? `${systemVoiceBasePath}/${file}` : null;
}
