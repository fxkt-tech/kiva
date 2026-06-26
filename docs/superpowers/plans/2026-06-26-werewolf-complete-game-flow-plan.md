# Werewolf Complete Game Flow Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extend the current editor MVP so a host can complete a full fixed 6-player Werewolf game without LLM: night/day loops, last words, speeches, voting, PK, exile, win checks, endgame reveal, structured draft editing, and playable public preview.

**Architecture:** Keep the official active event log as the only source of truth. Extend the pure core layer first: event model, derived state, vote/exile/win rules, and deterministic advance planner. Then expose structured draft editing through server actions and editor UI. Playback remains separate and reads only active public events.

**Tech Stack:** Next.js 16 App Router, React 19, TypeScript, Tailwind CSS 4, Node JSON repository, Vitest, pnpm.

---

## Scope

This plan implements the complete no-LLM game loop for the existing fixed 6-player ruleset:

- Role assignment
- Repeating night cycles
- Wolf kill
- Seer check/result
- Witch antidote/poison
- Night settlement
- Public death announcement
- Last words
- Day speeches
- Voting
- Tie detection
- PK speeches
- PK revote
- No-exile on second tie
- Exile settlement
- Win checks after night and exile
- Endgame reveal
- Structured draft payload editing for the above actions
- Playback controls for confirmed public events

Out of scope:

- LLM role generation
- Character library management
- Video recording
- Multi-board role selection
- Advanced visual polish
- God-view playback

The purpose is to prove the rules/content pipeline can produce a complete match before connecting model output.

## File Structure

- Modify `src/core/events.ts`: add missing full-game event payloads.
- Modify `src/core/types.ts`: add phase values only if current phases are insufficient.
- Modify `src/core/state.ts`: derive round/day state, deaths pending last words, speeches, votes, PK, exile, and ended state.
- Modify `src/core/rules.ts`: add vote tally, PK voter eligibility, exile settlement, full win check helpers, and target validation.
- Modify `src/core/advance-planner.ts`: replace first-night-only chain with a deterministic full-game planner.
- Create `src/core/draft-edit.ts`: validate and apply structured payload edits to a draft.
- Modify `src/core/playback.ts`: compile a sequence of public playback items, not just the latest display surface.
- Modify `src/server/game-actions.ts`: add structured draft edit action and keep existing display edit.
- Modify `src/app/actions.ts`: add server action for structured draft edits.
- Modify `src/components/editor/draft-panel.tsx`: render draft-type-specific controls.
- Modify `src/components/preview/playback-stage.tsx`: client playback controls for public timeline.
- Create/modify tests:
  - `src/core/__tests__/state.test.ts`
  - `src/core/__tests__/rules.test.ts`
  - `src/core/__tests__/advance-planner.test.ts`
  - `src/core/__tests__/draft-edit.test.ts`
  - `src/core/__tests__/playback.test.ts`
  - `src/server/__tests__/game-actions.test.ts`

## State Model Decisions

The planner must not infer from `game.players.gameRole` once active `role_assigned` events exist. Role truth after setup comes from active role events.

The planner should be deterministic but editable:

- Wolf kill defaults to first legal non-wolf alive target.
- Seer check defaults to first legal unchecked alive target, preferring a wolf if any is alive.
- Witch defaults to no antidote/no poison.
- Speeches and last words default to short draft text, then host can edit.
- Votes default to first legal non-self candidate, then host can edit.
- PK revote defaults to first legal tied candidate.

The planner must return `null` after `game_ended`.

## Task 1: Complete Event Model

**Files:**
- Modify: `src/core/events.ts`
- Test: `src/core/__tests__/events.test.ts`

- [ ] **Step 1: Write event type tests**

