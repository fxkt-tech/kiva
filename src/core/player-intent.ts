import { assertExactObjectKeys, isPlainObject } from "./model-binding";
import type { PlayerContextTimelineItem } from "./player-context";

export type PlayerSpeechEvidenceScope = {
  readonly publicFacts: readonly PlayerContextTimelineItem[];
  readonly publicClaims: readonly PlayerContextTimelineItem[];
  readonly privateFacts: readonly PlayerContextTimelineItem[];
  readonly factionDiscussion: readonly PlayerContextTimelineItem[];
};

export type PlayerSpeechIntent = {
  readonly objective: string;
  readonly conclusion: string;
  readonly evidenceEventIndexes: readonly number[];
  readonly uncertainty: string | null;
  readonly disclosure: "conceal" | "claim" | "not_applicable";
  readonly intendedEffect: string;
};

export function validatePlayerSpeechIntent(input: {
  readonly value: unknown;
  readonly evidenceScope: PlayerSpeechEvidenceScope;
  readonly publicSpeech: boolean;
  readonly requireDisclosure: boolean;
}): PlayerSpeechIntent {
  const value = input.value;
  if (!isPlainObject(value)) {
    throw new Error("Player speech intent must be an object");
  }
  assertExactObjectKeys(value, "Player speech intent", [
    "objective",
    "conclusion",
    "evidenceEventIndexes",
    "uncertainty",
    "disclosure",
    "intendedEffect",
  ]);
  const disclosure = value.disclosure;
  if (
    disclosure !== "conceal" &&
    disclosure !== "claim" &&
    disclosure !== "not_applicable"
  ) {
    throw new Error("Player speech intent disclosure is invalid");
  }
  if (input.requireDisclosure && disclosure === "not_applicable") {
    throw new Error("Special-role public intent must decide disclosure");
  }
  if (!input.requireDisclosure && disclosure !== "not_applicable") {
    throw new Error("This speech intent must use disclosure=not_applicable");
  }
  const objective = boundedText(value.objective, "objective", 160);
  const conclusion = boundedText(value.conclusion, "conclusion", 240);
  const intendedEffect = boundedText(
    value.intendedEffect,
    "intendedEffect",
    160,
  );
  const uncertainty =
    value.uncertainty === null
      ? null
      : boundedText(value.uncertainty, "uncertainty", 160);
  if (
    !Array.isArray(value.evidenceEventIndexes) ||
    value.evidenceEventIndexes.length > 3 ||
    value.evidenceEventIndexes.some(
      (index) => !Number.isInteger(index) || (index as number) < 1,
    ) ||
    new Set(value.evidenceEventIndexes).size !==
      value.evidenceEventIndexes.length
  ) {
    throw new Error("Player speech intent evidenceEventIndexes is invalid");
  }
  const evidenceEventIndexes = value.evidenceEventIndexes as number[];
  const allowed = new Set(
    allowedEvidence(input.evidenceScope, input.publicSpeech, disclosure).map(
      (item) => item.index,
    ),
  );
  if (evidenceEventIndexes.some((index) => !allowed.has(index))) {
    throw new Error("Player speech intent selected evidence outside visibility");
  }
  return {
    objective,
    conclusion,
    evidenceEventIndexes: [...evidenceEventIndexes],
    uncertainty,
    disclosure,
    intendedEffect,
  };
}

export function selectedIntentEvidence(input: {
  readonly intent: PlayerSpeechIntent;
  readonly evidenceScope: PlayerSpeechEvidenceScope;
  readonly publicSpeech: boolean;
}): readonly PlayerContextTimelineItem[] {
  const byIndex = new Map(
    allowedEvidence(
      input.evidenceScope,
      input.publicSpeech,
      input.intent.disclosure,
    ).map((item) => [item.index, item]),
  );
  return input.intent.evidenceEventIndexes.map((index) => {
    const item = byIndex.get(index);
    if (!item) throw new Error(`Selected intent evidence is unavailable: ${index}`);
    return item;
  });
}

function allowedEvidence(
  scope: PlayerSpeechEvidenceScope,
  publicSpeech: boolean,
  disclosure: PlayerSpeechIntent["disclosure"],
): readonly PlayerContextTimelineItem[] {
  const publicItems = [
    ...scope.publicFacts,
    ...scope.publicClaims,
  ];
  if (publicSpeech) {
    return disclosure === "claim"
      ? unique([...publicItems, ...scope.privateFacts])
      : unique(publicItems);
  }
  return unique([
    ...publicItems,
    ...scope.privateFacts,
    ...scope.factionDiscussion,
  ]);
}

function unique(
  items: readonly PlayerContextTimelineItem[],
): readonly PlayerContextTimelineItem[] {
  return [...new Map(items.map((item) => [item.index, item])).values()];
}

function boundedText(value: unknown, field: string, max: number): string {
  if (
    typeof value !== "string" ||
    value.trim().length === 0 ||
    value !== value.trim() ||
    value.length > max
  ) {
    throw new Error(`Player speech intent ${field} is invalid`);
  }
  return value;
}
