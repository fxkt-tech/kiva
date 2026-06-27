# Role Library UI Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a `/library` workspace for editing roles, characters, and presets, then create games explicitly from a selected preset.

**Architecture:** Keep domain validation in core/server modules, then render a server-driven Next.js workspace with small client components only where browser state is required. Library writes go through `LibraryRepository` and validate the combined record before persisting. Game creation consumes a preset id and redirects to the existing editor.

**Tech Stack:** Next.js App Router, React Server Components, server actions, TypeScript, Tailwind CSS, Vitest.

---

## File Structure

- Create `src/core/library-diagnostics.ts`: pure diagnostics and prompt-preview helpers for roles, characters, and presets.
- Create `src/core/__tests__/library-diagnostics.test.ts`: diagnostics coverage.
- Create `src/server/library-actions.ts`: server-side library use cases over `LibraryRepository` and `GameRepository`.
- Create `src/server/__tests__/library-actions.test.ts`: save, duplicate, toggle, and create-game tests.
- Create `src/app/library/actions.ts`: Next server actions and form parsing for the UI.
- Create `src/app/library/page.tsx`: `/library` route.
- Create `src/components/library/library-workspace.tsx`: tabbed workspace shell.
- Create `src/components/library/library-list.tsx`: selected collection list.
- Create `src/components/library/role-editor.tsx`: role detail form.
- Create `src/components/library/character-editor.tsx`: character detail form.
- Create `src/components/library/preset-editor.tsx`: preset seat table and create-game form.
- Create `src/components/library/validation-panel.tsx`: diagnostics and prompt preview.
- Create `src/components/library/dirty-form-guard.tsx`: client-only unsaved-change warning.
- Create focused component tests under `src/components/library/*.test.ts`.
- Modify `src/app/page.tsx`: add Library link and change `New game` to preset selection.
- Modify `src/app/actions.ts`: replace default create-game action with preset-specific creation, or delegate to `src/app/library/actions.ts`.
- Modify `src/server/game-actions.ts`: add `createGameFromPresetId(presetId)` while keeping `createGame()` as a compatibility wrapper if needed by tests.

---

### Task 1: Core Library Diagnostics

**Files:**
- Create: `src/core/library-diagnostics.ts`
- Test: `src/core/__tests__/library-diagnostics.test.ts`

- [ ] **Step 1: Write failing diagnostics tests**

Create `src/core/__tests__/library-diagnostics.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import {
  diagnoseCharacter,
  diagnosePreset,
  diagnoseRole,
  promptPreviewForPresetSeat,
} from "../library-diagnostics";
import type { CharacterDefinition } from "../character-definition";
import type { GamePreset } from "../game-preset";
import type { RoleDefinition } from "../role-definition";

const timestamp = "2026-06-27T00:00:00.000Z";

const werewolf = role({
  id: "werewolf",
  name: "狼人",
  faction: "wolves",
  team: "wolf",
  mechanicKey: "wolf_kill",
  systemPrompt: "你是狼人。",
  actionPrompt: "选择袭击目标。",
});
const seer = role({
  id: "seer",
  name: "预言家",
  faction: "good",
  team: "god",
  mechanicKey: "seer_check",
  systemPrompt: "你是预言家。",
  actionPrompt: "选择查验目标。",
});
const qin = character({ id: "qin", name: "秦川", systemPrompt: "你是秦川。" });
const lin = character({ id: "lin", name: "林夏", systemPrompt: "你是林夏。" });

describe("library diagnostics", () => {
  it("reports role references and built-in contract status", () => {
    const preset = presetWithSeats({
      roles: ["werewolf", "seer"],
      characters: ["qin", "lin"],
    });

    expect(
      diagnoseRole({
        role: werewolf,
        roles: [werewolf, seer],
        characters: [qin, lin],
        presets: [preset],
      }),
    ).toMatchObject({
      valid: true,
      references: ["two_player_test"],
      messages: expect.arrayContaining(["Built-in role contract locked"]),
      promptPreview: expect.stringContaining("你是狼人。"),
    });
  });

  it("reports character references without role fields", () => {
    const preset = presetWithSeats({
      roles: ["werewolf", "seer"],
      characters: ["qin", "lin"],
    });

    expect(
      diagnoseCharacter({
        character: qin,
        roles: [werewolf, seer],
        characters: [qin, lin],
        presets: [preset],
      }),
    ).toMatchObject({
      valid: true,
      references: ["two_player_test"],
      promptPreview: expect.stringContaining("你是秦川。"),
    });
  });

  it("marks presets invalid when they reference disabled items", () => {
    const disabledSeer = { ...seer, enabled: false };
    const preset = presetWithSeats({
      roles: ["werewolf", "seer"],
      characters: ["qin", "lin"],
    });

    expect(
      diagnosePreset({
        preset,
        roles: [werewolf, disabledSeer],
        characters: [qin, lin],
        presets: [preset],
      }),
    ).toMatchObject({
      valid: false,
      canCreateGame: false,
      messages: expect.arrayContaining([
        "Game preset two_player_test references disabled role: seer",
      ]),
    });
  });

  it("builds preset seat prompt preview from character, role, action, and model", () => {
    const preset = presetWithSeats({
      roles: ["werewolf", "seer"],
      characters: ["qin", "lin"],
    });

    expect(
      promptPreviewForPresetSeat({
        preset,
        seatNo: 1,
        roles: [werewolf, seer],
        characters: [qin, lin],
      }),
    ).toContain("你是秦川。\n你是狼人。\n选择袭击目标。\nmodel: mock/mock-model");
  });
});

function role(overrides: Partial<RoleDefinition>): RoleDefinition {
  return {
    id: "werewolf",
    name: "狼人",
    faction: "wolves",
    team: "wolf",
    mechanicKey: "wolf_kill",
    systemPrompt: "role prompt",
    actionPrompt: null,
    visibilityRules: ["own_role"],
    nightOrder: null,
    defaultModelBinding: {
      provider: "mock",
      model: "mock-model",
      temperature: 0.7,
      maxTokens: 1000,
      responseFormat: "json",
    },
    enabled: true,
    createdAt: timestamp,
    updatedAt: timestamp,
    ...overrides,
  };
}

function character(overrides: Partial<CharacterDefinition>): CharacterDefinition {
  return {
    id: "qin",
    name: "秦川",
    avatar: null,
    tags: [],
    persona: "冷静",
    speakingStyle: "短句",
    reasoningStyle: "证据优先",
    systemPrompt: "character prompt",
    defaultModelBinding: null,
    enabled: true,
    createdAt: timestamp,
    updatedAt: timestamp,
    ...overrides,
  };
}

function presetWithSeats(input: {
  roles: readonly string[];
  characters: readonly string[];
}): GamePreset {
  return {
    id: "two_player_test",
    name: "Two player test",
    rulesetId: "test_ruleset",
    playerCount: 2,
    roleIds: input.roles,
    characterIds: input.characters,
    seatAssignments: input.roles.map((roleId, index) => ({
      seatNo: index + 1,
      roleId,
      characterId: input.characters[index]!,
      modelBindingOverride: null,
    })),
    enabled: true,
    createdAt: timestamp,
    updatedAt: timestamp,
  };
}
```

- [ ] **Step 2: Run test to verify it fails**

Run:

```bash
pnpm vitest run src/core/__tests__/library-diagnostics.test.ts
```

Expected: FAIL because `src/core/library-diagnostics.ts` does not exist.

- [ ] **Step 3: Implement diagnostics helpers**

Create `src/core/library-diagnostics.ts`:

```ts
import type { CharacterDefinition } from "./character-definition";
import type { GamePreset } from "./game-preset";
import type { ModelBindingSnapshot } from "./player";
import type { RoleDefinition } from "./role-definition";

export type LibraryDiagnostic = {
  readonly valid: boolean;
  readonly canCreateGame?: boolean;
  readonly references: readonly string[];
  readonly messages: readonly string[];
  readonly promptPreview: string;
};

type LibraryContext = {
  readonly roles: readonly RoleDefinition[];
  readonly characters: readonly CharacterDefinition[];
  readonly presets: readonly GamePreset[];
};

export function diagnoseRole(
  input: LibraryContext & { readonly role: RoleDefinition },
): LibraryDiagnostic {
  const references = input.presets
    .filter((preset) => preset.roleIds.includes(input.role.id))
    .map((preset) => preset.id);
  const messages = [
    ...(isBuiltInRole(input.role.id) ? ["Built-in role contract locked"] : []),
    ...(input.role.enabled ? [] : ["Role is disabled"]),
  ];

  return {
    valid: messages.every((message) => message !== "Role is disabled"),
    references,
    messages,
    promptPreview: [input.role.systemPrompt, input.role.actionPrompt]
      .filter((value): value is string => Boolean(value))
      .join("\n"),
  };
}

export function diagnoseCharacter(
  input: LibraryContext & { readonly character: CharacterDefinition },
): LibraryDiagnostic {
  const references = input.presets
    .filter((preset) => preset.characterIds.includes(input.character.id))
    .map((preset) => preset.id);
  const messages = input.character.enabled ? [] : ["Character is disabled"];

  return {
    valid: input.character.enabled,
    references,
    messages,
    promptPreview: [
      input.character.systemPrompt,
      input.character.persona,
      input.character.speakingStyle,
      input.character.reasoningStyle,
    ].join("\n"),
  };
}

export function diagnosePreset(
  input: LibraryContext & { readonly preset: GamePreset },
): LibraryDiagnostic {
  const roleById = new Map(input.roles.map((role) => [role.id, role]));
  const characterById = new Map(
    input.characters.map((character) => [character.id, character]),
  );
  const messages: string[] = [];

  if (!input.preset.enabled) {
    messages.push("Preset is disabled");
  }

  for (const roleId of input.preset.roleIds) {
    const role = roleById.get(roleId);
    if (!role) {
      messages.push(`Game preset ${input.preset.id} references unknown role: ${roleId}`);
    } else if (!role.enabled) {
      messages.push(`Game preset ${input.preset.id} references disabled role: ${roleId}`);
    }
  }

  for (const characterId of input.preset.characterIds) {
    const character = characterById.get(characterId);
    if (!character) {
      messages.push(
        `Game preset ${input.preset.id} references unknown character: ${characterId}`,
      );
    } else if (!character.enabled) {
      messages.push(
        `Game preset ${input.preset.id} references disabled character: ${characterId}`,
      );
    }
  }

  if (!input.preset.seatAssignments) {
    messages.push(`Game preset ${input.preset.id} has no seat assignments`);
  }

  return {
    valid: messages.length === 0,
    canCreateGame: messages.length === 0,
    references: [],
    messages,
    promptPreview: promptPreviewForPresetSeat({
      preset: input.preset,
      seatNo: input.preset.seatAssignments?.[0]?.seatNo ?? 1,
      roles: input.roles,
      characters: input.characters,
    }),
  };
}

export function promptPreviewForPresetSeat(input: {
  readonly preset: GamePreset;
  readonly seatNo: number;
  readonly roles: readonly RoleDefinition[];
  readonly characters: readonly CharacterDefinition[];
}): string {
  const assignment = input.preset.seatAssignments?.find(
    (item) => item.seatNo === input.seatNo,
  );
  if (!assignment) {
    return "No seat assignment selected.";
  }

  const role = input.roles.find((item) => item.id === assignment.roleId);
  const character = input.characters.find(
    (item) => item.id === assignment.characterId,
  );
  if (!role || !character) {
    return "Seat assignment references missing library items.";
  }

  return [
    character.systemPrompt,
    role.systemPrompt,
    role.actionPrompt,
    `model: ${modelLabel(
      assignment.modelBindingOverride ??
        character.defaultModelBinding ??
        role.defaultModelBinding,
    )}`,
  ]
    .filter((value): value is string => Boolean(value))
    .join("\n");
}

function modelLabel(modelBinding: ModelBindingSnapshot | null): string {
  return modelBinding
    ? `${modelBinding.provider}/${modelBinding.model}`
    : "system/default";
}

function isBuiltInRole(roleId: string): boolean {
  return ["werewolf", "seer", "witch", "villager"].includes(roleId);
}
```

- [ ] **Step 4: Run tests**

Run:

```bash
pnpm vitest run src/core/__tests__/library-diagnostics.test.ts
pnpm typecheck
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/core/library-diagnostics.ts src/core/__tests__/library-diagnostics.test.ts
git commit -m "feat: add library diagnostics"
```

---

### Task 2: Server Library Use Cases

**Files:**
- Create: `src/server/library-actions.ts`
- Test: `src/server/__tests__/library-actions.test.ts`
- Modify: `src/server/game-actions.ts`
- Test: `src/server/__tests__/game-actions.test.ts`

- [ ] **Step 1: Write failing server action tests**

Create `src/server/__tests__/library-actions.test.ts`:

```ts
import { mkdtemp, rm } from "node:fs/promises";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { afterEach, describe, expect, it } from "vitest";
import { createGameRepository } from "../game-repository";
import { createLibraryActions } from "../library-actions";
import { createLibraryRepository } from "../library-repository";
import { seedCharacters } from "@/seeds/characters";
import { seedPresets } from "@/seeds/presets";
import { seedRoles } from "@/seeds/roles";

const tempDirs: string[] = [];

afterEach(async () => {
  await Promise.all(tempDirs.map((dir) => rm(dir, { recursive: true, force: true })));
  tempDirs.length = 0;
});

describe("library actions", () => {
  it("loads diagnostics with a complete library record", async () => {
    const actions = await actionsWithSeeds();

    const record = await actions.getLibrary();

    expect(record.roles.length).toBe(4);
    expect(record.characters.length).toBe(6);
    expect(record.presets.length).toBe(1);
    expect(record.diagnostics.presets[0]).toMatchObject({
      canCreateGame: true,
    });
  });

  it("saves a role and validates dependent presets", async () => {
    const actions = await actionsWithSeeds();
    const library = await actions.getLibrary();
    const role = library.roles.find((item) => item.id === "werewolf")!;

    await actions.saveRole({ ...role, systemPrompt: "新的狼人提示词" });

    const saved = await actions.getLibrary();
    expect(saved.roles.find((item) => item.id === "werewolf")?.systemPrompt).toBe(
      "新的狼人提示词",
    );
  });

  it("rejects saving a built-in role with mismatched mechanics", async () => {
    const actions = await actionsWithSeeds();
    const role = (await actions.getLibrary()).roles.find(
      (item) => item.id === "werewolf",
    )!;

    await expect(
      actions.saveRole({ ...role, mechanicKey: "none" }),
    ).rejects.toThrow("Role werewolf does not match the current ruleset contract");
  });

  it("duplicates and disables a character", async () => {
    const actions = await actionsWithSeeds();

    await actions.duplicateCharacter("qin_chuan");
    await actions.setCharacterEnabled("qin_chuan_copy", false);

    const saved = await actions.getLibrary();
    expect(saved.characters.some((item) => item.id === "qin_chuan_copy")).toBe(true);
    expect(saved.characters.find((item) => item.id === "qin_chuan_copy")?.enabled).toBe(false);
  });

  it("creates a game from a selected preset", async () => {
    const actions = await actionsWithSeeds();

    const record = await actions.createGameFromPreset("six_player_standard");

    expect(record.game.title).toBe("6人狼人杀试运行");
    expect(record.game.players).toHaveLength(6);
  });
});

async function actionsWithSeeds() {
  const root = await mkdtemp(join(tmpdir(), "kiva-library-actions-"));
  tempDirs.push(root);
  const libraryRepository = createLibraryRepository(join(root, "library"));
  await libraryRepository.saveAll({
    roles: seedRoles,
    characters: seedCharacters,
    presets: seedPresets,
  });

  return createLibraryActions({
    libraryRepository,
    gameRepository: createGameRepository(join(root, "games")),
  });
}
```

