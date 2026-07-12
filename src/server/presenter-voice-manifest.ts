import { readFile } from "node:fs/promises";
import { join } from "node:path";
import {
  type PresenterVoiceManifest,
} from "@/core/presenter-voice";
import { validateVoiceProfileSnapshot } from "@/core/voice";

export async function loadPresenterVoiceManifest(
  dataDir: string,
  presenterId: string,
): Promise<PresenterVoiceManifest> {
  if (!/^[a-zA-Z0-9_-]+$/.test(presenterId)) {
    throw new Error("Invalid presenter identifier");
  }
  const value = JSON.parse(
    await readFile(join(dataDir, "presenters", presenterId, "manifest.json"), "utf8"),
  ) as PresenterVoiceManifest;
  if (
    value.schemaVersion !== 1 ||
    value.presenterId !== presenterId ||
    !value.clips ||
    !value.plans
  ) {
    throw new Error(`Presenter voice manifest is invalid: ${presenterId}`);
  }
  validateVoiceProfileSnapshot(value.profile, `Presenter ${presenterId} voice profile`);
  for (const [clipId, clip] of Object.entries(value.clips)) {
    if (
      !/^[\p{L}\p{N}_.-]+$/u.test(clipId) ||
      !/^[a-zA-Z0-9_.-]+\.mp3$/.test(clip.file) ||
      !clip.text.trim() ||
      !Number.isFinite(clip.durationMs) ||
      clip.durationMs <= 0
    ) {
      throw new Error(`Presenter voice clip is invalid: ${clipId}`);
    }
  }
  return value;
}

export function presenterVoiceFilePath(
  dataDir: string,
  presenterId: string,
  file: string,
): string {
  if (
    !/^[a-zA-Z0-9_-]+$/.test(presenterId) ||
    !/^[a-zA-Z0-9_.-]+\.mp3$/.test(file)
  ) {
    throw new Error("Invalid presenter voice path");
  }
  return join(dataDir, "presenters", presenterId, "voice", file);
}
