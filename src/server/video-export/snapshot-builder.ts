import { copyFile, mkdir } from "node:fs/promises";
import { basename, join } from "node:path";
import { compilePublicPlayback } from "@/core/playback";
import { systemVoiceSourceForScene } from "@/components/preview/preview-audio";
import { createCompositionInput } from "@/components/preview-v2/composition/create-composition-input";
import type { VideoCompositionInput } from "@/components/preview-v2/composition/types";
import type { GameRecord } from "@/server/game-repository";
import { loadSystemVoiceDurations } from "@/server/preview-voice-assets";
import type { ExportJob } from "./types";

export async function buildExportSnapshot(input: {
  readonly dataDir: string;
  readonly record: GameRecord;
  readonly job: ExportJob;
  readonly jobDir: string;
}): Promise<{
  readonly composition: VideoCompositionInput;
  readonly warnings: readonly string[];
}> {
  const warnings: string[] = [];
  const assetsDir = join(input.jobDir, "assets");
  await mkdir(join(assetsDir, "avatars"), { recursive: true });
  await mkdir(join(assetsDir, "audio"), { recursive: true });

  const voiceDurations = await loadSystemVoiceDurations(input.dataDir);
  const items = compilePublicPlayback(
    input.record.events,
    input.record.game.players,
    {
      presenter: input.record.game.presenter,
      audience: "director",
      durationForScene: (scene) => {
        const source = systemVoiceSourceForScene(scene);
        return source ? voiceDurations.get(source) ?? null : null;
      },
    },
  );
  const composition = createCompositionInput({
    gameId: input.record.game.id,
    gameTitle: input.record.game.title,
    items,
  });
  const assetBase =
    "/api/preview-v2/exports/" + input.job.jobId + "/assets/";

  await Promise.all([
    copyFile(
      join(input.dataDir, "assets", "preview", "noto-sans-sc-900.ttf"),
      join(assetsDir, "font.ttf"),
    ),
    copyFile(
      join(input.dataDir, "assets", "preview", "day-background.png"),
      join(assetsDir, "day-background.png"),
    ),
    copyFile(
      join(input.dataDir, "assets", "preview", "night-background.png"),
      join(assetsDir, "night-background.png"),
    ),
  ]);

  const avatarUrls: Record<string, string> = {};
  for (const source of Object.keys(composition.assets.avatarUrls)) {
    const file = internalAssetFile(source, "/kivdb-assets/characters/");
    if (!file || !isSafeAssetFile(file)) {
      warnings.push("Avatar was not snapshot-compatible: " + source);
      continue;
    }
    try {
      await copyFile(
        join(input.dataDir, "assets", "characters", file),
        join(assetsDir, "avatars", file),
      );
      avatarUrls[source] = assetBase + "avatars/" + encodeURIComponent(file);
    } catch {
      warnings.push("Avatar was unavailable: " + file);
    }
  }

  const audioCues = [];
  for (const cue of composition.audioCues) {
    const file = internalAssetFile(cue.src, "/kivdb-assets/voice/system/");
    if (!file || !isSafeAssetFile(file)) {
      warnings.push("Audio cue was not snapshot-compatible: " + cue.src);
      continue;
    }
    try {
      await copyFile(
        join(input.dataDir, "assets", "voice", "system", file),
        join(assetsDir, "audio", file),
      );
      audioCues.push({
        ...cue,
        src: assetBase + "audio/" + encodeURIComponent(file),
      });
    } catch {
      warnings.push("Optional audio cue was unavailable: " + file);
    }
  }

  return {
    composition: {
      ...composition,
      assets: {
        fontUrl: assetBase + "font.ttf",
        dayBackgroundUrl: assetBase + "day-background.png",
        nightBackgroundUrl: assetBase + "night-background.png",
        avatarUrls,
      },
      audioCues,
    },
    warnings,
  };
}

function internalAssetFile(source: string, prefix: string): string | null {
  let pathname: string;
  try {
    pathname = new URL(source, "http://kiva.local").pathname;
  } catch {
    return null;
  }
  if (!pathname.startsWith(prefix)) {
    return null;
  }
  return basename(pathname.slice(prefix.length));
}

function isSafeAssetFile(file: string): boolean {
  return /^[a-zA-Z0-9_-]+\.(png|mp3)$/i.test(file);
}
