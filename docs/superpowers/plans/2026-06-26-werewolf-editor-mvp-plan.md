# Werewolf Editor MVP Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the first usable local Werewolf director loop: create a fixed 6-player game, advance confirmed events through host-reviewed drafts, persist the game, preview a separate 16:9 playback page, and reload/replay the saved event log.

**Architecture:** Keep the official event log as the only source of truth. Add a small local repository layer backed by JSON files, a deterministic advance planner that creates editable drafts, server actions that confirm/rollback drafts into active events, and separate editor/preview pages that read the same persisted game data but render different surfaces.

**Tech Stack:** Next.js 16 App Router, React 19, TypeScript, Tailwind CSS 4, Node `fs/promises`, Vitest, pnpm.

---

## Scope

This plan implements the next product slice after the core foundation:

- Local JSON persistence for games.
- Fixed 6-player seed game creation.
- Draft event model for host review.
- Deterministic advance planner for the first playable content chain.
- Server actions for create, continue, edit draft display text, confirm, rollback, and delete draft.
- Editor page with timeline, board state, draft controls, and embedded preview.
- Separate 16:9 preview page for future recording.
- Tests for persistence, planner behavior, server action boundaries, and playback compilation.

Out of scope for this plan:

- LLM provider integration.
- Character library management UI.
- Browser video recording.
- Full final visual polish.
- Multi-game dashboard beyond a minimal entry point.

The point is to validate that the content creation loop works before adding generation and recording complexity.

## File Structure

- `src/core/game.ts`: Game aggregate types and creation helpers.
- `src/core/drafts.ts`: Draft event types, validation helpers, and conversion to official events.
- `src/core/advance-planner.ts`: Deterministic planner that creates the next host-reviewable draft.
- `src/core/playback.ts`: Converts active events into public preview timeline items.
- `src/core/id.ts`: Local deterministic ID helpers for server-side creation.
- `src/core/__tests__/drafts.test.ts`: Draft conversion tests.
- `src/core/__tests__/advance-planner.test.ts`: Continue-flow tests.
- `src/core/__tests__/playback.test.ts`: Preview timeline tests.
- `src/server/game-repository.ts`: JSON file repository for local games.
- `src/server/game-actions.ts`: Server actions for editor operations.
- `src/server/__tests__/game-repository.test.ts`: Repository tests using temporary directories.
- `src/server/__tests__/game-actions.test.ts`: Action behavior tests against an injected temp repository.
- `src/app/actions.ts`: Thin Next.js server action exports.
- `src/app/page.tsx`: Minimal game launcher.
- `src/app/games/[gameId]/editor/page.tsx`: Editor route.
- `src/app/games/[gameId]/preview/page.tsx`: Standalone preview route.
- `src/components/editor/game-board.tsx`: Seat/state board.
- `src/components/editor/event-timeline.tsx`: Official event timeline.
- `src/components/editor/draft-panel.tsx`: Draft review/edit/confirm controls.
- `src/components/preview/playback-stage.tsx`: 16:9 playback surface.
- `src/app/globals.css`: Global theme adjustments.

## Data Decisions

Use local JSON files under `.kiva-data/games/<gameId>.json`.

Reason: this is the smallest persistence layer that proves the product loop. SQLite can replace the repository implementation later without changing the editor pages or core planner.

The JSON record shape:

```ts
export type GameRecord = {
  readonly game: Game;
  readonly events: readonly GameEvent[];
  readonly draft: DraftEvent | null;
};
```

`.kiva-data/` must be ignored by git.

## Task 1: Add Game Aggregate and Draft Model

**Files:**
- Create: `src/core/game.ts`
- Create: `src/core/drafts.ts`
- Create: `src/core/__tests__/drafts.test.ts`

- [ ] **Step 1: Write draft tests**

Create `src/core/__tests__/drafts.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import { confirmDraftEvent, createDraftEvent } from "../drafts";
import { createDefaultRuleset } from "../types";

const gameId = "game_1" as never;
const draftId = "draft_1" as never;
const eventId = "event_1" as never;
const playerId = "player_1" as never;

describe("drafts", () => {
  it("creates an editable draft that is not an official fact", () => {
    const draft = createDraftEvent({
      id: draftId,
      gameId,
      type: "death_announced",
      phase: "day",
      actorPlayerId: undefined,
      targetPlayerIds: [playerId],
      visibility: { kind: "public" },
      payload: { deadPlayerIds: [playerId] },
      display: { title: "昨夜死讯", text: "昨夜 1 号玩家死亡。" },
      createdAt: "2026-06-26T00:00:00.000Z",
    });

    expect(draft.status).toBe("draft");
    expect(draft.payload).toEqual({ deadPlayerIds: [playerId] });
  });

  it("confirms a draft into the next active official event", () => {
    const draft = createDraftEvent({
      id: draftId,
      gameId,
      type: "death_announced",
      phase: "day",
      actorPlayerId: undefined,
      targetPlayerIds: [playerId],
      visibility: { kind: "public" },
      payload: { deadPlayerIds: [playerId] },
      display: { title: "昨夜死讯", text: "昨夜 1 号玩家死亡。" },
      createdAt: "2026-06-26T00:00:00.000Z",
    });

    const event = confirmDraftEvent({
      draft,
      eventId,
      index: 3,
      createdAt: "2026-06-26T00:01:00.000Z",
    });

    expect(event).toMatchObject({
      id: eventId,
      gameId,
      index: 3,
      status: "active",
      createdFromDraftId: draftId,
      type: "death_announced",
      phase: "day",
    });
  });

  it("creates a fixed six-player game aggregate", async () => {
    const { createSeedGame } = await import("../game");
    const record = createSeedGame({
      gameId,
      createdAt: "2026-06-26T00:00:00.000Z",
      ruleset: createDefaultRuleset(),
    });

    expect(record.players).toHaveLength(6);
    expect(record.players.map((player) => player.seatNo)).toEqual([
      1, 2, 3, 4, 5, 6,
    ]);
  });
});
```

- [ ] **Step 2: Run tests to verify they fail**

Run:

```bash
pnpm test src/core/__tests__/drafts.test.ts
```

Expected: FAIL because `src/core/drafts.ts` and `src/core/game.ts` do not exist.

- [ ] **Step 3: Implement game aggregate**

Create `src/core/game.ts`:

