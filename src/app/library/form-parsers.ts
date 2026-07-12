import type { CharacterDefinition } from "@/core/character-definition";
import {
  type GamePreset,
  type GamePresetSeatAssignment,
} from "@/core/game-preset";
import { validateModelBindingSnapshot } from "@/core/model-binding";
import type { ModelBindingSnapshot } from "@/core/player";
import { edgeVoiceProfile } from "@/core/voice";
import {
  PRESENTER_LINE_VARIABLES,
  type PresenterCopyKey,
  type PresenterDefinition,
  type PresenterLineCatalog,
} from "@/core/presenter-definition";
import type {
  RoleDefinition,
  RoleKnowledgeRule,
} from "@/core/role-definition";

export function roleFromFormData(formData: FormData, now: string): RoleDefinition {
  return {
    id: text(formData, "id"),
    name: text(formData, "name"),
    enabled: checkbox(formData, "enabled"),
    faction: text(formData, "faction") as RoleDefinition["faction"],
    team: text(formData, "team") as RoleDefinition["team"],
    mechanicKey: text(formData, "mechanicKey") as RoleDefinition["mechanicKey"],
    visibilityRules: list(formData, "visibilityRules") as RoleKnowledgeRule[],
    nightOrder: nullableNumber(formData, "nightOrder"),
    systemPrompt: text(formData, "systemPrompt"),
    actionPrompt: nullableText(formData, "actionPrompt"),
    defaultModelBinding: nullableModelBinding(formData, "defaultModelBinding"),
    createdAt: textOrDefault(formData, "createdAt", now),
    updatedAt: now,
  };
}

export function characterFromFormData(
  formData: FormData,
  now: string,
): CharacterDefinition {
  return {
    id: text(formData, "id"),
    name: text(formData, "name"),
    avatar: nullableText(formData, "avatar"),
    tags: list(formData, "tags"),
    persona: text(formData, "persona"),
    speakingStyle: text(formData, "speakingStyle"),
    reasoningStyle: text(formData, "reasoningStyle"),
    systemPrompt: text(formData, "systemPrompt"),
    defaultModelBinding: characterModelBindingFromFormData(formData),
    voiceProfile: edgeVoiceProfile(
      textOrDefault(formData, "voiceProfile.voice", "zh-CN-XiaoxiaoNeural"),
      {
        pitch: textOrDefault(formData, "voiceProfile.pitch", "+0Hz"),
        rate: textOrDefault(formData, "voiceProfile.rate", "+0%"),
        volume: textOrDefault(formData, "voiceProfile.volume", "+0%"),
      },
    ),
    enabled: checkbox(formData, "enabled"),
    createdAt: textOrDefault(formData, "createdAt", now),
    updatedAt: now,
  };
}

export function presetFromFormData(formData: FormData, now: string): GamePreset {
  const playerCount = number(formData, "playerCount");
  const seatAssignments = seatAssignmentsFromFormData(formData, playerCount);

  return {
    id: text(formData, "id"),
    name: text(formData, "name"),
    rulesetId: text(formData, "rulesetId"),
    playerCount,
    roleIds: listOrDefault(
      formData,
      "roleIds",
      seatAssignments.map((assignment) => assignment.roleId),
    ),
    characterIds: listOrDefault(
      formData,
      "characterIds",
      seatAssignments.map((assignment) => assignment.characterId),
    ),
    seatAssignments,
    enabled: checkbox(formData, "enabled"),
    createdAt: textOrDefault(formData, "createdAt", now),
    updatedAt: now,
  };
}

export function presenterFromFormData(
  formData: FormData,
  now: string,
): PresenterDefinition {
  return {
    id: text(formData, "id"),
    name: text(formData, "name"),
    avatar: nullableText(formData, "avatar"),
    voiceProfile: edgeVoiceProfile(
      textOrDefault(formData, "voiceProfile.voice", "zh-CN-XiaoxiaoNeural"),
      {
        pitch: textOrDefault(formData, "voiceProfile.pitch", "+0Hz"),
        rate: textOrDefault(formData, "voiceProfile.rate", "+0%"),
        volume: textOrDefault(formData, "voiceProfile.volume", "+0%"),
      },
    ),
    lines: presenterLinesFromFormData(formData),
    enabled: checkbox(formData, "enabled"),
    createdAt: textOrDefault(formData, "createdAt", now),
    updatedAt: now,
  };
}

