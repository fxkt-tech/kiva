import { appendEvent } from "./event-log";
import { planNextDraft } from "./advance-planner";
import { confirmDraftEvent, type DraftEvent } from "./drafts";
import type { GameEvent, VoteType } from "./events";
import type { Game } from "./game";
import {
  isGenerationAttemptSnapshot,
  isLlmTokenUsage,
  validateGenerationRequestSnapshot,
  type GenerationAttemptSnapshot,
  type GenerationRequestSnapshot,
} from "./generation-record";
import type { LlmTokenUsage } from "./llm";
import {
  assertExactObjectKeys,
  isPlainObject,
  validateModelBindingSnapshot,
} from "./model-binding";
import { isLlmSpeechDraft, type LlmSpeechDraft } from "./llm-task-specs";
import {
  estimateSpeechDurationMs,
  speechBudgetForDraft,
  type SpeechBudget,
} from "./speech-budget";
import type { ModelBindingSnapshot, PlayerSnapshot } from "./player";
import type { DraftId, EventId, Phase, PlayerId } from "./types";

export const EPISODE_SCRIPT_SCHEMA_VERSION = 2;
export const EPISODE_COMPILER_VERSION = "episode-compiler:v1";
export const EPISODE_AUTHOR_PROMPT_VERSION = "episode-author:v3" as const;
export const MAX_EPISODE_STORY_ACTS = 4;
export const MAX_EPISODE_RELATIONSHIP_SEEDS = 6;
const MAX_EPISODE_STEPS = 240;

export type EpisodeCharacterProfile = {
  readonly playerId: PlayerId;
  readonly seatNo: number;
  readonly name: string;
  readonly gameRole: PlayerSnapshot["gameRole"];
  readonly persona: string;
  readonly speakingStyle: string;
  readonly reasoningStyle: string;
};

export type EpisodeCastDirection = {
  readonly playerId: PlayerId;
  readonly dramaticWeight: "primary" | "supporting";
  readonly dramaticFunction: string;
  readonly baseline: string;
  readonly pressure: string;
  readonly change: string;
  readonly payoff: string;
  readonly signatureMoment: {
    readonly stepIndex: number;
    readonly description: string;
  };
};

export type EpisodeRelationshipKind =
  | "rivalry"
  | "alliance"
  | "contrast"
  | "trust_shift";

export type EpisodeRelationshipDirection = {
  readonly playerIds: readonly [PlayerId, PlayerId];
  readonly kind: EpisodeRelationshipKind;
  readonly setup: string;
  readonly development: string;
  readonly payoff: string;
};

export type DraftSlot = {
  readonly type: DraftEvent["type"];
  readonly phase: Phase;
  readonly actorPlayerId: PlayerId | null;
  readonly dayNumber: number | null;
  readonly round: number | null;
  readonly voteType: VoteType | null;
};

export type EpisodeSpeechBeat = {
  readonly stepIndex: number;
  readonly objective: string;
  readonly stance: string;
  readonly disclosure: "conceal" | "claim" | "not_applicable";
  readonly themeHook: string;
  readonly characterHook: string | null;
  readonly arcMove: string | null;
  readonly relationshipMove: string | null;
  readonly budget: SpeechBudget;
};

export type EpisodePlanStep = {
  readonly index: number;
  readonly slot: DraftSlot;
  readonly plannedPayload: Readonly<Record<string, unknown>>;
  readonly speechBeat: EpisodeSpeechBeat | null;
  readonly summary: string;
};

export type EpisodeAct = {
  readonly title: string;
  readonly summary: string;
};

export type EpisodeScriptSnapshot = {
  readonly schemaVersion: typeof EPISODE_SCRIPT_SCHEMA_VERSION;
  readonly id: string;
  readonly gameId: Game["id"];
  readonly compilerVersion: string;
  readonly inputHash: string;
  readonly title: string;
  readonly logline: string;
  readonly plannedWinner: "wolves" | "good";
  readonly plannedDayCount: number;
  readonly targetDurationMs: number;
  readonly acts: readonly EpisodeAct[];
  readonly castDirections: readonly EpisodeCastDirection[];
  readonly relationships: readonly EpisodeRelationshipDirection[];
  readonly steps: readonly EpisodePlanStep[];
  readonly createdAt: string;
  readonly author: {
    readonly provider: string;
    readonly model: string;
  };
};

export type EpisodeScriptReport = {
  readonly valid: boolean;
  readonly stepCount: number;
  readonly speechCount: number;
  readonly targetDurationMs: number;
  readonly warnings: readonly string[];
};

export type EpisodeStorySpine = {
  readonly title: string;
  readonly logline: string;
  readonly acts: readonly EpisodeAct[];
};

export type EpisodeCastAssignment = {
  readonly playerId: PlayerId;
  readonly dramaticWeight: EpisodeCastDirection["dramaticWeight"];
  readonly dramaticFunction: string;
  readonly signatureStepIndex: number;
};

export type EpisodeRelationshipSeed = {
  readonly playerIds: readonly [PlayerId, PlayerId];
  readonly kind: EpisodeRelationshipKind;
};

export type EpisodeEnsembleMap = {
  readonly castAssignments: readonly EpisodeCastAssignment[];
  readonly relationshipSeeds: readonly EpisodeRelationshipSeed[];
};

export type EpisodeAuthorWorkspace = {
  readonly inputHash: string;
  readonly modelBinding: ModelBindingSnapshot;
  readonly planStepCount: number;
  readonly speechStepCount: number;
  readonly story: EpisodeStorySpine | null;
  readonly ensemble: EpisodeEnsembleMap | null;
  readonly castDirections: readonly EpisodeCastDirection[];
  readonly relationships: readonly EpisodeRelationshipDirection[];
  readonly beats: readonly Omit<EpisodeSpeechBeat, "budget">[];
};

export type EpisodeAuthorTask =
  | { readonly kind: "story" }
  | { readonly kind: "ensemble" }
  | { readonly kind: "character"; readonly playerId: PlayerId }
  | {
      readonly kind: "relationship";
      readonly playerIds: readonly [PlayerId, PlayerId];
    }
  | { readonly kind: "beats"; readonly stepIndexes: readonly number[] };

export type EpisodeAuthorRequestRecord = {
  readonly id: string;
  readonly task: EpisodeAuthorTask;
  readonly status: "success" | "failed";
  readonly promptVersion: typeof EPISODE_AUTHOR_PROMPT_VERSION;
  readonly provider: string;
  readonly model: string;
  readonly request: GenerationRequestSnapshot;
  readonly tokenUsage: LlmTokenUsage | null;
  readonly finishReason: string | null;
  readonly rawOutput: string | null;
  readonly parsedOutput: Record<string, unknown> | null;
  readonly error: string | null;
  readonly createdAt: string;
  readonly attempts?: readonly GenerationAttemptSnapshot[];
};