```ts
import { createPlayerSnapshot, type PlayerSnapshot } from "./player";
import {
  createDefaultRuleset,
  type GameId,
  type PlayerId,
  type Ruleset,
} from "./types";

export type GameStatus = "drafting" | "ended";

export type Game = {
  readonly id: GameId;
  readonly title: string;
  readonly status: GameStatus;
  readonly ruleset: Ruleset;
  readonly players: readonly PlayerSnapshot[];
  readonly createdAt: string;
  readonly updatedAt: string;
};

export type CreateSeedGameInput = {
  readonly gameId: GameId;
  readonly createdAt: string;
  readonly ruleset?: Ruleset;
};

export function createSeedGame(input: CreateSeedGameInput): Game {
  const ruleset = input.ruleset ?? createDefaultRuleset();
  const players = [
    ["p1", 1, "秦川", "werewolf"],
    ["p2", 2, "林夏", "werewolf"],
    ["p3", 3, "周知", "seer"],
    ["p4", 4, "许棠", "witch"],
    ["p5", 5, "陈墨", "villager"],
    ["p6", 6, "沈岚", "villager"],
  ] as const;

  return {
    id: input.gameId,
    title: "6人狼人杀试运行",
    status: "drafting",
    ruleset,
    players: players.map(([suffix, seatNo, name, role]) =>
      createPlayerSnapshot({
        playerId: `${input.gameId}_${suffix}` as PlayerId,
        seatNo,
        name,
        gameRole: role,
        persona: "冷静、愿意表达判断",
        speakingStyle: "短句、直接、有推进感",
        reasoningStyle: "根据自己可见的信息给出结论",
        systemPrompt: "你是狼人杀对局中的一名玩家，只能依据你可见的信息行动。",
      }),
    ),
    createdAt: input.createdAt,
    updatedAt: input.createdAt,
  };
}
```

- [ ] **Step 4: Implement draft helpers**

Create `src/core/drafts.ts`:

```ts
import type {
  DeathAnnouncedEvent,
  GameEvent,
  GameEventBase,
  NightResolvedEvent,
  PhaseStartedEvent,
  RoleAssignedEvent,
  SeerCheckResultEvent,
  SeerCheckSelectedEvent,
  WitchAntidoteDecidedEvent,
  WitchDeathInfoShownEvent,
  WitchPoisonDecidedEvent,
  WolfKillSelectedEvent,
} from "./events";
import type { DraftId, EventId, GameId, Phase, PlayerId } from "./types";

export type DraftStatus = "draft";

type DraftBase<Type extends GameEvent["type"], Payload> = Omit<
  GameEventBase<Type, Payload>,
  "id" | "index" | "status" | "createdFromDraftId"
> & {
  readonly id: DraftId;
  readonly status: DraftStatus;
  readonly reason: string;
};

export type DraftEvent =
  | DraftBase<PhaseStartedEvent["type"], PhaseStartedEvent["payload"]>
  | DraftBase<RoleAssignedEvent["type"], RoleAssignedEvent["payload"]>
  | DraftBase<WolfKillSelectedEvent["type"], WolfKillSelectedEvent["payload"]>
  | DraftBase<SeerCheckSelectedEvent["type"], SeerCheckSelectedEvent["payload"]>
  | DraftBase<SeerCheckResultEvent["type"], SeerCheckResultEvent["payload"]>
  | DraftBase<WitchDeathInfoShownEvent["type"], WitchDeathInfoShownEvent["payload"]>
  | DraftBase<WitchAntidoteDecidedEvent["type"], WitchAntidoteDecidedEvent["payload"]>
  | DraftBase<WitchPoisonDecidedEvent["type"], WitchPoisonDecidedEvent["payload"]>
  | DraftBase<NightResolvedEvent["type"], NightResolvedEvent["payload"]>
  | DraftBase<DeathAnnouncedEvent["type"], DeathAnnouncedEvent["payload"]>;

export type CreateDraftEventInput = Omit<
  DraftEvent,
  "status" | "reason"
> & {
  readonly reason?: string;
};

export type ConfirmDraftEventInput = {
  readonly draft: DraftEvent;
  readonly eventId: EventId;
  readonly index: number;
  readonly createdAt: string;
};

export function createDraftEvent(input: CreateDraftEventInput): DraftEvent {
  return {
    ...input,
    status: "draft",
    reason: input.reason ?? "等待主理人确认",
  } as DraftEvent;
}

export function confirmDraftEvent(input: ConfirmDraftEventInput): GameEvent {
  const { draft, eventId, index, createdAt } = input;

  return {
    id: eventId,
    gameId: draft.gameId as GameId,
    index,
    status: "active",
    type: draft.type,
    phase: draft.phase as Phase,
    actorPlayerId: draft.actorPlayerId as PlayerId | undefined,
    targetPlayerIds: draft.targetPlayerIds,
    visibility: draft.visibility,
    payload: draft.payload,
    display: draft.display,
    createdFromDraftId: draft.id,
    createdAt,
  } as GameEvent;
}
```

- [ ] **Step 5: Run draft tests**

Run:

```bash
pnpm test src/core/__tests__/drafts.test.ts
```

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add src/core/game.ts src/core/drafts.ts src/core/__tests__/drafts.test.ts
git commit -m "feat: add game aggregate and draft model"
```

## Task 2: Add Deterministic Advance Planner

**Files:**
- Create: `src/core/advance-planner.ts`
- Create: `src/core/__tests__/advance-planner.test.ts`
- Modify: `src/core/events.ts`

- [ ] **Step 1: Extend event types for speeches and votes**

Modify `src/core/events.ts` by adding these event types before the `GameEvent` union:

```ts
export type LastWordsGivenEvent = GameEventBase<
  "last_words_given",
  { readonly playerId: PlayerId; readonly text: string }
>;

export type DaySpeechGivenEvent = GameEventBase<
  "day_speech_given",
  { readonly playerId: PlayerId; readonly text: string }
>;

export type VoteCastEvent = GameEventBase<
  "vote_cast",
  { readonly voterPlayerId: PlayerId; readonly targetPlayerId: PlayerId | null }
>;

export type ExileResolvedEvent = GameEventBase<
  "exile_resolved",
  { readonly exiledPlayerId: PlayerId | null; readonly tiedPlayerIds: readonly PlayerId[] }
>;

export type GameEndedEvent = GameEventBase<
  "game_ended",
  { readonly winner: "wolves" | "good"; readonly reason: string }
>;
```

Then include them in the `GameEvent` union:

```ts
  | LastWordsGivenEvent
  | DaySpeechGivenEvent
  | VoteCastEvent
  | ExileResolvedEvent
  | GameEndedEvent;
```

- [ ] **Step 2: Write planner tests**

Create `src/core/__tests__/advance-planner.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import { appendEvent } from "../event-log";
import { confirmDraftEvent } from "../drafts";
import { createSeedGame } from "../game";
import { planNextDraft } from "../advance-planner";
import { createDefaultRuleset } from "../types";

const gameId = "game_1" as never;

function seed() {
  const game = createSeedGame({
    gameId,
    createdAt: "2026-06-26T00:00:00.000Z",
    ruleset: createDefaultRuleset(),
  });
  return { game, events: [] };
}