- [ ] **Step 2: Run tests to verify failure**

Run:

```bash
pnpm vitest run src/server/__tests__/library-actions.test.ts
```

Expected: FAIL because `createLibraryActions` does not exist.

- [ ] **Step 3: Implement `createGameFromPresetId`**

Modify `src/server/game-actions.ts`:

```ts
async function createGameFromPresetId(presetId: string): Promise<GameRecord> {
  const createdAt = now();
  const library = await loadGameLibrary(options.libraryRepository);
  const preset = library.presets.find((item) => item.id === presetId);
  if (!preset) {
    throw new Error(`Game preset not found: ${presetId}`);
  }

  const game = createGameFromPreset({
    gameId: createGameId(),
    title: preset.name,
    createdAt,
    ruleset: createDefaultRuleset(),
    preset,
    roles: library.roles,
    characters: library.characters,
  });
  const record: GameRecord = {
    game,
    events: [],
    draft: null,
    generations: [],
  };

  await repository.save(record);
  return record;
}
```

Return it from `createGameActions`:

```ts
async createGame(): Promise<GameRecord> {
  return createGameFromPresetId(options.defaultPresetId ?? "six_player_standard");
},

createGameFromPresetId,
```

- [ ] **Step 4: Implement `createLibraryActions`**

Create `src/server/library-actions.ts`:

```ts
import { createGameActions } from "./game-actions";
import type { GameRepository } from "./game-repository";
import type { LibraryRecord, LibraryRepository } from "./library-repository";
import type { CharacterDefinition } from "@/core/character-definition";
import type { GamePreset } from "@/core/game-preset";
import {
  diagnoseCharacter,
  diagnosePreset,
  diagnoseRole,
  type LibraryDiagnostic,
} from "@/core/library-diagnostics";
import type { RoleDefinition } from "@/core/role-definition";

export type LibraryActionsInput = {
  readonly libraryRepository: LibraryRepository;
  readonly gameRepository: GameRepository;
};

export type LibraryViewRecord = LibraryRecord & {
  readonly diagnostics: {
    readonly roles: readonly LibraryDiagnostic[];
    readonly characters: readonly LibraryDiagnostic[];
    readonly presets: readonly LibraryDiagnostic[];
  };
};

export function createLibraryActions(input: LibraryActionsInput) {
  async function load(): Promise<LibraryRecord> {
    return input.libraryRepository.loadAll();
  }

  async function saveAll(record: LibraryRecord): Promise<void> {
    await input.libraryRepository.saveAll(record);
  }

  return {
    async getLibrary(): Promise<LibraryViewRecord> {
      const record = await load();
      return {
        ...record,
        diagnostics: {
          roles: record.roles.map((role) => diagnoseRole({ ...record, role })),
          characters: record.characters.map((character) =>
            diagnoseCharacter({ ...record, character }),
          ),
          presets: record.presets.map((preset) => diagnosePreset({ ...record, preset })),
        },
      };
    },

    async saveRole(role: RoleDefinition): Promise<void> {
      const record = await load();
      await saveAll({
        ...record,
        roles: replaceById(record.roles, role),
      });
    },

    async saveCharacter(character: CharacterDefinition): Promise<void> {
      const record = await load();
      await saveAll({
        ...record,
        characters: replaceById(record.characters, character),
      });
    },

    async savePreset(preset: GamePreset): Promise<void> {
      const record = await load();
      await saveAll({
        ...record,
        presets: replaceById(record.presets, preset),
      });
    },

    async duplicateRole(roleId: string): Promise<void> {
      const record = await load();
      const role = required(record.roles.find((item) => item.id === roleId), roleId);
      await saveAll({
        ...record,
        roles: [...record.roles, { ...role, id: nextCopyId(role.id, record.roles) }],
      });
    },

    async duplicateCharacter(characterId: string): Promise<void> {
      const record = await load();
      const character = required(
        record.characters.find((item) => item.id === characterId),
        characterId,
      );
      await saveAll({
        ...record,
        characters: [
          ...record.characters,
          { ...character, id: nextCopyId(character.id, record.characters) },
        ],
      });
    },

    async duplicatePreset(presetId: string): Promise<void> {
      const record = await load();
      const preset = required(record.presets.find((item) => item.id === presetId), presetId);
      await saveAll({
        ...record,
        presets: [...record.presets, { ...preset, id: nextCopyId(preset.id, record.presets) }],
      });
    },

    async setRoleEnabled(roleId: string, enabled: boolean): Promise<void> {
      const record = await load();
      await saveAll({
        ...record,
        roles: record.roles.map((role) =>
          role.id === roleId ? { ...role, enabled } : role,
        ),
      });
    },

    async setCharacterEnabled(characterId: string, enabled: boolean): Promise<void> {
      const record = await load();
      await saveAll({
        ...record,
        characters: record.characters.map((character) =>
          character.id === characterId ? { ...character, enabled } : character,
        ),
      });
    },

    async setPresetEnabled(presetId: string, enabled: boolean): Promise<void> {
      const record = await load();
      await saveAll({
        ...record,
        presets: record.presets.map((preset) =>
          preset.id === presetId ? { ...preset, enabled } : preset,
        ),
      });
    },

    async createGameFromPreset(presetId: string) {
      return createGameActions(input.gameRepository, {
        libraryRepository: input.libraryRepository,
        defaultPresetId: presetId,
      }).createGameFromPresetId(presetId);
    },
  };
}

function replaceById<T extends { readonly id: string }>(
  items: readonly T[],
  nextItem: T,
): readonly T[] {
  return items.some((item) => item.id === nextItem.id)
    ? items.map((item) => (item.id === nextItem.id ? nextItem : item))
    : [...items, nextItem];
}

function required<T>(value: T | undefined, id: string): T {
  if (!value) {
    throw new Error(`Library item not found: ${id}`);
  }
  return value;
}

function nextCopyId(
  baseId: string,
  items: readonly { readonly id: string }[],
): string {
  const ids = new Set(items.map((item) => item.id));
  let candidate = `${baseId}_copy`;
  let index = 2;
  while (ids.has(candidate)) {
    candidate = `${baseId}_copy_${index}`;
    index += 1;
  }
  return candidate;
}
```

- [ ] **Step 5: Run server tests**

Run:

```bash
pnpm vitest run src/server/__tests__/library-actions.test.ts src/server/__tests__/game-actions.test.ts
pnpm typecheck
```

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add src/server/library-actions.ts src/server/__tests__/library-actions.test.ts src/server/game-actions.ts src/server/__tests__/game-actions.test.ts
git commit -m "feat: add library management actions"
```

---

### Task 3: Library Server Actions And Form Parsing

**Files:**
- Create: `src/app/library/actions.ts`
- Create: `src/app/library/form-parsers.ts`
- Test: `src/app/library/form-parsers.test.ts`
- Modify: `src/app/actions.ts`

- [ ] **Step 1: Write failing form parser tests**

Create `src/app/library/form-parsers.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import { characterFromFormData, presetFromFormData, roleFromFormData } from "./form-parsers";

const timestamp = "2026-06-27T00:00:00.000Z";