export type EpisodeScriptState =
  | { readonly status: "idle" }
  | {
      readonly status: "generating";
      readonly jobId: string;
      readonly startedAt: string;
      readonly workspace: EpisodeAuthorWorkspace;
      readonly requests: readonly EpisodeAuthorRequestRecord[];
    }
  | {
      readonly status: "review";
      readonly jobId: string;
      readonly candidate: EpisodeScriptSnapshot;
      readonly report: EpisodeScriptReport;
      readonly requests: readonly EpisodeAuthorRequestRecord[];
    }
  | {
      readonly status: "approved";
      readonly jobId: string;
      readonly script: EpisodeScriptSnapshot;
      readonly report: EpisodeScriptReport;
      readonly approvedAt: string;
      readonly requests: readonly EpisodeAuthorRequestRecord[];
    }
  | {
      readonly status: "failed";
      readonly jobId: string;
      readonly error: string;
      readonly failedAt: string;
      readonly workspace: EpisodeAuthorWorkspace;
      readonly requests: readonly EpisodeAuthorRequestRecord[];
    };

export type CompiledEpisodePlan = {
  readonly inputHash: string;
  readonly plannedWinner: "wolves" | "good";
  readonly plannedDayCount: number;
  readonly targetDurationMs: number;
  readonly steps: readonly EpisodePlanStep[];
  readonly simulatedEvents: readonly GameEvent[];
};

export type EpisodeActorBrief = {
  readonly stepIndex: number;
  readonly scene: string;
  readonly objective: string;
  readonly stance: string;
  readonly disclosure: EpisodeSpeechBeat["disclosure"];
  readonly themeHook: string;
  readonly characterHook: string | null;
  readonly arcMove: string | null;
  readonly relationshipMove: string | null;
  readonly budget: SpeechBudget;
};

export function episodeCharacterProfile(
  player: PlayerSnapshot,
): EpisodeCharacterProfile {
  return {
    playerId: player.playerId,
    seatNo: player.seatNo,
    name: player.name,
    gameRole: player.gameRole,
    persona: player.persona,
    speakingStyle: player.speakingStyle,
    reasoningStyle: player.reasoningStyle,
  };
}

export function episodeInputHash(game: Game): string {
  return characterDrivenEpisodeInputHash(game);
}

function characterDrivenEpisodeInputHash(game: Game): string {
  return stableHash(
    JSON.stringify({
      players: game.players.map(episodeCharacterProfile),
      ruleset: game.ruleset,
      script: game.script,
    }),
  );
}

export function compileEpisodePlan(game: Game): CompiledEpisodePlan {
  let events: readonly GameEvent[] = [];
  const steps: EpisodePlanStep[] = [];
  let speechDurationMs = 0;
  let nonSpeechDurationMs = 0;

  for (let index = 1; index <= MAX_EPISODE_STEPS; index += 1) {
    const draft = planNextDraft({
      game,
      events,
      draftId: `episode_draft_${index}` as DraftId,
      createdAt: episodeTime(index),
    });
    if (!draft) break;

    const hasPriorDaySpeech =
      draft.type === "day_speech_given" &&
      events.some(
        (event) =>
          event.type === "day_speech_given" &&
          event.payload.dayNumber === draft.payload.dayNumber,
      );
    const speechDraft = isLlmSpeechDraft(draft) ? draft : null;
    const budget = speechDraft
      ? speechBudgetForDraft({
          draft: speechDraft,
          hasPriorDaySpeech,
          tier: "critical",
        })
      : null;
    const speechBeat = budget && speechDraft
      ? defaultSpeechBeat(index, speechDraft, budget, game)
      : null;
    steps.push({
      index,
      slot: draftSlotFor(draft),
      plannedPayload: cloneRecord(draft.payload),
      speechBeat,
      summary: draftSummary(draft),
    });
    speechDurationMs += budget
      ? estimateSpeechDurationMs(budget.targetMaxCharacters)
      : 0;
    nonSpeechDurationMs += budget ? 900 : 2_200;

    events = appendEvent(
      events,
      confirmDraftEvent({
        draft,
        eventId: `episode_event_${index}` as EventId,
        index,
        createdAt: episodeTime(index),
      }),
    );
    if (draft.type === "game_ended") break;
  }

  const ending = events.at(-1);
  if (ending?.type !== "game_ended") {
    throw new Error(
      `Episode dry-run did not reach game_ended within ${MAX_EPISODE_STEPS} steps`,
    );
  }

  return {
    inputHash: episodeInputHash(game),
    plannedWinner: ending.payload.winner,
    plannedDayCount: ending.payload.dayNumber,
    targetDurationMs: speechDurationMs + nonSpeechDurationMs,
    steps,
    simulatedEvents: events,
  };
}

export function createEpisodeScriptSnapshot(input: {
  readonly id: string;
  readonly game: Game;
  readonly plan: CompiledEpisodePlan;
  readonly title: string;
  readonly logline: string;
  readonly acts: readonly EpisodeAct[];
  readonly castDirections: readonly EpisodeCastDirection[];
  readonly relationships: readonly EpisodeRelationshipDirection[];
  readonly beats: readonly Omit<EpisodeSpeechBeat, "budget">[];
  readonly createdAt: string;
  readonly provider: string;
  readonly model: string;
}): EpisodeScriptSnapshot {
  if (input.plan.inputHash !== episodeInputHash(input.game)) {
    throw new Error("Episode plan input hash does not match game");
  }
  const authoredBeats = new Map(
    input.beats.map((beat) => [beat.stepIndex, beat]),
  );
  const steps = input.plan.steps.map((step) => {
    if (!step.speechBeat) return step;
    const authored = authoredBeats.get(step.index);
    return authored
      ? {
          ...step,
          speechBeat: { ...authored, budget: step.speechBeat.budget },
        }
      : step;
  });

  assertEpisodeDramaturgy({
    game: input.game,
    plan: input.plan,
    castDirections: input.castDirections,
    relationships: input.relationships,
  });

  return validateEpisodeScriptSnapshot({
    schemaVersion: EPISODE_SCRIPT_SCHEMA_VERSION,
    id: input.id,
    gameId: input.game.id,
    compilerVersion: EPISODE_COMPILER_VERSION,
    inputHash: input.plan.inputHash,
    title: nonEmpty(input.title, "Episode title"),
    logline: nonEmpty(input.logline, "Episode logline"),
    plannedWinner: input.plan.plannedWinner,
    plannedDayCount: input.plan.plannedDayCount,
    targetDurationMs: input.plan.targetDurationMs,
    acts: input.acts.length > 0
      ? input.acts
      : [{ title: "序幕", summary: input.logline }],
    castDirections: input.castDirections,
    relationships: input.relationships,
    steps,
    createdAt: input.createdAt,
    author: { provider: input.provider, model: input.model },
  });
}