Create `src/core/__tests__/events.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import type { GameEvent } from "../events";
import type { EventId, GameId, PlayerId } from "../types";

const gameId = "game_1" as GameId;
const player1 = "player_1" as PlayerId;
const player2 = "player_2" as PlayerId;

describe("full game event model", () => {
  it("supports last words, speeches, voting, exile, and game end events", () => {
    const events: readonly GameEvent[] = [
      {
        id: "event_1" as EventId,
        gameId,
        index: 1,
        status: "active",
        type: "last_words_given",
        phase: "last_words",
        actorPlayerId: player1,
        visibility: { kind: "public" },
        payload: { playerId: player1, text: "我觉得 2 号很可疑。" },
        createdAt: "2026-06-26T00:00:00.000Z",
      },
      {
        id: "event_2" as EventId,
        gameId,
        index: 2,
        status: "active",
        type: "day_speech_given",
        phase: "speech",
        actorPlayerId: player2,
        visibility: { kind: "public" },
        payload: { playerId: player2, round: 1, text: "我会听后置位。" },
        createdAt: "2026-06-26T00:01:00.000Z",
      },
      {
        id: "event_3" as EventId,
        gameId,
        index: 3,
        status: "active",
        type: "vote_cast",
        phase: "vote",
        actorPlayerId: player1,
        targetPlayerIds: [player2],
        visibility: { kind: "public" },
        payload: {
          voterPlayerId: player1,
          targetPlayerId: player2,
          round: 1,
          voteType: "daily",
        },
        createdAt: "2026-06-26T00:02:00.000Z",
      },
      {
        id: "event_4" as EventId,
        gameId,
        index: 4,
        status: "active",
        type: "exile_resolved",
        phase: "vote",
        targetPlayerIds: [player2],
        visibility: { kind: "public" },
        payload: {
          exiledPlayerId: player2,
          tiedPlayerIds: [],
          voteType: "daily",
          voteTable: [{ voterPlayerId: player1, targetPlayerId: player2 }],
        },
        createdAt: "2026-06-26T00:03:00.000Z",
      },
      {
        id: "event_5" as EventId,
        gameId,
        index: 5,
        status: "active",
        type: "game_ended",
        phase: "ended",
        visibility: { kind: "public" },
        payload: {
          winner: "good",
          reason: "all_wolves_dead",
          revealedRoles: [{ playerId: player1, role: "seer", faction: "good" }],
        },
        createdAt: "2026-06-26T00:04:00.000Z",
      },
    ];

    expect(events.map((event) => event.type)).toEqual([
      "last_words_given",
      "day_speech_given",
      "vote_cast",
      "exile_resolved",
      "game_ended",
    ]);
  });
});
```

- [ ] **Step 2: Run the test and verify failure**

Run:

```bash
pnpm test src/core/__tests__/events.test.ts
```

Expected: FAIL because current event payloads do not include `round`, `voteType`, `voteTable`, or `revealedRoles`.

- [ ] **Step 3: Update event types**

Modify `src/core/events.ts`:

```ts
export type VoteType = "daily" | "pk";

export type VoteTableEntry = {
  readonly voterPlayerId: PlayerId;
  readonly targetPlayerId: PlayerId | null;
};

export type RevealedRole = {
  readonly playerId: PlayerId;
  readonly role: GameRole;
  readonly faction: Faction;
};
```

Change existing full-game event definitions to:

```ts
export type LastWordsGivenEvent = GameEventBase<
  "last_words_given",
  { readonly playerId: PlayerId; readonly text: string }
>;

export type DaySpeechGivenEvent = GameEventBase<
  "day_speech_given",
  { readonly playerId: PlayerId; readonly round: number; readonly text: string }
>;

export type VoteCastEvent = GameEventBase<
  "vote_cast",
  {
    readonly voterPlayerId: PlayerId;
    readonly targetPlayerId: PlayerId | null;
    readonly round: number;
    readonly voteType: VoteType;
  }
>;

export type PkSpeechGivenEvent = GameEventBase<
  "pk_speech_given",
  { readonly playerId: PlayerId; readonly round: number; readonly text: string }
>;

export type ExileResolvedEvent = GameEventBase<
  "exile_resolved",
  {
    readonly exiledPlayerId: PlayerId | null;
    readonly tiedPlayerIds: readonly PlayerId[];
    readonly voteType: VoteType;
    readonly voteTable: readonly VoteTableEntry[];
  }
>;

export type GameEndedEvent = GameEventBase<
  "game_ended",
  {
    readonly winner: "wolves" | "good";
    readonly reason:
      | "all_wolves_dead"
      | "all_gods_dead"
      | "all_villagers_dead"
      | "all_good_dead";
    readonly revealedRoles: readonly RevealedRole[];
  }
>;
```

Include `PkSpeechGivenEvent` in `GameEvent`.

- [ ] **Step 4: Run event tests and existing core tests**

Run:

