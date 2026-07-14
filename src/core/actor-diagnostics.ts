import type { ActorDefinition } from "./actor-definition";

export type ActorComparisonRow = {
  readonly actorId: string;
  readonly name: string;
  readonly enabled: boolean;
  readonly tableFunction: string;
  readonly attention: string;
  readonly decisionPolicy: string;
  readonly pressureResponse: string;
  readonly cadence: string;
  readonly conflictAxes: readonly string[];
  readonly model: string;
  readonly voice: string;
};

export type ActorSimilarityPair = {
  readonly actorIds: readonly [string, string];
  readonly score: number;
};

export type ActorPoolDiagnostics = {
  readonly enabledActorCount: number;
  readonly distinctTableFunctionCount: number;
  readonly distinctConflictAxisCount: number;
  readonly distinctVoiceCount: number;
  readonly voiceUsage: Readonly<Record<string, readonly string[]>>;
  readonly nearDuplicatePairs: readonly ActorSimilarityPair[];
  readonly hardIssues: readonly string[];
};

export type ActorSelectionDiagnostics = {
  readonly selectedActorCount: number;
  readonly uniqueActorCount: number;
  readonly distinctTableFunctionCount: number;
  readonly distinctConflictAxisCount: number;
  readonly distinctVoiceCount: number;
  readonly hardIssues: readonly string[];
  readonly nearDuplicatePairs: readonly ActorSimilarityPair[];
};

const MIN_ENABLED_ACTORS = 12;
const MIN_POOL_VOICES = 6;
const NEAR_DUPLICATE_THRESHOLD = 0.72;

export function compileActorComparisonMatrix(
  actors: readonly ActorDefinition[],
): readonly ActorComparisonRow[] {
  return actors.map((actor) => ({
    actorId: actor.id,
    name: actor.identity.name,
    enabled: actor.enabled,
    tableFunction: actor.interaction.tableFunction,
    attention: actor.cognition.attention,
    decisionPolicy: actor.cognition.decisionPolicy,
    pressureResponse: actor.interaction.pressureResponse,
    cadence: actor.expression.cadence,
    conflictAxes: [...actor.interaction.conflictAxes],
    model: `${actor.production.modelBinding.provider}/${actor.production.modelBinding.model}`,
    voice: actor.production.voice.voice,
  }));
}

export function diagnoseActorPool(
  actors: readonly ActorDefinition[],
): ActorPoolDiagnostics {
  const enabled = actors.filter((actor) => actor.enabled);
  const voiceUsage = groupIds(enabled, (actor) => actor.production.voice.voice);
  const hardIssues: string[] = [];
  if (enabled.length < MIN_ENABLED_ACTORS) {
    hardIssues.push(
      `At least ${MIN_ENABLED_ACTORS} enabled Actors are required; found ${enabled.length}`,
    );
  }
  if (Object.keys(voiceUsage).length < MIN_POOL_VOICES) {
    hardIssues.push(
      `At least ${MIN_POOL_VOICES} distinct Actor voices are required; found ${Object.keys(voiceUsage).length}`,
    );
  }
  return {
    enabledActorCount: enabled.length,
    distinctTableFunctionCount: distinctCount(
      enabled.map((actor) => actor.interaction.tableFunction),
    ),
    distinctConflictAxisCount: distinctCount(
      enabled.flatMap((actor) => actor.interaction.conflictAxes),
    ),
    distinctVoiceCount: Object.keys(voiceUsage).length,
    voiceUsage,
    nearDuplicatePairs: nearDuplicatePairs(enabled),
    hardIssues,
  };
}