export function validateEpisodeScriptSnapshot(
  value: unknown,
): EpisodeScriptSnapshot {
  if (!isPlainObject(value)) {
    throw new Error("Episode script must be an object");
  }
  assertExactObjectKeys(value, "Episode script", [
    "schemaVersion",
    "id",
    "gameId",
    "compilerVersion",
    "inputHash",
    "title",
    "logline",
    "plannedWinner",
    "plannedDayCount",
    "targetDurationMs",
    "acts",
    "castDirections",
    "relationships",
    "steps",
    "createdAt",
    "author",
  ]);
  const snapshot = value as EpisodeScriptSnapshot;
  if (snapshot.schemaVersion !== EPISODE_SCRIPT_SCHEMA_VERSION) {
    throw new Error(
      `Unsupported episode script schema: ${String(snapshot.schemaVersion)}`,
    );
  }
  if (snapshot.compilerVersion !== EPISODE_COMPILER_VERSION) {
    throw new Error(`Unsupported episode compiler: ${snapshot.compilerVersion}`);
  }
  requiredString(snapshot.id, "Episode script id");
  requiredString(snapshot.gameId, "Episode script gameId");
  requiredString(snapshot.inputHash, "Episode script inputHash");
  requiredString(snapshot.createdAt, "Episode script createdAt");
  if (
    (snapshot.plannedWinner !== "wolves" &&
      snapshot.plannedWinner !== "good") ||
    !Number.isInteger(snapshot.plannedDayCount) ||
    snapshot.plannedDayCount < 1 ||
    !Number.isFinite(snapshot.targetDurationMs) ||
    snapshot.targetDurationMs <= 0
  ) {
    throw new Error("Episode script plan metadata is invalid");
  }
  if (!Array.isArray(snapshot.acts) || snapshot.acts.length === 0) {
    throw new Error("Episode script must include acts");
  }
  snapshot.acts.forEach((act, index) => {
    const current = objectRecord(act, `Episode act ${index}`);
    assertExactObjectKeys(current, `Episode act ${index}`, ["title", "summary"]);
    requiredString(current.title, `Episode act ${index} title`);
    requiredString(current.summary, `Episode act ${index} summary`);
  });
  const author = objectRecord(snapshot.author, "Episode script author");
  assertExactObjectKeys(author, "Episode script author", ["provider", "model"]);
  requiredString(author.provider, "Episode script author provider");
  requiredString(author.model, "Episode script author model");
  if (!Array.isArray(snapshot.steps) || snapshot.steps.length === 0) {
    throw new Error("Episode script must include steps");
  }
  const steps = snapshot.steps.map((step, offset) => {
    const currentStep = objectRecord(step, `Episode step ${offset + 1}`);
    assertExactObjectKeys(currentStep, `Episode step ${offset + 1}`, [
      "index",
      "slot",
      "plannedPayload",
      "speechBeat",
      "summary",
    ]);
    if (step.index !== offset + 1) {
      throw new Error(`Episode step index mismatch at ${offset + 1}`);
    }
    if (!step.slot || typeof step.slot.type !== "string") {
      throw new Error(`Episode step ${step.index} has invalid slot`);
    }
    const slot = objectRecord(step.slot, `Episode step ${step.index} slot`);
    assertExactObjectKeys(slot, `Episode step ${step.index} slot`, [
      "type",
      "phase",
      "actorPlayerId",
      "dayNumber",
      "round",
      "voteType",
    ]);
    if (!step.plannedPayload || typeof step.plannedPayload !== "object") {
      throw new Error(`Episode step ${step.index} has invalid payload`);
    }
    if (step.speechBeat && step.speechBeat.stepIndex !== step.index) {
      throw new Error(`Episode speech beat mismatch at step ${step.index}`);
    }
    if (step.speechBeat) {
      validateSpeechBeat(step.speechBeat, step.index);
    }
    requiredString(step.summary, `Episode step ${step.index} summary`);
    return {
      ...step,
      speechBeat: step.speechBeat ? { ...step.speechBeat } : null,
    };
  });
  const last = snapshot.steps.at(-1);
  if (last?.slot.type !== "game_ended") {
    throw new Error("Episode script must end with game_ended");
  }
  nonEmpty(snapshot.title, "Episode title");
  nonEmpty(snapshot.logline, "Episode logline");
  const castDirections = validateCastDirectionShapes(snapshot.castDirections);
  const relationships = validateRelationshipShapes(snapshot.relationships);
  return structuredClone({
    ...snapshot,
    castDirections,
    relationships,
    steps,
  });
}

export function episodeScriptReport(
  script: EpisodeScriptSnapshot,
): EpisodeScriptReport {
  const warnings = [
    ...(script.targetDurationMs > 32 * 60_000
      ? ["预计时长高于 32 分钟目标"]
      : []),
  ];
  return {
    valid: script.targetDurationMs <= 35 * 60_000,
    stepCount: script.steps.length,
    speechCount: script.steps.filter((step) => step.speechBeat !== null).length,
    targetDurationMs: script.targetDurationMs,
    warnings,
  };
}

export function assertEpisodeScriptMatchesGame(input: {
  readonly game: Game;
  readonly script: EpisodeScriptSnapshot;
}): void {
  const { game, script } = input;
  if (script.gameId !== game.id) {
    throw new Error("Episode script belongs to another game");
  }
  if (script.inputHash !== episodeInputHash(game)) {
    throw new Error("Episode script input no longer matches game");
  }

  const plan = compileEpisodePlan(game);
  if (
    script.plannedWinner !== plan.plannedWinner ||
    script.plannedDayCount !== plan.plannedDayCount ||
    script.targetDurationMs !== plan.targetDurationMs ||
    script.steps.length !== plan.steps.length
  ) {
    throw new Error("Episode script does not match the compiled episode plan");
  }

  for (const [offset, expected] of plan.steps.entries()) {
    const actual = script.steps[offset];
    if (
      !actual ||
      actual.index !== expected.index ||
      JSON.stringify(actual.slot) !== JSON.stringify(expected.slot) ||
      JSON.stringify(actual.plannedPayload) !==
        JSON.stringify(expected.plannedPayload) ||
      JSON.stringify(actual.speechBeat?.budget ?? null) !==
        JSON.stringify(expected.speechBeat?.budget ?? null)
    ) {
      throw new Error(
        `Episode script changed compiler-owned step ${expected.index}`,
      );
    }
  }

  assertEpisodeDramaturgy({
    game,
    plan,
    castDirections: script.castDirections,
    relationships: script.relationships,
  });
}