```bash
pnpm test src/core/__tests__/events.test.ts src/core/__tests__/playback.test.ts
pnpm typecheck
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/core/events.ts src/core/__tests__/events.test.ts
git commit -m "feat: extend full game event model"
```

## Task 2: Derive Full Game State

**Files:**
- Modify: `src/core/state.ts`
- Test: `src/core/__tests__/state.test.ts`

- [ ] **Step 1: Add failing state tests**

Append to `src/core/__tests__/state.test.ts`:

```ts
import type { GameEvent } from "../events";
import { createSeedGame } from "../game";
import type { EventId, GameId, PlayerId } from "../types";

describe("full game derived state", () => {
  const gameId = "game_full_state" as GameId;
  const game = createSeedGame({
    gameId,
    createdAt: "2026-06-26T00:00:00.000Z",
  });
  const [p1, p2, p3] = game.players.map((player) => player.playerId);

  it("tracks pending last words after death announcement", () => {
    const state = deriveGameState(game.players, [
      event(1, "night_resolved", "night", { deadPlayerIds: [p1] }),
      event(2, "death_announced", "day", { deadPlayerIds: [p1] }),
    ]);

    expect(state.pendingLastWordPlayerIds).toEqual([p1]);
  });

  it("removes a player from pending last words after they speak", () => {
    const state = deriveGameState(game.players, [
      event(1, "night_resolved", "night", { deadPlayerIds: [p1] }),
      event(2, "death_announced", "day", { deadPlayerIds: [p1] }),
      event(3, "last_words_given", "last_words", {
        playerId: p1,
        text: "我留遗言。",
      }),
    ]);

    expect(state.pendingLastWordPlayerIds).toEqual([]);
  });

  it("tracks speeches and votes for the current day round", () => {
    const state = deriveGameState(game.players, [
      event(1, "phase_started", "speech", { phase: "speech", dayNumber: 1 }),
      event(2, "day_speech_given", "speech", {
        playerId: p1,
        round: 1,
        text: "发言。",
      }),
      event(3, "vote_cast", "vote", {
        voterPlayerId: p1,
        targetPlayerId: p2,
        round: 1,
        voteType: "daily",
      }),
    ]);

    expect(state.spokenPlayerIdsByRound[1]).toEqual([p1]);
    expect(state.votesByRound[1]).toEqual([
      { voterPlayerId: p1, targetPlayerId: p2, voteType: "daily" },
    ]);
  });

  it("marks exiled players dead and detects ended games", () => {
    const state = deriveGameState(game.players, [
      event(1, "exile_resolved", "vote", {
        exiledPlayerId: p3,
        tiedPlayerIds: [],
        voteType: "daily",
        voteTable: [{ voterPlayerId: p1, targetPlayerId: p3 }],
      }),
      event(2, "game_ended", "ended", {
        winner: "wolves",
        reason: "all_gods_dead",
        revealedRoles: [],
      }),
    ]);

    expect(state.deadPlayerIds).toContain(p3);
    expect(state.currentPhase).toBe("ended");
    expect(state.ended).toEqual({ winner: "wolves", reason: "all_gods_dead" });
  });

  function event<Type extends GameEvent["type"]>(
    index: number,
    type: Type,
    phase: GameEvent["phase"],
    payload: Extract<GameEvent, { type: Type }>["payload"],
  ): Extract<GameEvent, { type: Type }> {
    return {
      id: `event_${index}` as EventId,
      gameId,
      index,
      status: "active",
      type,
      phase,
      visibility: { kind: "public" },
      payload,
      createdAt: "2026-06-26T00:00:00.000Z",
    } as Extract<GameEvent, { type: Type }>;
  }
});
```

- [ ] **Step 2: Run state tests and verify failure**

Run:

```bash
pnpm test src/core/__tests__/state.test.ts
```

Expected: FAIL because the new derived state fields do not exist.

- [ ] **Step 3: Extend `DerivedGameState`**

Modify `src/core/state.ts`:

```ts
export type DerivedVote = {
  readonly voterPlayerId: PlayerId;
  readonly targetPlayerId: PlayerId | null;
  readonly voteType: "daily" | "pk";
};

export type DerivedGameState = {
  readonly currentPhase: Phase;
  readonly dayNumber: number;
  readonly alivePlayerIds: readonly PlayerId[];
  readonly deadPlayerIds: readonly PlayerId[];
  readonly pendingLastWordPlayerIds: readonly PlayerId[];
  readonly spokenPlayerIdsByRound: Readonly<Record<number, readonly PlayerId[]>>;
  readonly pkSpokenPlayerIdsByRound: Readonly<Record<number, readonly PlayerId[]>>;
  readonly votesByRound: Readonly<Record<number, readonly DerivedVote[]>>;
  readonly latestTiePlayerIds: readonly PlayerId[];
  readonly ended: null | {
    readonly winner: "wolves" | "good";
    readonly reason:
      | "all_wolves_dead"
      | "all_gods_dead"
      | "all_villagers_dead"
      | "all_good_dead";
  };
  readonly witch: {
    readonly antidoteAvailable: boolean;
    readonly poisonAvailable: boolean;
  };
};
```

Implementation rules:

- `night_resolved.deadPlayerIds` and `exile_resolved.exiledPlayerId` add to dead set.
- `death_announced.deadPlayerIds` append pending last words only for dead players.
- `last_words_given.playerId` removes pending last words.
- `day_speech_given` adds to `spokenPlayerIdsByRound[round]`.
- `pk_speech_given` adds to `pkSpokenPlayerIdsByRound[round]`.
- `vote_cast` appends to `votesByRound[round]`.
- `exile_resolved.tiedPlayerIds` becomes `latestTiePlayerIds`.
- `game_ended` sets current phase to `ended` and sets `ended`.

- [ ] **Step 4: Run state tests and full core tests**

Run:

```bash
pnpm test src/core/__tests__/state.test.ts
pnpm test src/core
pnpm typecheck
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/core/state.ts src/core/__tests__/state.test.ts
git commit -m "feat: derive full game state"
```

## Task 3: Add Vote, Exile, and Endgame Rules

**Files:**
- Modify: `src/core/rules.ts`
- Test: `src/core/__tests__/rules.test.ts`

- [ ] **Step 1: Add failing rules tests**

Append to `src/core/__tests__/rules.test.ts`:

```ts
import {
  resolveVote,
  getEligibleVoters,
  createEndgameReveal,
} from "../rules";

describe("full game vote and endgame rules", () => {
  it("resolves a daily vote to a single exile", () => {
    const result = resolveVote({
      votes: [
        { voterPlayerId: "p1" as never, targetPlayerId: "p3" as never },
        { voterPlayerId: "p2" as never, targetPlayerId: "p3" as never },
        { voterPlayerId: "p3" as never, targetPlayerId: "p1" as never },
      ],
      allowAbstainVote: true,
    });

    expect(result).toEqual({
      exiledPlayerId: "p3",
      tiedPlayerIds: [],
      voteTable: [
        { voterPlayerId: "p1", targetPlayerId: "p3" },
        { voterPlayerId: "p2", targetPlayerId: "p3" },
        { voterPlayerId: "p3", targetPlayerId: "p1" },
      ],
    });
  });

  it("returns tied players when highest vote count ties", () => {
    const result = resolveVote({
      votes: [
        { voterPlayerId: "p1" as never, targetPlayerId: "p3" as never },
        { voterPlayerId: "p2" as never, targetPlayerId: "p4" as never },
      ],
      allowAbstainVote: true,
    });

    expect(result.exiledPlayerId).toBeNull();
    expect(result.tiedPlayerIds).toEqual(["p3", "p4"]);
  });

  it("excludes PK players from PK revote when ruleset says non_pk_only", () => {
    expect(
      getEligibleVoters({
        alivePlayerIds: ["p1", "p2", "p3"] as never,
        voteType: "pk",
        pkPlayerIds: ["p1", "p2"] as never,
        pkVoters: "non_pk_only",
      }),
    ).toEqual(["p3"]);
  });

  it("creates endgame role reveal from player snapshots", () => {
    const game = createSeedGame({
      gameId: "game_rules" as never,
      createdAt: "2026-06-26T00:00:00.000Z",
    });

    expect(createEndgameReveal(game.players)).toHaveLength(6);
  });
});
```

- [ ] **Step 2: Run rules tests and verify failure**

Run:

```bash
pnpm test src/core/__tests__/rules.test.ts
```

Expected: FAIL because these functions do not exist.

- [ ] **Step 3: Implement rules**

Modify `src/core/rules.ts` and export:

```ts
export type VoteInput = {
  readonly voterPlayerId: PlayerId;
  readonly targetPlayerId: PlayerId | null;
};

export type VoteResolution = {
  readonly exiledPlayerId: PlayerId | null;
  readonly tiedPlayerIds: readonly PlayerId[];
  readonly voteTable: readonly VoteInput[];
};

export function resolveVote(input: {
  readonly votes: readonly VoteInput[];
  readonly allowAbstainVote: boolean;
}): VoteResolution;

export function getEligibleVoters(input: {
  readonly alivePlayerIds: readonly PlayerId[];
  readonly voteType: "daily" | "pk";
  readonly pkPlayerIds: readonly PlayerId[];
  readonly pkVoters: PkVoters;
}): readonly PlayerId[];

export function createEndgameReveal(
  players: readonly PlayerSnapshot[],
): readonly RevealedRole[];
```

Rules:

- Abstain votes (`targetPlayerId: null`) appear in `voteTable` but do not count toward exile.
- If no non-null targets receive votes, return no exile and no tie.
- If one target has strictly highest votes, exile that player.
- If multiple targets tie for highest votes, return no exile and tied ids in first-seen order.
- For PK vote with `pkVoters: "non_pk_only"`, exclude tied PK players from voters.
- For `all_living_non_self`, caller still prevents self-vote per candidate UI; return all alive ids.

- [ ] **Step 4: Run rules tests and core tests**

Run:

```bash
pnpm test src/core/__tests__/rules.test.ts
pnpm test src/core
pnpm typecheck
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/core/rules.ts src/core/__tests__/rules.test.ts
git commit -m "feat: add vote and endgame rules"
```

## Task 4: Build Full Advance Planner Loop

**Files:**
- Modify: `src/core/advance-planner.ts`
- Test: `src/core/__tests__/advance-planner.test.ts`

- [ ] **Step 1: Add full-loop planner tests**

Append to `src/core/__tests__/advance-planner.test.ts`:

```ts
describe("complete deterministic game flow", () => {
  it("can advance from setup to game_ended without LLM", () => {
    const game = createGame();
    let events: readonly GameEvent[] = [];
    const plannedTypes: GameEvent["type"][] = [];

    for (let step = 1; step <= 80; step += 1) {
      const draft = planNextDraft({
        game,
        events,
        draftId: draftId(100 + step),
        createdAt,
      });
      if (!draft) break;

      plannedTypes.push(draft.type);
      events = confirmNext(events, draft);

      if (draft.type === "game_ended") break;
    }

    expect(plannedTypes).toContain("last_words_given");
    expect(plannedTypes).toContain("day_speech_given");
    expect(plannedTypes).toContain("vote_cast");
    expect(plannedTypes).toContain("exile_resolved");
    expect(plannedTypes).toContain("game_ended");
    expect(plannedTypes.at(-1)).toBe("game_ended");
  });

  it("plans PK speeches and revote after a tied daily vote", () => {
    const game = createGame();
    const events = createEventsThroughDailyTie(game);

    const draft = planNextDraft({
      game,
      events,
      draftId: draftId(300),
      createdAt,
    });

    expect(draft?.type).toBe("phase_started");
    expect(draft?.payload).toEqual({ phase: "pk", dayNumber: 1 });
  });
});
```

Create test helpers in the same file. `confirmAllUntil(game, stopType)` repeatedly calls `planNextDraft`, confirms each draft, and stops immediately after confirming a draft with `draft.type === stopType`. `createEventsThroughDailyTie(game)` should call `confirmAllUntil(game, "vote_cast")`, then replace the final daily vote events with a deterministic tie between two alive players before returning the event list. Do not import test helpers from production files.

- [ ] **Step 2: Run planner tests and verify failure**

Run:

```bash
pnpm test src/core/__tests__/advance-planner.test.ts
```

Expected: FAIL because planner currently ends after `death_announced`.

- [ ] **Step 3: Refactor planner into step helpers**

Modify `src/core/advance-planner.ts` to keep `planNextDraft(input)` public but split private helpers:

```ts
function planSetupDraft(...): DraftEvent | null;
function planNightDraft(...): DraftEvent | null;
function planLastWordsDraft(...): DraftEvent | null;
function planDaySpeechDraft(...): DraftEvent | null;
function planDailyVoteDraft(...): DraftEvent | null;
function planPkDraft(...): DraftEvent | null;
function planExileOrEndDraft(...): DraftEvent | null;
```

