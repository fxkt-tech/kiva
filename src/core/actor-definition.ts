import {
  assertExactObjectKeys,
  isPlainObject,
  validateModelBindingSnapshot,
  type ModelBindingSnapshot,
} from "./model-binding";
import {
  validateVoiceProfileSnapshot,
  type VoiceProfileSnapshot,
} from "./voice";

export type ActorIdentity = {
  readonly name: string;
  readonly portrait: string;
  readonly tags: readonly string[];
  readonly visualAnchor: string;
};

export type ActorCore = {
  readonly stableCore: string;
  readonly drive: string;
  readonly blindSpot: string;
  readonly changeBoundary: string;
};

export type ActorCognition = {
  readonly attention: string;
  readonly evidencePolicy: string;
  readonly decisionPolicy: string;
  readonly correctionTrigger: string;
};

export type ActorInteraction = {
  readonly tableFunction: string;
  readonly socialStrategy: string;
  readonly pressureResponse: string;
  readonly conflictAxes: readonly string[];
};

export type ActorExpression = {
  readonly cadence: string;
  readonly diction: string;
  readonly rhetoricalMoves: readonly string[];
  readonly avoid: readonly string[];
};

export type ActorProduction = {
  readonly modelBinding: ModelBindingSnapshot;
  readonly voice: VoiceProfileSnapshot;
};

export type ActorDefinition = {
  readonly id: string;
  readonly identity: ActorIdentity;
  readonly core: ActorCore;
  readonly cognition: ActorCognition;
  readonly interaction: ActorInteraction;
  readonly expression: ActorExpression;
  readonly production: ActorProduction;
  readonly enabled: boolean;
  readonly revision: number;
};

export type ActorSnapshot = {
  readonly sourceId: string;
  readonly revision: number;
  readonly identity: ActorIdentity;
  readonly core: ActorCore;
  readonly cognition: ActorCognition;
  readonly interaction: ActorInteraction;
  readonly expression: ActorExpression;
  readonly production: ActorProduction;
};

export type ActorRuntimeCard = {
  readonly actorId: string;
  readonly name: string;
  readonly core: Pick<ActorCore, "stableCore" | "drive">;
  readonly cognition: ActorCognition;
  readonly interaction: Pick<
    ActorInteraction,
    "socialStrategy" | "pressureResponse"
  >;
  readonly expression: ActorExpression;
};

export type ActorAuthorCard = {
  readonly actorId: string;
  readonly identity: Pick<ActorIdentity, "name" | "tags">;
  readonly core: Pick<
    ActorCore,
    "stableCore" | "blindSpot" | "changeBoundary"
  >;
  readonly tableFunction: string;
  readonly pressureResponse: string;
  readonly correctionTrigger: string;
  readonly conflictAxes: readonly string[];
};

const ACTOR_FIELDS = [
  "id",
  "identity",
  "core",
  "cognition",
  "interaction",
  "expression",
  "production",
  "enabled",
  "revision",
] as const;

const ACTOR_SNAPSHOT_FIELDS = [
  "sourceId",
  "revision",
  "identity",
  "core",
  "cognition",
  "interaction",
  "expression",
  "production",
] as const;

const MAX_CLAUSE_LENGTH = 240;
const MAX_LIST_ITEM_LENGTH = 80;

export function validateActorDefinitions(
  value: unknown,
): readonly ActorDefinition[] {
  if (!Array.isArray(value)) {
    throw new Error("Actor definitions must be an array");
  }

  const ids = new Set<string>();
  const actors = value.map((actor, index) => {
    const path = `Actor definition[${index}]`;
    if (!isPlainObject(actor)) {
      throw new Error(`${path} must be an object`);
    }
    assertExactObjectKeys(actor, path, ACTOR_FIELDS);
    const id = stableText(actor.id, `${path}.id`, 80);
    if (ids.has(id)) {
      throw new Error(`Duplicate Actor definition id: ${id}`);
    }
    ids.add(id);
    validateActorBody(actor, id, path);
    if (typeof actor.enabled !== "boolean") {
      throw new Error(`${path}.enabled must be a boolean`);
    }
    positiveInteger(actor.revision, `${path}.revision`);
    const validatedActor = actor as unknown as ActorDefinition;
    if ((id === "qin_chuan") !== (validatedActor.identity.name === "秦川")) {
      throw new Error('Stable Actor must remain exactly "秦川" / qin_chuan');
    }
    return validatedActor;
  });

  rejectNamedActorDependencies(actors);
  return value as readonly ActorDefinition[];
}