export function validateEpisodeScriptState(
  value: unknown,
  runMode: Game["runMode"],
): EpisodeScriptState | null {
  if (runMode === "game") {
    if (value !== null) {
      throw new Error("Game-mode records must store a null episode script state");
    }
    return null;
  }
  const state = objectRecord(value, "Episode script state");
  switch (state.status) {
    case "idle":
      assertExactObjectKeys(state, "Episode script state", ["status"]);
      return { status: "idle" };
    case "generating":
      assertExactObjectKeys(
        state,
        "Episode script state",
        ["status", "jobId", "startedAt", "workspace", "requests"],
      );
      return {
        status: "generating",
        jobId: requiredString(state.jobId, "Episode script state jobId"),
        startedAt: requiredString(
          state.startedAt,
          "Episode script state startedAt",
        ),
        workspace: validateEpisodeAuthorWorkspace(state.workspace),
        requests: validateEpisodeAuthorRequests(state.requests),
      };
    case "review":
      assertExactObjectKeys(
        state,
        "Episode script state",
        ["status", "jobId", "candidate", "report", "requests"],
      );
      return {
        status: "review",
        jobId: requiredString(state.jobId, "Episode script state jobId"),
        candidate: validateEpisodeScriptSnapshot(state.candidate),
        report: validateEpisodeScriptReport(state.report),
        requests: validateEpisodeAuthorRequests(state.requests),
      };
    case "approved":
      assertExactObjectKeys(
        state,
        "Episode script state",
        ["status", "jobId", "script", "report", "approvedAt", "requests"],
      );
      return {
        status: "approved",
        jobId: requiredString(state.jobId, "Episode script state jobId"),
        script: validateEpisodeScriptSnapshot(state.script),
        report: validateEpisodeScriptReport(state.report),
        approvedAt: requiredString(
          state.approvedAt,
          "Episode script state approvedAt",
        ),
        requests: validateEpisodeAuthorRequests(state.requests),
      };
    case "failed":
      assertExactObjectKeys(
        state,
        "Episode script state",
        [
          "status",
          "jobId",
          "error",
          "failedAt",
          "workspace",
          "requests",
        ],
      );
      return {
        status: "failed",
        jobId: requiredString(state.jobId, "Episode script state jobId"),
        error: requiredString(state.error, "Episode script state error"),
        failedAt: requiredString(
          state.failedAt,
          "Episode script state failedAt",
        ),
        workspace: validateEpisodeAuthorWorkspace(state.workspace),
        requests: validateEpisodeAuthorRequests(state.requests),
      };
    default:
      throw new Error("Episode script state has invalid status");
  }
}

export function validateEpisodeAuthorWorkspace(
  value: unknown,
): EpisodeAuthorWorkspace {
  const workspace = objectRecord(value, "Episode author workspace");
  assertExactObjectKeys(workspace, "Episode author workspace", [
    "inputHash",
    "modelBinding",
    "planStepCount",
    "speechStepCount",
    "story",
    "ensemble",
    "castDirections",
    "relationships",
    "beats",
  ]);
  if (!isPlainObject(workspace.modelBinding)) {
    throw new Error("Episode author workspace modelBinding must be an object");
  }
  validateModelBindingSnapshot(
    workspace.modelBinding,
    "Episode author workspace modelBinding",
  );
  if (
    !Number.isInteger(workspace.planStepCount) ||
    (workspace.planStepCount as number) < 1 ||
    !Number.isInteger(workspace.speechStepCount) ||
    (workspace.speechStepCount as number) < 1
  ) {
    throw new Error("Episode author workspace plan counts are invalid");
  }
  if (!Array.isArray(workspace.castDirections)) {
    throw new Error("Episode author workspace castDirections must be an array");
  }
  if (!Array.isArray(workspace.beats)) {
    throw new Error("Episode author workspace beats must be an array");
  }
  return {
    inputHash: requiredString(
      workspace.inputHash,
      "Episode author workspace inputHash",
    ),
    modelBinding: structuredClone(workspace.modelBinding) as ModelBindingSnapshot,
    planStepCount: workspace.planStepCount as number,
    speechStepCount: workspace.speechStepCount as number,
    story:
      workspace.story === null
        ? null
        : validateEpisodeStorySpine(workspace.story),
    ensemble:
      workspace.ensemble === null
        ? null
        : validateEpisodeEnsembleMap(workspace.ensemble),
    castDirections:
      workspace.castDirections.length === 0
        ? []
        : validateCastDirectionShapes(workspace.castDirections),
    relationships: validateRelationshipShapes(workspace.relationships),
    beats: workspace.beats.map(validateWorkspaceBeat),
  };
}

function validateEpisodeStorySpine(value: unknown): EpisodeStorySpine {
  const story = objectRecord(value, "Episode author story");
  assertExactObjectKeys(story, "Episode author story", [
    "title",
    "logline",
    "acts",
  ]);
  if (
    !Array.isArray(story.acts) ||
    story.acts.length === 0 ||
    story.acts.length > MAX_EPISODE_STORY_ACTS
  ) {
    throw new Error(
      `Episode author story acts must contain between 1 and ${MAX_EPISODE_STORY_ACTS} items`,
    );
  }
  return {
    title: requiredString(story.title, "Episode author story title"),
    logline: requiredString(story.logline, "Episode author story logline"),
    acts: story.acts.map((value, index) => {
      const act = objectRecord(value, `Episode author story act ${index + 1}`);
      assertExactObjectKeys(act, `Episode author story act ${index + 1}`, [
        "title",
        "summary",
      ]);
      return {
        title: requiredString(
          act.title,
          `Episode author story act ${index + 1} title`,
        ),
        summary: requiredString(
          act.summary,
          `Episode author story act ${index + 1} summary`,
        ),
      };
    }),
  };
}

