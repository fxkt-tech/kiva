import { copyFile, mkdir } from "node:fs/promises";
import { basename, join } from "node:path";
import { compilePublicPlayback } from "@/core/playback";
import { createCompositionInput } from "@/components/preview-v2/composition/create-composition-input";
import type { VideoCompositionInput } from "@/components/preview-v2/composition/types";
import type { GameRecord } from "@/server/game-repository";
import { loadPresenterVoiceManifest } from "@/server/presenter-voice-manifest";
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
  if (
    input.record.game.runMode === "scripted" &&
    input.record.episodeScript?.status !== "approved"
  ) {
    throw new Error("Episode script must be approved before video export");
  }
  const warnings: string[] = [];
  const assetsDir = join(input.jobDir, "assets");
  await mkdir(join(assetsDir, "avatars"), { recursive: true });
  await mkdir(join(assetsDir, "audio"), { recursive: true });

  const presenterVoiceManifest = await loadPresenterVoiceManifest(
    input.dataDir,
    input.record.game.presenter.presenterSourceId,
  );
  const items = compilePublicPlayback(
    input.record.events,
    input.record.game.players,
    {
      presenter: input.record.game.presenter,
      audience: "director",
      voiceArtifactsByEventId: input.record.voiceArtifactsByEventId,
      presenterVoiceManifest,
    },
  );
  const composition = createCompositionInput({
    gameId: input.record.game.id,
    gameTitle: input.record.game.title,
    script: input.record.game.script,
    items,
  });
  const missingPlayerVoice = items.find(
    (item) => item.transcriptSpeaker === "player" && !item.playerVoice,
  );
  if (missingPlayerVoice) {
    throw new Error(
      `Player voice is missing for playback event ${missingPlayerVoice.index}`,
    );
  }
  const assetBase =
    "/api/preview-v2/exports/" + input.job.jobId + "/assets/";

  await Promise.all([
    copyFile(
      join(input.dataDir, "assets", "preview", "noto-sans-sc-900.ttf"),
      join(assetsDir, "font.ttf"),
    ),
    copyFile(
      compositionBackgroundPath(input.dataDir, composition.assets.dayBackgroundUrl),
      join(assetsDir, "day-background.png"),
    ),
    copyFile(
      compositionBackgroundPath(input.dataDir, composition.assets.nightBackgroundUrl),
      join(assetsDir, "night-background.png"),
    ),
  ]);

  const avatarUrls: Record<string, string> = {};
  for (const source of Object.keys(composition.assets.avatarUrls)) {
    const asset = internalAvatarAsset(source);
    if (!asset || !isSafeAssetFile(asset.file)) {
      warnings.push("Avatar cannot be copied into the export snapshot: " + source);
      continue;
    }
    try {
      await copyFile(
        join(input.dataDir, "assets", asset.directory, asset.file),
        join(assetsDir, "avatars", asset.file),
      );
      avatarUrls[source] = assetBase + "avatars/" + encodeURIComponent(asset.file);
    } catch {
      warnings.push("Avatar was unavailable: " + asset.file);
    }
  }

  const audioCues = [];
  for (const cue of composition.audioCues) {
    if (cue.kind === "player-voice") {
      const eventId = cue.id.slice("player-voice:".length);
      const artifact = input.record.voiceArtifactsByEventId[eventId];
      if (!artifact || !isSafeAssetFile(artifact.audio.file)) {
        throw new Error(`Player voice artifact is invalid: ${eventId}`);
      }
      await copyFile(
        join(
          input.dataDir,
          "games",
          input.record.game.id,
          "voice",
          artifact.audio.file,
        ),
        join(assetsDir, "audio", artifact.audio.file),
      );
      audioCues.push({
        ...cue,
        src: assetBase + "audio/" + encodeURIComponent(artifact.audio.file),
      });
      continue;
    }
    if (cue.src.startsWith("/api/presenters/")) {
      const file = decodeURIComponent(cue.src.split("/").at(-1) ?? "");
      if (!isSafeAssetFile(file)) {
        throw new Error(`Presenter voice artifact is invalid: ${cue.src}`);
      }
      await copyFile(
        join(
          input.dataDir,
          "presenters",
          input.record.game.presenter.presenterSourceId,
          "voice",
          file,
        ),
        join(assetsDir, "audio", file),
      );
      audioCues.push({ ...cue, src: assetBase + "audio/" + encodeURIComponent(file) });
      continue;
    }
    throw new Error(`Audio cue cannot be copied into the export snapshot: ${cue.src}`);
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

function internalAvatarAsset(
  source: string,
): { readonly directory: "characters" | "presenters"; readonly file: string } | null {
  const character = internalAssetFile(source, "/kivdb-assets/characters/");
  if (character) return { directory: "characters", file: character };
  const presenter = internalAssetFile(source, "/kivdb-assets/presenters/");
  return presenter ? { directory: "presenters", file: presenter } : null;
}

function compositionBackgroundPath(
  dataDir: string,
  source: string,
): string {
  const script = internalAssetFile(source, "/kivdb-assets/scripts/");
  if (script && isSafeAssetFile(script)) {
    return join(dataDir, "assets", "scripts", script);
  }
  throw new Error(`Script background asset is invalid: ${source}`);
}

function isSafeAssetFile(file: string): boolean {
  return /^[a-zA-Z0-9_-]+\.(png|mp3)$/i.test(file);
}