describe("library form parsers", () => {
  it("parses a role form", () => {
    const form = new FormData();
    form.set("id", "werewolf");
    form.set("name", "狼人");
    form.set("enabled", "on");
    form.set("faction", "wolves");
    form.set("team", "wolf");
    form.set("mechanicKey", "wolf_kill");
    form.set("visibilityRules", "own_role,wolf_teammates");
    form.set("nightOrder", "10");
    form.set("systemPrompt", "你是狼人。");
    form.set("actionPrompt", "选择袭击目标。");

    expect(roleFromFormData(form, timestamp)).toMatchObject({
      id: "werewolf",
      enabled: true,
      visibilityRules: ["own_role", "wolf_teammates"],
      nightOrder: 10,
      actionPrompt: "选择袭击目标。",
    });
  });

  it("parses a character form", () => {
    const form = new FormData();
    form.set("id", "qin");
    form.set("name", "秦川");
    form.set("enabled", "on");
    form.set("tags", "冷静,强势");
    form.set("persona", "冷静");
    form.set("speakingStyle", "短句");
    form.set("reasoningStyle", "证据优先");
    form.set("systemPrompt", "你是秦川。");

    expect(characterFromFormData(form, timestamp)).toMatchObject({
      id: "qin",
      tags: ["冷静", "强势"],
      avatar: null,
      enabled: true,
    });
  });

  it("parses a preset seat table", () => {
    const form = new FormData();
    form.set("id", "six_player_standard");
    form.set("name", "6人狼人杀试运行");
    form.set("rulesetId", "six_player_v1");
    form.set("playerCount", "2");
    form.set("enabled", "on");
    form.set("seat.1.roleId", "werewolf");
    form.set("seat.1.characterId", "qin");
    form.set("seat.2.roleId", "seer");
    form.set("seat.2.characterId", "lin");

    expect(presetFromFormData(form, timestamp)).toMatchObject({
      roleIds: ["werewolf", "seer"],
      characterIds: ["qin", "lin"],
      seatAssignments: [
        { seatNo: 1, roleId: "werewolf", characterId: "qin" },
        { seatNo: 2, roleId: "seer", characterId: "lin" },
      ],
    });
  });
});
```

- [ ] **Step 2: Run parser tests to verify failure**

Run:

```bash
pnpm vitest run src/app/library/form-parsers.test.ts
```

Expected: FAIL because parser module does not exist.

- [ ] **Step 3: Implement form parsers**

Create `src/app/library/form-parsers.ts` with exported functions:

```ts
import type { CharacterDefinition } from "@/core/character-definition";
import type { GamePreset, GamePresetSeatAssignment } from "@/core/game-preset";
import type { RoleDefinition } from "@/core/role-definition";

export function roleFromFormData(formData: FormData, now: string): RoleDefinition {
  return {
    id: text(formData, "id"),
    name: text(formData, "name"),
    enabled: checkbox(formData, "enabled"),
    faction: text(formData, "faction") as RoleDefinition["faction"],
    team: text(formData, "team") as RoleDefinition["team"],
    mechanicKey: text(formData, "mechanicKey") as RoleDefinition["mechanicKey"],
    visibilityRules: csv(formData, "visibilityRules") as RoleDefinition["visibilityRules"],
    nightOrder: nullableNumber(formData, "nightOrder"),
    systemPrompt: text(formData, "systemPrompt"),
    actionPrompt: nullableText(formData, "actionPrompt"),
    defaultModelBinding: null,
    createdAt: textOrDefault(formData, "createdAt", now),
    updatedAt: now,
  };
}

export function characterFromFormData(
  formData: FormData,
  now: string,
): CharacterDefinition {
  return {
    id: text(formData, "id"),
    name: text(formData, "name"),
    avatar: nullableText(formData, "avatar"),
    tags: csv(formData, "tags"),
    persona: text(formData, "persona"),
    speakingStyle: text(formData, "speakingStyle"),
    reasoningStyle: text(formData, "reasoningStyle"),
    systemPrompt: text(formData, "systemPrompt"),
    defaultModelBinding: null,
    enabled: checkbox(formData, "enabled"),
    createdAt: textOrDefault(formData, "createdAt", now),
    updatedAt: now,
  };
}

export function presetFromFormData(formData: FormData, now: string): GamePreset {
  const playerCount = Number.parseInt(text(formData, "playerCount"), 10);
  const seatAssignments: GamePresetSeatAssignment[] = [];

  for (let seatNo = 1; seatNo <= playerCount; seatNo += 1) {
    seatAssignments.push({
      seatNo,
      roleId: text(formData, `seat.${seatNo}.roleId`),
      characterId: text(formData, `seat.${seatNo}.characterId`),
      modelBindingOverride: null,
    });
  }

  return {
    id: text(formData, "id"),
    name: text(formData, "name"),
    rulesetId: text(formData, "rulesetId"),
    playerCount,
    roleIds: seatAssignments.map((assignment) => assignment.roleId),
    characterIds: seatAssignments.map((assignment) => assignment.characterId),
    seatAssignments,
    enabled: checkbox(formData, "enabled"),
    createdAt: textOrDefault(formData, "createdAt", now),
    updatedAt: now,
  };
}

function text(formData: FormData, key: string): string {
  const value = formData.get(key);
  return typeof value === "string" ? value.trim() : "";
}

function textOrDefault(formData: FormData, key: string, defaultValue: string): string {
  return text(formData, key) || defaultValue;
}

function nullableText(formData: FormData, key: string): string | null {
  const value = text(formData, key);
  return value === "" ? null : value;
}

function nullableNumber(formData: FormData, key: string): number | null {
  const value = text(formData, key);
  return value === "" ? null : Number(value);
}

function checkbox(formData: FormData, key: string): boolean {
  const value = formData.get(key);
  return value === "on" || value === "true" || value === "1";
}

function csv(formData: FormData, key: string): readonly string[] {
  return text(formData, key)
    .split(",")
    .map((value) => value.trim())
    .filter(Boolean);
}
```

- [ ] **Step 4: Implement server actions**

Create `src/app/library/actions.ts`:

```ts
"use server";

import { revalidatePath } from "next/cache";
import { redirect } from "next/navigation";
import { createGameRepository } from "@/server/game-repository";
import { createLibraryActions } from "@/server/library-actions";
import { createLibraryRepository } from "@/server/library-repository";
import {
  characterFromFormData,
  presetFromFormData,
  roleFromFormData,
} from "./form-parsers";

const libraryActions = createLibraryActions({
  libraryRepository: createLibraryRepository(process.env.KIVA_DATA_DIR),
  gameRepository: createGameRepository(),
});

export async function saveRoleAction(formData: FormData) {
  await libraryActions.saveRole(roleFromFormData(formData, now()));
  revalidatePath("/library");
}

export async function saveCharacterAction(formData: FormData) {
  await libraryActions.saveCharacter(characterFromFormData(formData, now()));
  revalidatePath("/library");
}

export async function savePresetAction(formData: FormData) {
  await libraryActions.savePreset(presetFromFormData(formData, now()));
  revalidatePath("/library");
}

export async function duplicateRoleAction(roleId: string) {
  await libraryActions.duplicateRole(roleId);
  revalidatePath("/library");
}

export async function duplicateCharacterAction(characterId: string) {
  await libraryActions.duplicateCharacter(characterId);
  revalidatePath("/library");
}

export async function duplicatePresetAction(presetId: string) {
  await libraryActions.duplicatePreset(presetId);
  revalidatePath("/library");
}

export async function setRoleEnabledAction(roleId: string, enabled: boolean) {
  await libraryActions.setRoleEnabled(roleId, enabled);
  revalidatePath("/library");
}

export async function setCharacterEnabledAction(characterId: string, enabled: boolean) {
  await libraryActions.setCharacterEnabled(characterId, enabled);
  revalidatePath("/library");
}

export async function setPresetEnabledAction(presetId: string, enabled: boolean) {
  await libraryActions.setPresetEnabled(presetId, enabled);
  revalidatePath("/library");
}

export async function createGameFromPresetAction(presetId: string) {
  const record = await libraryActions.createGameFromPreset(presetId);
  redirect(`/games/${record.game.id}/editor`);
}

function now(): string {
  return new Date().toISOString();
}
```

Update `src/app/actions.ts` so `createGameAction` no longer creates the default game. Replace its body with:

```ts
export async function createGameAction() {
  redirect("/library?tab=presets");
}
```

- [ ] **Step 5: Run tests**

Run:

```bash
pnpm vitest run src/app/library/form-parsers.test.ts
pnpm typecheck
```

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add src/app/library/actions.ts src/app/library/form-parsers.ts src/app/library/form-parsers.test.ts src/app/actions.ts
git commit -m "feat: add library form actions"
```