function validateEpisodeEnsembleMap(value: unknown): EpisodeEnsembleMap {
  const ensemble = objectRecord(value, "Episode author ensemble");
  assertExactObjectKeys(ensemble, "Episode author ensemble", [
    "castAssignments",
    "relationshipSeeds",
  ]);
  if (!Array.isArray(ensemble.castAssignments)) {
    throw new Error("Episode author castAssignments must be an array");
  }
  if (!Array.isArray(ensemble.relationshipSeeds)) {
    throw new Error("Episode author relationshipSeeds must be an array");
  }
  if (
    ensemble.relationshipSeeds.length === 0 ||
    ensemble.relationshipSeeds.length > MAX_EPISODE_RELATIONSHIP_SEEDS
  ) {
    throw new Error(
      `Episode author relationshipSeeds must contain between 1 and ${MAX_EPISODE_RELATIONSHIP_SEEDS} items`,
    );
  }
  const assigned = new Set<string>();
  const castAssignments = ensemble.castAssignments.map(
    (value, index): EpisodeCastAssignment => {
      const assignment = objectRecord(
        value,
        `Episode author cast assignment ${index + 1}`,
      );
      assertExactObjectKeys(
        assignment,
        `Episode author cast assignment ${index + 1}`,
        [
          "playerId",
          "dramaticWeight",
          "dramaticFunction",
          "signatureStepIndex",
        ],
      );
      const playerId = requiredString(
        assignment.playerId,
        `Episode author cast assignment ${index + 1} playerId`,
      ) as PlayerId;
      if (assigned.has(playerId)) {
        throw new Error(
          `Duplicate episode author cast assignment: ${playerId}`,
        );
      }
      assigned.add(playerId);
      if (
        assignment.dramaticWeight !== "primary" &&
        assignment.dramaticWeight !== "supporting"
      ) {
        throw new Error(
          `Episode author cast assignment ${index + 1} dramaticWeight is invalid`,
        );
      }
      if (!Number.isInteger(assignment.signatureStepIndex)) {
        throw new Error(
          `Episode author cast assignment ${index + 1} signatureStepIndex is invalid`,
        );
      }
      return {
        playerId,
        dramaticWeight: assignment.dramaticWeight,
        dramaticFunction: requiredString(
          assignment.dramaticFunction,
          `Episode author cast assignment ${index + 1} dramaticFunction`,
        ),
        signatureStepIndex: assignment.signatureStepIndex as number,
      };
    },
  );
  const pairs = new Set<string>();
  const relationshipSeeds = ensemble.relationshipSeeds.map((value, index) => {
    const seed = objectRecord(
      value,
      `Episode author relationship seed ${index + 1}`,
    );
    assertExactObjectKeys(
      seed,
      `Episode author relationship seed ${index + 1}`,
      ["playerIds", "kind"],
    );
    const playerIds = requiredPlayerPair(
      seed.playerIds,
      `Episode author relationship seed ${index + 1} playerIds`,
    );
    const pairKey = [...playerIds].sort().join(":");
    if (pairs.has(pairKey)) {
      throw new Error(`Duplicate episode author relationship seed: ${pairKey}`);
    }
    pairs.add(pairKey);
    if (!isRelationshipKind(seed.kind)) {
      throw new Error(
        `Episode author relationship seed ${index + 1} kind is invalid`,
      );
    }
    return { playerIds, kind: seed.kind };
  });
  return { castAssignments, relationshipSeeds };
}

function validateWorkspaceBeat(
  value: unknown,
  index: number,
): Omit<EpisodeSpeechBeat, "budget"> {
  const beat = objectRecord(value, `Episode author beat ${index + 1}`);
  assertExactObjectKeys(beat, `Episode author beat ${index + 1}`, [
    "stepIndex",
    "objective",
    "stance",
    "disclosure",
    "themeHook",
    "characterHook",
    "arcMove",
    "relationshipMove",
  ]);
  if (!Number.isInteger(beat.stepIndex) || (beat.stepIndex as number) < 1) {
    throw new Error(`Episode author beat ${index + 1} stepIndex is invalid`);
  }
  if (
    beat.disclosure !== "conceal" &&
    beat.disclosure !== "claim" &&
    beat.disclosure !== "not_applicable"
  ) {
    throw new Error(`Episode author beat ${index + 1} disclosure is invalid`);
  }
  return {
    stepIndex: beat.stepIndex as number,
    objective: requiredString(beat.objective, `Episode author beat ${index + 1} objective`),
    stance: requiredString(beat.stance, `Episode author beat ${index + 1} stance`),
    disclosure: beat.disclosure,
    themeHook: requiredString(beat.themeHook, `Episode author beat ${index + 1} themeHook`),
    characterHook: nullableWorkspaceString(
      beat.characterHook,
      `Episode author beat ${index + 1} characterHook`,
    ),
    arcMove: nullableWorkspaceString(
      beat.arcMove,
      `Episode author beat ${index + 1} arcMove`,
    ),
    relationshipMove: nullableWorkspaceString(
      beat.relationshipMove,
      `Episode author beat ${index + 1} relationshipMove`,
    ),
  };
}

function validateEpisodeAuthorRequests(
  value: unknown,
): readonly EpisodeAuthorRequestRecord[] {
  if (!Array.isArray(value)) {
    throw new Error("Episode author requests must be an array");
  }
  return value.map((request, index) =>
    validateEpisodeAuthorRequest(request, index),
  );
}

function validateEpisodeAuthorRequest(
  value: unknown,
  index: number,
): EpisodeAuthorRequestRecord {
  const label = `Episode author request ${index + 1}`;
  const request = objectRecord(value, label);
  assertExactObjectKeys(
    request,
    label,
    [
      "id",
      "task",
      "status",
      "promptVersion",
      "provider",
      "model",
      "request",
      "tokenUsage",
      "finishReason",
      "rawOutput",
      "parsedOutput",
      "error",
      "createdAt",
    ],
    ["attempts"],
  );
  const { task, schemaName } = validateEpisodeAuthorTask(request.task, label);
  validateGenerationRequestSnapshot(request.request, [schemaName]);
  if (
    (request.status !== "success" && request.status !== "failed") ||
    request.promptVersion !== EPISODE_AUTHOR_PROMPT_VERSION ||
    !isLlmTokenUsage(request.tokenUsage) ||
    (request.finishReason !== null && typeof request.finishReason !== "string") ||
    (request.rawOutput !== null && typeof request.rawOutput !== "string") ||
    (request.parsedOutput !== null && !isPlainObject(request.parsedOutput)) ||
    (request.error !== null && typeof request.error !== "string") ||
    (request.status === "success" &&
      (request.parsedOutput === null || request.error !== null)) ||
    (request.status === "failed" &&
      (request.parsedOutput !== null || typeof request.error !== "string"))
  ) {
    throw new Error(`${label} payload is invalid`);
  }
  for (const field of ["id", "provider", "model", "createdAt"] as const) {
    requiredString(request[field], `${label} ${field}`);
  }
  if (
    request.attempts !== undefined &&
    (!Array.isArray(request.attempts) ||
      request.attempts.some(
        (attempt) => !isGenerationAttemptSnapshot(attempt, [schemaName]),
      ))
  ) {
    throw new Error(`${label} attempts are invalid`);
  }
  return structuredClone({ ...request, task }) as EpisodeAuthorRequestRecord;
}