describe("advance planner", () => {
  it("starts with private role assignment drafts", () => {
    const { game, events } = seed();
    const draft = planNextDraft({
      game,
      events,
      draftId: "draft_1" as never,
      createdAt: "2026-06-26T00:00:01.000Z",
    });

    expect(draft?.type).toBe("role_assigned");
    expect(draft?.visibility.kind).toBe("player_private");
  });

  it("plans night after all roles are assigned", () => {
    const { game } = seed();
    let events = [];

    for (let index = 0; index < game.players.length; index += 1) {
      const draft = planNextDraft({
        game,
        events,
        draftId: `draft_${index}` as never,
        createdAt: "2026-06-26T00:00:01.000Z",
      });
      if (!draft) throw new Error("Expected draft");
      events = appendEvent(
        events,
        confirmDraftEvent({
          draft,
          eventId: `event_${index}` as never,
          index: index + 1,
          createdAt: "2026-06-26T00:00:02.000Z",
        }),
      ) as never[];
    }

    const next = planNextDraft({
      game,
      events,
      draftId: "draft_night" as never,
      createdAt: "2026-06-26T00:00:03.000Z",
    });

    expect(next?.type).toBe("phase_started");
    expect(next?.payload).toEqual({ phase: "night", dayNumber: 1 });
  });

  it("produces a deterministic first night chain", () => {
    const { game } = seed();
    let events = [];
    const types: string[] = [];

    for (let index = 0; index < 13; index += 1) {
      const draft = planNextDraft({
        game,
        events,
        draftId: `draft_${index}` as never,
        createdAt: "2026-06-26T00:00:01.000Z",
      });
      if (!draft) break;
      types.push(draft.type);
      events = appendEvent(
        events,
        confirmDraftEvent({
          draft,
          eventId: `event_${index}` as never,
          index: index + 1,
          createdAt: "2026-06-26T00:00:02.000Z",
        }),
      ) as never[];
    }

    expect(types).toContain("wolf_kill_selected");
    expect(types).toContain("seer_check_selected");
    expect(types).toContain("seer_check_result");
    expect(types).toContain("witch_death_info_shown");
    expect(types).toContain("night_resolved");
    expect(types).toContain("death_announced");
  });
});
```

- [ ] **Step 3: Run planner tests to verify they fail**

Run:

```bash
pnpm test src/core/__tests__/advance-planner.test.ts
```

Expected: FAIL because `planNextDraft` does not exist.

- [ ] **Step 4: Implement planner**

Create `src/core/advance-planner.ts`:

```ts
import { createDraftEvent, type DraftEvent } from "./drafts";
import { getActiveEvents } from "./event-log";
import type { GameEvent } from "./events";
import type { Game } from "./game";
import { resolveNightDeaths } from "./rules";
import type { DraftId } from "./types";

export type PlanNextDraftInput = {
  readonly game: Game;
  readonly events: readonly GameEvent[];
  readonly draftId: DraftId;
  readonly createdAt: string;
};

export function planNextDraft(input: PlanNextDraftInput): DraftEvent | null {
  const events = getActiveEvents(input.events);
  const assignedPlayerIds = new Set(
    events
      .filter((event) => event.type === "role_assigned")
      .map((event) => event.payload.playerId),
  );
  const unassigned = input.game.players.find(
    (player) => !assignedPlayerIds.has(player.playerId),
  );

  if (unassigned) {
    return createDraftEvent({
      id: input.draftId,
      gameId: input.game.id,
      type: "role_assigned",
      phase: "setup",
      actorPlayerId: undefined,
      targetPlayerIds: [unassigned.playerId],
      visibility: { kind: "player_private", playerIds: [unassigned.playerId] },
      payload: {
        playerId: unassigned.playerId,
        role: unassigned.gameRole,
        faction: unassigned.faction,
      },
      display: {
        title: `${unassigned.seatNo}号身份确认`,
        text: `${unassigned.name} 的身份已写入私密日志。`,
      },
      createdAt: input.createdAt,
      reason: "开局需要把每名玩家的身份作为私密事实写入日志",
    });
  }

  if (!hasEvent(events, "phase_started", (event) => event.payload.phase === "night")) {
    return createDraftEvent({
      id: input.draftId,
      gameId: input.game.id,
      type: "phase_started",
      phase: "night",
      visibility: { kind: "public" },
      payload: { phase: "night", dayNumber: 1 },
      display: { title: "夜晚开始", text: "天黑请闭眼。" },
      createdAt: input.createdAt,
      reason: "身份确认完成后进入第一夜",
    });
  }

  const firstWolfTarget = input.game.players.find((player) => player.gameRole !== "werewolf");
  if (!hasEvent(events, "wolf_kill_selected") && firstWolfTarget) {
    return createDraftEvent({
      id: input.draftId,
      gameId: input.game.id,
      type: "wolf_kill_selected",
      phase: "night",
      actorPlayerId: input.game.players.find((player) => player.gameRole === "werewolf")?.playerId,
      targetPlayerIds: [firstWolfTarget.playerId],
      visibility: { kind: "faction_private", faction: "wolves" },
      payload: { targetPlayerId: firstWolfTarget.playerId },
      display: { title: "狼人刀人", text: `狼人选择袭击 ${firstWolfTarget.seatNo}号。` },
      createdAt: input.createdAt,
      reason: "第一版使用确定性目标，后续由人工编辑或 LLM 替换",
    });
  }

  const seer = input.game.players.find((player) => player.gameRole === "seer");
  const wolf = input.game.players.find((player) => player.gameRole === "werewolf");
  if (!hasEvent(events, "seer_check_selected") && seer && wolf) {
    return createDraftEvent({
      id: input.draftId,
      gameId: input.game.id,
      type: "seer_check_selected",
      phase: "night",
      actorPlayerId: seer.playerId,
      targetPlayerIds: [wolf.playerId],
      visibility: { kind: "player_private", playerIds: [seer.playerId] },
      payload: { targetPlayerId: wolf.playerId },
      display: { title: "预言家查验", text: `预言家选择查验 ${wolf.seatNo}号。` },
      createdAt: input.createdAt,
      reason: "第一版使用确定性查验目标，先验证事件链",
    });
  }

  const seerCheck = events.find((event) => event.type === "seer_check_selected");
  if (!hasEvent(events, "seer_check_result") && seer && seerCheck?.type === "seer_check_selected") {
    const target = input.game.players.find(
      (player) => player.playerId === seerCheck.payload.targetPlayerId,
    );
    if (!target) return null;
    return createDraftEvent({
      id: input.draftId,
      gameId: input.game.id,
      type: "seer_check_result",
      phase: "night",
      actorPlayerId: seer.playerId,
      targetPlayerIds: [target.playerId],
      visibility: { kind: "player_private", playerIds: [seer.playerId] },
      payload: { targetPlayerId: target.playerId, result: target.faction },
      display: { title: "查验结果", text: `${target.seatNo}号的阵营是${target.faction === "wolves" ? "狼人" : "好人"}。` },
      createdAt: input.createdAt,
      reason: "查验动作确认后，系统结算私密结果",
    });
  }

  const kill = events.find((event) => event.type === "wolf_kill_selected");
  const witch = input.game.players.find((player) => player.gameRole === "witch");
  if (!hasEvent(events, "witch_death_info_shown") && witch && kill?.type === "wolf_kill_selected") {
    return createDraftEvent({
      id: input.draftId,
      gameId: input.game.id,
      type: "witch_death_info_shown",
      phase: "night",
      actorPlayerId: witch.playerId,
      targetPlayerIds: [kill.payload.targetPlayerId],
      visibility: { kind: "player_private", playerIds: [witch.playerId] },
      payload: { killedPlayerId: kill.payload.targetPlayerId },
      display: { title: "女巫死亡信息", text: "女巫看到了今晚被袭击的玩家。" },
      createdAt: input.createdAt,
      reason: "女巫行动前需要先看到夜晚死亡信息",
    });
  }

  if (!hasEvent(events, "witch_antidote_decided") && witch) {
    return createDraftEvent({
      id: input.draftId,
      gameId: input.game.id,
      type: "witch_antidote_decided",
      phase: "night",
      actorPlayerId: witch.playerId,
      visibility: { kind: "player_private", playerIds: [witch.playerId] },
      payload: { used: false, targetPlayerId: null },
      display: { title: "女巫解药", text: "女巫选择不使用解药。" },
      createdAt: input.createdAt,
      reason: "第一版默认不救，主理人可编辑草稿",
    });
  }

  if (!hasEvent(events, "witch_poison_decided") && witch) {
    return createDraftEvent({
      id: input.draftId,
      gameId: input.game.id,
      type: "witch_poison_decided",
      phase: "night",
      actorPlayerId: witch.playerId,
      visibility: { kind: "player_private", playerIds: [witch.playerId] },
      payload: { used: false, targetPlayerId: null },
      display: { title: "女巫毒药", text: "女巫选择不使用毒药。" },
      createdAt: input.createdAt,
      reason: "第一版默认不毒，主理人可编辑草稿",
    });
  }

  if (!hasEvent(events, "night_resolved")) {
    const antidote = events.find((event) => event.type === "witch_antidote_decided");
    const poison = events.find((event) => event.type === "witch_poison_decided");
    const deadPlayerIds = resolveNightDeaths({
      wolfKillTargetId: kill?.type === "wolf_kill_selected" ? kill.payload.targetPlayerId : null,
      antidoteTargetId: antidote?.type === "witch_antidote_decided" ? antidote.payload.targetPlayerId : null,
      poisonTargetId: poison?.type === "witch_poison_decided" ? poison.payload.targetPlayerId : null,
    });
    return createDraftEvent({
      id: input.draftId,
      gameId: input.game.id,
      type: "night_resolved",
      phase: "night",
      visibility: { kind: "host_only" },
      payload: { deadPlayerIds },
      display: { title: "夜晚结算", text: `夜晚死亡人数：${deadPlayerIds.length}` },
      createdAt: input.createdAt,
      reason: "所有夜间行动确认后，系统结算死亡",
    });
  }

  const resolved = events.find((event) => event.type === "night_resolved");
  if (!hasEvent(events, "death_announced") && resolved?.type === "night_resolved") {
    return createDraftEvent({
      id: input.draftId,
      gameId: input.game.id,
      type: "death_announced",
      phase: "day",
      targetPlayerIds: resolved.payload.deadPlayerIds,
      visibility: { kind: "public" },
      payload: { deadPlayerIds: resolved.payload.deadPlayerIds },
      display: { title: "昨夜死讯", text: resolved.payload.deadPlayerIds.length === 0 ? "昨夜平安夜。" : "昨夜有玩家死亡。" },
      createdAt: input.createdAt,
      reason: "夜晚结算需要转换为公开死讯，供预览页展示",
    });
  }

  return null;
}