export function diagnoseActorSelection(input: {
  readonly actors: readonly ActorDefinition[];
  readonly actorIds: readonly string[];
}): ActorSelectionDiagnostics {
  const actorsById = new Map(input.actors.map((actor) => [actor.id, actor]));
  const selected = input.actorIds.flatMap((id) => {
    const actor = actorsById.get(id);
    return actor ? [actor] : [];
  });
  const uniqueIds = new Set(input.actorIds);
  const hardIssues: string[] = [];
  if (input.actorIds.length !== 12) {
    hardIssues.push(`A Lineup must select exactly 12 Actors; found ${input.actorIds.length}`);
  }
  if (uniqueIds.size !== input.actorIds.length) {
    hardIssues.push("A Lineup cannot select the same Actor more than once");
  }
  const missing = [...uniqueIds].filter((id) => !actorsById.has(id));
  if (missing.length > 0) {
    hardIssues.push(`Unknown Actors: ${missing.join(", ")}`);
  }
  const disabled = selected.filter((actor) => !actor.enabled).map((actor) => actor.id);
  if (disabled.length > 0) {
    hardIssues.push(`Disabled Actors: ${disabled.join(", ")}`);
  }
  return {
    selectedActorCount: input.actorIds.length,
    uniqueActorCount: uniqueIds.size,
    distinctTableFunctionCount: distinctCount(
      selected.map((actor) => actor.interaction.tableFunction),
    ),
    distinctConflictAxisCount: distinctCount(
      selected.flatMap((actor) => actor.interaction.conflictAxes),
    ),
    distinctVoiceCount: distinctCount(
      selected.map((actor) => actor.production.voice.voice),
    ),
    hardIssues,
    nearDuplicatePairs: nearDuplicatePairs(selected),
  };
}

function nearDuplicatePairs(
  actors: readonly ActorDefinition[],
): readonly ActorSimilarityPair[] {
  const pairs: ActorSimilarityPair[] = [];
  for (let leftIndex = 0; leftIndex < actors.length; leftIndex += 1) {
    for (let rightIndex = leftIndex + 1; rightIndex < actors.length; rightIndex += 1) {
      const left = actors[leftIndex]!;
      const right = actors[rightIndex]!;
      const score = behaviorSimilarity(left, right);
      if (score >= NEAR_DUPLICATE_THRESHOLD) {
        pairs.push({
          actorIds: [left.id, right.id],
          score: Number(score.toFixed(3)),
        });
      }
    }
  }
  return pairs.sort(
    (left, right) =>
      right.score - left.score ||
      left.actorIds.join(":").localeCompare(right.actorIds.join(":")),
  );
}

function behaviorSimilarity(
  left: ActorDefinition,
  right: ActorDefinition,
): number {
  const leftGrams = characterBigrams(behaviorText(left));
  const rightGrams = characterBigrams(behaviorText(right));
  const union = new Set([...leftGrams, ...rightGrams]);
  if (union.size === 0) return 0;
  let intersection = 0;
  for (const gram of leftGrams) {
    if (rightGrams.has(gram)) intersection += 1;
  }
  return intersection / union.size;
}

function behaviorText(actor: ActorDefinition): string {
  return [
    actor.core.stableCore,
    actor.core.drive,
    actor.core.blindSpot,
    actor.cognition.attention,
    actor.cognition.evidencePolicy,
    actor.cognition.decisionPolicy,
    actor.cognition.correctionTrigger,
    actor.interaction.tableFunction,
    actor.interaction.socialStrategy,
    actor.interaction.pressureResponse,
    ...actor.interaction.conflictAxes,
    actor.expression.cadence,
    actor.expression.diction,
    ...actor.expression.rhetoricalMoves,
    ...actor.expression.avoid,
  ].join("");
}

function characterBigrams(value: string): ReadonlySet<string> {
  const normalized = [...value.toLowerCase()]
    .filter((character) => /[\p{L}\p{N}]/u.test(character))
    .join("");
  const grams = new Set<string>();
  for (let index = 0; index < normalized.length - 1; index += 1) {
    grams.add(normalized.slice(index, index + 2));
  }
  return grams;
}

function distinctCount(values: readonly string[]): number {
  return new Set(values).size;
}

function groupIds(
  actors: readonly ActorDefinition[],
  keyFor: (actor: ActorDefinition) => string,
): Readonly<Record<string, readonly string[]>> {
  const groups = new Map<string, string[]>();
  for (const actor of actors) {
    const key = keyFor(actor);
    groups.set(key, [...(groups.get(key) ?? []), actor.id]);
  }
  return Object.fromEntries(
    [...groups.entries()].sort(([left], [right]) => left.localeCompare(right)),
  );
}