function validateEpisodeAuthorTask(
  value: unknown,
  label: string,
): { readonly task: EpisodeAuthorTask; readonly schemaName: string } {
  const task = objectRecord(value, `${label} task`);
  switch (task.kind) {
    case "story":
      assertExactObjectKeys(task, `${label} task`, ["kind"]);
      return { task: { kind: "story" }, schemaName: "werewolf_episode_story_v3" };
    case "ensemble":
      assertExactObjectKeys(task, `${label} task`, ["kind"]);
      return {
        task: { kind: "ensemble" },
        schemaName: "werewolf_episode_ensemble_v3",
      };
    case "character":
      assertExactObjectKeys(task, `${label} task`, ["kind", "playerId"]);
      return {
        task: {
          kind: "character",
          playerId: requiredString(task.playerId, `${label} task playerId`) as PlayerId,
        },
        schemaName: "werewolf_episode_character_v3",
      };
    case "relationship": {
      assertExactObjectKeys(task, `${label} task`, ["kind", "playerIds"]);
      const playerIds = requiredPlayerPair(task.playerIds, `${label} task playerIds`);
      return {
        task: { kind: "relationship", playerIds },
        schemaName: "werewolf_episode_relationship_v3",
      };
    }
    case "beats":
      assertExactObjectKeys(task, `${label} task`, ["kind", "stepIndexes"]);
      if (!requiredIntegerArray(task.stepIndexes) || task.stepIndexes.length === 0) {
        throw new Error(`${label} task stepIndexes are invalid`);
      }
      return {
        task: { kind: "beats", stepIndexes: [...task.stepIndexes] },
        schemaName: "werewolf_episode_beats_v3",
      };
    default:
      throw new Error(`${label} task kind is invalid`);
  }
}

function requiredIntegerArray(value: unknown): value is readonly number[] {
  return (
    Array.isArray(value) &&
    value.every((item) => Number.isInteger(item) && item > 0) &&
    new Set(value).size === value.length
  );
}

function requiredPlayerPair(
  value: unknown,
  label: string,
): readonly [PlayerId, PlayerId] {
  if (
    !Array.isArray(value) ||
    value.length !== 2 ||
    value.some((playerId) => typeof playerId !== "string" || playerId.length === 0) ||
    value[0] === value[1]
  ) {
    throw new Error(`${label} must contain two different player IDs`);
  }
  return [value[0] as PlayerId, value[1] as PlayerId];
}

function nullableWorkspaceString(value: unknown, label: string): string | null {
  return value === null ? null : requiredString(value, label);
}

function validateEpisodeScriptReport(value: unknown): EpisodeScriptReport {
  const report = objectRecord(value, "Episode script report");
  assertExactObjectKeys(report, "Episode script report", [
    "valid",
    "stepCount",
    "speechCount",
    "targetDurationMs",
    "warnings",
  ]);
  if (
    typeof report.valid !== "boolean" ||
    !Number.isInteger(report.stepCount) ||
    (report.stepCount as number) < 0 ||
    !Number.isInteger(report.speechCount) ||
    (report.speechCount as number) < 0 ||
    typeof report.targetDurationMs !== "number" ||
    !Number.isFinite(report.targetDurationMs) ||
    report.targetDurationMs < 0 ||
    !Array.isArray(report.warnings) ||
    report.warnings.some((warning) => typeof warning !== "string")
  ) {
    throw new Error("Episode script report is invalid");
  }
  return structuredClone(report) as EpisodeScriptReport;
}

export function actorBriefForStep(
  script: EpisodeScriptSnapshot,
  stepIndex: number,
): EpisodeActorBrief | null {
  const step = script.steps[stepIndex - 1];
  if (!step?.speechBeat) return null;
  return {
    stepIndex,
    scene: step.summary,
    objective: step.speechBeat.objective,
    stance: step.speechBeat.stance,
    disclosure: step.speechBeat.disclosure,
    themeHook: step.speechBeat.themeHook,
    characterHook: step.speechBeat.characterHook,
    arcMove: step.speechBeat.arcMove,
    relationshipMove: step.speechBeat.relationshipMove,
    budget: step.speechBeat.budget,
  };
}

export function assertEpisodeDraftMatchesStep(input: {
  readonly script: EpisodeScriptSnapshot;
  readonly activeEventCount: number;
  readonly draft: DraftEvent;
}): void {
  const stepIndex = input.activeEventCount + 1;
  const step = input.script.steps[stepIndex - 1];
  if (!step || !sameSlot(draftSlotFor(input.draft), step.slot)) {
    throw new Error(`Episode script diverged at step ${stepIndex}`);
  }
  const actual = cloneRecord(input.draft.payload);
  const expected = cloneRecord(step.plannedPayload);
  if (step.speechBeat) {
    deleteMutableText(actual);
    deleteMutableText(expected);
  }
  if (JSON.stringify(actual) !== JSON.stringify(expected)) {
    throw new Error(`Episode script payload diverged at step ${stepIndex}`);
  }
}

export function draftSlotFor(draft: DraftEvent): DraftSlot {
  const payload = draft.payload as Record<string, unknown>;
  return {
    type: draft.type,
    phase: draft.phase,
    actorPlayerId: draft.actorPlayerId ?? null,
    dayNumber: numberOrNull(payload.dayNumber),
    round: numberOrNull(payload.round),
    voteType:
      payload.voteType === "sheriff" ||
      payload.voteType === "exile" ||
      payload.voteType === "pk"
        ? payload.voteType
        : null,
  };
}

