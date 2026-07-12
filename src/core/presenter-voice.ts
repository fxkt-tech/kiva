import type { PresenterCopyKey } from "./presenter-definition";
import type { PresenterCue } from "./presenter";
import type { VoiceProfileSnapshot } from "./voice";

export type PresenterVoicePlanToken =
  | { readonly kind: "clip"; readonly clipId: string }
  | {
      readonly kind: "seat-variant";
      readonly clipIdPrefix: string;
      readonly variableName: string;
    }
  | { readonly kind: "variable"; readonly name: string };

export type PresenterVoiceManifest = {
  readonly schemaVersion: 1;
  readonly presenterId: string;
  readonly profile: VoiceProfileSnapshot;
  readonly clips: Readonly<
    Record<string, { readonly file: string; readonly text: string; readonly durationMs: number }>
  >;
  readonly plans: Readonly<Record<PresenterCopyKey, readonly PresenterVoicePlanToken[]>>;
};

export type ResolvedPresenterVoiceClip = {
  readonly clipId: string;
  readonly file: string;
  readonly durationMs: number;
};

export function resolvePresenterVoiceClips(
  manifest: PresenterVoiceManifest,
  cue: PresenterCue,
): readonly ResolvedPresenterVoiceClip[] {
  return (manifest.plans[cue.copyKey] ?? []).flatMap((token) => {
    const clipIds = token.kind === "clip"
      ? [token.clipId]
      : token.kind === "seat-variant"
        ? [`${token.clipIdPrefix}.${seatNumberForValue(
            token.variableName,
            cue.values?.[token.variableName] ?? "",
          )}`]
        : clipIdsForValue(token.name, cue.values?.[token.name] ?? "");
    return clipIds.map((clipId) => {
      const clip = manifest.clips[clipId];
      if (!clip) throw new Error(`Presenter voice clip is missing: ${clipId}`);
      return { clipId, file: clip.file, durationMs: clip.durationMs };
    });
  });
}

function clipIdsForValue(name: string, value: string): readonly string[] {
  if (name === "role") return [`role.${value}`];
  if (name === "players") {
    return Array.from(value.matchAll(/(\d+)号/gu), (match) => `seat.${match[1]}`);
  }
  if (name === "seatNo") return value ? [`seat-number.${value}`] : [];
  const seat = value.match(/(\d+)号/u)?.[1] ?? null;
  return seat ? [`seat.${seat}`] : [];
}

export function presenterPlanForTemplate(
  copyKey: PresenterCopyKey,
  template: string,
): {
  readonly tokens: readonly PresenterVoicePlanToken[];
  readonly literalClips: Readonly<Record<string, string>>;
} {
  const placeholders = Array.from(
    template.matchAll(/\{([a-zA-Z][a-zA-Z0-9]*)\}/g),
    (match) => match[1]!,
  );
  const seatVariable = placeholders.length === 1 &&
    ["seatNo", "player", "target", "voter"].includes(placeholders[0]!)
    ? placeholders[0]!
    : null;
  if (seatVariable) {
    const clipIdPrefix = `line.${safeKey(copyKey)}.seat`;
    return {
      tokens: [{ kind: "seat-variant", clipIdPrefix, variableName: seatVariable }],
      literalClips: Object.fromEntries(
        Array.from({ length: 12 }, (_, index) => {
          const seatNo = index + 1;
          return [
            `${clipIdPrefix}.${seatNo}`,
            template.replace(
              `{${seatVariable}}`,
              seatVariable === "seatNo" ? String(seatNo) : `${seatNo}号`,
            ),
          ];
        }),
      ),
    };
  }
  const tokens: PresenterVoicePlanToken[] = [];
  const literalClips: Record<string, string> = {};
  let cursor = 0;
  let literalIndex = 0;
  for (const match of template.matchAll(/\{([a-zA-Z][a-zA-Z0-9]*)\}/g)) {
    const index = match.index ?? 0;
    const literal = template.slice(cursor, index).trim();
    if (hasSpokenContent(literal)) {
      const clipId = `line.${safeKey(copyKey)}.${literalIndex++}`;
      literalClips[clipId] = literal;
      tokens.push({ kind: "clip", clipId });
    }
    tokens.push({ kind: "variable", name: match[1]! });
    cursor = index + match[0].length;
  }
  const tail = template.slice(cursor).trim();
  if (hasSpokenContent(tail)) {
    const clipId = `line.${safeKey(copyKey)}.${literalIndex}`;
    literalClips[clipId] = tail;
    tokens.push({ kind: "clip", clipId });
  }
  return { tokens, literalClips };
}

function seatNumberForValue(name: string, value: string): string {
  if (name === "seatNo") return value;
  return value.match(/(\d+)号/u)?.[1] ?? "";
}

function hasSpokenContent(value: string): boolean {
  return /[\p{L}\p{N}]/u.test(value);
}

function safeKey(value: string): string {
  return value.replace(/[^a-zA-Z0-9_-]/g, "_");
}