Hard requirements:

- `game_ended` stops all future planning.
- Night phase increments by `dayNumber`.
- After `night_resolved`, check win condition before public death announcement. If ended, draft `game_ended`.
- After `death_announced`, if dead players exist, draft `phase_started(last_words)`, then `last_words_given` for each pending dead player.
- After last words complete, draft `phase_started(speech)`.
- Draft `day_speech_given` once per alive player for current round.
- After speeches complete, draft `phase_started(vote)`.
- Draft one `vote_cast` for each eligible living voter.
- After all daily votes, use `resolveVote`.
- If daily vote ties, draft `exile_resolved` with `exiledPlayerId: null` and tied ids, then `phase_started(pk)`.
- In PK phase, draft `pk_speech_given` for each tied player.
- Then draft PK `vote_cast` for eligible voters.
- After PK votes, use `resolveVote`.
- If PK revote ties again, draft `exile_resolved` with `exiledPlayerId: null` and no exile.
- If exile kills a player, check win condition. If ended, draft `game_ended`; otherwise draft next `phase_started(night)` with dayNumber + 1.
- `game_ended` includes `createEndgameReveal(playersWithActiveRoles)`.

- [ ] **Step 4: Run planner tests and full core tests**

Run:

```bash
pnpm test src/core/__tests__/advance-planner.test.ts
pnpm test src/core
pnpm typecheck
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/core/advance-planner.ts src/core/__tests__/advance-planner.test.ts
git commit -m "feat: advance through complete game flow"
```

## Task 5: Structured Draft Editing

**Files:**
- Create: `src/core/draft-edit.ts`
- Create: `src/core/__tests__/draft-edit.test.ts`
- Modify: `src/server/game-actions.ts`
- Modify: `src/server/__tests__/game-actions.test.ts`
- Modify: `src/app/actions.ts`

- [ ] **Step 1: Write draft edit tests**

Create `src/core/__tests__/draft-edit.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import { createDraftEvent } from "../drafts";
import { applyDraftPayloadEdit } from "../draft-edit";
import type { DraftId, GameId, PlayerId } from "../types";

const gameId = "game_edit" as GameId;
const draftId = "draft_edit" as DraftId;
const p1 = "p1" as PlayerId;
const p2 = "p2" as PlayerId;

describe("draft payload editing", () => {
  it("edits wolf kill target", () => {
    const draft = createDraftEvent({
      id: draftId,
      gameId,
      type: "wolf_kill_selected",
      phase: "night",
      visibility: { kind: "faction_private", faction: "wolves" },
      payload: { targetPlayerId: p1 },
      targetPlayerIds: [p1],
      createdAt: "2026-06-26T00:00:00.000Z",
    });

    const edited = applyDraftPayloadEdit(draft, {
      targetPlayerId: p2,
    });

    expect(edited.payload).toEqual({ targetPlayerId: p2 });
    expect(edited.targetPlayerIds).toEqual([p2]);
  });

  it("edits speech text", () => {
    const draft = createDraftEvent({
      id: draftId,
      gameId,
      type: "day_speech_given",
      phase: "speech",
      actorPlayerId: p1,
      visibility: { kind: "public" },
      payload: { playerId: p1, round: 1, text: "old" },
      createdAt: "2026-06-26T00:00:00.000Z",
    });

    expect(applyDraftPayloadEdit(draft, { text: "new" }).payload).toEqual({
      playerId: p1,
      round: 1,
      text: "new",
    });
  });

  it("rejects unknown edit keys for the draft type", () => {
    const draft = createDraftEvent({
      id: draftId,
      gameId,
      type: "witch_antidote_decided",
      phase: "night",
      visibility: { kind: "player_private", playerIds: [p1] },
      payload: { used: false, targetPlayerId: null },
      createdAt: "2026-06-26T00:00:00.000Z",
    });

    expect(() => applyDraftPayloadEdit(draft, { text: "bad" })).toThrow(
      "Unsupported draft edit",
    );
  });
});
```

- [ ] **Step 2: Run tests and verify failure**

Run:

```bash
pnpm test src/core/__tests__/draft-edit.test.ts
```

Expected: FAIL because `draft-edit.ts` does not exist.

- [ ] **Step 3: Implement draft edit helper**

Create `src/core/draft-edit.ts`:

```ts
import type { DraftEvent } from "./drafts";
import type { PlayerId } from "./types";

export type DraftPayloadEdit = {
  readonly targetPlayerId?: PlayerId | null;
  readonly used?: boolean;
  readonly text?: string;
};

export function applyDraftPayloadEdit(
  draft: DraftEvent,
  edit: DraftPayloadEdit,
): DraftEvent;
```

Supported edits:

- `wolf_kill_selected`, `seer_check_selected`: `targetPlayerId`.
- `witch_antidote_decided`, `witch_poison_decided`: `used`, `targetPlayerId`.
- `last_words_given`, `day_speech_given`, `pk_speech_given`: `text`.
- `vote_cast`: `targetPlayerId`.

For target edits, update `targetPlayerIds`.

Reject unsupported edit keys with:

```ts
throw new Error(`Unsupported draft edit for ${draft.type}`);
```

- [ ] **Step 4: Add server action support**

Modify `src/server/game-actions.ts`:

```ts
import { applyDraftPayloadEdit, type DraftPayloadEdit } from "@/core/draft-edit";
```

Add method:

```ts
async editDraftPayload(gameId: GameId, edit: DraftPayloadEdit): Promise<GameRecord> {
  return repository.withGameLock(gameId, async () => {
    const record = await loadGame(gameId);
    if (!record.draft) return record;
    const updatedAt = now();
    const nextRecord: GameRecord = {
      ...record,
      game: { ...record.game, updatedAt },
      draft: applyDraftPayloadEdit(record.draft, edit),
    };
    await repository.save(nextRecord);
    return nextRecord;
  });
}
```

Modify `src/app/actions.ts`:

```ts
export async function editDraftPayloadAction(gameId: GameId, formData: FormData) {
  await gameActions.editDraftPayload(gameId, {
    targetPlayerId: formData.get("targetPlayerId")
      ? (String(formData.get("targetPlayerId")) as PlayerId)
      : null,
    used: formData.get("used") === "true",
    text: formData.get("text") ? String(formData.get("text")) : undefined,
  });
  revalidatePath(editorPath(gameId));
  revalidatePath(previewPath(gameId));
}
```

Import `PlayerId`.

- [ ] **Step 5: Add server tests**

Append to `src/server/__tests__/game-actions.test.ts`:

```ts
it("edits current draft payload before confirmation", async () => {
  const { actions, repository } = await createActions();
  const created = await actions.createGame();
  const playerId = created.game.players[0].playerId;
  const draft = createDraftEvent({
    id: "draft_payload_edit" as DraftId,
    gameId: created.game.id,
    type: "day_speech_given",
    phase: "speech",
    actorPlayerId: playerId,
    visibility: { kind: "public" },
    payload: { playerId, round: 1, text: "old text" },
    display: { title: "发言", text: "old text" },
    createdAt: "2026-06-26T00:00:00.000Z",
  });

  await repository.save({ ...created, draft });

  const edited = await actions.editDraftPayload(created.game.id, {
    text: "new text",
  });

  expect(edited.draft?.payload).toMatchObject({ text: "new text" });

  const confirmed = await actions.confirmDraft(created.game.id);
  expect(confirmed.events[0]?.payload).toMatchObject({ text: "new text" });
});
```

Add imports for `createDraftEvent` and `DraftId`.

- [ ] **Step 6: Run draft edit and server tests**

Run:

```bash
pnpm test src/core/__tests__/draft-edit.test.ts src/server/__tests__/game-actions.test.ts
pnpm typecheck
```

Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add src/core/draft-edit.ts src/core/__tests__/draft-edit.test.ts src/server/game-actions.ts src/server/__tests__/game-actions.test.ts src/app/actions.ts
git commit -m "feat: add structured draft editing"
```

## Task 6: Add Draft-Type Editor Controls

**Files:**
- Modify: `src/components/editor/draft-panel.tsx`

- [ ] **Step 1: Add draft controls**

Modify `DraftPanel` to render a second form for structured payload edits.

Add helper:

```tsx
function DraftPayloadControls({ draft }: { readonly draft: DraftEvent }) {
  switch (draft.type) {
    case "wolf_kill_selected":
    case "seer_check_selected":
    case "vote_cast":
      return <input name="targetPlayerId" defaultValue={draft.payload.targetPlayerId ?? ""} />;
    case "witch_antidote_decided":
    case "witch_poison_decided":
      return (
        <>
          <select name="used" defaultValue={draft.payload.used ? "true" : "false"}>
            <option value="false">不使用</option>
            <option value="true">使用</option>
          </select>
          <input name="targetPlayerId" defaultValue={draft.payload.targetPlayerId ?? ""} />
        </>
      );
    case "last_words_given":
    case "day_speech_given":
    case "pk_speech_given":
      return <textarea name="text" defaultValue={draft.payload.text} />;
    default:
      return null;
  }
}
```

Wrap controls in:

```tsx
<form action={editDraftPayloadAction.bind(null, gameId)}>
  <DraftPayloadControls draft={draft} />
  <button type="submit">Save action</button>
