import { readFile } from "node:fs/promises";
import { join } from "node:path";
import {
  PRESENTER_LINE_VARIABLES,
  type PresenterCopyKey,
} from "@/core/presenter-definition";
import {
  type PresenterVoiceManifest,
} from "@/core/presenter-voice";
import { assertExactObjectKeys, isPlainObject } from "@/core/model-binding";
import { validateVoiceProfileSnapshot } from "@/core/voice";

export async function loadPresenterVoiceManifest(
  dataDir: string,
  presenterId: string,
): Promise<PresenterVoiceManifest> {
  if (!/^[a-zA-Z0-9_-]+$/.test(presenterId)) {
    throw new Error("Invalid presenter identifier");
  }
  const value: unknown = JSON.parse(
    await readFile(join(dataDir, "presenters", presenterId, "manifest.json"), "utf8"),
  );
  if (!isPlainObject(value)) {
    throw new Error(`Presenter voice manifest is invalid: ${presenterId}`);
  }
  if (value.schemaVersion !== 1) {
    throw new Error(`Unsupported presenter voice manifest schema: ${presenterId}`);
  }
  assertExactObjectKeys(value, "Presenter voice manifest", [
    "schemaVersion",
    "presenterId",
    "profile",
    "clips",
    "plans",
  ]);
  if (
    value.presenterId !== presenterId ||
    !isPlainObject(value.clips) ||
    !isPlainObject(value.plans)
  ) {
    throw new Error(`Presenter voice manifest is invalid: ${presenterId}`);
  }
  validateVoiceProfileSnapshot(value.profile, `Presenter ${presenterId} voice profile`);
  for (const [clipId, clip] of Object.entries(value.clips)) {
    if (!isPlainObject(clip)) {
      throw new Error(`Presenter voice clip is invalid: ${clipId}`);
    }
    if (
      !hasExactKeys(clip, ["file", "text", "durationMs"]) ||
      !/^[\p{L}\p{N}_.-]+$/u.test(clipId) ||
      typeof clip.file !== "string" ||
      !/^[a-zA-Z0-9_.-]+\.mp3$/.test(clip.file) ||
      typeof clip.text !== "string" ||
      !clip.text.trim() ||
      typeof clip.durationMs !== "number" ||
      !Number.isFinite(clip.durationMs) ||
      clip.durationMs <= 0
    ) {
      throw new Error(`Presenter voice clip is invalid: ${clipId}`);
    }
  }
  const planKeys = Object.keys(PRESENTER_LINE_VARIABLES);
  if (!hasExactKeys(value.plans, planKeys)) {
    throw new Error(`Presenter voice plans are invalid: ${presenterId}`);
  }
  for (const key of planKeys as PresenterCopyKey[]) {
    const plan = value.plans[key];
    if (!Array.isArray(plan) || plan.some((token) => !isPlanToken(token))) {
      throw new Error(`Presenter voice plan is invalid: ${key}`);
    }
  }
  return structuredClone(value) as PresenterVoiceManifest;
}

function isPlanToken(value: unknown): boolean {
  if (!isPlainObject(value)) return false;
  switch (value.kind) {
    case "clip":
      return hasExactKeys(value, ["kind", "clipId"]) && isNonBlank(value.clipId);
    case "seat-variant":
      return (
        hasExactKeys(value, ["kind", "clipIdPrefix", "variableName"]) &&
        isNonBlank(value.clipIdPrefix) &&
        isNonBlank(value.variableName)
      );
    case "variable":
      return hasExactKeys(value, ["kind", "name"]) && isNonBlank(value.name);
    default:
      return false;
  }
}

function hasExactKeys(
  value: Record<string, unknown>,
  keys: readonly string[],
): boolean {
  return Object.keys(value).length === keys.length && keys.every((key) => key in value);
}

function isNonBlank(value: unknown): value is string {
  return typeof value === "string" && value.trim().length > 0;
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