function presenterLinesFromFormData(formData: FormData): PresenterLineCatalog {
  const entries = (Object.keys(PRESENTER_LINE_VARIABLES) as PresenterCopyKey[])
    .map((key) => {
      const template = text(formData, `line.${key}.template`);
      const variables = PRESENTER_LINE_VARIABLES[key];
      return [
        key,
        {
          template,
          variables,
        },
      ] as const;
    });

  return Object.fromEntries(entries) as unknown as PresenterLineCatalog;
}

function seatAssignmentsFromFormData(
  formData: FormData,
  playerCount: number,
): readonly GamePresetSeatAssignment[] {
  if (hasText(formData, "seatAssignments")) {
    const parsed = JSON.parse(text(formData, "seatAssignments"));
    if (!Array.isArray(parsed)) {
      throw new Error("seatAssignments must be a JSON array");
    }
    return parsed.map((assignment) => {
      const seat = assignment as Partial<GamePresetSeatAssignment>;
      const nextAssignment: GamePresetSeatAssignment = {
        seatNo: Number(seat.seatNo),
        roleId: String(seat.roleId ?? ""),
        characterId: String(seat.characterId ?? ""),
        modelBindingOverride: modelBindingFromUnknown(
          seat.modelBindingOverride ?? null,
          "modelBindingOverride",
        ),
      };
      return nextAssignment;
    });
  }

  return Array.from({ length: playerCount }, (_, index) => {
    const seatNo = index + 1;
    return {
      seatNo,
      roleId: text(formData, `seat.${seatNo}.roleId`),
      characterId: text(formData, `seat.${seatNo}.characterId`),
      modelBindingOverride: nullableModelBinding(
        formData,
        `seat.${seatNo}.modelBindingOverride`,
      ),
    };
  });
}

function text(formData: FormData, key: string): string {
  const value = formData.get(key);
  return typeof value === "string" ? value.trim() : "";
}

function textOrDefault(
  formData: FormData,
  key: string,
  defaultValue: string,
): string {
  return text(formData, key) || defaultValue;
}

function nullableText(formData: FormData, key: string): string | null {
  const value = text(formData, key);
  return value === "" ? null : value;
}

function number(formData: FormData, key: string): number {
  const value = Number(text(formData, key));
  if (!Number.isFinite(value)) {
    throw new Error(`${key} must be a finite number`);
  }
  return value;
}

function nullableNumber(formData: FormData, key: string): number | null {
  const value = text(formData, key);
  return value === "" ? null : number(formData, key);
}

function checkbox(formData: FormData, key: string): boolean {
  const value = formData.get(key);
  return value === "on" || value === "true" || value === "1";
}

function list(formData: FormData, key: string): readonly string[] {
  const value = text(formData, key);
  if (value === "") {
    return [];
  }

  if (value.startsWith("[")) {
    const parsed = JSON.parse(value);
    if (!Array.isArray(parsed)) {
      throw new Error(`${key} must be a JSON array`);
    }
    return parsed.map((item) => String(item).trim()).filter(Boolean);
  }

  return value
    .split(",")
    .map((item) => item.trim())
    .filter(Boolean);
}

function listOrDefault(
  formData: FormData,
  key: string,
  defaultValue: readonly string[],
): readonly string[] {
  return hasText(formData, key) ? list(formData, key) : defaultValue;
}

function nullableModelBinding(
  formData: FormData,
  key: string,
): ModelBindingSnapshot | null {
  if (!hasText(formData, key)) {
    return null;
  }

  return modelBindingFromUnknown(JSON.parse(text(formData, key)), key);
}

function characterModelBindingFromFormData(
  formData: FormData,
): ModelBindingSnapshot | null {
  if (
    formData.has("modelBinding.provider") ||
    formData.has("modelBinding.model") ||
    formData.has("modelBinding.responseFormat")
  ) {
    return modelBindingFromUnknown(
      {
        provider: text(formData, "modelBinding.provider"),
        model: text(formData, "modelBinding.model"),
        responseFormat: text(formData, "modelBinding.responseFormat"),
        ...(hasText(formData, "modelBinding.fallbackModel")
          ? { fallbackModel: text(formData, "modelBinding.fallbackModel") }
          : {}),
      },
      "modelBinding",
    );
  }

  return nullableModelBinding(formData, "defaultModelBinding");
}

function modelBindingFromUnknown(
  value: unknown,
  path: string,
): ModelBindingSnapshot | null {
  validateModelBindingSnapshot(value, path);
  return value as ModelBindingSnapshot | null;
}

function hasText(formData: FormData, key: string): boolean {
  return text(formData, key) !== "";
}