</form>
```

Keep the existing display edit form.

- [ ] **Step 2: Run typecheck and build**

Run:

```bash
pnpm typecheck
pnpm build
```

Expected: PASS.

- [ ] **Step 3: Commit**

```bash
git add src/components/editor/draft-panel.tsx
git commit -m "feat: add draft action controls"
```

## Task 7: Add Playback Sequence Controls

**Files:**
- Modify: `src/components/preview/playback-stage.tsx`
- Modify: `src/core/playback.ts`
- Modify: `src/core/__tests__/playback.test.ts`

- [ ] **Step 1: Ensure playback compiler emits stable duration**

Add optional duration to `PlaybackItem`:

```ts
readonly durationMs: number;
```

Default public event duration:

- `phase_started`: 1600
- `death_announced`: 2200
- `last_words_given`, `day_speech_given`, `pk_speech_given`: 4200
- `vote_cast`: 1200
- `exile_resolved`: 2400
- `game_ended`: 5000
- fallback: 2200

Update playback tests to assert `durationMs`.

- [ ] **Step 2: Make playback stage client-controlled**

Modify `src/components/preview/playback-stage.tsx`:

```tsx
"use client";
```

Add state:

```tsx
const [index, setIndex] = useState(0);
const [playing, setPlaying] = useState(false);
const [speed, setSpeed] = useState(1);
```

Use `useEffect` to advance after `current.durationMs / speed`.

Controls:

- Play/Pause
- Restart
- Previous
- Next
- Speed select: 0.5, 1, 1.5, 2

Keep controls outside the 16:9 stage or as a bottom overlay that can later be hidden for recording. Do not show editor/debug/global truth.

- [ ] **Step 3: Run tests and build**

Run:

```bash
pnpm test src/core/__tests__/playback.test.ts
pnpm typecheck
pnpm build
```

Expected: PASS.

- [ ] **Step 4: Commit**

```bash
git add src/core/playback.ts src/core/__tests__/playback.test.ts src/components/preview/playback-stage.tsx
git commit -m "feat: add playback sequence controls"
```

## Task 8: Full Verification and Smoke Test

**Files:**
- No planned code files unless fixing verification failures.

- [ ] **Step 1: Run all automated verification**

Run:

```bash
pnpm test
pnpm typecheck
pnpm build
```

Expected: PASS.

- [ ] **Step 2: Start dev server**

Run:

```bash
pnpm dev
```

Expected: local URL printed, usually `http://localhost:3000` or next available port.

- [ ] **Step 3: Manual smoke**

In the running app:

1. Open `/`.
2. Create a game.
3. Repeatedly click `Continue` and `Confirm draft`.
4. Edit at least one speech draft payload before confirmation.
5. Continue until a `game_ended` event appears.
6. Open preview.
7. Verify preview controls can play through public events.
8. Verify no private wolf/seer/witch events appear in public preview.
9. Roll back to an earlier event and verify later events become superseded.

- [ ] **Step 4: Commit verification fixes if needed**

If fixes are needed:

```bash
git add <changed-files>
git commit -m "fix: stabilize complete game flow"
```

If no fixes are needed, do not create an empty commit.

## Self-Review

- Spec coverage: covers complete fixed-board rules from setup to endgame reveal, no-LLM deterministic content generation, structured host edits, public-only playback, and editor/playback separation.
- Placeholder scan: no unresolved implementation markers remain.
- Type consistency: new event payloads feed derived state, rules, planner, draft editing, server actions, editor controls, and playback.
- Scope check: LLM and video recording are intentionally deferred. This plan creates a complete no-LLM game, which is the prerequisite for both.