export function assertActorLibraryInvariants(
  actors: readonly ActorDefinition[],
): void {
  const qinChuanById = actors.filter((actor) => actor.id === "qin_chuan");
  const qinChuanByName = actors.filter(
    (actor) => actor.identity.name === "秦川",
  );
  if (
    qinChuanById.length !== 1 ||
    qinChuanByName.length !== 1 ||
    qinChuanById[0] !== qinChuanByName[0]
  ) {
    throw new Error('Actor library must contain exactly one "秦川" / qin_chuan');
  }
}

export function createActorSnapshot(actor: ActorDefinition): ActorSnapshot {
  return structuredClone({
    sourceId: actor.id,
    revision: actor.revision,
    identity: actor.identity,
    core: actor.core,
    cognition: actor.cognition,
    interaction: actor.interaction,
    expression: actor.expression,
    production: actor.production,
  });
}

export function validateActorSnapshot(value: unknown): ActorSnapshot {
  if (!isPlainObject(value)) {
    throw new Error("Actor snapshot must be an object");
  }
  assertExactObjectKeys(value, "Actor snapshot", ACTOR_SNAPSHOT_FIELDS);
  const sourceId = stableText(value.sourceId, "Actor snapshot.sourceId", 80);
  positiveInteger(value.revision, "Actor snapshot.revision");
  validateActorBody(value, sourceId, "Actor snapshot");
  const snapshot = value as unknown as ActorSnapshot;
  if ((sourceId === "qin_chuan") !== (snapshot.identity.name === "秦川")) {
    throw new Error('Stable Actor snapshot must remain exactly "秦川" / qin_chuan');
  }
  return structuredClone(snapshot);
}

export function compileActorRuntimeCard(
  actor: ActorDefinition | ActorSnapshot,
): ActorRuntimeCard {
  return {
    actorId: actorId(actor),
    name: actor.identity.name,
    core: {
      stableCore: actor.core.stableCore,
      drive: actor.core.drive,
    },
    cognition: structuredClone(actor.cognition),
    interaction: {
      socialStrategy: actor.interaction.socialStrategy,
      pressureResponse: actor.interaction.pressureResponse,
    },
    expression: structuredClone(actor.expression),
  };
}

export function compileActorAuthorCard(
  actor: ActorDefinition | ActorSnapshot,
): ActorAuthorCard {
  return {
    actorId: actorId(actor),
    identity: {
      name: actor.identity.name,
      tags: [...actor.identity.tags],
    },
    core: {
      stableCore: actor.core.stableCore,
      blindSpot: actor.core.blindSpot,
      changeBoundary: actor.core.changeBoundary,
    },
    tableFunction: actor.interaction.tableFunction,
    pressureResponse: actor.interaction.pressureResponse,
    correctionTrigger: actor.cognition.correctionTrigger,
    conflictAxes: [...actor.interaction.conflictAxes],
  };
}