function hasEvent<Type extends GameEvent["type"]>(
  events: readonly GameEvent[],
  type: Type,
  predicate: (event: Extract<GameEvent, { type: Type }>) => boolean = () => true,
): boolean {
  return events.some(
    (event): event is Extract<GameEvent, { type: Type }> =>
      event.type === type && predicate(event as Extract<GameEvent, { type: Type }>),
  );
}
```

- [ ] **Step 5: Run planner tests**

Run:

```bash
pnpm test src/core/__tests__/advance-planner.test.ts
```

Expected: PASS.

- [ ] **Step 6: Run existing core tests**

Run:

```bash
pnpm test src/core
```

Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add src/core/events.ts src/core/advance-planner.ts src/core/__tests__/advance-planner.test.ts
git commit -m "feat: add deterministic advance planner"
```

## Task 3: Add Local Game Repository

**Files:**
- Create: `src/server/game-repository.ts`
- Create: `src/server/__tests__/game-repository.test.ts`
- Modify: `.gitignore`

- [ ] **Step 1: Ignore local game data**

Modify `.gitignore`:

```gitignore
.kiva-data/
```

- [ ] **Step 2: Write repository tests**

Create `src/server/__tests__/game-repository.test.ts`:

```ts
import { mkdtemp, rm } from "node:fs/promises";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { afterEach, describe, expect, it } from "vitest";
import { createGameRepository } from "../game-repository";
import { createSeedGame } from "../../core/game";
import { createDefaultRuleset } from "../../core/types";

let tempDir: string | null = null;

afterEach(async () => {
  if (tempDir) {
    await rm(tempDir, { recursive: true, force: true });
    tempDir = null;
  }
});

describe("game repository", () => {
  it("saves and loads a game record", async () => {
    tempDir = await mkdtemp(join(tmpdir(), "kiva-repo-"));
    const repo = createGameRepository(tempDir);
    const game = createSeedGame({
      gameId: "game_1" as never,
      createdAt: "2026-06-26T00:00:00.000Z",
      ruleset: createDefaultRuleset(),
    });

    await repo.save({ game, events: [], draft: null });
    const loaded = await repo.get(game.id);

    expect(loaded?.game.id).toBe(game.id);
    expect(loaded?.events).toEqual([]);
    expect(loaded?.draft).toBeNull();
  });

  it("lists records newest first", async () => {
    tempDir = await mkdtemp(join(tmpdir(), "kiva-repo-"));
    const repo = createGameRepository(tempDir);
    const first = createSeedGame({
      gameId: "game_1" as never,
      createdAt: "2026-06-26T00:00:00.000Z",
      ruleset: createDefaultRuleset(),
    });
    const second = createSeedGame({
      gameId: "game_2" as never,
      createdAt: "2026-06-26T00:01:00.000Z",
      ruleset: createDefaultRuleset(),
    });

    await repo.save({ game: first, events: [], draft: null });
    await repo.save({ game: second, events: [], draft: null });

    await expect(repo.list()).resolves.toMatchObject([
      { game: { id: "game_2" } },
      { game: { id: "game_1" } },
    ]);
  });
});
```

