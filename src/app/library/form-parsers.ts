import type { ActorDefinition } from "@/core/actor-definition";
import type { GameScriptDefinition } from "@/core/game-script";
import type { Lineup } from "@/core/lineup";
import type { ModelBindingSnapshot } from "@/core/model-binding";
import {
  PRESENTER_LINE_VARIABLES,
  type PresenterCopyKey,
  type PresenterDefinition,
  type PresenterLineCatalog,
} from "@/core/presenter-definition";
import type { RuleRoleId } from "@/core/rule-role";
import { edgeVoiceProfile } from "@/core/voice";

export function actorFromFormData(formData: FormData): ActorDefinition {
  const id = text(formData, "id");
  const name = text(formData, "identity.name");
  if ((id === "qin_chuan") !== (name === "秦川")) {
    throw new Error('稳定 Actor 必须保持 "秦川" / qin_chuan');
  }
  return {
    id,
    identity: {
      name,
      portrait: text(formData, "identity.portrait"),
      tags: list(formData, "identity.tags"),
      visualAnchor: text(formData, "identity.visualAnchor"),
    },
    core: {
      stableCore: text(formData, "core.stableCore"),
      drive: text(formData, "core.drive"),
      blindSpot: text(formData, "core.blindSpot"),
      changeBoundary: text(formData, "core.changeBoundary"),
    },
    cognition: {
      attention: text(formData, "cognition.attention"),
      evidencePolicy: text(formData, "cognition.evidencePolicy"),
      decisionPolicy: text(formData, "cognition.decisionPolicy"),
      correctionTrigger: text(formData, "cognition.correctionTrigger"),
    },
    interaction: {
      tableFunction: text(formData, "interaction.tableFunction"),
      socialStrategy: text(formData, "interaction.socialStrategy"),
      pressureResponse: text(formData, "interaction.pressureResponse"),
      conflictAxes: list(formData, "interaction.conflictAxes"),
    },
    expression: {
      cadence: text(formData, "expression.cadence"),
      diction: text(formData, "expression.diction"),
      rhetoricalMoves: list(formData, "expression.rhetoricalMoves"),
      avoid: list(formData, "expression.avoid"),
    },
    production: {
      modelBinding: modelBinding(formData),
      voice: edgeVoiceProfile(text(formData, "production.voice.voice"), {
        rate: text(formData, "production.voice.rate"),
        pitch: text(formData, "production.voice.pitch"),
        volume: text(formData, "production.voice.volume"),
      }),
    },
    enabled: checkbox(formData, "enabled"),
    revision: integer(formData, "revision") + 1,
  };
}

export function lineupFromFormData(formData: FormData): Lineup {
  return {
    id: text(formData, "id"),
    name: text(formData, "name"),
    rulesetId: "classic_twelve",
    seats: Array.from({ length: 12 }, (_, index) => {
      const seatNo = index + 1;
      return {
        seatNo,
        ruleRoleId: text(
          formData,
          `seat.${seatNo}.ruleRoleId`,
        ) as RuleRoleId,
        actorId: text(formData, `seat.${seatNo}.actorId`),
      };
    }),
    enabled: checkbox(formData, "enabled"),
    revision: integer(formData, "revision") + 1,
  };
}

export function scriptFromFormData(
  formData: FormData,
  now: string,
): GameScriptDefinition {
  return {
    id: text(formData, "id"),
    name: text(formData, "name"),
    theme: text(formData, "theme"),
    background: text(formData, "background"),
    atmosphere: list(formData, "atmosphere"),
    presentation: {
      styleKey: "midnight_archive_v1",
      coverImage: text(formData, "presentation.coverImage"),
      dayBackground: text(formData, "presentation.dayBackground"),
      nightBackground: text(formData, "presentation.nightBackground"),
      colors: {
        ink: text(formData, "presentation.colors.ink"),
        paper: text(formData, "presentation.colors.paper"),
        accent: text(formData, "presentation.colors.accent"),
        signal: text(formData, "presentation.colors.signal"),
        night: text(formData, "presentation.colors.night"),
      },
    },
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

function modelBinding(formData: FormData): ModelBindingSnapshot {
  return {
    provider: text(formData, "production.modelBinding.provider"),
    model: text(formData, "production.modelBinding.model"),
    responseFormat: "json",
    ...(hasText(formData, "production.modelBinding.fallbackModel")
      ? {
          fallbackModel: text(
            formData,
            "production.modelBinding.fallbackModel",
          ),
        }
      : {}),
  };
}

function presenterLinesFromFormData(formData: FormData): PresenterLineCatalog {
  return Object.fromEntries(
    (Object.keys(PRESENTER_LINE_VARIABLES) as PresenterCopyKey[]).map((key) => [
      key,
      {
        template: text(formData, `line.${key}.template`),
        variables: PRESENTER_LINE_VARIABLES[key],
      },
    ]),
  ) as unknown as PresenterLineCatalog;
}

function text(formData: FormData, key: string): string {
  const value = formData.get(key);
  return typeof value === "string" ? value.trim() : "";
}

function textOrDefault(
  formData: FormData,
  key: string,
  fallback: string,
): string {
  return text(formData, key) || fallback;
}

function nullableText(formData: FormData, key: string): string | null {
  return text(formData, key) || null;
}

function integer(formData: FormData, key: string): number {
  const value = Number(text(formData, key));
  if (!Number.isInteger(value)) throw new Error(`${key} must be an integer`);
  return value;
}

function checkbox(formData: FormData, key: string): boolean {
  const value = formData.get(key);
  return value === "on" || value === "true" || value === "1";
}

function list(formData: FormData, key: string): readonly string[] {
  const value = text(formData, key);
  if (!value) return [];
  return value
    .split(/[，,\n]/)
    .map((item) => item.trim())
    .filter(Boolean);
}

function hasText(formData: FormData, key: string): boolean {
  return text(formData, key).length > 0;
}