export function planNextEpisodeDraft(input: {
  readonly game: Game;
  readonly events: readonly GameEvent[];
  readonly script: EpisodeScriptSnapshot;
  readonly draftId: DraftId;
  readonly createdAt: string;
}): { readonly draft: DraftEvent | null; readonly actorBrief: EpisodeActorBrief | null } {
  if (
    input.script.inputHash !== episodeInputHash(input.game)
  ) {
    throw new Error("Approved episode script no longer matches game input");
  }
  const nextIndex = input.events.filter((event) => event.status === "active").length + 1;
  const step = input.script.steps[nextIndex - 1];
  const draft = planNextDraft(input);
  if (!step || !draft) {
    if (!step && !draft) return { draft: null, actorBrief: null };
    throw new Error(`Episode script diverged at step ${nextIndex}`);
  }
  if (!sameSlot(draftSlotFor(draft), step.slot)) {
    throw new Error(
      `Episode script diverged at step ${nextIndex}: expected ${step.slot.type}, received ${draft.type}`,
    );
  }
  const boundDraft = bindPlannedPayload(draft, step.plannedPayload);
  return {
    draft: boundDraft,
    actorBrief: step.speechBeat
      ? {
          stepIndex: step.index,
          scene: step.summary,
          objective: step.speechBeat.objective,
          stance: step.speechBeat.stance,
          disclosure: step.speechBeat.disclosure,
          themeHook: step.speechBeat.themeHook,
          characterHook: step.speechBeat.characterHook,
          arcMove: step.speechBeat.arcMove,
          relationshipMove: step.speechBeat.relationshipMove,
          budget: step.speechBeat.budget,
        }
      : null,
  };
}

function bindPlannedPayload(
  draft: DraftEvent,
  plannedPayload: Readonly<Record<string, unknown>>,
): DraftEvent {
  return {
    ...draft,
    payload: structuredClone(plannedPayload),
    targetPlayerIds: targetPlayerIdsFromPayload(draft, plannedPayload),
  } as DraftEvent;
}

function targetPlayerIdsFromPayload(
  draft: DraftEvent,
  payload: Readonly<Record<string, unknown>>,
): readonly PlayerId[] | undefined {
  if ("targetPlayerId" in payload) {
    return typeof payload.targetPlayerId === "string"
      ? [payload.targetPlayerId as PlayerId]
      : [];
  }
  return draft.targetPlayerIds;
}

function defaultSpeechBeat(
  stepIndex: number,
  draft: LlmSpeechDraft,
  budget: SpeechBudget,
  game: Game,
): EpisodeSpeechBeat {
  return {
    stepIndex,
    objective: `完成${draftSummary(draft)}，推动当前阵营冲突`,
    stance: "围绕当前可见事实给出明确立场，并留下后续可验证点",
    disclosure: draft.phase === "night" ? "not_applicable" : "conceal",
    themeHook: `把本轮冲突自然映射到“${game.script.theme}”，不要只重复主题词`,
    characterHook: null,
    arcMove: null,
    relationshipMove: null,
    budget,
  };
}

export function episodePerformanceOpportunities(
  game: Game,
  plan: CompiledEpisodePlan,
): ReadonlyMap<PlayerId, readonly number[]> {
  const playerIds = new Set(game.players.map((player) => player.playerId));
  const opportunities = new Map(
    game.players.map((player) => [player.playerId, [] as number[]]),
  );

  for (const step of plan.steps) {
    if (step.slot.type === "role_assigned" || step.slot.type === "phase_started") {
      continue;
    }
    const involved = new Set<PlayerId>();
    if (step.slot.actorPlayerId && playerIds.has(step.slot.actorPlayerId)) {
      involved.add(step.slot.actorPlayerId);
    }
    collectPlayerIds(step.plannedPayload, playerIds, involved);
    for (const playerId of involved) {
      opportunities.get(playerId)?.push(step.index);
    }
  }

  return opportunities;
}

export function assertEpisodeDramaturgy(input: {
  readonly game: Game;
  readonly plan: CompiledEpisodePlan;
  readonly castDirections: readonly EpisodeCastDirection[];
  readonly relationships: readonly EpisodeRelationshipDirection[];
}): void {
  const expectedPlayerIds = [...input.game.players]
    .map((player) => player.playerId)
    .sort();
  const actualPlayerIds = input.castDirections
    .map((direction) => direction.playerId)
    .sort();
  if (JSON.stringify(actualPlayerIds) !== JSON.stringify(expectedPlayerIds)) {
    throw new Error(
      "Episode cast directions must cover every player exactly once",
    );
  }

  const opportunities = episodePerformanceOpportunities(input.game, input.plan);
  for (const direction of input.castDirections) {
    if (!(opportunities.get(direction.playerId) ?? []).includes(
      direction.signatureMoment.stepIndex,
    )) {
      throw new Error(
        `Episode cast direction ${direction.playerId} has an invalid signature step`,
      );
    }
  }

  const playerIds = new Set(expectedPlayerIds);
  const pairs = new Set<string>();
  for (const relationship of input.relationships) {
    const [left, right] = relationship.playerIds;
    if (!playerIds.has(left) || !playerIds.has(right) || left === right) {
      throw new Error("Episode relationship references invalid players");
    }
    const pair = [left, right].sort().join(":");
    if (pairs.has(pair)) {
      throw new Error(`Duplicate episode relationship: ${pair}`);
    }
    pairs.add(pair);
  }
}

function validateCastDirectionShapes(
  value: unknown,
): readonly EpisodeCastDirection[] {
  if (!Array.isArray(value) || value.length === 0) {
    throw new Error("Episode script v2 must include cast directions");
  }
  const playerIds = new Set<string>();
  return value.map((raw, index) => {
    const direction = objectRecord(raw, `castDirections[${index}]`);
    assertExactObjectKeys(direction, `castDirections[${index}]`, [
      "playerId",
      "dramaticWeight",
      "dramaticFunction",
      "baseline",
      "pressure",
      "change",
      "payoff",
      "signatureMoment",
    ]);
    const playerId = requiredString(
      direction.playerId,
      `castDirections[${index}].playerId`,
    ) as PlayerId;
    if (playerIds.has(playerId)) {
      throw new Error(`Duplicate episode cast direction: ${playerId}`);
    }
    playerIds.add(playerId);
    if (
      direction.dramaticWeight !== "primary" &&
      direction.dramaticWeight !== "supporting"
    ) {
      throw new Error(`castDirections[${index}].dramaticWeight is invalid`);
    }
    const signatureMoment = objectRecord(
      direction.signatureMoment,
      `castDirections[${index}].signatureMoment`,
    );
    assertExactObjectKeys(
      signatureMoment,
      `castDirections[${index}].signatureMoment`,
      ["stepIndex", "description"],
    );
    if (!Number.isInteger(signatureMoment.stepIndex)) {
      throw new Error(
        `castDirections[${index}].signatureMoment.stepIndex must be an integer`,
      );
    }
    return {
      playerId,
      dramaticWeight: direction.dramaticWeight,
      dramaticFunction: requiredString(
        direction.dramaticFunction,
        `castDirections[${index}].dramaticFunction`,
      ),
      baseline: requiredString(
        direction.baseline,
        `castDirections[${index}].baseline`,
      ),
      pressure: requiredString(
        direction.pressure,
        `castDirections[${index}].pressure`,
      ),
      change: requiredString(
        direction.change,
        `castDirections[${index}].change`,
      ),
      payoff: requiredString(
        direction.payoff,
        `castDirections[${index}].payoff`,
      ),
      signatureMoment: {
        stepIndex: signatureMoment.stepIndex as number,
        description: requiredString(
          signatureMoment.description,
          `castDirections[${index}].signatureMoment.description`,
        ),
      },
    };
  });
}

