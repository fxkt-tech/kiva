import { copyFile, mkdir, readFile, readdir, rename, stat, unlink, writeFile } from "node:fs/promises";
import { join } from "node:path";
import { createHash } from "node:crypto";
import {
  presenterPlanForTemplate,
  type PresenterVoiceManifest,
} from "@/core/presenter-voice";
import type { VoiceProfileSnapshot } from "@/core/voice";
import { PRESENTER_EDGE_VOICE } from "@/core/voice";
import { createEdgeVoiceAdapter } from "@/server/voice-synthesis/edge-adapter";
import { createLibraryRepository } from "@/server/library-repository";
import { mp3DurationMs } from "@/server/voice-synthesis/mp3-duration";
import { loadPresenterVoiceManifest } from "@/server/presenter-voice-manifest";

const ROLE_NAMES = ["狼人", "预言家", "女巫", "猎人", "守卫", "平民"];

async function main(): Promise<void> {
  const dataDir = process.env.KIVA_DATA_DIR ?? "kivdb";
  const adapter = createEdgeVoiceAdapter({ proxy: process.env.EDGE_TTS_PROXY });

  for (const presenter of await createLibraryRepository(dataDir).getPresenters()) {
    const profile = presenter.voiceProfile;
    const root = join(dataDir, "presenters", presenter.id);
    const voiceDir = join(root, "voice");
    await mkdir(voiceDir, { recursive: true });
    const previousManifest = await readExistingManifest(dataDir, presenter.id);
    if (profile.voice !== PRESENTER_EDGE_VOICE) {
      throw new Error(`Presenter ${presenter.id} must use ${PRESENTER_EDGE_VOICE}`);
    }

    const texts: Record<string, string> = {};
    const plans = {} as PresenterVoiceManifest["plans"];
    for (const [copyKey, line] of Object.entries(presenter.lines)) {
      const planned = presenterPlanForTemplate(
        copyKey as keyof typeof presenter.lines,
        line.template,
      );
      Object.assign(texts, planned.literalClips);
      (plans as Record<string, typeof planned.tokens>)[copyKey] = planned.tokens;
    }
    for (let seat = 1; seat <= 12; seat += 1) {
      texts[`seat.${seat}`] = `${seat}号`;
    }
    for (const role of ROLE_NAMES) texts[`role.${role}`] = role;

    const clips: Record<string, { file: string; text: string; durationMs: number }> = {};
    for (const [clipId, text] of Object.entries(texts)) {
      const safeClipId = /^[a-zA-Z0-9_.-]+$/.test(clipId)
        ? clipId
        : `clip-${createHash("sha256").update(clipId).digest("hex").slice(0, 20)}`;
      const fingerprint = createHash("sha256")
        .update(JSON.stringify({ profile, text }))
        .digest("hex")
        .slice(0, 12);
      const file = `${safeClipId}-${fingerprint}.mp3`;
      const target = join(voiceDir, file);
      const temp = `${target}.tmp`;
      const previousClip = previousManifest?.clips[clipId];
      const reusable = sameProfile(previousManifest?.profile, profile) &&
        previousClip?.text === text;
      const previousPath = previousClip ? join(voiceDir, previousClip.file) : null;
      const existingDuration = reusable && previousPath
        ? await durationOfExisting(previousPath)
        : null;
      if (existingDuration !== null && previousPath) {
        if (previousPath !== target) await copyFile(previousPath, target);
        clips[clipId] = { file, text, durationMs: existingDuration };
        process.stdout.write(`reused ${presenter.id}/${clipId}\n`);
        continue;
      }
      const result = await synthesizeWithRetry(text, profile, temp);
      await rename(temp, target);
      clips[clipId] = {
        file,
        text: result.synthesizedText,
        durationMs: result.durationMs,
      };
      process.stdout.write(`built ${presenter.id}/${clipId}\n`);
    }

    const manifest: PresenterVoiceManifest = {
      schemaVersion: 1,
      presenterId: presenter.id,
      profile,
      clips,
      plans,
    };
    await writeFile(join(root, "manifest.json"), `${JSON.stringify(manifest, null, 2)}\n`, "utf8");
    const declaredFiles = new Set(Object.values(clips).map((clip) => clip.file));
    for (const file of await readdir(voiceDir)) {
      if (!declaredFiles.has(file)) await unlink(join(voiceDir, file));
    }
    async function synthesizeWithRetry(
      text: string,
      voiceProfile: VoiceProfileSnapshot,
      outputPath: string,
    ) {
      let lastError: unknown;
      for (let attempt = 1; attempt <= 3; attempt += 1) {
        try {
          return await adapter.synthesize({
            sourceText: text,
            profile: voiceProfile,
            outputPath,
          });
        } catch (error) {
          lastError = error;
          process.stderr.write(
            `Edge TTS attempt ${attempt} failed: ${String(error)}\n`,
          );
        }
      }
      throw lastError;
    }
  }

  async function durationOfExisting(path: string): Promise<number | null> {
    try {
      if ((await stat(path)).size === 0) return null;
      const duration = Math.ceil(mp3DurationMs(await readFile(path)));
      return duration > 0 ? duration : null;
    } catch {
      return null;
    }
  }

  async function readExistingManifest(
    rootDir: string,
    presenterId: string,
  ): Promise<PresenterVoiceManifest | null> {
    try {
      return await loadPresenterVoiceManifest(rootDir, presenterId);
    } catch {
      return null;
    }
  }

  function sameProfile(
    left: VoiceProfileSnapshot | undefined,
    right: VoiceProfileSnapshot,
  ): boolean {
    return left !== undefined &&
      left.provider === right.provider &&
      left.adapterVersion === right.adapterVersion &&
      left.voice === right.voice &&
      left.lang === right.lang &&
      left.pitch === right.pitch &&
      left.rate === right.rate &&
      left.volume === right.volume;
  }
}

void main();