- [ ] **Step 3: Run repository tests to verify they fail**

Run:

```bash
pnpm test src/server/__tests__/game-repository.test.ts
```

Expected: FAIL because `game-repository.ts` does not exist.

- [ ] **Step 4: Implement repository**

Create `src/server/game-repository.ts`:

```ts
import { mkdir, readFile, readdir, writeFile } from "node:fs/promises";
import { join } from "node:path";
import type { DraftEvent } from "@/core/drafts";
import type { GameEvent } from "@/core/events";
import type { Game } from "@/core/game";
import type { GameId } from "@/core/types";

export type GameRecord = {
  readonly game: Game;
  readonly events: readonly GameEvent[];
  readonly draft: DraftEvent | null;
};

export type GameRepository = {
  readonly get: (gameId: GameId) => Promise<GameRecord | null>;
  readonly list: () => Promise<readonly GameRecord[]>;
  readonly save: (record: GameRecord) => Promise<void>;
};

export function createGameRepository(rootDir = ".kiva-data"): GameRepository {
  const gamesDir = join(rootDir, "games");

  return {
    async get(gameId) {
      try {
        const raw = await readFile(gamePath(gamesDir, gameId), "utf8");
        return JSON.parse(raw) as GameRecord;
      } catch (error) {
        if (isNotFound(error)) return null;
        throw error;
      }
    },

    async list() {
      await mkdir(gamesDir, { recursive: true });
      const files = await readdir(gamesDir);
      const records = await Promise.all(
        files
          .filter((file) => file.endsWith(".json"))
          .map(async (file) => JSON.parse(await readFile(join(gamesDir, file), "utf8")) as GameRecord),
      );
      return records.sort((a, b) => b.game.updatedAt.localeCompare(a.game.updatedAt));
    },

    async save(record) {
      await mkdir(gamesDir, { recursive: true });
      await writeFile(
        gamePath(gamesDir, record.game.id),
        `${JSON.stringify(record, null, 2)}\n`,
        "utf8",
      );
    },
  };
}

function gamePath(gamesDir: string, gameId: GameId): string {
  return join(gamesDir, `${gameId}.json`);
}

function isNotFound(error: unknown): boolean {
  return (
    typeof error === "object" &&
    error !== null &&
    "code" in error &&
    error.code === "ENOENT"
  );
}
```

- [ ] **Step 5: Run repository tests**

Run:

```bash
pnpm test src/server/__tests__/game-repository.test.ts
```

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add .gitignore src/server/game-repository.ts src/server/__tests__/game-repository.test.ts
git commit -m "feat: add local game repository"
```

## Task 4: Add Server Actions for Editor Operations

**Files:**
- Create: `src/core/id.ts`
- Create: `src/server/game-actions.ts`
- Create: `src/server/__tests__/game-actions.test.ts`
- Create: `src/app/actions.ts`

- [ ] **Step 1: Write server action tests**

Create `src/server/__tests__/game-actions.test.ts`:

```ts
import { mkdtemp, rm } from "node:fs/promises";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { afterEach, describe, expect, it } from "vitest";
import { createGameRepository } from "../game-repository";
import { createGameActions } from "../game-actions";

let tempDir: string | null = null;

afterEach(async () => {
  if (tempDir) {
    await rm(tempDir, { recursive: true, force: true });
    tempDir = null;
  }
});

describe("game actions", () => {
  it("creates a game record", async () => {
    tempDir = await mkdtemp(join(tmpdir(), "kiva-actions-"));
    const actions = createGameActions(createGameRepository(tempDir));

    const record = await actions.createGame();

    expect(record.game.players).toHaveLength(6);
    await expect(actions.getGame(record.game.id)).resolves.toMatchObject({
      game: { id: record.game.id },
    });
  });

  it("continues into a draft and confirms it", async () => {
    tempDir = await mkdtemp(join(tmpdir(), "kiva-actions-"));
    const actions = createGameActions(createGameRepository(tempDir));
    const record = await actions.createGame();

    const withDraft = await actions.continueGame(record.game.id);
    expect(withDraft.draft?.type).toBe("role_assigned");

    const confirmed = await actions.confirmDraft(record.game.id);
    expect(confirmed.events).toHaveLength(1);
    expect(confirmed.draft).toBeNull();
  });

  it("rolls back active events after a selected index", async () => {
    tempDir = await mkdtemp(join(tmpdir(), "kiva-actions-"));
    const actions = createGameActions(createGameRepository(tempDir));
    let record = await actions.createGame();
    record = await actions.continueGame(record.game.id);
    record = await actions.confirmDraft(record.game.id);
    record = await actions.continueGame(record.game.id);
    record = await actions.confirmDraft(record.game.id);

    const rolledBack = await actions.rollbackAfter(record.game.id, 1);

    expect(rolledBack.events.filter((event) => event.status === "active")).toHaveLength(1);
    expect(rolledBack.events.filter((event) => event.status === "superseded")).toHaveLength(1);
  });

  it("edits the current draft display text before confirmation", async () => {
    tempDir = await mkdtemp(join(tmpdir(), "kiva-actions-"));
    const actions = createGameActions(createGameRepository(tempDir));
    const record = await actions.createGame();
    await actions.continueGame(record.game.id);

    const edited = await actions.editDraftDisplay(record.game.id, {
      title: "自定义标题",
      text: "自定义展示文案",
    });

    expect(edited.draft?.display).toEqual({
      title: "自定义标题",
      text: "自定义展示文案",
    });
  });
});
```

- [ ] **Step 2: Run action tests to verify they fail**

Run:

```bash
pnpm test src/server/__tests__/game-actions.test.ts
```

Expected: FAIL because `game-actions.ts` does not exist.

- [ ] **Step 3: Implement ID helpers**

Create `src/core/id.ts`:

```ts
import type { DraftId, EventId, GameId } from "./types";

export function createGameId(now = new Date()): GameId {
  return `game_${now.getTime().toString(36)}` as GameId;
}

export function createDraftId(now = new Date()): DraftId {
  return `draft_${now.getTime().toString(36)}` as DraftId;
}

export function createEventId(index: number, now = new Date()): EventId {
  return `event_${index}_${now.getTime().toString(36)}` as EventId;
}
```

- [ ] **Step 4: Implement server actions service**

Create `src/server/game-actions.ts`:

```ts
import { planNextDraft } from "@/core/advance-planner";
import { confirmDraftEvent } from "@/core/drafts";
import { appendEvent, getActiveEvents, rollbackAfterIndex } from "@/core/event-log";
import { createSeedGame } from "@/core/game";
import { createDraftId, createEventId, createGameId } from "@/core/id";
import { createDefaultRuleset, type GameId } from "@/core/types";
import type { GameRecord, GameRepository } from "./game-repository";

