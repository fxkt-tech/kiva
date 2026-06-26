# Werewolf Director Foundation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the first testable foundation for the Werewolf director: Next.js project scaffold, domain types, event log, visibility projection, and fixed 6-player rule engine.

**Architecture:** Implement the game core as pure TypeScript modules under `src/core` so it can be tested without UI, database, or LLM calls. The official event log is the only source of truth; derived state, player viewpoints, and legal actions are computed from active events.

**Tech Stack:** Next.js 16, TypeScript, Vitest, Tailwind CSS 4, shadcn/ui-ready structure, pure TypeScript domain modules.

---

## Scope

This plan implements only the foundation slice from the approved spec:

- Project scaffold
- Domain identifiers and enums
- Player snapshots and rule set types
- Official event model
- Event log helpers
- Visibility projector
- Fixed 6-player board validation
- State derivation from active events
- Legal actions for the first night
- Night settlement for wolf kill, seer check, and witch decisions

Out of scope for this first plan:

- Database persistence
- Character library UI
- Editor UI
- Playback UI
- Recording
- LLM provider integration
- Draft confirmation workflow

Those depend on the correctness of this foundation.

## File Structure

- `package.json`: project scripts and dependencies.
- `tsconfig.json`: TypeScript configuration and `@/*` alias.
- `next.config.ts`: Next.js configuration.
- `vitest.config.ts`: unit test configuration.
- `src/app/layout.tsx`: minimal app shell.
- `postcss.config.mjs`: Tailwind CSS PostCSS plugin configuration.
- `src/app/page.tsx`: minimal landing page.
- `src/core/types.ts`: shared domain IDs, roles, factions, phases, rule settings.
- `src/core/events.ts`: official event union and event visibility types.
- `src/core/event-log.ts`: active event filtering, append validation, rollback helper.
- `src/core/player.ts`: player snapshot factory helpers and board validation.
- `src/core/visibility.ts`: hard visibility projector.
- `src/core/state.ts`: derived game state from active events.
- `src/core/rules.ts`: legal actions, night settlement, win checks.
- `src/core/__tests__/event-log.test.ts`: event log tests.
- `src/core/__tests__/visibility.test.ts`: viewpoint isolation tests.
- `src/core/__tests__/state.test.ts`: derived state tests.
- `src/core/__tests__/rules.test.ts`: board and night rule tests.

## Task 1: Scaffold Next.js and Test Harness

**Files:**
- Create: `package.json`
- Create: `tsconfig.json`
- Create: `next.config.ts`
- Create: `postcss.config.mjs`
- Create: `vitest.config.ts`
- Create: `src/app/layout.tsx`
- Create: `src/app/page.tsx`
- Create: `src/app/globals.css`

- [ ] **Step 1: Create project metadata and scripts**

Create `package.json`:

```json
{
  "name": "kiva",
  "version": "0.1.0",
  "private": true,
  "scripts": {
    "dev": "next dev",
    "build": "next build",
    "start": "next start",
    "test": "vitest run",
    "test:watch": "vitest",
    "typecheck": "tsc --noEmit"
  },
  "dependencies": {
    "@radix-ui/react-slot": "^1.3.0",
    "class-variance-authority": "^0.7.1",
    "clsx": "^2.1.1",
    "lucide-react": "^1.21.0",
    "next": "^16.2.9",
    "react": "^19.2.7",
    "react-dom": "^19.2.7",
    "tailwind-merge": "^3.6.0",
    "tailwindcss": "^4.3.1"
  },
  "devDependencies": {
    "@tailwindcss/postcss": "^4.3.1",
    "@types/node": "^26.0.1",
    "@types/react": "^19.2.17",
    "@types/react-dom": "^19.2.3",
    "postcss": "^8.5.15",
    "typescript": "^6.0.3",
    "vitest": "^4.1.9"
  }
}
```

- [ ] **Step 2: Create TypeScript config**

Create `tsconfig.json`:

```json
{
  "compilerOptions": {
    "target": "ES2022",
    "lib": ["dom", "dom.iterable", "es2022"],
    "allowJs": false,
    "skipLibCheck": true,
    "strict": true,
    "noEmit": true,
    "esModuleInterop": true,
    "module": "esnext",
    "moduleResolution": "bundler",
    "resolveJsonModule": true,
    "isolatedModules": true,
    "jsx": "preserve",
    "incremental": true,
    "baseUrl": ".",
    "paths": {
      "@/*": ["./src/*"]
    },
    "plugins": [
      {
        "name": "next"
      }
    ]
  },
  "include": ["next-env.d.ts", "**/*.ts", "**/*.tsx", ".next/types/**/*.ts"],
  "exclude": ["node_modules"]
}
```

- [ ] **Step 3: Create Next config**

Create `next.config.ts`:

```ts
import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  reactStrictMode: true,
};

export default nextConfig;
```

- [ ] **Step 4: Create PostCSS config**

Create `postcss.config.mjs`:

```js
const config = {
  plugins: {
    "@tailwindcss/postcss": {},
  },
};

export default config;
```

- [ ] **Step 5: Create Vitest config**

Create `vitest.config.ts`:

```ts
import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    environment: "node",
    include: ["src/**/*.test.ts"],
  },
  resolve: {
    alias: {
      "@": new URL("./src", import.meta.url).pathname,
    },
  },
});
```

- [ ] **Step 6: Create minimal app shell**

Create `src/app/layout.tsx`:

```tsx
import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "Kiva Werewolf Director",
  description: "Werewolf match director and replay recorder",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="zh-CN">
      <body>{children}</body>
    </html>
  );
}
```

Create `src/app/page.tsx`:

```tsx
export default function HomePage() {
  return (
    <main className="min-h-screen bg-neutral-950 px-8 py-10 text-neutral-100">
      <h1 className="text-2xl font-semibold">Kiva Werewolf Director</h1>
      <p className="mt-3 max-w-2xl text-sm text-neutral-300">
        Foundation build: event log, visibility projection, and rules engine.
      </p>
    </main>
  );
}
```

Create `src/app/globals.css`:

```css
@import "tailwindcss";

:root {
  color-scheme: dark;
}

body {
  margin: 0;
}
```

- [ ] **Step 7: Install dependencies**

Run:

```bash
npm install
```

Expected: `package-lock.json` is created and install exits successfully.

- [ ] **Step 8: Verify scaffold**

Run:

```bash
npm run typecheck
npm test
```

Expected:

- Typecheck passes.
- Vitest exits with no test files found or no tests yet, depending Vitest version.

- [ ] **Step 9: Commit scaffold**

```bash
git add package.json package-lock.json tsconfig.json next.config.ts postcss.config.mjs vitest.config.ts src/app
git commit -m "chore: scaffold next app and test harness"
```

## Task 2: Define Core Domain Types

**Files:**
- Create: `src/core/types.ts`
- Test: `src/core/__tests__/types.test.ts`

- [ ] **Step 1: Write failing type/runtime tests**

Create `src/core/__tests__/types.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import {
  createDefaultRuleset,
  GAME_ROLES,
  isWerewolfRole,
  type GameRole,
} from "../types";

describe("core types", () => {
  it("defines the fixed first-version roles", () => {
    expect(GAME_ROLES).toEqual(["werewolf", "seer", "witch", "villager"]);
  });

  it("detects werewolf role", () => {
    expect(isWerewolfRole("werewolf")).toBe(true);
    expect(isWerewolfRole("seer")).toBe(false);
  });

  it("creates default ruleset from approved spec", () => {
    expect(createDefaultRuleset()).toMatchObject({
      winCondition: "slaughter_side",
      witchFirstNightSelfSave: true,
      witchAllowSameNightAntidoteAndPoison: false,
      voteReveal: "after_all_votes",
      deadRoleReveal: "endgame",
      pkVoters: "non_pk_only",
    });
  });

  it("keeps GameRole as a narrow union", () => {
    const role: GameRole = "witch";
    expect(role).toBe("witch");
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

Run:

```bash
npm test -- src/core/__tests__/types.test.ts
```

Expected: FAIL because `src/core/types.ts` does not exist.

- [ ] **Step 3: Implement domain types**

Create `src/core/types.ts`:

```ts
export type Brand<T, Name extends string> = T & { readonly __brand: Name };

export type GameId = Brand<string, "GameId">;
export type PlayerId = Brand<string, "PlayerId">;
export type EventId = Brand<string, "EventId">;
export type DraftId = Brand<string, "DraftId">;

export const GAME_ROLES = ["werewolf", "seer", "witch", "villager"] as const;
export type GameRole = (typeof GAME_ROLES)[number];

export type Faction = "wolves" | "good";
export type Phase =
  | "setup"
  | "night"
  | "day"
  | "last_words"
  | "speech"
  | "vote"
  | "pk"
  | "ended";

export type WinCondition = "slaughter_side" | "slaughter_all";
export type VoteReveal = "after_all_votes" | "immediate";
export type DeadRoleReveal = "endgame" | "on_death";
export type PkVoters = "non_pk_only" | "all_living_non_self";

export type Ruleset = {
  readonly playerCount: 6;
  readonly roleCounts: Readonly<Record<GameRole, number>>;
  readonly winCondition: WinCondition;
  readonly witchFirstNightSelfSave: boolean;
  readonly witchAllowSameNightAntidoteAndPoison: boolean;
  readonly voteReveal: VoteReveal;
  readonly deadRoleReveal: DeadRoleReveal;
  readonly pkVoters: PkVoters;
  readonly allowAbstainVote: boolean;
};

export function isWerewolfRole(role: GameRole): role is "werewolf" {
  return role === "werewolf";
}

export function factionForRole(role: GameRole): Faction {
  return role === "werewolf" ? "wolves" : "good";
}

export function createDefaultRuleset(): Ruleset {
  return {
    playerCount: 6,
    roleCounts: {
      werewolf: 2,
      seer: 1,
      witch: 1,
      villager: 2,
    },
    winCondition: "slaughter_side",
    witchFirstNightSelfSave: true,
    witchAllowSameNightAntidoteAndPoison: false,
    voteReveal: "after_all_votes",
    deadRoleReveal: "endgame",
    pkVoters: "non_pk_only",
    allowAbstainVote: true,
  };
}
```

- [ ] **Step 4: Run test to verify it passes**

Run:

```bash
npm test -- src/core/__tests__/types.test.ts
```

Expected: PASS.

- [ ] **Step 5: Commit domain types**

```bash
git add src/core/types.ts src/core/__tests__/types.test.ts
git commit -m "feat: define werewolf core domain types"
```

## Task 3: Define Players and Board Validation

**Files:**
- Create: `src/core/player.ts`
- Test: `src/core/__tests__/player.test.ts`

- [ ] **Step 1: Write failing board validation tests**

Create `src/core/__tests__/player.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import { createDefaultRuleset, type PlayerId } from "../types";
import {
  createPlayerSnapshot,
  validateSixPlayerBoard,
  type PlayerSnapshot,
} from "../player";

function player(id: string, seatNo: number, role: PlayerSnapshot["gameRole"]) {
  return createPlayerSnapshot({
    playerId: id as PlayerId,
    seatNo,
    name: `P${seatNo}`,
    gameRole: role,
  });
}