function validateRelationshipShapes(
  value: unknown,
): readonly EpisodeRelationshipDirection[] {
  if (!Array.isArray(value)) {
    throw new Error("Episode script v2 relationships must be an array");
  }
  return value.map((raw, index) => {
    const relationship = objectRecord(raw, `relationships[${index}]`);
    assertExactObjectKeys(relationship, `relationships[${index}]`, [
      "playerIds",
      "kind",
      "setup",
      "development",
      "payoff",
    ]);
    if (
      !Array.isArray(relationship.playerIds) ||
      relationship.playerIds.length !== 2 ||
      relationship.playerIds.some((playerId) => typeof playerId !== "string")
    ) {
      throw new Error(
        `relationships[${index}].playerIds must contain two players`,
      );
    }
    if (!isRelationshipKind(relationship.kind)) {
      throw new Error(`relationships[${index}].kind is invalid`);
    }
    return {
      playerIds: [
        relationship.playerIds[0] as PlayerId,
        relationship.playerIds[1] as PlayerId,
      ],
      kind: relationship.kind,
      setup: requiredString(
        relationship.setup,
        `relationships[${index}].setup`,
      ),
      development: requiredString(
        relationship.development,
        `relationships[${index}].development`,
      ),
      payoff: requiredString(
        relationship.payoff,
        `relationships[${index}].payoff`,
      ),
    };
  });
}

function validateSpeechBeat(
  beat: EpisodeSpeechBeat,
  stepIndex: number,
): void {
  const current = objectRecord(beat, `Episode step ${stepIndex} speech beat`);
  assertExactObjectKeys(current, `Episode step ${stepIndex} speech beat`, [
    "stepIndex",
    "objective",
    "stance",
    "disclosure",
    "themeHook",
    "characterHook",
    "arcMove",
    "relationshipMove",
    "budget",
  ]);
  const budget = objectRecord(beat.budget, `Episode step ${stepIndex} budget`);
  assertExactObjectKeys(budget, `Episode step ${stepIndex} budget`, [
    "key",
    "tier",
    "targetMinCharacters",
    "targetMaxCharacters",
    "hardMaxCharacters",
  ]);
  nonEmpty(beat.objective, `Episode step ${stepIndex} objective`);
  nonEmpty(beat.stance, `Episode step ${stepIndex} stance`);
  nonEmpty(beat.themeHook, `Episode step ${stepIndex} theme hook`);
  if (
    beat.disclosure !== "conceal" &&
    beat.disclosure !== "claim" &&
    beat.disclosure !== "not_applicable"
  ) {
    throw new Error(`Episode step ${stepIndex} disclosure is invalid`);
  }
  nonEmpty(
    beat.characterHook ?? "",
    `Episode step ${stepIndex} character hook`,
  );
  nonEmpty(beat.arcMove ?? "", `Episode step ${stepIndex} arc move`);
  if (
    beat.relationshipMove !== null &&
    typeof beat.relationshipMove !== "string"
  ) {
    throw new Error(`Episode step ${stepIndex} relationship move is invalid`);
  }
  if (typeof beat.relationshipMove === "string") {
    nonEmpty(
      beat.relationshipMove,
      `Episode step ${stepIndex} relationship move`,
    );
  }
}

function collectPlayerIds(
  value: unknown,
  playerIds: ReadonlySet<PlayerId>,
  result: Set<PlayerId>,
): void {
  if (typeof value === "string") {
    if (playerIds.has(value as PlayerId)) result.add(value as PlayerId);
    return;
  }
  if (Array.isArray(value)) {
    value.forEach((entry) => collectPlayerIds(entry, playerIds, result));
    return;
  }
  if (value && typeof value === "object") {
    Object.values(value).forEach((entry) =>
      collectPlayerIds(entry, playerIds, result),
    );
  }
}

function isRelationshipKind(value: unknown): value is EpisodeRelationshipKind {
  return (
    value === "rivalry" ||
    value === "alliance" ||
    value === "contrast" ||
    value === "trust_shift"
  );
}

function objectRecord(value: unknown, label: string): Record<string, unknown> {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    throw new Error(`${label} must be an object`);
  }
  return value as Record<string, unknown>;
}

function requiredString(value: unknown, label: string): string {
  if (typeof value !== "string" || value.trim().length === 0) {
    throw new Error(`${label} must be a non-empty string`);
  }
  return value.trim();
}

function draftSummary(draft: DraftEvent): string {
  const payload = draft.payload as Record<string, unknown>;
  const day = numberOrNull(payload.dayNumber);
  return `${day ? `第 ${day} 天 ` : ""}${draft.type}`;
}

function sameSlot(left: DraftSlot, right: DraftSlot): boolean {
  return JSON.stringify(left) === JSON.stringify(right);
}

function cloneRecord(value: object): Readonly<Record<string, unknown>> {
  return structuredClone(value) as Readonly<Record<string, unknown>>;
}

function deleteMutableText(value: Readonly<Record<string, unknown>>): void {
  delete (value as Record<string, unknown>).text;
}

function numberOrNull(value: unknown): number | null {
  return typeof value === "number" && Number.isFinite(value) ? value : null;
}

function nonEmpty(value: string, label: string): string {
  if (typeof value !== "string" || value.trim().length === 0) {
    throw new Error(`${label} cannot be blank`);
  }
  return value.trim();
}

function stableHash(value: string): string {
  let hash = 2166136261;
  for (const character of value) {
    hash ^= character.codePointAt(0) ?? 0;
    hash = Math.imul(hash, 16777619);
  }
  return `episode_${(hash >>> 0).toString(16).padStart(8, "0")}`;
}

function episodeTime(index: number): string {
  return new Date(Date.UTC(2026, 0, 1, 0, 0, index)).toISOString();
}