export type GameActions = ReturnType<typeof createGameActions>;

export function createGameActions(repository: GameRepository) {
  return {
    async createGame(): Promise<GameRecord> {
      const now = new Date().toISOString();
      const game = createSeedGame({
        gameId: createGameId(),
        createdAt: now,
        ruleset: createDefaultRuleset(),
      });
      const record: GameRecord = { game, events: [], draft: null };
      await repository.save(record);
      return record;
    },

    async getGame(gameId: GameId): Promise<GameRecord | null> {
      return repository.get(gameId);
    },

    async listGames(): Promise<readonly GameRecord[]> {
      return repository.list();
    },

    async continueGame(gameId: GameId): Promise<GameRecord> {
      const record = await mustGet(repository, gameId);
      if (record.draft) return record;
      const now = new Date().toISOString();
      const draft = planNextDraft({
        game: record.game,
        events: record.events,
        draftId: createDraftId(),
        createdAt: now,
      });
      const updated: GameRecord = {
        ...record,
        game: { ...record.game, updatedAt: now },
        draft,
      };
      await repository.save(updated);
      return updated;
    },

    async confirmDraft(gameId: GameId): Promise<GameRecord> {
      const record = await mustGet(repository, gameId);
      if (!record.draft) return record;
      const now = new Date().toISOString();
      const nextIndex = getActiveEvents(record.events).length + 1;
      const event = confirmDraftEvent({
        draft: record.draft,
        eventId: createEventId(nextIndex),
        index: nextIndex,
        createdAt: now,
      });
      const updated: GameRecord = {
        ...record,
        game: { ...record.game, updatedAt: now },
        events: appendEvent(record.events, event),
        draft: null,
      };
      await repository.save(updated);
      return updated;
    },

    async editDraftDisplay(
      gameId: GameId,
      display: { readonly title: string; readonly text: string },
    ): Promise<GameRecord> {
      const record = await mustGet(repository, gameId);
      if (!record.draft) return record;
      const now = new Date().toISOString();
      const updated: GameRecord = {
        ...record,
        game: { ...record.game, updatedAt: now },
        draft: {
          ...record.draft,
          display: {
            title: display.title.trim(),
            text: display.text.trim(),
          },
        },
      };
      await repository.save(updated);
      return updated;
    },

    async deleteDraft(gameId: GameId): Promise<GameRecord> {
      const record = await mustGet(repository, gameId);
      const updated = {
        ...record,
        game: { ...record.game, updatedAt: new Date().toISOString() },
        draft: null,
      };
      await repository.save(updated);
      return updated;
    },

    async rollbackAfter(gameId: GameId, index: number): Promise<GameRecord> {
      const record = await mustGet(repository, gameId);
      const now = new Date().toISOString();
      const updated: GameRecord = {
        ...record,
        game: { ...record.game, updatedAt: now },
        events: rollbackAfterIndex(record.events, index),
        draft: null,
      };
      await repository.save(updated);
      return updated;
    },
  };
}

async function mustGet(repository: GameRepository, gameId: GameId): Promise<GameRecord> {
  const record = await repository.get(gameId);
  if (!record) throw new Error(`Game not found: ${gameId}`);
  return record;
}
```

- [ ] **Step 5: Create Next server action exports**

Create `src/app/actions.ts`:

```ts
"use server";

import { redirect } from "next/navigation";
import { revalidatePath } from "next/cache";
import { createGameActions } from "@/server/game-actions";
import { createGameRepository } from "@/server/game-repository";
import type { GameId } from "@/core/types";

const actions = createGameActions(createGameRepository());

export async function createGameAction() {
  const record = await actions.createGame();
  redirect(`/games/${record.game.id}/editor`);
}

export async function continueGameAction(gameId: GameId) {
  await actions.continueGame(gameId);
  revalidatePath(`/games/${gameId}/editor`);
}

export async function confirmDraftAction(gameId: GameId) {
  await actions.confirmDraft(gameId);
  revalidatePath(`/games/${gameId}/editor`);
  revalidatePath(`/games/${gameId}/preview`);
}

export async function editDraftDisplayAction(gameId: GameId, formData: FormData) {
  await actions.editDraftDisplay(gameId, {
    title: String(formData.get("title") ?? ""),
    text: String(formData.get("text") ?? ""),
  });
  revalidatePath(`/games/${gameId}/editor`);
}

export async function deleteDraftAction(gameId: GameId) {
  await actions.deleteDraft(gameId);
  revalidatePath(`/games/${gameId}/editor`);
}

export async function rollbackAfterAction(gameId: GameId, index: number) {
  await actions.rollbackAfter(gameId, index);
  revalidatePath(`/games/${gameId}/editor`);
  revalidatePath(`/games/${gameId}/preview`);
}
```

- [ ] **Step 6: Run action tests**

Run:

```bash
pnpm test src/server/__tests__/game-actions.test.ts
```

Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add src/core/id.ts src/server/game-actions.ts src/server/__tests__/game-actions.test.ts src/app/actions.ts
git commit -m "feat: add editor server actions"
```

## Task 5: Add Playback Compiler

**Files:**
- Create: `src/core/playback.ts`
- Create: `src/core/__tests__/playback.test.ts`

- [ ] **Step 1: Write playback tests**

Create `src/core/__tests__/playback.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import { compilePublicPlayback } from "../playback";

describe("playback compiler", () => {
  it("includes only public active events", () => {
    const items = compilePublicPlayback([
      {
        id: "event_1",
        gameId: "game_1",
        index: 1,
        status: "active",
        type: "phase_started",
        phase: "night",
        visibility: { kind: "public" },
        payload: { phase: "night", dayNumber: 1 },
        display: { title: "夜晚开始", text: "天黑请闭眼。" },
        createdAt: "2026-06-26T00:00:00.000Z",
      },
      {
        id: "event_2",
        gameId: "game_1",
        index: 2,
        status: "active",
        type: "wolf_kill_selected",
        phase: "night",
        visibility: { kind: "faction_private", faction: "wolves" },
        payload: { targetPlayerId: "p1" },
        display: { title: "狼人刀人", text: "狼人选择袭击 1号。" },
        createdAt: "2026-06-26T00:00:01.000Z",
      },
    ] as never);

    expect(items).toEqual([
      {
        index: 1,
        phase: "night",
        title: "夜晚开始",
        text: "天黑请闭眼。",
      },
    ]);
  });
});
```

- [ ] **Step 2: Run playback tests to verify they fail**

Run:

```bash
pnpm test src/core/__tests__/playback.test.ts
```

Expected: FAIL because `playback.ts` does not exist.

- [ ] **Step 3: Implement playback compiler**

Create `src/core/playback.ts`:

```ts
import { getActiveEvents } from "./event-log";
import type { GameEvent } from "./events";
import type { Phase } from "./types";

export type PlaybackItem = {
  readonly index: number;
  readonly phase: Phase;
  readonly title: string;
  readonly text: string;
};

export function compilePublicPlayback(events: readonly GameEvent[]): readonly PlaybackItem[] {
  return getActiveEvents(events)
    .filter((event) => event.visibility.kind === "public")
    .map((event) => ({
      index: event.index,
      phase: event.phase,
      title: event.display?.title ?? event.type,
      text: event.display?.text ?? "",
    }));
}
```

- [ ] **Step 4: Run playback tests**

Run:

```bash
pnpm test src/core/__tests__/playback.test.ts
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/core/playback.ts src/core/__tests__/playback.test.ts
git commit -m "feat: add public playback compiler"
```

## Task 6: Build Editor and Preview UI

**Files:**
- Modify: `src/app/page.tsx`
- Create: `src/app/games/[gameId]/editor/page.tsx`
- Create: `src/app/games/[gameId]/preview/page.tsx`
- Create: `src/components/editor/game-board.tsx`
- Create: `src/components/editor/event-timeline.tsx`
- Create: `src/components/editor/draft-panel.tsx`
- Create: `src/components/preview/playback-stage.tsx`
- Modify: `src/app/globals.css`

- [ ] **Step 1: Implement editor components**

Create `src/components/editor/game-board.tsx`:

```tsx
import { deriveGameState } from "@/core/state";
import type { GameEvent } from "@/core/events";
import type { Game } from "@/core/game";

export function GameBoard({
  game,
  events,
}: {
  readonly game: Game;
  readonly events: readonly GameEvent[];
}) {
  const state = deriveGameState(game.players, events);
  const alive = new Set(state.alivePlayerIds);

  return (
    <section className="grid grid-cols-3 gap-3">
      {game.players.map((player) => (
        <div key={player.playerId} className="rounded border border-neutral-800 bg-neutral-950 p-3">
          <div className="text-xs text-neutral-500">{player.seatNo}号</div>
          <div className="mt-1 font-medium">{player.name}</div>
          <div className="mt-2 text-xs text-neutral-400">{player.gameRole}</div>
          <div className={alive.has(player.playerId) ? "mt-2 text-xs text-emerald-400" : "mt-2 text-xs text-red-400"}>
            {alive.has(player.playerId) ? "存活" : "死亡"}
          </div>
        </div>
      ))}
    </section>
  );
}
```

Create `src/components/editor/event-timeline.tsx`:

```tsx
import type { GameEvent } from "@/core/events";
import { rollbackAfterAction } from "@/app/actions";
import type { GameId } from "@/core/types";

export function EventTimeline({
  gameId,
  events,
}: {
  readonly gameId: GameId;
  readonly events: readonly GameEvent[];
}) {
  return (
    <section className="space-y-2">
      {events.map((event) => (
        <div key={event.id} className={event.status === "active" ? "rounded border border-neutral-800 p-3" : "rounded border border-neutral-900 p-3 opacity-40"}>
          <div className="flex items-center justify-between gap-3">
            <div>
              <div className="text-xs text-neutral-500">#{event.index} · {event.phase} · {event.visibility.kind}</div>
              <div className="mt-1 text-sm font-medium">{event.display?.title ?? event.type}</div>
              <div className="mt-1 text-sm text-neutral-400">{event.display?.text}</div>
            </div>
            {event.status === "active" ? (
              <form action={rollbackAfterAction.bind(null, gameId, event.index)}>
                <button className="rounded bg-neutral-800 px-3 py-2 text-xs hover:bg-neutral-700">回滚到此处</button>
              </form>
            ) : null}
          </div>
        </div>
      ))}
    </section>
  );
}
```

Create `src/components/editor/draft-panel.tsx`:

```tsx
import { confirmDraftAction, deleteDraftAction, editDraftDisplayAction } from "@/app/actions";
import type { DraftEvent } from "@/core/drafts";
import type { GameId } from "@/core/types";

export function DraftPanel({
  gameId,
  draft,
}: {
  readonly gameId: GameId;
  readonly draft: DraftEvent | null;
}) {
  if (!draft) {
    return (
      <section className="rounded border border-dashed border-neutral-800 p-4 text-sm text-neutral-400">
        当前没有待确认草稿。
      </section>
    );
  }

  return (
    <section className="rounded border border-amber-500/50 bg-amber-500/10 p-4">
      <div className="text-xs text-amber-300">待确认 · {draft.visibility.kind}</div>
      <h2 className="mt-2 text-lg font-semibold">{draft.display?.title ?? draft.type}</h2>
      <p className="mt-2 text-sm text-neutral-200">{draft.display?.text}</p>
      <p className="mt-3 text-xs text-neutral-400">{draft.reason}</p>
      <form action={editDraftDisplayAction.bind(null, gameId)} className="mt-4 space-y-2">
        <input name="title" defaultValue={draft.display?.title ?? draft.type} className="w-full rounded border border-neutral-700 bg-neutral-950 px-3 py-2 text-sm" />
        <textarea name="text" defaultValue={draft.display?.text ?? ""} rows={4} className="w-full resize-none rounded border border-neutral-700 bg-neutral-950 px-3 py-2 text-sm" />
        <button className="rounded bg-neutral-800 px-4 py-2 text-sm hover:bg-neutral-700">保存文案</button>
      </form>
      <div className="mt-4 flex gap-2">
        <form action={confirmDraftAction.bind(null, gameId)}>
          <button className="rounded bg-amber-300 px-4 py-2 text-sm font-medium text-neutral-950 hover:bg-amber-200">确认写入</button>
        </form>
        <form action={deleteDraftAction.bind(null, gameId)}>
          <button className="rounded bg-neutral-800 px-4 py-2 text-sm hover:bg-neutral-700">删除草稿</button>
        </form>
      </div>
    </section>
  );
}
```

- [ ] **Step 2: Implement preview component**

Create `src/components/preview/playback-stage.tsx`:

```tsx
import type { PlaybackItem } from "@/core/playback";

export function PlaybackStage({
  items,
}: {
  readonly items: readonly PlaybackItem[];
}) {
  const current = items.at(-1);

  return (
    <main className="min-h-screen bg-black p-6 text-white">
      <section className="mx-auto aspect-video max-h-[calc(100vh-48px)] max-w-[calc((100vh-48px)*16/9)] overflow-hidden rounded bg-neutral-950 shadow-2xl">
        <div className="flex h-full flex-col justify-between p-12">
          <div className="text-2xl font-semibold tracking-normal">Kiva Werewolf</div>
          <div>
            <div className="text-sm uppercase text-neutral-500">Public Replay</div>
            <h1 className="mt-4 text-5xl font-bold tracking-normal">{current?.title ?? "等待事件"}</h1>
            <p className="mt-6 max-w-3xl text-2xl leading-relaxed text-neutral-200">{current?.text ?? "确认事件后，这里会显示公开回放内容。"}</p>
          </div>
          <div className="text-sm text-neutral-500">{items.length} public events</div>
        </div>
      </section>
    </main>
  );
}
```

