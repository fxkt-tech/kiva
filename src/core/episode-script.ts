import { appendEvent } from "./event-log";
import { planNextDraft } from "./advance-planner";
import { confirmDraftEvent, type DraftEvent } from "./drafts";
import type { GameEvent, VoteType } from "./events";
import type { Game } from "./game";
import { isLlmSpeechDraft, type LlmSpeechDraft } from "./llm-task-specs";
import {
  estimateSpeechDurationMs,
  speechBudgetForDraft,
  type SpeechBudget,
} from "./speech-budget";
import type { DraftId, EventId, Phase, PlayerId } from "./types";

export const EPISODE_SCRIPT_SCHEMA_VERSION = 1;
export const EPISODE_COMPILER_VERSION = "episode-compiler:v1";
const MAX_EPISODE_STEPS = 240;

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
  readonly schemaVersion: 1;
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

export type EpisodeScriptState =
  | { readonly status: "idle" }
  | {
      readonly status: "generating";
      readonly jobId: string;
      readonly startedAt: string;
    }
  | {
      readonly status: "review";
      readonly jobId: string;
      readonly candidate: EpisodeScriptSnapshot;
      readonly report: EpisodeScriptReport;
    }
  | {
      readonly status: "approved";
      readonly jobId: string;
      readonly script: EpisodeScriptSnapshot;
      readonly report: EpisodeScriptReport;
      readonly approvedAt: string;
    }
  | {
      readonly status: "failed";
      readonly jobId: string;
      readonly error: string;
      readonly failedAt: string;
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
  readonly budget: SpeechBudget;
};

export function episodeInputHash(game: Game): string {
  return stableHash(
    JSON.stringify({
      players: game.players.map((player) => ({
        playerId: player.playerId,
        role: player.gameRole,
        characterSourceId: player.characterSourceId,
      })),
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
  readonly beats?: readonly Omit<EpisodeSpeechBeat, "budget">[];
  readonly createdAt: string;
  readonly provider: string;
  readonly model: string;
}): EpisodeScriptSnapshot {
  if (input.plan.inputHash !== episodeInputHash(input.game)) {
    throw new Error("Episode plan input hash does not match game");
  }
  const authoredBeats = new Map(
    (input.beats ?? []).map((beat) => [beat.stepIndex, beat]),
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
    steps,
    createdAt: input.createdAt,
    author: { provider: input.provider, model: input.model },
  });
}

export function validateEpisodeScriptSnapshot(
  value: unknown,
): EpisodeScriptSnapshot {
  if (!value || typeof value !== "object") {
    throw new Error("Episode script must be an object");
  }
  const snapshot = value as EpisodeScriptSnapshot;
  if (snapshot.schemaVersion !== 1) {
    throw new Error(`Unsupported episode script schema: ${String(snapshot.schemaVersion)}`);
  }
  if (snapshot.compilerVersion !== EPISODE_COMPILER_VERSION) {
    throw new Error(`Unsupported episode compiler: ${snapshot.compilerVersion}`);
  }
  if (!Array.isArray(snapshot.steps) || snapshot.steps.length === 0) {
    throw new Error("Episode script must include steps");
  }
  snapshot.steps.forEach((step, offset) => {
    if (step.index !== offset + 1) {
      throw new Error(`Episode step index mismatch at ${offset + 1}`);
    }
    if (!step.slot || typeof step.slot.type !== "string") {
      throw new Error(`Episode step ${step.index} has invalid slot`);
    }
    if (!step.plannedPayload || typeof step.plannedPayload !== "object") {
      throw new Error(`Episode step ${step.index} has invalid payload`);
    }
    if (step.speechBeat && step.speechBeat.stepIndex !== step.index) {
      throw new Error(`Episode speech beat mismatch at step ${step.index}`);
    }
  });
  const last = snapshot.steps.at(-1);
  if (last?.slot.type !== "game_ended") {
    throw new Error("Episode script must end with game_ended");
  }
  nonEmpty(snapshot.title, "Episode title");
  nonEmpty(snapshot.logline, "Episode logline");
  return structuredClone(snapshot);
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

export function normalizeEpisodeScriptState(
  value: unknown,
  runMode: Game["runMode"],
): EpisodeScriptState | null {
  if (runMode === "game") return null;
  if (value === undefined || value === null) return { status: "idle" };
  if (typeof value !== "object") throw new Error("Episode script state must be an object");
  const state = value as EpisodeScriptState;
  switch (state.status) {
    case "idle":
    case "generating":
    case "failed":
      return structuredClone(state);
    case "review":
      return { ...structuredClone(state), candidate: validateEpisodeScriptSnapshot(state.candidate) };
    case "approved":
      return { ...structuredClone(state), script: validateEpisodeScriptSnapshot(state.script) };
    default:
      throw new Error("Episode script state has invalid status");
  }
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
  if (input.script.inputHash !== episodeInputHash(input.game)) {
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
    budget,
  };
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