function validateActorBody(
  actor: Record<string, unknown>,
  actorIdValue: string,
  path: string,
): void {
  const identity = object(actor.identity, `${path}.identity`);
  assertExactObjectKeys(identity, `${path}.identity`, [
    "name",
    "portrait",
    "tags",
    "visualAnchor",
  ]);
  stableText(identity.name, `${path}.identity.name`, 40);
  stableText(identity.portrait, `${path}.identity.portrait`, 240);
  textList(identity.tags, `${path}.identity.tags`, { min: 1, max: 8 });
  stableText(identity.visualAnchor, `${path}.identity.visualAnchor`);

  const core = object(actor.core, `${path}.core`);
  validateTextObject(core, `${path}.core`, [
    "stableCore",
    "drive",
    "blindSpot",
    "changeBoundary",
  ]);

  const cognition = object(actor.cognition, `${path}.cognition`);
  validateTextObject(cognition, `${path}.cognition`, [
    "attention",
    "evidencePolicy",
    "decisionPolicy",
    "correctionTrigger",
  ]);

  const interaction = object(actor.interaction, `${path}.interaction`);
  assertExactObjectKeys(interaction, `${path}.interaction`, [
    "tableFunction",
    "socialStrategy",
    "pressureResponse",
    "conflictAxes",
  ]);
  for (const key of [
    "tableFunction",
    "socialStrategy",
    "pressureResponse",
  ] as const) {
    stableText(interaction[key], `${path}.interaction.${key}`);
  }
  textList(interaction.conflictAxes, `${path}.interaction.conflictAxes`, {
    min: 1,
    max: 6,
  });

  const expression = object(actor.expression, `${path}.expression`);
  assertExactObjectKeys(expression, `${path}.expression`, [
    "cadence",
    "diction",
    "rhetoricalMoves",
    "avoid",
  ]);
  stableText(expression.cadence, `${path}.expression.cadence`);
  stableText(expression.diction, `${path}.expression.diction`);
  textList(expression.rhetoricalMoves, `${path}.expression.rhetoricalMoves`, {
    min: 1,
    max: 6,
  });
  textList(expression.avoid, `${path}.expression.avoid`, { min: 1, max: 6 });

  const production = object(actor.production, `${path}.production`);
  assertExactObjectKeys(production, `${path}.production`, [
    "modelBinding",
    "voice",
  ]);
  if (production.modelBinding === null) {
    throw new Error(`${path}.production.modelBinding must be set`);
  }
  validateModelBindingSnapshot(
    production.modelBinding,
    `${path}.production.modelBinding`,
  );
  validateVoiceProfileSnapshot(production.voice, `${path}.production.voice`);

  if (actorIdValue.trim().length === 0) {
    throw new Error(`${path} id must be set`);
  }
}

function rejectNamedActorDependencies(actors: readonly ActorDefinition[]): void {
  for (const actor of actors) {
    const dependencyText = [
      ...actor.identity.tags,
      actor.identity.visualAnchor,
      actor.core.stableCore,
      actor.core.drive,
      actor.core.blindSpot,
      actor.core.changeBoundary,
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
    ].join("\n");
    for (const other of actors) {
      if (other.id === actor.id) continue;
      if (
        dependencyText.includes(other.id) ||
        dependencyText.includes(other.identity.name)
      ) {
        throw new Error(
          `Actor ${actor.id} interaction must not reference Actor ${other.id}`,
        );
      }
    }
  }
}

function actorId(actor: ActorDefinition | ActorSnapshot): string {
  return "id" in actor ? actor.id : actor.sourceId;
}

function validateTextObject(
  value: Record<string, unknown>,
  path: string,
  keys: readonly string[],
): void {
  assertExactObjectKeys(value, path, keys);
  for (const key of keys) {
    stableText(value[key], `${path}.${key}`);
  }
}

function object(value: unknown, path: string): Record<string, unknown> {
  if (!isPlainObject(value)) {
    throw new Error(`${path} must be an object`);
  }
  return value;
}

function stableText(
  value: unknown,
  path: string,
  maxLength = MAX_CLAUSE_LENGTH,
): string {
  if (typeof value !== "string" || value.trim().length === 0) {
    throw new Error(`${path} must be a non-blank string`);
  }
  if (value !== value.trim()) {
    throw new Error(`${path} must not include surrounding whitespace`);
  }
  if ([...value].length > maxLength) {
    throw new Error(`${path} must be at most ${maxLength} characters`);
  }
  return value;
}

function textList(
  value: unknown,
  path: string,
  bounds: { readonly min: number; readonly max: number },
): readonly string[] {
  if (!Array.isArray(value)) {
    throw new Error(`${path} must be an array`);
  }
  if (value.length < bounds.min || value.length > bounds.max) {
    throw new Error(
      `${path} must contain ${bounds.min}-${bounds.max} items`,
    );
  }
  const items = value.map((item, index) =>
    stableText(item, `${path}[${index}]`, MAX_LIST_ITEM_LENGTH),
  );
  if (new Set(items).size !== items.length) {
    throw new Error(`${path} must not contain duplicates`);
  }
  return items;
}

function positiveInteger(value: unknown, path: string): number {
  if (!Number.isInteger(value) || (value as number) < 1) {
    throw new Error(`${path} must be a positive integer`);
  }
  return value as number;
}