- [ ] **Step 3: Implement pages**

Modify `src/app/page.tsx`:

```tsx
import { createGameAction } from "./actions";
import { createGameActions } from "@/server/game-actions";
import { createGameRepository } from "@/server/game-repository";

export default async function HomePage() {
  const records = await createGameActions(createGameRepository()).listGames();

  return (
    <main className="min-h-screen bg-neutral-950 px-8 py-10 text-neutral-100">
      <div className="mx-auto max-w-4xl">
        <div className="flex items-center justify-between gap-4">
          <div>
            <h1 className="text-2xl font-semibold">Kiva Werewolf Director</h1>
            <p className="mt-2 text-sm text-neutral-400">创建、推进、预览一局狼人杀内容。</p>
          </div>
          <form action={createGameAction}>
            <button className="rounded bg-white px-4 py-2 text-sm font-medium text-neutral-950 hover:bg-neutral-200">新建对局</button>
          </form>
        </div>
        <section className="mt-8 space-y-3">
          {records.map((record) => (
            <a key={record.game.id} href={`/games/${record.game.id}/editor`} className="block rounded border border-neutral-800 p-4 hover:bg-neutral-900">
              <div className="font-medium">{record.game.title}</div>
              <div className="mt-1 text-xs text-neutral-500">{record.game.id}</div>
            </a>
          ))}
        </section>
      </div>
    </main>
  );
}
```

Create `src/app/games/[gameId]/editor/page.tsx`:

```tsx
import { notFound } from "next/navigation";
import { continueGameAction } from "@/app/actions";
import { DraftPanel } from "@/components/editor/draft-panel";
import { EventTimeline } from "@/components/editor/event-timeline";
import { GameBoard } from "@/components/editor/game-board";
import { createGameActions } from "@/server/game-actions";
import { createGameRepository } from "@/server/game-repository";
import type { GameId } from "@/core/types";

export default async function EditorPage({
  params,
}: {
  readonly params: Promise<{ readonly gameId: string }>;
}) {
  const { gameId } = await params;
  const record = await createGameActions(createGameRepository()).getGame(gameId as GameId);
  if (!record) notFound();

  return (
    <main className="min-h-screen bg-neutral-950 p-6 text-neutral-100">
      <div className="mx-auto grid max-w-7xl grid-cols-[1fr_420px] gap-6">
        <section>
          <div className="flex items-center justify-between gap-3">
            <div>
              <h1 className="text-2xl font-semibold">{record.game.title}</h1>
              <p className="mt-1 text-sm text-neutral-500">{record.game.id}</p>
            </div>
            <div className="flex gap-2">
              <a className="rounded bg-neutral-800 px-4 py-2 text-sm hover:bg-neutral-700" href={`/games/${record.game.id}/preview`} target="_blank">打开预览</a>
              <form action={continueGameAction.bind(null, record.game.id)}>
                <button className="rounded bg-white px-4 py-2 text-sm font-medium text-neutral-950 hover:bg-neutral-200">继续</button>
              </form>
            </div>
          </div>
          <div className="mt-6">
            <GameBoard game={record.game} events={record.events} />
          </div>
          <div className="mt-6">
            <EventTimeline gameId={record.game.id} events={record.events} />
          </div>
        </section>
        <aside className="space-y-4">
          <DraftPanel gameId={record.game.id} draft={record.draft} />
          <iframe className="aspect-video w-full rounded border border-neutral-800" src={`/games/${record.game.id}/preview`} />
        </aside>
      </div>
    </main>
  );
}
```

Create `src/app/games/[gameId]/preview/page.tsx`:

```tsx
import { notFound } from "next/navigation";
import { PlaybackStage } from "@/components/preview/playback-stage";
import { compilePublicPlayback } from "@/core/playback";
import type { GameId } from "@/core/types";
import { createGameActions } from "@/server/game-actions";
import { createGameRepository } from "@/server/game-repository";

export default async function PreviewPage({
  params,
}: {
  readonly params: Promise<{ readonly gameId: string }>;
}) {
  const { gameId } = await params;
  const record = await createGameActions(createGameRepository()).getGame(gameId as GameId);
  if (!record) notFound();

  return <PlaybackStage items={compilePublicPlayback(record.events)} />;
}
```

- [ ] **Step 4: Run typecheck**

Run:

```bash
pnpm typecheck
```

Expected: PASS.

- [ ] **Step 5: Run build**

Run:

```bash
pnpm build
```

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add src/app/page.tsx src/app/games src/components src/app/globals.css
git commit -m "feat: add editor and preview pages"
```

## Task 7: Full Verification and Local Run

**Files:**
- No code files unless fixing issues found by verification.

- [ ] **Step 1: Run all tests**

Run:

```bash
pnpm test
```

Expected: all tests PASS.

- [ ] **Step 2: Run typecheck**

Run:

```bash
pnpm typecheck
```

Expected: PASS.

- [ ] **Step 3: Run production build**

Run:

```bash
pnpm build
```

Expected: PASS.

- [ ] **Step 4: Start local dev server**

Run:

```bash
pnpm dev
```

Expected: Next.js starts and prints a local URL, normally `http://localhost:3000`.

- [ ] **Step 5: Manual smoke test**

In the browser:

1. Open `/`.
2. Click `新建对局`.
3. Click `继续`.
4. Confirm the draft.
5. Repeat until public preview shows `夜晚开始` and later `昨夜死讯`.
6. Open `/games/<gameId>/preview` in a separate tab.
7. Confirm the preview page is 16:9, has no editor controls, and only shows public events.
8. Use `回滚到此处` on an event and confirm later active events become superseded.

- [ ] **Step 6: Commit verification fixes if needed**

If any verification fix was required:

```bash
git add <changed-files>
git commit -m "fix: stabilize editor mvp verification"
```

If no fixes were required, do not create an empty commit.

## Self-Review

- Spec coverage: this plan implements the local creation loop, official event log, draft display editing, draft confirmation, rollback, editor/preview separation, public-only preview, and local persistence. It intentionally excludes LLM and recording because those depend on a working content pipeline.
- Placeholder scan: no `TBD`, `TODO`, or unresolved implementation instruction remains.
- Type consistency: `GameRecord`, `DraftEvent`, `Game`, `GameEvent`, and `GameId` are introduced before use. Server actions use the repository abstraction, and UI pages read through the same actions/repository boundary.
- Risk: the deterministic planner only covers the initial content chain through first public death announcement. That is intentional for this MVP; after this works, the next plan should extend planner states for last words, speeches, voting, PK, exile, and win checks.