---

### Task 4: Library Workspace Shell

**Files:**
- Create: `src/app/library/page.tsx`
- Create: `src/components/library/library-workspace.tsx`
- Create: `src/components/library/library-list.tsx`
- Test: `src/components/library/library-workspace.test.tsx`

- [ ] **Step 1: Write failing render test**

Create `src/components/library/library-workspace.test.tsx`:

```ts
import React from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { LibraryWorkspace } from "./library-workspace";
import { seedCharacters } from "@/seeds/characters";
import { seedPresets } from "@/seeds/presets";
import { seedRoles } from "@/seeds/roles";

describe("LibraryWorkspace", () => {
  it("renders tabs, object list, detail region, and validation region", () => {
    const html = renderToStaticMarkup(
      React.createElement(LibraryWorkspace, {
        activeTab: "roles",
        selectedId: "werewolf",
        library: {
          roles: seedRoles,
          characters: seedCharacters,
          presets: seedPresets,
          diagnostics: { roles: [], characters: [], presets: [] },
        },
      }),
    );

    expect(html).toContain("Kiva Library");
    expect(html).toContain("Roles");
    expect(html).toContain("Characters");
    expect(html).toContain("Presets");
    expect(html).toContain("狼人");
    expect(html).toContain("Validation");
  });
});
```

- [ ] **Step 2: Run test to verify failure**

Run:

```bash
pnpm vitest run src/components/library/library-workspace.test.tsx
```

Expected: FAIL because component does not exist.

- [ ] **Step 3: Implement workspace and list**

Create `src/components/library/library-list.tsx`:

```tsx
import Link from "next/link";

type LibraryListProps = {
  readonly tab: "roles" | "characters" | "presets";
  readonly selectedId: string | null;
  readonly items: readonly { readonly id: string; readonly name: string; readonly enabled: boolean }[];
};

export function LibraryList({ tab, selectedId, items }: LibraryListProps) {
  return (
    <aside className="min-h-0 overflow-y-auto border-r border-zinc-800">
      <div className="border-b border-zinc-800 px-3 py-2 text-xs font-medium uppercase tracking-[0.14em] text-zinc-500">
        {tab}
      </div>
      <div className="divide-y divide-zinc-800">
        {items.map((item) => (
          <Link
            key={item.id}
            href={`/library?tab=${tab}&id=${item.id}`}
            className={
              item.id === selectedId
                ? "block bg-zinc-800/70 px-3 py-3"
                : "block px-3 py-3 hover:bg-zinc-900"
            }
          >
            <div className="truncate text-sm font-medium text-zinc-100">{item.name}</div>
            <div className="mt-1 truncate text-xs text-zinc-500">
              {item.id} · {item.enabled ? "enabled" : "disabled"}
            </div>
          </Link>
        ))}
      </div>
    </aside>
  );
}
```

Create `src/components/library/library-workspace.tsx`:

```tsx
import Link from "next/link";
import type { LibraryViewRecord } from "@/server/library-actions";
import { LibraryList } from "./library-list";
import { ValidationPanel } from "./validation-panel";
import { RoleEditor } from "./role-editor";
import { CharacterEditor } from "./character-editor";
import { PresetEditor } from "./preset-editor";

export type LibraryTab = "roles" | "characters" | "presets";

type LibraryWorkspaceProps = {
  readonly activeTab: LibraryTab;
  readonly selectedId: string | null;
  readonly library: LibraryViewRecord;
};

export function LibraryWorkspace({
  activeTab,
  selectedId,
  library,
}: LibraryWorkspaceProps) {
  const selected = selectItem(library, activeTab, selectedId);

  return (
    <main className="h-screen overflow-hidden bg-zinc-950 p-3 text-zinc-100 sm:p-4">
      <div className="flex h-full min-h-0 flex-col gap-3">
        <header className="flex h-12 shrink-0 items-center justify-between border-b border-zinc-800 pb-3">
          <div className="flex items-center gap-3">
            <Link href="/" className="rounded-md border border-zinc-800 px-3 py-1.5 text-xs text-zinc-400">
              Back
            </Link>
            <h1 className="text-lg font-semibold">Kiva Library</h1>
          </div>
          <Link href="/library?tab=presets" className="rounded-md border border-zinc-700 px-3 py-1.5 text-xs text-zinc-300">
            Create game
          </Link>
        </header>

        <div className="grid min-h-0 flex-1 grid-cols-[150px_260px_minmax(0,1fr)_320px] overflow-hidden rounded-lg border border-zinc-800 bg-zinc-900/45">
          <nav className="border-r border-zinc-800 p-2">
            {(["roles", "characters", "presets"] as const).map((tab) => (
              <Link
                key={tab}
                href={`/library?tab=${tab}`}
                className={tab === activeTab ? "block rounded-md bg-zinc-800 px-3 py-2 text-sm" : "block rounded-md px-3 py-2 text-sm text-zinc-400 hover:bg-zinc-900"}
              >
                {labelForTab(tab)}
              </Link>
            ))}
          </nav>
          <LibraryList
            tab={activeTab}
            selectedId={selected?.id ?? null}
            items={itemsForTab(library, activeTab)}
          />
          <section className="min-h-0 overflow-y-auto p-4">
            {activeTab === "roles" && selected?.kind === "role" ? (
              <RoleEditor role={selected.item} />
            ) : null}
            {activeTab === "characters" && selected?.kind === "character" ? (
              <CharacterEditor character={selected.item} />
            ) : null}
            {activeTab === "presets" && selected?.kind === "preset" ? (
              <PresetEditor
                preset={selected.item}
                roles={library.roles}
                characters={library.characters}
              />
            ) : null}
          </section>
          <ValidationPanel library={library} activeTab={activeTab} selectedId={selected?.id ?? null} />
        </div>
      </div>
    </main>
  );
}

function labelForTab(tab: LibraryTab): string {
  return tab === "roles" ? "Roles" : tab === "characters" ? "Characters" : "Presets";
}

function itemsForTab(library: LibraryViewRecord, tab: LibraryTab) {
  return tab === "roles"
    ? library.roles
    : tab === "characters"
      ? library.characters
      : library.presets;
}

function selectItem(library: LibraryViewRecord, tab: LibraryTab, selectedId: string | null) {
  const items = itemsForTab(library, tab);
  const item = items.find((candidate) => candidate.id === selectedId) ?? items[0];
  if (!item) return null;
  if (tab === "roles") return { kind: "role" as const, id: item.id, item: library.roles.find((role) => role.id === item.id)! };
  if (tab === "characters") return { kind: "character" as const, id: item.id, item: library.characters.find((character) => character.id === item.id)! };
  return { kind: "preset" as const, id: item.id, item: library.presets.find((preset) => preset.id === item.id)! };
}
```

Create `src/app/library/page.tsx`:

```tsx
import { LibraryWorkspace, type LibraryTab } from "@/components/library/library-workspace";
import { createGameRepository } from "@/server/game-repository";
import { createLibraryActions } from "@/server/library-actions";
import { createLibraryRepository } from "@/server/library-repository";

export const dynamic = "force-dynamic";

type LibraryPageProps = {
  readonly searchParams: Promise<{
    readonly tab?: string;
    readonly id?: string;
  }>;
};

export default async function LibraryPage({ searchParams }: LibraryPageProps) {
  const params = await searchParams;
  const activeTab = parseTab(params.tab);
  const library = await createLibraryActions({
    libraryRepository: createLibraryRepository(process.env.KIVA_DATA_DIR),
    gameRepository: createGameRepository(),
  }).getLibrary();

  return (
    <LibraryWorkspace
      activeTab={activeTab}
      selectedId={params.id ?? null}
      library={library}
    />
  );
}

function parseTab(value: string | undefined): LibraryTab {
  return value === "characters" || value === "presets" ? value : "roles";
}
```