describe("player snapshots", () => {
  it("assigns faction from role", () => {
    expect(player("p1", 1, "werewolf").faction).toBe("wolves");
    expect(player("p2", 2, "seer").faction).toBe("good");
  });

  it("accepts the approved fixed six player board", () => {
    const players = [
      player("p1", 1, "werewolf"),
      player("p2", 2, "werewolf"),
      player("p3", 3, "seer"),
      player("p4", 4, "witch"),
      player("p5", 5, "villager"),
      player("p6", 6, "villager"),
    ];

    expect(validateSixPlayerBoard(players, createDefaultRuleset())).toEqual({
      ok: true,
    });
  });

  it("rejects duplicate seats", () => {
    const players = [
      player("p1", 1, "werewolf"),
      player("p2", 1, "werewolf"),
      player("p3", 3, "seer"),
      player("p4", 4, "witch"),
      player("p5", 5, "villager"),
      player("p6", 6, "villager"),
    ];

    expect(validateSixPlayerBoard(players, createDefaultRuleset())).toEqual({
      ok: false,
      reason: "duplicate_seat",
    });
  });

  it("rejects wrong role counts", () => {
    const players = [
      player("p1", 1, "werewolf"),
      player("p2", 2, "villager"),
      player("p3", 3, "seer"),
      player("p4", 4, "witch"),
      player("p5", 5, "villager"),
      player("p6", 6, "villager"),
    ];

    expect(validateSixPlayerBoard(players, createDefaultRuleset())).toEqual({
      ok: false,
      reason: "invalid_role_count",
      role: "werewolf",
      expected: 2,
      actual: 1,
    });
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

Run:

```bash
npm test -- src/core/__tests__/player.test.ts
```

Expected: FAIL because `src/core/player.ts` does not exist.

- [ ] **Step 3: Implement player snapshots and validation**

Create `src/core/player.ts`:

```ts
import {
  factionForRole,
  GAME_ROLES,
  type Faction,
  type GameRole,
  type PlayerId,
  type Ruleset,
} from "./types";

export type ModelBindingSnapshot = {
  readonly provider: string;
  readonly model: string;
  readonly temperature: number;
  readonly maxTokens: number;
  readonly responseFormat: "json";
  readonly fallbackModel?: string;
};

export type PlayerSnapshot = {
  readonly playerId: PlayerId;
  readonly seatNo: number;
  readonly profileSourceId?: string;
  readonly name: string;
  readonly avatar?: string;
  readonly persona: string;
  readonly speakingStyle: string;
  readonly reasoningStyle: string;
  readonly systemPrompt: string;
  readonly modelBindingSnapshot: ModelBindingSnapshot;
  readonly gameRole: GameRole;
  readonly faction: Faction;
  readonly initialPrivateKnowledge: readonly string[];
};

export type CreatePlayerSnapshotInput = {
  readonly playerId: PlayerId;
  readonly seatNo: number;
  readonly name: string;
  readonly gameRole: GameRole;
  readonly profileSourceId?: string;
  readonly avatar?: string;
  readonly persona?: string;
  readonly speakingStyle?: string;
  readonly reasoningStyle?: string;
  readonly systemPrompt?: string;
  readonly modelBindingSnapshot?: ModelBindingSnapshot;
};

export type BoardValidationResult =
  | { readonly ok: true }
  | { readonly ok: false; readonly reason: "invalid_player_count"; readonly expected: number; readonly actual: number }
  | { readonly ok: false; readonly reason: "duplicate_player_id" }
  | { readonly ok: false; readonly reason: "duplicate_seat" }
  | { readonly ok: false; readonly reason: "invalid_role_count"; readonly role: GameRole; readonly expected: number; readonly actual: number };

const defaultModelBinding: ModelBindingSnapshot = {
  provider: "mock",
  model: "mock-role-model",
  temperature: 0.7,
  maxTokens: 1200,
  responseFormat: "json",
};

export function createPlayerSnapshot(input: CreatePlayerSnapshotInput): PlayerSnapshot {
  const faction = factionForRole(input.gameRole);

  return {
    playerId: input.playerId,
    seatNo: input.seatNo,
    profileSourceId: input.profileSourceId,
    name: input.name,
    avatar: input.avatar,
    persona: input.persona ?? "",
    speakingStyle: input.speakingStyle ?? "",
    reasoningStyle: input.reasoningStyle ?? "",
    systemPrompt: input.systemPrompt ?? "",
    modelBindingSnapshot: input.modelBindingSnapshot ?? defaultModelBinding,
    gameRole: input.gameRole,
    faction,
    initialPrivateKnowledge: createInitialPrivateKnowledge(input.gameRole),
  };
}

function createInitialPrivateKnowledge(role: GameRole): readonly string[] {
  if (role === "werewolf") return ["own_role", "wolf_teammates"];
  if (role === "witch") return ["own_role", "witch_medicines"];
  return ["own_role"];
}

export function validateSixPlayerBoard(
  players: readonly PlayerSnapshot[],
  ruleset: Ruleset,
): BoardValidationResult {
  if (players.length !== ruleset.playerCount) {
    return {
      ok: false,
      reason: "invalid_player_count",
      expected: ruleset.playerCount,
      actual: players.length,
    };
  }

  if (new Set(players.map((p) => p.playerId)).size !== players.length) {
    return { ok: false, reason: "duplicate_player_id" };
  }

  if (new Set(players.map((p) => p.seatNo)).size !== players.length) {
    return { ok: false, reason: "duplicate_seat" };
  }

  for (const role of GAME_ROLES) {
    const actual = players.filter((p) => p.gameRole === role).length;
    const expected = ruleset.roleCounts[role];
    if (actual !== expected) {
      return { ok: false, reason: "invalid_role_count", role, expected, actual };
    }
  }

  return { ok: true };
}
```

- [ ] **Step 4: Run player tests**

Run:

```bash
npm test -- src/core/__tests__/player.test.ts
```

Expected: PASS.

- [ ] **Step 5: Commit player model**

```bash
git add src/core/player.ts src/core/__tests__/player.test.ts
git commit -m "feat: add player snapshots and board validation"
```

## Task 4: Define Official Events and Event Log Operations

**Files:**
- Create: `src/core/events.ts`
- Create: `src/core/event-log.ts`
- Test: `src/core/__tests__/event-log.test.ts`

- [ ] **Step 1: Write failing event log tests**

Create `src/core/__tests__/event-log.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import { getActiveEvents, appendEvent, rollbackAfterIndex } from "../event-log";
import type { GameEvent } from "../events";
import type { EventId, GameId } from "../types";

function event(index: number, status: GameEvent["status"] = "active"): GameEvent {
  return {
    id: `e${index}` as EventId,
    gameId: "g1" as GameId,
    index,
    status,
    type: "phase_started",
    phase: "night",
    visibility: { kind: "public" },
    payload: { phase: "night", dayNumber: 1 },
    createdAt: new Date("2026-06-26T00:00:00.000Z").toISOString(),
  };
}

describe("event log", () => {
  it("filters only active events in index order", () => {
    expect(getActiveEvents([event(2), event(1, "superseded")])).toEqual([event(2)]);
  });

  it("appends the next sequential active event", () => {
    const appended = appendEvent([event(1)], event(2));
    expect(appended.map((e) => e.index)).toEqual([1, 2]);
  });

  it("rejects non-sequential append", () => {
    expect(() => appendEvent([event(1)], event(3))).toThrow("Expected next event index 2 but received 3");
  });

  it("marks events after rollback index as superseded", () => {
    const rolledBack = rollbackAfterIndex([event(1), event(2), event(3)], 1);
    expect(rolledBack.map((e) => [e.index, e.status])).toEqual([
      [1, "active"],
      [2, "superseded"],
      [3, "superseded"],
    ]);
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

Run:

```bash
npm test -- src/core/__tests__/event-log.test.ts
```

Expected: FAIL because event modules do not exist.

- [ ] **Step 3: Implement event types**

Create `src/core/events.ts`:

```ts
import type { EventId, GameId, Phase, PlayerId } from "./types";

export type EventStatus = "active" | "superseded";

export type EventVisibility =
  | { readonly kind: "public" }
  | { readonly kind: "host_only" }
  | { readonly kind: "player_private"; readonly playerIds: readonly PlayerId[] }
  | { readonly kind: "faction_private"; readonly faction: "wolves" }
  | { readonly kind: "custom"; readonly playerIds: readonly PlayerId[] };

export type GameEventBase<Type extends string, Payload> = {
  readonly id: EventId;
  readonly gameId: GameId;
  readonly index: number;
  readonly status: EventStatus;
  readonly type: Type;
  readonly phase: Phase;
  readonly actorPlayerId?: PlayerId;
  readonly targetPlayerIds?: readonly PlayerId[];
  readonly visibility: EventVisibility;
  readonly payload: Payload;
  readonly display?: {
    readonly title?: string;
    readonly text?: string;
  };
  readonly createdFromDraftId?: string;
  readonly createdAt: string;
};

export type PhaseStartedEvent = GameEventBase<
  "phase_started",
  { readonly phase: Phase; readonly dayNumber: number }
>;

export type RoleAssignedEvent = GameEventBase<
  "role_assigned",
  { readonly playerId: PlayerId; readonly role: string; readonly faction: string }
>;

export type WolfKillSelectedEvent = GameEventBase<
  "wolf_kill_selected",
  { readonly targetPlayerId: PlayerId }
>;

export type SeerCheckSelectedEvent = GameEventBase<
  "seer_check_selected",
  { readonly targetPlayerId: PlayerId }
>;

export type SeerCheckResultEvent = GameEventBase<
  "seer_check_result",
  { readonly targetPlayerId: PlayerId; readonly result: "wolves" | "good" }
>;

export type WitchDeathInfoShownEvent = GameEventBase<
  "witch_death_info_shown",
  { readonly killedPlayerId: PlayerId | null }
>;

export type WitchAntidoteDecidedEvent = GameEventBase<
  "witch_antidote_decided",
  { readonly used: boolean; readonly targetPlayerId: PlayerId | null }
>;

export type WitchPoisonDecidedEvent = GameEventBase<
  "witch_poison_decided",
  { readonly used: boolean; readonly targetPlayerId: PlayerId | null }
>;

export type NightResolvedEvent = GameEventBase<
  "night_resolved",
  { readonly deadPlayerIds: readonly PlayerId[] }
>;

export type DeathAnnouncedEvent = GameEventBase<
  "death_announced",
  { readonly deadPlayerIds: readonly PlayerId[] }
>;

export type GameEvent =
  | PhaseStartedEvent
  | RoleAssignedEvent
  | WolfKillSelectedEvent
  | SeerCheckSelectedEvent
  | SeerCheckResultEvent
  | WitchDeathInfoShownEvent
  | WitchAntidoteDecidedEvent
  | WitchPoisonDecidedEvent
  | NightResolvedEvent
  | DeathAnnouncedEvent;
```

- [ ] **Step 4: Implement event log helpers**

Create `src/core/event-log.ts`:

```ts
import type { GameEvent } from "./events";

export function getActiveEvents(events: readonly GameEvent[]): readonly GameEvent[] {
  return events
    .filter((event) => event.status === "active")
    .slice()
    .sort((a, b) => a.index - b.index);
}

export function appendEvent(events: readonly GameEvent[], nextEvent: GameEvent): readonly GameEvent[] {
  const activeEvents = getActiveEvents(events);
  const lastIndex = activeEvents.at(-1)?.index ?? 0;
  const expectedIndex = lastIndex + 1;

  if (nextEvent.index !== expectedIndex) {
    throw new Error(`Expected next event index ${expectedIndex} but received ${nextEvent.index}`);
  }

  if (nextEvent.status !== "active") {
    throw new Error("Only active events can be appended");
  }

  return [...events, nextEvent];
}

export function rollbackAfterIndex(events: readonly GameEvent[], index: number): readonly GameEvent[] {
  return events.map((event) => {
    if (event.status === "active" && event.index > index) {
      return { ...event, status: "superseded" };
    }
    return event;
  });
}
```

- [ ] **Step 5: Run event log tests**

Run:

```bash
npm test -- src/core/__tests__/event-log.test.ts
```

Expected: PASS.

- [ ] **Step 6: Run all tests and typecheck**

Run:

```bash
npm test
npm run typecheck
```

Expected: PASS.

- [ ] **Step 7: Commit events**

```bash
git add src/core/events.ts src/core/event-log.ts src/core/__tests__/event-log.test.ts
git commit -m "feat: add official event log foundation"
```

## Task 5: Implement Visibility Projector

**Files:**
- Create: `src/core/visibility.ts`
- Test: `src/core/__tests__/visibility.test.ts`

- [ ] **Step 1: Write failing visibility tests**

Create `src/core/__tests__/visibility.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import { projectVisibleEvents } from "../visibility";
import type { GameEvent } from "../events";
import type { EventId, GameId, PlayerId } from "../types";

const gameId = "g1" as GameId;
const wolf = "p1" as PlayerId;
const seer = "p3" as PlayerId;
const witch = "p4" as PlayerId;

function baseEvent(overrides: Partial<GameEvent>): GameEvent {
  return {
    id: "e1" as EventId,
    gameId,
    index: 1,
    status: "active",
    type: "phase_started",
    phase: "night",
    visibility: { kind: "public" },
    payload: { phase: "night", dayNumber: 1 },
    createdAt: "2026-06-26T00:00:00.000Z",
    ...overrides,
  } as GameEvent;
}

describe("visibility projector", () => {
  it("shows public events to every player", () => {
    const events = [baseEvent({ index: 1, visibility: { kind: "public" } })];
    expect(projectVisibleEvents(events, seer, { wolfPlayerIds: [wolf] })).toHaveLength(1);
  });

  it("hides host-only events from players", () => {
    const events = [baseEvent({ index: 1, visibility: { kind: "host_only" } })];
    expect(projectVisibleEvents(events, seer, { wolfPlayerIds: [wolf] })).toHaveLength(0);
  });

  it("shows private player events only to listed players", () => {
    const events = [
      baseEvent({
        index: 1,
        visibility: { kind: "player_private", playerIds: [seer] },
      }),
    ];

    expect(projectVisibleEvents(events, seer, { wolfPlayerIds: [wolf] })).toHaveLength(1);
    expect(projectVisibleEvents(events, witch, { wolfPlayerIds: [wolf] })).toHaveLength(0);
  });

  it("shows wolf faction events only to wolves", () => {
    const events = [
      baseEvent({
        index: 1,
        visibility: { kind: "faction_private", faction: "wolves" },
      }),
    ];

    expect(projectVisibleEvents(events, wolf, { wolfPlayerIds: [wolf] })).toHaveLength(1);
    expect(projectVisibleEvents(events, seer, { wolfPlayerIds: [wolf] })).toHaveLength(0);
  });

  it("does not show superseded events", () => {
    const events = [
      baseEvent({
        index: 1,
        status: "superseded",
        visibility: { kind: "public" },
      }),
    ];

    expect(projectVisibleEvents(events, seer, { wolfPlayerIds: [wolf] })).toHaveLength(0);
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

Run:

```bash
npm test -- src/core/__tests__/visibility.test.ts
```

Expected: FAIL because `visibility.ts` does not exist.

- [ ] **Step 3: Implement visibility projector**

Create `src/core/visibility.ts`:

```ts
import { getActiveEvents } from "./event-log";
import type { EventVisibility, GameEvent } from "./events";
import type { PlayerId } from "./types";

export type VisibilityContext = {
  readonly wolfPlayerIds: readonly PlayerId[];
};

export function projectVisibleEvents(
  events: readonly GameEvent[],
  viewerPlayerId: PlayerId,
  context: VisibilityContext,
): readonly GameEvent[] {
  return getActiveEvents(events).filter((event) =>
    canPlayerSee(event.visibility, viewerPlayerId, context),
  );
}

export function canPlayerSee(
  visibility: EventVisibility,
  viewerPlayerId: PlayerId,
  context: VisibilityContext,
): boolean {
  if (visibility.kind === "public") return true;
  if (visibility.kind === "host_only") return false;
  if (visibility.kind === "player_private") {
    return visibility.playerIds.includes(viewerPlayerId);
  }
  if (visibility.kind === "custom") {
    return visibility.playerIds.includes(viewerPlayerId);
  }
  if (visibility.kind === "faction_private") {
    return visibility.faction === "wolves" && context.wolfPlayerIds.includes(viewerPlayerId);
  }
  return false;
}
```

- [ ] **Step 4: Run visibility tests**

Run:

```bash
npm test -- src/core/__tests__/visibility.test.ts
```

Expected: PASS.

- [ ] **Step 5: Run all tests and typecheck**

Run:

```bash
npm test
npm run typecheck
```

Expected: PASS.

- [ ] **Step 6: Commit visibility projector**

```bash
git add src/core/visibility.ts src/core/__tests__/visibility.test.ts
git commit -m "feat: add hard visibility projection"
```

## Task 6: Derive Game State From Active Events

**Files:**
- Create: `src/core/state.ts`
- Test: `src/core/__tests__/state.test.ts`

- [ ] **Step 1: Write failing state derivation tests**

Create `src/core/__tests__/state.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import { createPlayerSnapshot } from "../player";
import { deriveGameState } from "../state";
import type { GameEvent } from "../events";
import type { EventId, GameId, PlayerId } from "../types";

const gameId = "g1" as GameId;
const p1 = "p1" as PlayerId;
const p2 = "p2" as PlayerId;
const p3 = "p3" as PlayerId;

function event(index: number, event: Partial<GameEvent>): GameEvent {
  return {
    id: `e${index}` as EventId,
    gameId,
    index,
    status: "active",
    type: "phase_started",
    phase: "night",
    visibility: { kind: "public" },
    payload: { phase: "night", dayNumber: 1 },
    createdAt: "2026-06-26T00:00:00.000Z",
    ...event,
  } as GameEvent;
}

describe("deriveGameState", () => {
  const players = [
    createPlayerSnapshot({ playerId: p1, seatNo: 1, name: "P1", gameRole: "werewolf" }),
    createPlayerSnapshot({ playerId: p2, seatNo: 2, name: "P2", gameRole: "werewolf" }),
    createPlayerSnapshot({ playerId: p3, seatNo: 3, name: "P3", gameRole: "seer" }),
  ];

  it("starts all players alive and tracks current phase", () => {
    const state = deriveGameState(players, [
      event(1, {
        type: "phase_started",
        phase: "night",
        payload: { phase: "night", dayNumber: 1 },
      }),
    ]);

    expect(state.currentPhase).toBe("night");
    expect(state.dayNumber).toBe(1);
    expect(state.alivePlayerIds).toEqual([p1, p2, p3]);
  });

  it("marks night-resolved deaths as dead", () => {
    const state = deriveGameState(players, [
      event(1, {
        type: "night_resolved",
        payload: { deadPlayerIds: [p3] },
      }),
    ]);

    expect(state.alivePlayerIds).toEqual([p1, p2]);
    expect(state.deadPlayerIds).toEqual([p3]);
  });

  it("tracks witch medicine usage", () => {
    const state = deriveGameState(players, [
      event(1, {
        type: "witch_antidote_decided",
        actorPlayerId: p3,
        payload: { used: true, targetPlayerId: p2 },
      }),
      event(2, {
        type: "witch_poison_decided",
        actorPlayerId: p3,
        payload: { used: false, targetPlayerId: null },
      }),
    ]);

    expect(state.witch.antidoteAvailable).toBe(false);
    expect(state.witch.poisonAvailable).toBe(true);
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

Run:

```bash
npm test -- src/core/__tests__/state.test.ts
```

Expected: FAIL because `state.ts` does not exist.

- [ ] **Step 3: Implement state derivation**

Create `src/core/state.ts`:

```ts
import { getActiveEvents } from "./event-log";
import type { GameEvent } from "./events";
import type { PlayerSnapshot } from "./player";
import type { Phase, PlayerId } from "./types";

export type DerivedGameState = {
  readonly currentPhase: Phase;
  readonly dayNumber: number;
  readonly alivePlayerIds: readonly PlayerId[];
  readonly deadPlayerIds: readonly PlayerId[];
  readonly witch: {
    readonly antidoteAvailable: boolean;
    readonly poisonAvailable: boolean;
  };
};

export function deriveGameState(
  players: readonly PlayerSnapshot[],
  events: readonly GameEvent[],
): DerivedGameState {
  let currentPhase: Phase = "setup";
  let dayNumber = 0;
  const dead = new Set<PlayerId>();
  let antidoteAvailable = true;
  let poisonAvailable = true;

  for (const event of getActiveEvents(events)) {
    if (event.type === "phase_started") {
      currentPhase = event.payload.phase;
      dayNumber = event.payload.dayNumber;
    }

    if (event.type === "night_resolved") {
      for (const playerId of event.payload.deadPlayerIds) {
        dead.add(playerId);
      }
    }

    if (event.type === "witch_antidote_decided" && event.payload.used) {
      antidoteAvailable = false;
    }

    if (event.type === "witch_poison_decided" && event.payload.used) {
      poisonAvailable = false;
    }
  }

  const allPlayerIds = players.map((player) => player.playerId);

  return {
    currentPhase,
    dayNumber,
    alivePlayerIds: allPlayerIds.filter((playerId) => !dead.has(playerId)),
    deadPlayerIds: allPlayerIds.filter((playerId) => dead.has(playerId)),
    witch: {
      antidoteAvailable,
      poisonAvailable,
    },
  };
}
```

- [ ] **Step 4: Run state tests**

Run:

```bash
npm test -- src/core/__tests__/state.test.ts
```

Expected: PASS.

- [ ] **Step 5: Run all tests and typecheck**

Run:

```bash
npm test
npm run typecheck
```

Expected: PASS.

- [ ] **Step 6: Commit state derivation**

```bash
git add src/core/state.ts src/core/__tests__/state.test.ts
git commit -m "feat: derive game state from event log"
```

## Task 7: Implement First-Night Rules

**Files:**
- Create: `src/core/rules.ts`
- Test: `src/core/__tests__/rules.test.ts`

- [ ] **Step 1: Write failing rules tests**

Create `src/core/__tests__/rules.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import { createPlayerSnapshot } from "../player";
import {
  getLegalNightTargets,
  resolveNightDeaths,
  checkWinCondition,
} from "../rules";
import { createDefaultRuleset, type PlayerId } from "../types";

const p1 = "p1" as PlayerId;
const p2 = "p2" as PlayerId;
const p3 = "p3" as PlayerId;
const p4 = "p4" as PlayerId;
const p5 = "p5" as PlayerId;
const p6 = "p6" as PlayerId;

const players = [
  createPlayerSnapshot({ playerId: p1, seatNo: 1, name: "P1", gameRole: "werewolf" }),
  createPlayerSnapshot({ playerId: p2, seatNo: 2, name: "P2", gameRole: "werewolf" }),
  createPlayerSnapshot({ playerId: p3, seatNo: 3, name: "P3", gameRole: "seer" }),
  createPlayerSnapshot({ playerId: p4, seatNo: 4, name: "P4", gameRole: "witch" }),
  createPlayerSnapshot({ playerId: p5, seatNo: 5, name: "P5", gameRole: "villager" }),
  createPlayerSnapshot({ playerId: p6, seatNo: 6, name: "P6", gameRole: "villager" }),
];

describe("rules", () => {
  it("allows wolves to target living non-wolves", () => {
    expect(getLegalNightTargets("wolf_kill", players, [p1, p2, p3, p4, p5, p6])).toEqual([
      p3,
      p4,
      p5,
      p6,
    ]);
  });

  it("allows seer to check living players except self", () => {
    expect(getLegalNightTargets("seer_check", players, [p1, p2, p3, p4], p3)).toEqual([
      p1,
      p2,
      p4,
    ]);
  });

  it("rescues wolf kill with antidote", () => {
    expect(
      resolveNightDeaths({
        wolfKillTargetId: p3,
        antidoteTargetId: p3,
        poisonTargetId: null,
      }),
    ).toEqual([]);
  });

  it("kills poison target even when wolf kill is rescued", () => {
    expect(
      resolveNightDeaths({
        wolfKillTargetId: p3,
        antidoteTargetId: p3,
        poisonTargetId: p5,
      }),
    ).toEqual([p5]);
  });

  it("good team wins when all wolves are dead", () => {
    expect(checkWinCondition(players, [p1, p2], createDefaultRuleset())).toEqual({
      ended: true,
      winner: "good",
      reason: "all_wolves_dead",
    });
  });

  it("wolves win by slaughter side when all gods are dead", () => {
    expect(checkWinCondition(players, [p3, p4], createDefaultRuleset())).toEqual({
      ended: true,
      winner: "wolves",
      reason: "all_gods_dead",
    });
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

Run:

```bash
npm test -- src/core/__tests__/rules.test.ts
```

Expected: FAIL because `rules.ts` does not exist.

- [ ] **Step 3: Implement first-night rules**

Create `src/core/rules.ts`:

```ts
import type { PlayerSnapshot } from "./player";
import type { PlayerId, Ruleset } from "./types";

export type NightActionKind = "wolf_kill" | "seer_check" | "witch_poison";

export type NightResolutionInput = {
  readonly wolfKillTargetId: PlayerId | null;
  readonly antidoteTargetId: PlayerId | null;
  readonly poisonTargetId: PlayerId | null;
};

export type WinCheckResult =
  | { readonly ended: false }
  | {
      readonly ended: true;
      readonly winner: "wolves" | "good";
      readonly reason: "all_wolves_dead" | "all_gods_dead" | "all_villagers_dead" | "all_good_dead";
    };

export function getLegalNightTargets(
  action: NightActionKind,
  players: readonly PlayerSnapshot[],
  alivePlayerIds: readonly PlayerId[],
  actorPlayerId?: PlayerId,
): readonly PlayerId[] {
  const alive = new Set(alivePlayerIds);

  if (action === "wolf_kill") {
    return players
      .filter((player) => alive.has(player.playerId))
      .filter((player) => player.gameRole !== "werewolf")
      .map((player) => player.playerId);
  }

  if (action === "seer_check") {
    return players
      .filter((player) => alive.has(player.playerId))
      .filter((player) => player.playerId !== actorPlayerId)
      .map((player) => player.playerId);
  }

  return players
    .filter((player) => alive.has(player.playerId))
    .filter((player) => player.playerId !== actorPlayerId)
    .map((player) => player.playerId);
}

export function resolveNightDeaths(input: NightResolutionInput): readonly PlayerId[] {
  const dead = new Set<PlayerId>();

  if (input.wolfKillTargetId && input.wolfKillTargetId !== input.antidoteTargetId) {
    dead.add(input.wolfKillTargetId);
  }

  if (input.poisonTargetId) {
    dead.add(input.poisonTargetId);
  }

  return [...dead];
}

export function checkWinCondition(
  players: readonly PlayerSnapshot[],
  deadPlayerIds: readonly PlayerId[],
  ruleset: Ruleset,
): WinCheckResult {
  const dead = new Set(deadPlayerIds);
  const livingPlayers = players.filter((player) => !dead.has(player.playerId));
  const livingWolves = livingPlayers.filter((player) => player.gameRole === "werewolf");

  if (livingWolves.length === 0) {
    return { ended: true, winner: "good", reason: "all_wolves_dead" };
  }

  if (ruleset.winCondition === "slaughter_all") {
    const livingGood = livingPlayers.filter((player) => player.faction === "good");
    if (livingGood.length === 0) {
      return { ended: true, winner: "wolves", reason: "all_good_dead" };
    }
    return { ended: false };
  }

  const livingGods = livingPlayers.filter(
    (player) => player.gameRole === "seer" || player.gameRole === "witch",
  );
  if (livingGods.length === 0) {
    return { ended: true, winner: "wolves", reason: "all_gods_dead" };
  }

  const livingVillagers = livingPlayers.filter((player) => player.gameRole === "villager");
  if (livingVillagers.length === 0) {
    return { ended: true, winner: "wolves", reason: "all_villagers_dead" };
  }

  return { ended: false };
}
```

- [ ] **Step 4: Run rules tests**

Run:

```bash
npm test -- src/core/__tests__/rules.test.ts
```

Expected: PASS.

- [ ] **Step 5: Run all tests and typecheck**

Run:

```bash
npm test
npm run typecheck
```

Expected: PASS.

- [ ] **Step 6: Commit rules foundation**

```bash
git add src/core/rules.ts src/core/__tests__/rules.test.ts
git commit -m "feat: add six player night rules"
```

## Task 8: Add Foundation Acceptance Test

**Files:**
- Create: `src/core/__tests__/foundation-flow.test.ts`

- [ ] **Step 1: Write end-to-end foundation test**

Create `src/core/__tests__/foundation-flow.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import { createPlayerSnapshot } from "../player";
import { projectVisibleEvents } from "../visibility";
import { deriveGameState } from "../state";
import { resolveNightDeaths } from "../rules";
import type { GameEvent } from "../events";
import type { EventId, GameId, PlayerId } from "../types";

const gameId = "g1" as GameId;
const wolf1 = "p1" as PlayerId;
const wolf2 = "p2" as PlayerId;
const seer = "p3" as PlayerId;
const witch = "p4" as PlayerId;
const villager1 = "p5" as PlayerId;
const villager2 = "p6" as PlayerId;

const players = [
  createPlayerSnapshot({ playerId: wolf1, seatNo: 1, name: "Wolf 1", gameRole: "werewolf" }),
  createPlayerSnapshot({ playerId: wolf2, seatNo: 2, name: "Wolf 2", gameRole: "werewolf" }),
  createPlayerSnapshot({ playerId: seer, seatNo: 3, name: "Seer", gameRole: "seer" }),
  createPlayerSnapshot({ playerId: witch, seatNo: 4, name: "Witch", gameRole: "witch" }),
  createPlayerSnapshot({ playerId: villager1, seatNo: 5, name: "Villager 1", gameRole: "villager" }),
  createPlayerSnapshot({ playerId: villager2, seatNo: 6, name: "Villager 2", gameRole: "villager" }),
];

function event(index: number, event: Partial<GameEvent>): GameEvent {
  return {
    id: `e${index}` as EventId,
    gameId,
    index,
    status: "active",
    type: "phase_started",
    phase: "night",
    visibility: { kind: "public" },
    payload: { phase: "night", dayNumber: 1 },
    createdAt: "2026-06-26T00:00:00.000Z",
    ...event,
  } as GameEvent;
}

describe("foundation flow", () => {
  it("keeps hidden night truth out of non-owner viewpoints while deriving public death state", () => {
    const dead = resolveNightDeaths({
      wolfKillTargetId: seer,
      antidoteTargetId: null,
      poisonTargetId: null,
    });

    const events = [
      event(1, {
        type: "phase_started",
        phase: "night",
        payload: { phase: "night", dayNumber: 1 },
      }),
      event(2, {
        type: "wolf_kill_selected",
        actorPlayerId: wolf1,
        targetPlayerIds: [seer],
        visibility: { kind: "faction_private", faction: "wolves" },
        payload: { targetPlayerId: seer },
      }),
      event(3, {
        type: "seer_check_result",
        actorPlayerId: seer,
        targetPlayerIds: [wolf1],
        visibility: { kind: "player_private", playerIds: [seer] },
        payload: { targetPlayerId: wolf1, result: "wolves" },
      }),
      event(4, {
        type: "night_resolved",
        visibility: { kind: "host_only" },
        payload: { deadPlayerIds: dead },
      }),
      event(5, {
        type: "death_announced",
        phase: "day",
        visibility: { kind: "public" },
        payload: { deadPlayerIds: dead },
      }),
    ];

    const villagerView = projectVisibleEvents(events, villager1, {
      wolfPlayerIds: [wolf1, wolf2],
    });
    const wolfView = projectVisibleEvents(events, wolf1, {
      wolfPlayerIds: [wolf1, wolf2],
    });
    const seerView = projectVisibleEvents(events, seer, {
      wolfPlayerIds: [wolf1, wolf2],
    });
    const state = deriveGameState(players, events);

    expect(villagerView.map((visibleEvent) => visibleEvent.type)).toEqual([
      "phase_started",
      "death_announced",
    ]);
    expect(wolfView.map((visibleEvent) => visibleEvent.type)).toContain("wolf_kill_selected");
    expect(seerView.map((visibleEvent) => visibleEvent.type)).toContain("seer_check_result");
    expect(state.deadPlayerIds).toEqual([seer]);
  });
});
```

- [ ] **Step 2: Run acceptance test**

Run:

```bash
npm test -- src/core/__tests__/foundation-flow.test.ts
```

Expected: PASS.

- [ ] **Step 3: Run full verification**

Run:

```bash
npm test
npm run typecheck
npm run build
```

Expected:

- All tests pass.
- Typecheck passes.
- Next.js build succeeds.

- [ ] **Step 4: Commit foundation acceptance flow**

```bash
git add src/core/__tests__/foundation-flow.test.ts
git commit -m "test: cover foundation werewolf event flow"
```

## Self-Review Checklist

- Spec coverage in this plan:
  - Official event log: Tasks 4 and 8.
  - Visibility hard boundary: Task 5 and Task 8.
  - Fixed 6-player board: Task 3.
  - Ruleset defaults: Task 2.
  - Derived game state from active events: Task 6.
  - First-night wolf, seer, witch rules: Task 7.
  - Rollback with superseded events: Task 4.
- Intentional gaps for later plans:
  - Draft workflow and host confirmation.
  - LLM orchestration.
  - SQLite persistence.
  - Editor UI.
  - Playback compiler and playback UI.
  - Recording.
- Red-flag scan:
  - No unresolved markers or unspecified implementation steps.
  - Each code-changing step includes exact code.
- Type consistency:
  - `PlayerId`, `GameId`, `EventId`, `GameEvent`, `PlayerSnapshot`, and `Ruleset` are introduced before use.
  - Event type names match between tests and event union.
  - Rule function names match tests and implementation.