Create temporary placeholder components so this task compiles before the real editors are built:

```tsx
// src/components/library/role-editor.tsx
import type { RoleDefinition } from "@/core/role-definition";

export function RoleEditor({ role }: { readonly role: RoleDefinition }) {
  return <div>Role editor: {role.name}</div>;
}
```

```tsx
// src/components/library/character-editor.tsx
import type { CharacterDefinition } from "@/core/character-definition";

export function CharacterEditor({
  character,
}: {
  readonly character: CharacterDefinition;
}) {
  return <div>Character editor: {character.name}</div>;
}
```

```tsx
// src/components/library/preset-editor.tsx
import type { CharacterDefinition } from "@/core/character-definition";
import type { GamePreset } from "@/core/game-preset";
import type { RoleDefinition } from "@/core/role-definition";

export function PresetEditor({
  preset,
}: {
  readonly preset: GamePreset;
  readonly roles: readonly RoleDefinition[];
  readonly characters: readonly CharacterDefinition[];
}) {
  return <div>Preset editor: {preset.name}</div>;
}
```

```tsx
// src/components/library/validation-panel.tsx
import type { LibraryTab } from "./library-workspace";
import type { LibraryViewRecord } from "@/server/library-actions";

export function ValidationPanel({
  activeTab,
}: {
  readonly library: LibraryViewRecord;
  readonly activeTab: LibraryTab;
  readonly selectedId: string | null;
}) {
  return <aside>Validation · {activeTab}</aside>;
}
```

- [ ] **Step 4: Run workspace test and typecheck**

Run:

```bash
pnpm vitest run src/components/library/library-workspace.test.tsx
pnpm typecheck
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/app/library/page.tsx src/components/library/library-workspace.tsx src/components/library/library-list.tsx src/components/library/*.tsx src/components/library/library-workspace.test.tsx
git commit -m "feat: add library workspace shell"
```

---

### Task 5: Role And Character Editors

**Files:**
- Modify: `src/components/library/role-editor.tsx`
- Modify: `src/components/library/character-editor.tsx`
- Create: `src/components/library/dirty-form-guard.tsx`
- Test: `src/components/library/role-editor.test.tsx`
- Test: `src/components/library/character-editor.test.tsx`

- [ ] **Step 1: Write failing editor tests**

Create tests that render forms and assert field boundaries:

```ts
import React from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { RoleEditor } from "./role-editor";
import { CharacterEditor } from "./character-editor";
import { seedRoles } from "@/seeds/roles";
import { seedCharacters } from "@/seeds/characters";

describe("library editors", () => {
  it("renders role mechanics and prompts", () => {
    const html = renderToStaticMarkup(
      React.createElement(RoleEditor, { role: seedRoles[0] }),
    );

    expect(html).toContain('name="mechanicKey"');
    expect(html).toContain('name="systemPrompt"');
    expect(html).toContain('name="actionPrompt"');
    expect(html).toContain("Save role");
  });

  it("does not render role-only fields in character editor", () => {
    const html = renderToStaticMarkup(
      React.createElement(CharacterEditor, { character: seedCharacters[0] }),
    );

    expect(html).toContain('name="persona"');
    expect(html).toContain('name="speakingStyle"');
    expect(html).not.toContain('name="mechanicKey"');
    expect(html).not.toContain('name="faction"');
  });
});
```

- [ ] **Step 2: Run tests to verify failure**

Run:

```bash
pnpm vitest run src/components/library/role-editor.test.tsx src/components/library/character-editor.test.tsx
```

Expected: FAIL until editors are implemented.

- [ ] **Step 3: Implement dirty guard**

Create `src/components/library/dirty-form-guard.tsx`:

```tsx
"use client";

import { useEffect, useState } from "react";

export function DirtyFormGuard() {
  const [dirty, setDirty] = useState(false);

  useEffect(() => {
    function beforeUnload(event: BeforeUnloadEvent) {
      if (!dirty) return;
      event.preventDefault();
      event.returnValue = "";
    }

    window.addEventListener("beforeunload", beforeUnload);
    return () => window.removeEventListener("beforeunload", beforeUnload);
  }, [dirty]);

  return (
    <input
      type="hidden"
      aria-hidden="true"
      onChange={() => setDirty(true)}
      data-dirty-form-guard
    />
  );
}
```

- [ ] **Step 4: Implement editors**

Replace `src/components/library/role-editor.tsx` with:

```tsx
import { saveRoleAction } from "@/app/library/actions";
import type { RoleDefinition } from "@/core/role-definition";
import { DirtyFormGuard } from "./dirty-form-guard";

const BUILT_IN_ROLE_IDS = ["werewolf", "seer", "witch", "villager"] as const;

export function RoleEditor({ role }: { readonly role: RoleDefinition }) {
  const lockedContract = BUILT_IN_ROLE_IDS.includes(
    role.id as (typeof BUILT_IN_ROLE_IDS)[number],
  );

  return (
    <form action={saveRoleAction} className="space-y-4">
      <DirtyFormGuard />
      <input type="hidden" name="createdAt" value={role.createdAt} />
      <FormField label="Id" name="id" defaultValue={role.id} />
      <FormField label="Name" name="name" defaultValue={role.name} />
      <label className="flex items-center gap-2 text-sm text-zinc-300">
        <input type="checkbox" name="enabled" defaultChecked={role.enabled} />
        Enabled
      </label>
      <SelectField
        label="Faction"
        name="faction"
        value={role.faction}
        options={["wolves", "good"]}
        disabled={lockedContract}
      />
      <SelectField
        label="Team"
        name="team"
        value={role.team}
        options={["wolf", "god", "villager"]}
        disabled={lockedContract}
      />
      <SelectField
        label="Mechanic"
        name="mechanicKey"
        value={role.mechanicKey}
        options={["wolf_kill", "seer_check", "witch_medicine", "none"]}
        disabled={lockedContract}
      />
      {lockedContract ? (
        <>
          <input type="hidden" name="faction" value={role.faction} />
          <input type="hidden" name="team" value={role.team} />
          <input type="hidden" name="mechanicKey" value={role.mechanicKey} />
        </>
      ) : null}
      <FormField
        label="Visibility rules"
        name="visibilityRules"
        defaultValue={role.visibilityRules.join(",")}
      />
      <FormField
        label="Night order"
        name="nightOrder"
        defaultValue={role.nightOrder?.toString() ?? ""}
      />
      <TextAreaField label="System prompt" name="systemPrompt" defaultValue={role.systemPrompt} />
      <TextAreaField label="Action prompt" name="actionPrompt" defaultValue={role.actionPrompt ?? ""} />
      <button type="submit" className="rounded-md bg-zinc-100 px-4 py-2 text-sm font-medium text-zinc-950">
        Save role
      </button>
    </form>
  );
}

function FormField(props: {
  readonly label: string;
  readonly name: string;
  readonly defaultValue: string;
}) {
  return (
    <label className="block text-sm text-zinc-300">
      {props.label}
      <input name={props.name} defaultValue={props.defaultValue} className="mt-1 w-full rounded-md border border-zinc-700 bg-zinc-950 px-3 py-2" />
    </label>
  );
}

function SelectField(props: {
  readonly label: string;
  readonly name: string;
  readonly value: string;
  readonly options: readonly string[];
  readonly disabled?: boolean;
}) {
  return (
    <label className="block text-sm text-zinc-300">
      {props.label}
      <select name={props.name} defaultValue={props.value} disabled={props.disabled} className="mt-1 w-full rounded-md border border-zinc-700 bg-zinc-950 px-3 py-2">
        {props.options.map((option) => (
          <option key={option} value={option}>{option}</option>
        ))}
      </select>
    </label>
  );
}

function TextAreaField(props: {
  readonly label: string;
  readonly name: string;
  readonly defaultValue: string;
}) {
  return (
    <label className="block text-sm text-zinc-300">
      {props.label}
      <textarea name={props.name} defaultValue={props.defaultValue} rows={5} className="mt-1 w-full rounded-md border border-zinc-700 bg-zinc-950 px-3 py-2" />
    </label>
  );
}
```

Replace `src/components/library/character-editor.tsx` with:

```tsx
import { saveCharacterAction } from "@/app/library/actions";
import type { CharacterDefinition } from "@/core/character-definition";
import { DirtyFormGuard } from "./dirty-form-guard";

export function CharacterEditor({
  character,
}: {
  readonly character: CharacterDefinition;
}) {
  return (
    <form action={saveCharacterAction} className="space-y-4">
      <DirtyFormGuard />
      <input type="hidden" name="createdAt" value={character.createdAt} />
      <FormField label="Id" name="id" defaultValue={character.id} />
      <FormField label="Name" name="name" defaultValue={character.name} />
      <FormField label="Avatar" name="avatar" defaultValue={character.avatar ?? ""} />
      <FormField label="Tags" name="tags" defaultValue={character.tags.join(",")} />
      <label className="flex items-center gap-2 text-sm text-zinc-300">
        <input type="checkbox" name="enabled" defaultChecked={character.enabled} />
        Enabled
      </label>
      <TextAreaField label="Persona" name="persona" defaultValue={character.persona} />
      <TextAreaField label="Speaking style" name="speakingStyle" defaultValue={character.speakingStyle} />
      <TextAreaField label="Reasoning style" name="reasoningStyle" defaultValue={character.reasoningStyle} />
      <TextAreaField label="System prompt" name="systemPrompt" defaultValue={character.systemPrompt} />
      <button type="submit" className="rounded-md bg-zinc-100 px-4 py-2 text-sm font-medium text-zinc-950">
        Save character
      </button>
    </form>
  );
}

function FormField(props: {
  readonly label: string;
  readonly name: string;
  readonly defaultValue: string;
}) {
  return (
    <label className="block text-sm text-zinc-300">
      {props.label}
      <input name={props.name} defaultValue={props.defaultValue} className="mt-1 w-full rounded-md border border-zinc-700 bg-zinc-950 px-3 py-2" />
    </label>
  );
}

function TextAreaField(props: {
  readonly label: string;
  readonly name: string;
  readonly defaultValue: string;
}) {
  return (
    <label className="block text-sm text-zinc-300">
      {props.label}
      <textarea name={props.name} defaultValue={props.defaultValue} rows={5} className="mt-1 w-full rounded-md border border-zinc-700 bg-zinc-950 px-3 py-2" />
    </label>
  );
}
```

- [ ] **Step 5: Run tests**

Run:

```bash
pnpm vitest run src/components/library/role-editor.test.tsx src/components/library/character-editor.test.tsx
pnpm typecheck
```

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add src/components/library/role-editor.tsx src/components/library/character-editor.tsx src/components/library/dirty-form-guard.tsx src/components/library/*editor.test.tsx
git commit -m "feat: add role and character editors"
```

---

### Task 6: Preset Editor And Create Game Flow

**Files:**
- Modify: `src/components/library/preset-editor.tsx`
- Test: `src/components/library/preset-editor.test.tsx`
- Modify: `src/app/page.tsx`

- [ ] **Step 1: Write failing preset editor test**

Create `src/components/library/preset-editor.test.tsx`:

```ts
import React from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { PresetEditor } from "./preset-editor";
import { seedCharacters } from "@/seeds/characters";
import { seedPresets } from "@/seeds/presets";
import { seedRoles } from "@/seeds/roles";

describe("PresetEditor", () => {
  it("renders a seat table and create-game action", () => {
    const html = renderToStaticMarkup(
      React.createElement(PresetEditor, {
        preset: seedPresets[0],
        roles: seedRoles,
        characters: seedCharacters,
      }),
    );

    expect(html).toContain("Seat 1");
    expect(html).toContain('name="seat.1.roleId"');
    expect(html).toContain('name="seat.1.characterId"');
    expect(html).toContain("Create game");
  });
});
```

- [ ] **Step 2: Run test to verify failure**

Run:

```bash
pnpm vitest run src/components/library/preset-editor.test.tsx
```

Expected: FAIL until preset editor is implemented.

- [ ] **Step 3: Implement preset editor**

Replace `src/components/library/preset-editor.tsx` with:

```tsx
import {
  createGameFromPresetAction,
  savePresetAction,
} from "@/app/library/actions";
import type { CharacterDefinition } from "@/core/character-definition";
import type { GamePreset } from "@/core/game-preset";
import type { RoleDefinition } from "@/core/role-definition";
import { DirtyFormGuard } from "./dirty-form-guard";

export function PresetEditor({
  preset,
  roles,
  characters,
}: {
  readonly preset: GamePreset;
  readonly roles: readonly RoleDefinition[];
  readonly characters: readonly CharacterDefinition[];
}) {
  const seats = preset.seatAssignments ?? [];

  return (
    <div className="space-y-4">
      <form action={savePresetAction} className="space-y-4">
        <DirtyFormGuard />
        <input type="hidden" name="createdAt" value={preset.createdAt} />
        <FormField label="Id" name="id" defaultValue={preset.id} />
        <FormField label="Name" name="name" defaultValue={preset.name} />
        <FormField label="Ruleset" name="rulesetId" defaultValue={preset.rulesetId} />
        <FormField label="Player count" name="playerCount" defaultValue={preset.playerCount.toString()} />
        <label className="flex items-center gap-2 text-sm text-zinc-300">
          <input type="checkbox" name="enabled" defaultChecked={preset.enabled} />
          Enabled
        </label>
        <div className="overflow-hidden rounded-md border border-zinc-800">
          {seats.map((seat) => (
            <div key={seat.seatNo} className="grid grid-cols-[90px_1fr_1fr] gap-3 border-b border-zinc-800 px-3 py-2 last:border-b-0">
              <div className="text-sm text-zinc-400">Seat {seat.seatNo}</div>
              <select name={`seat.${seat.seatNo}.roleId`} defaultValue={seat.roleId} className="rounded-md border border-zinc-700 bg-zinc-950 px-2 py-1">
                {roles.map((role) => (
                  <option key={role.id} value={role.id}>{role.name}</option>
                ))}
              </select>
              <select name={`seat.${seat.seatNo}.characterId`} defaultValue={seat.characterId} className="rounded-md border border-zinc-700 bg-zinc-950 px-2 py-1">
                {characters.map((character) => (
                  <option key={character.id} value={character.id}>{character.name}</option>
                ))}
              </select>
            </div>
          ))}
        </div>
        <button type="submit" className="rounded-md bg-zinc-100 px-4 py-2 text-sm font-medium text-zinc-950">
          Save preset
        </button>
      </form>
      <form action={createGameFromPresetAction.bind(null, preset.id)}>
        <button type="submit" className="rounded-md border border-emerald-700 px-4 py-2 text-sm font-medium text-emerald-200">
          Create game
        </button>
      </form>
    </div>
  );
}

function FormField(props: {
  readonly label: string;
  readonly name: string;
  readonly defaultValue: string;
}) {
  return (
    <label className="block text-sm text-zinc-300">
      {props.label}
      <input name={props.name} defaultValue={props.defaultValue} className="mt-1 w-full rounded-md border border-zinc-700 bg-zinc-950 px-3 py-2" />
    </label>
  );
}
```

- [ ] **Step 4: Update home page navigation**

Modify `src/app/page.tsx`:

- Add a `Library` link in the header.
- Keep `New game` button, but its server action redirects to `/library?tab=presets`.
- Do not create a game directly from the home page.

The header action area should become:

```tsx
<div className="flex flex-col gap-2 sm:flex-row">
  <Link
    href="/library"
    className="rounded-md border border-zinc-700 px-4 py-2 text-sm font-medium text-zinc-200 transition hover:border-zinc-500 hover:text-white"
  >
    Library
  </Link>
  <form action={createGameAction}>
    <button
      type="submit"
      className="w-full rounded-md bg-zinc-100 px-4 py-2 text-sm font-medium text-zinc-950 transition hover:bg-white sm:w-auto"
    >
      New game
    </button>
  </form>
</div>
```

- [ ] **Step 5: Run tests**

Run:

```bash
pnpm vitest run src/components/library/preset-editor.test.tsx
pnpm typecheck
```

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add src/components/library/preset-editor.tsx src/components/library/preset-editor.test.tsx src/app/page.tsx
git commit -m "feat: add preset editor and explicit game creation"
```

---

### Task 7: Validation Panel And Save Error Visibility

**Files:**
- Modify: `src/components/library/validation-panel.tsx`
- Test: `src/components/library/validation-panel.test.tsx`
- Modify: `src/components/library/library-workspace.tsx`

- [ ] **Step 1: Write failing validation panel test**

Create `src/components/library/validation-panel.test.tsx`:

```ts
import React from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { ValidationPanel } from "./validation-panel";
import { seedCharacters } from "@/seeds/characters";
import { seedPresets } from "@/seeds/presets";
import { seedRoles } from "@/seeds/roles";
import { diagnosePreset, diagnoseRole } from "@/core/library-diagnostics";

describe("ValidationPanel", () => {
  it("renders diagnostics and prompt preview", () => {
    const roleDiagnostic = diagnoseRole({
      role: seedRoles[0],
      roles: seedRoles,
      characters: seedCharacters,
      presets: seedPresets,
    });
    const presetDiagnostic = diagnosePreset({
      preset: seedPresets[0],
      roles: seedRoles,
      characters: seedCharacters,
      presets: seedPresets,
    });

    const html = renderToStaticMarkup(
      React.createElement(ValidationPanel, {
        activeTab: "roles",
        selectedId: "werewolf",
        library: {
          roles: seedRoles,
          characters: seedCharacters,
          presets: seedPresets,
          diagnostics: {
            roles: [roleDiagnostic],
            characters: [],
            presets: [presetDiagnostic],
          },
        },
      }),
    );

    expect(html).toContain("Validation");
    expect(html).toContain("Built-in role contract locked");
    expect(html).toContain("Prompt preview");
  });
});
```

- [ ] **Step 2: Run test to verify failure**

Run:

```bash
pnpm vitest run src/components/library/validation-panel.test.tsx
```

Expected: FAIL until panel is implemented.

- [ ] **Step 3: Implement panel**

Replace `src/components/library/validation-panel.tsx` with:

```tsx
import type { LibraryDiagnostic } from "@/core/library-diagnostics";
import type { LibraryViewRecord } from "@/server/library-actions";
import type { LibraryTab } from "./library-workspace";

type ValidationPanelProps = {
  readonly library: LibraryViewRecord;
  readonly activeTab: LibraryTab;
  readonly selectedId: string | null;
};

export function ValidationPanel({
  library,
  activeTab,
  selectedId,
}: ValidationPanelProps) {
  const diagnostic = selectedDiagnostic(library, activeTab, selectedId);

  return (
    <aside className="min-h-0 overflow-y-auto border-l border-zinc-800 p-4">
      <h2 className="text-sm font-semibold text-zinc-100">Validation</h2>
      {!diagnostic ? (
        <p className="mt-3 text-sm text-zinc-500">No item selected.</p>
      ) : (
        <div className="mt-3 space-y-4">
          <span className={diagnostic.valid ? "text-sm text-emerald-300" : "text-sm text-red-300"}>
            {diagnostic.valid ? "Valid" : "Invalid"}
          </span>
          {typeof diagnostic.canCreateGame === "boolean" ? (
            <div className="text-xs text-zinc-400">
              Can create game: {diagnostic.canCreateGame ? "yes" : "no"}
            </div>
          ) : null}
          <PanelBlock title="References">
            {diagnostic.references.length === 0 ? (
              <p className="text-xs text-zinc-500">No references.</p>
            ) : (
              <ul className="space-y-1 text-xs text-zinc-400">
                {diagnostic.references.map((reference) => (
                  <li key={reference}>{reference}</li>
                ))}
              </ul>
            )}
          </PanelBlock>
          <PanelBlock title="Messages">
            {diagnostic.messages.length === 0 ? (
              <p className="text-xs text-zinc-500">No messages.</p>
            ) : (
              <ul className="space-y-1 text-xs text-zinc-400">
                {diagnostic.messages.map((message) => (
                  <li key={message}>{message}</li>
                ))}
              </ul>
            )}
          </PanelBlock>
          <PanelBlock title="Prompt preview">
            <pre className="whitespace-pre-wrap break-words text-xs leading-5 text-zinc-400">
              {diagnostic.promptPreview || "No prompt preview."}
            </pre>
          </PanelBlock>
        </div>
      )}
    </aside>
  );
}

function selectedDiagnostic(
  library: LibraryViewRecord,
  activeTab: LibraryTab,
  selectedId: string | null,
): LibraryDiagnostic | null {
  if (activeTab === "roles") {
    const index = library.roles.findIndex((role) => role.id === selectedId);
    return index >= 0 ? library.diagnostics.roles[index] ?? null : null;
  }
  if (activeTab === "characters") {
    const index = library.characters.findIndex(
      (character) => character.id === selectedId,
    );
    return index >= 0 ? library.diagnostics.characters[index] ?? null : null;
  }
  const index = library.presets.findIndex((preset) => preset.id === selectedId);
  return index >= 0 ? library.diagnostics.presets[index] ?? null : null;
}

function PanelBlock({
  title,
  children,
}: {
  readonly title: string;
  readonly children: React.ReactNode;
}) {
  return (
    <section>
      <h3 className="text-xs font-medium uppercase tracking-[0.14em] text-zinc-500">
        {title}
      </h3>
      <div className="mt-2">{children}</div>
    </section>
  );
}
```

- [ ] **Step 4: Run tests**

Run:

```bash
pnpm vitest run src/components/library/validation-panel.test.tsx src/components/library/library-workspace.test.tsx
pnpm typecheck
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/components/library/validation-panel.tsx src/components/library/validation-panel.test.tsx src/components/library/library-workspace.tsx
git commit -m "feat: add library validation panel"
```

---

### Task 8: Full Regression And Browser Verification

**Files:**
- Modify only if verification finds defects.

- [ ] **Step 1: Run full automated verification**

Run:

```bash
pnpm typecheck
pnpm test
pnpm build
KIVA_DATA_DIR=$(mktemp -d) pnpm seed:library
```

Expected:

- TypeScript passes.
- Vitest passes.
- Next build passes.
- Seed script prints seeded role library counts.

- [ ] **Step 2: Run local app**

Run:

```bash
pnpm dev
```

Open the local URL printed by Next.js.

- [ ] **Step 3: Manual browser checks**

Verify:

- Home page shows `Library`.
- Home page `New game` opens preset selection in `/library?tab=presets`.
- `/library` opens without server errors.
- Switching `Roles`, `Characters`, and `Presets` changes the list and form.
- Editing a role prompt and saving persists after refresh.
- Editing a character prompt and saving persists after refresh.
- Editing a preset seat assignment and saving persists after refresh.
- `Create game` from preset redirects to `/games/:gameId/editor`.
- The editor still auto-generates the next draft.
- Preview still renders.

- [ ] **Step 4: Commit fixes if needed**

If verification required fixes:

```bash
git add <changed-files>
git commit -m "fix: complete role library ui regression"
```

If no fixes were needed, do not create an empty commit.

---

## Self-Review

- Spec coverage: `/library`, tabs, list/detail/validation layout, explicit save, disable/duplicate, no hard delete, prompt previews, preset-based game creation, and validation diagnostics are each covered by tasks.
- Placeholder scan: no `TBD`, `TODO`, or unspecified implementation tasks remain.
- Type consistency: plan uses existing `RoleDefinition`, `CharacterDefinition`, `GamePreset`, `LibraryRepository`, `GameRepository`, and `createGameActions` names consistently.
