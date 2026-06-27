# Role Library Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the first role-library foundation: separate role definitions from AI character definitions, create games from presets, snapshot library data into each game, and feed those snapshots into LLM prompts without breaking the current 6-player flow.

**Architecture:** Keep runtime game state event-driven. Add repository-backed local JSON libraries for `RoleDefinition`, `CharacterDefinition`, and `GamePreset`, then create immutable `PlayerSnapshot` objects from them at game creation time. Use a small mechanic adapter boundary for current action prompt generation while keeping existing rule functions as the source of truth.

**Tech Stack:** Next.js 16 App Router, TypeScript, Vitest, local JSON repositories under `.kiva-data`, pnpm scripts.

---

## File Structure

- Create `src/core/role-definition.ts`: role library types, validation, seed helpers.
- Create `src/core/character-definition.ts`: AI character library types and validation.
- Create `src/core/game-preset.ts`: preset types, validation, snapshot creation input.
- Create `src/core/library-validation.ts`: shared duplicate-id and reference validation utilities.
- Create `src/seeds/roles.ts`: current werewolf/seer/witch/villager seed definitions.
- Create `src/seeds/characters.ts`: current six AI character seed definitions.
- Create `src/seeds/presets.ts`: `six_player_standard` preset matching the current game.
- Create `src/server/library-repository.ts`: JSON repository for role/character/preset library data.
- Create `src/scripts/seed-library.ts`: imports seed definitions into `.kiva-data`.
- Modify `package.json`: add `seed:roles`, `seed:characters`, `seed:presets`, `seed:library`.
- Modify `src/core/player.ts`: extend `PlayerSnapshot` with source ids and prompt snapshots while keeping old records readable.
- Modify `src/core/game.ts`: add preset-based game creation and keep `createSeedGame` as a compatibility wrapper if needed by tests.
- Modify `src/server/game-actions.ts`: create games from default preset through the library repository.
- Modify `src/server/game-repository.ts`: normalize old player snapshots missing new fields.
- Modify `src/core/player-context.ts`: expose character and role prompt snapshots to prompt builders.
- Modify `src/core/prompt-builders.ts`: compose speech prompts from character + role snapshots.
- Modify `src/core/action-generation.ts`: use a mechanic adapter boundary for current action prompts.
- Modify tests in `src/core/__tests__` and `src/server/__tests__`.

---

### Task 1: Add Role Definition Model

**Files:**
- Create: `src/core/role-definition.ts`
- Create: `src/core/__tests__/role-definition.test.ts`

- [x] **Step 1: Write failing tests for role validation**

Tests must assert the real `RoleDefinition` contract:

- accepts valid unique definitions and returns the original readonly array.
- rejects blank `id`, `name`, and enabled `systemPrompt`.
- rejects duplicate ids.
- rejects ids with leading or trailing whitespace.
- rejects invalid runtime `faction`, `team`, `mechanicKey`, `visibilityRules`, and `nightOrder` values.
- allows disabled definitions without a prompt.

- [x] **Step 2: Run test to verify it fails**

Run:

```bash
pnpm vitest run src/core/__tests__/role-definition.test.ts
```

Expected: FAIL because `src/core/role-definition.ts` does not exist.

- [x] **Step 3: Implement role definition model**

`src/core/role-definition.ts` exports:

- `RoleTeam`: `"wolf" | "god" | "villager"`.
- `RoleMechanicKey`: `"wolf_kill" | "seer_check" | "witch_medicine" | "none"`.
- `RoleKnowledgeRule`: `"own_role" | "wolf_teammates" | "witch_medicines"`.
- `RoleDefinition`: readonly role-library configuration with `id`, `name`, `faction`, `team`, `mechanicKey`, `systemPrompt`, `actionPrompt`, `visibilityRules`, `nightOrder`, `defaultModelBinding`, `enabled`, `createdAt`, and `updatedAt`.
- `validateRoleDefinitions(roles): readonly RoleDefinition[]`.

The validator should reject malformed JSON-boundary values with field-specific errors instead of relying only on TypeScript.

- [x] **Step 4: Run test to verify it passes**

Run:

```bash
pnpm vitest run src/core/__tests__/role-definition.test.ts
pnpm typecheck
pnpm test
```

Expected: PASS.

- [x] **Step 5: Commit**

```bash
git add src/core/role-definition.ts src/core/__tests__/role-definition.test.ts
git commit -m "feat: add role definition model"
```

---

### Task 2: Add Character Definition Model

**Files:**
- Create: `src/core/character-definition.ts`
- Create: `src/core/__tests__/character-definition.test.ts`

- [ ] **Step 1: Write failing tests for character validation**

Create `src/core/__tests__/character-definition.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import {
  validateCharacterDefinitions,
  type CharacterDefinition,
} from "../character-definition";

const character: CharacterDefinition = {
  id: "qin_chuan",
  name: "秦川",
  avatar: null,
  tags: ["default"],
  persona: "冷静、愿意表达判断",
  speakingStyle: "短句、直接、有推进感",
  reasoningStyle: "根据自己可见的信息给出结论",
  systemPrompt: "你是秦川。",
  defaultModelBinding: null,
  enabled: true,
  createdAt: "2026-06-27T00:00:00.000Z",
  updatedAt: "2026-06-27T00:00:00.000Z",
};

describe("character definitions", () => {
  it("accepts valid unique character definitions", () => {
    expect(validateCharacterDefinitions([character])).toEqual([character]);
  });

  it("rejects duplicate character ids", () => {
    expect(() =>
      validateCharacterDefinitions([character, character]),
    ).toThrow("Duplicate character definition id: qin_chuan");
  });

  it("rejects empty enabled character prompts", () => {
    expect(() =>
      validateCharacterDefinitions([{ ...character, systemPrompt: "" }]),
    ).toThrow("Character qin_chuan must include a systemPrompt");
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

Run:

```bash
pnpm test src/core/__tests__/character-definition.test.ts
```

Expected: FAIL because `src/core/character-definition.ts` does not exist.

- [ ] **Step 3: Implement character definition model**

Create `src/core/character-definition.ts`:

```ts
import type { ModelBindingSnapshot } from "./player";

export type CharacterDefinition = {
  readonly id: string;
  readonly name: string;
  readonly avatar: string | null;
  readonly tags: readonly string[];
  readonly persona: string;
  readonly speakingStyle: string;
  readonly reasoningStyle: string;
  readonly systemPrompt: string;
  readonly defaultModelBinding: ModelBindingSnapshot | null;
  readonly enabled: boolean;
  readonly createdAt: string;
  readonly updatedAt: string;
};

export function validateCharacterDefinitions(
  characters: readonly CharacterDefinition[],
): readonly CharacterDefinition[] {
  const seen = new Set<string>();

  for (const character of characters) {
    if (seen.has(character.id)) {
      throw new Error(`Duplicate character definition id: ${character.id}`);
    }
    seen.add(character.id);

    if (character.enabled && character.name.trim().length === 0) {
      throw new Error(`Character ${character.id} must include a name`);
    }

    if (character.enabled && character.systemPrompt.trim().length === 0) {
      throw new Error(`Character ${character.id} must include a systemPrompt`);
    }
  }

  return characters;
}
```

- [ ] **Step 4: Run test to verify it passes**

Run:

```bash
pnpm test src/core/__tests__/character-definition.test.ts
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/core/character-definition.ts src/core/__tests__/character-definition.test.ts
git commit -m "feat: add character definition model"
```

---

### Task 3: Add Game Preset Model and Validation

**Files:**
- Create: `src/core/game-preset.ts`
- Create: `src/core/__tests__/game-preset.test.ts`

- [ ] **Step 1: Write failing tests for preset validation**

Create `src/core/__tests__/game-preset.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import type { CharacterDefinition } from "../character-definition";
import type { RoleDefinition } from "../role-definition";
import {
  validateGamePresets,
  type GamePreset,
} from "../game-preset";

const createdAt = "2026-06-27T00:00:00.000Z";

const role: RoleDefinition = {
  id: "werewolf",
  name: "狼人",
  faction: "wolves",
  team: "wolf",
  mechanicKey: "wolf_kill",
  systemPrompt: "你是狼人。",
  actionPrompt: "夜晚选择击杀目标。",
  visibilityRules: ["own_role", "wolf_teammates"],
  nightOrder: 10,
  defaultModelBinding: null,
  enabled: true,
  createdAt,
  updatedAt: createdAt,
};

const character: CharacterDefinition = {
  id: "qin_chuan",
  name: "秦川",
  avatar: null,
  tags: [],
  persona: "冷静",
  speakingStyle: "直接",
  reasoningStyle: "根据可见信息判断",
  systemPrompt: "你是秦川。",
  defaultModelBinding: null,
  enabled: true,
  createdAt,
  updatedAt: createdAt,
};

const preset: GamePreset = {
  id: "one_player_test",
  name: "测试预设",
  rulesetId: "six_player_slaughter_side",
  playerCount: 1,
  roleIds: ["werewolf"],
  characterIds: ["qin_chuan"],
  seatAssignments: [
    { seatNo: 1, roleId: "werewolf", characterId: "qin_chuan", modelBinding: null },
  ],
  enabled: true,
  createdAt,
  updatedAt: createdAt,
};

describe("game presets", () => {
  it("accepts presets with valid role and character references", () => {
    expect(validateGamePresets([preset], [role], [character])).toEqual([preset]);
  });

  it("rejects missing role references", () => {
    expect(() =>
      validateGamePresets(
        [{ ...preset, roleIds: ["missing"], seatAssignments: [] }],
        [role],
        [character],
      ),
    ).toThrow("Preset one_player_test references missing role: missing");
  });

  it("rejects wrong player counts", () => {
    expect(() =>
      validateGamePresets([{ ...preset, playerCount: 2 }], [role], [character]),
    ).toThrow("Preset one_player_test playerCount must match roleIds length");
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

Run:

```bash
pnpm test src/core/__tests__/game-preset.test.ts
```

Expected: FAIL because `src/core/game-preset.ts` does not exist.

- [ ] **Step 3: Implement preset model**

Create `src/core/game-preset.ts`:

```ts
import type { ModelBindingSnapshot } from "./player";
import type { CharacterDefinition } from "./character-definition";
import type { RoleDefinition } from "./role-definition";

export type PresetSeatAssignment = {
  readonly seatNo: number;
  readonly roleId: string;
  readonly characterId: string;
  readonly modelBinding: ModelBindingSnapshot | null;
};

export type GamePreset = {
  readonly id: string;
  readonly name: string;
  readonly rulesetId: string;
  readonly playerCount: number;
  readonly roleIds: readonly string[];
  readonly characterIds: readonly string[];
  readonly seatAssignments: readonly PresetSeatAssignment[];
  readonly enabled: boolean;
  readonly createdAt: string;
  readonly updatedAt: string;
};

export function validateGamePresets(
  presets: readonly GamePreset[],
  roles: readonly RoleDefinition[],
  characters: readonly CharacterDefinition[],
): readonly GamePreset[] {
  const roleIds = new Set(roles.map((role) => role.id));
  const characterIds = new Set(characters.map((character) => character.id));
  const seen = new Set<string>();

  for (const preset of presets) {
    if (seen.has(preset.id)) {
      throw new Error(`Duplicate preset id: ${preset.id}`);
    }
    seen.add(preset.id);

    if (preset.playerCount !== preset.roleIds.length) {
      throw new Error(`Preset ${preset.id} playerCount must match roleIds length`);
    }

    if (preset.playerCount !== preset.characterIds.length) {
      throw new Error(`Preset ${preset.id} playerCount must match characterIds length`);
    }

    for (const roleId of preset.roleIds) {
      if (!roleIds.has(roleId)) {
        throw new Error(`Preset ${preset.id} references missing role: ${roleId}`);
      }
    }

    for (const characterId of preset.characterIds) {
      if (!characterIds.has(characterId)) {
        throw new Error(
          `Preset ${preset.id} references missing character: ${characterId}`,
        );
      }
    }
  }

  return presets;
}
```

- [ ] **Step 4: Run test to verify it passes**

Run:

```bash
pnpm test src/core/__tests__/game-preset.test.ts
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/core/game-preset.ts src/core/__tests__/game-preset.test.ts
git commit -m "feat: add game preset model"
```

---

### Task 4: Add Seed Definitions

**Files:**
- Create: `src/seeds/roles.ts`
- Create: `src/seeds/characters.ts`
- Create: `src/seeds/presets.ts`
- Create: `src/seeds/__tests__/seed-library.test.ts`

- [ ] **Step 1: Write failing seed tests**

Create `src/seeds/__tests__/seed-library.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import { seedCharacters } from "../characters";
import { seedPresets } from "../presets";
import { seedRoles } from "../roles";
import { validateCharacterDefinitions } from "@/core/character-definition";
import { validateGamePresets } from "@/core/game-preset";
import { validateRoleDefinitions } from "@/core/role-definition";

describe("seed role library", () => {
  it("contains the current four role definitions", () => {
    const roles = validateRoleDefinitions(seedRoles);
    expect(roles.map((role) => role.id).sort()).toEqual([
      "seer",
      "villager",
      "werewolf",
      "witch",
    ]);
  });

  it("contains the current six character definitions", () => {
    const characters = validateCharacterDefinitions(seedCharacters);
    expect(characters.map((character) => character.name)).toEqual([
      "秦川",
      "林夏",
      "周知",
      "许棠",
      "陈墨",
      "沈岚",
    ]);
  });

  it("contains a default 6-player preset matching the current setup", () => {
    const presets = validateGamePresets(seedPresets, seedRoles, seedCharacters);
    expect(presets[0]).toMatchObject({
      id: "six_player_standard",
      playerCount: 6,
      roleIds: ["werewolf", "werewolf", "seer", "witch", "villager", "villager"],
      characterIds: [
        "qin_chuan",
        "lin_xia",
        "zhou_zhi",
        "xu_tang",
        "chen_mo",
        "shen_lan",
      ],
    });
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

Run:

```bash
pnpm test src/seeds/__tests__/seed-library.test.ts
```

Expected: FAIL because seed files do not exist.

- [ ] **Step 3: Create role seeds**

Create `src/seeds/roles.ts` with four roles:

```ts
import type { RoleDefinition } from "@/core/role-definition";

const timestamp = "2026-06-27T00:00:00.000Z";

export const seedRoles = [
  {
    id: "werewolf",
    name: "狼人",
    faction: "wolves",
    team: "wolf",
    mechanicKey: "wolf_kill",
    systemPrompt: "你本局身份是狼人。你知道狼人队友，只能根据狼人可见信息行动。",
    actionPrompt: "夜晚选择一名非狼人玩家作为击杀目标，并给出简短 reasoning。",
    visibilityRules: ["own_role", "wolf_teammates"],
    nightOrder: 10,
    defaultModelBinding: null,
    enabled: true,
    createdAt: timestamp,
    updatedAt: timestamp,
  },
  {
    id: "seer",
    name: "预言家",
    faction: "good",
    team: "god",
    mechanicKey: "seer_check",
    systemPrompt: "你本局身份是预言家。你每晚可以查验一名玩家阵营。",
    actionPrompt: "夜晚选择一名玩家查验阵营，并给出简短 reasoning。",
    visibilityRules: ["own_role"],
    nightOrder: 20,
    defaultModelBinding: null,
    enabled: true,
    createdAt: timestamp,
    updatedAt: timestamp,
  },
  {
    id: "witch",
    name: "女巫",
    faction: "good",
    team: "god",
    mechanicKey: "witch_medicine",
    systemPrompt: "你本局身份是女巫。你拥有解药和毒药，依据夜晚死亡信息行动。",
    actionPrompt: "根据死亡信息和药品状态决定是否用药，并给出简短 reasoning。",
    visibilityRules: ["own_role", "witch_medicines"],
    nightOrder: 30,
    defaultModelBinding: null,
    enabled: true,
    createdAt: timestamp,
    updatedAt: timestamp,
  },
  {
    id: "villager",
    name: "平民",
    faction: "good",
    team: "villager",
    mechanicKey: "none",
    systemPrompt: "你本局身份是平民。你没有夜晚技能，只能通过发言和投票找狼人。",
    actionPrompt: null,
    visibilityRules: ["own_role"],
    nightOrder: null,
    defaultModelBinding: null,
    enabled: true,
    createdAt: timestamp,
    updatedAt: timestamp,
  },
] satisfies readonly RoleDefinition[];
```

- [ ] **Step 4: Create character and preset seeds**

Create `src/seeds/characters.ts`:

```ts
import type { CharacterDefinition } from "@/core/character-definition";

const timestamp = "2026-06-27T00:00:00.000Z";

function character(id: string, name: string): CharacterDefinition {
  return {
    id,
    name,
    avatar: null,
    tags: ["default"],
    persona: "冷静、愿意表达判断",
    speakingStyle: "短句、直接、有推进感",
    reasoningStyle: "根据自己可见的信息给出结论",
    systemPrompt: `你是${name}，一名狼人杀玩家。你的表达冷静直接，应该像真实玩家一样基于可见信息判断。`,
    defaultModelBinding: null,
    enabled: true,
    createdAt: timestamp,
    updatedAt: timestamp,
  };
}

export const seedCharacters = [
  character("qin_chuan", "秦川"),
  character("lin_xia", "林夏"),
  character("zhou_zhi", "周知"),
  character("xu_tang", "许棠"),
  character("chen_mo", "陈墨"),
  character("shen_lan", "沈岚"),
] satisfies readonly CharacterDefinition[];
```

Create `src/seeds/presets.ts`:

```ts
import type { GamePreset } from "@/core/game-preset";

const timestamp = "2026-06-27T00:00:00.000Z";

export const seedPresets = [
  {
    id: "six_player_standard",
    name: "6人标准局",
    rulesetId: "six_player_slaughter_side",
    playerCount: 6,
    roleIds: ["werewolf", "werewolf", "seer", "witch", "villager", "villager"],
    characterIds: [
      "qin_chuan",
      "lin_xia",
      "zhou_zhi",
      "xu_tang",
      "chen_mo",
      "shen_lan",
    ],
    seatAssignments: [
      { seatNo: 1, roleId: "werewolf", characterId: "qin_chuan", modelBinding: null },
      { seatNo: 2, roleId: "werewolf", characterId: "lin_xia", modelBinding: null },
      { seatNo: 3, roleId: "seer", characterId: "zhou_zhi", modelBinding: null },
      { seatNo: 4, roleId: "witch", characterId: "xu_tang", modelBinding: null },
      { seatNo: 5, roleId: "villager", characterId: "chen_mo", modelBinding: null },
      { seatNo: 6, roleId: "villager", characterId: "shen_lan", modelBinding: null },
    ],
    enabled: true,
    createdAt: timestamp,
    updatedAt: timestamp,
  },
] satisfies readonly GamePreset[];
```

- [ ] **Step 5: Run test to verify it passes**

Run:

```bash
pnpm test src/seeds/__tests__/seed-library.test.ts
```

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add src/seeds src/core
git commit -m "feat: add default role library seeds"
```

---

### Task 5: Add Library Repository and Seed Script

**Files:**
- Create: `src/server/library-repository.ts`
- Create: `src/server/__tests__/library-repository.test.ts`
- Create: `src/scripts/seed-library.ts`
- Modify: `package.json`

- [ ] **Step 1: Write failing repository tests**

Create `src/server/__tests__/library-repository.test.ts`:

```ts
import { mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { afterEach, describe, expect, it } from "vitest";
import { seedCharacters } from "@/seeds/characters";
import { seedPresets } from "@/seeds/presets";
import { seedRoles } from "@/seeds/roles";
import { createLibraryRepository } from "../library-repository";

const tempDirs: string[] = [];

afterEach(async () => {
  await Promise.all(
    tempDirs.splice(0).map((dir) => rm(dir, { recursive: true, force: true })),
  );
});

async function createTempDir(): Promise<string> {
  const dir = await mkdtemp(join(tmpdir(), "kiva-library-"));
  tempDirs.push(dir);
  return dir;
}

describe("library repository", () => {
  it("saves and loads role, character, and preset libraries", async () => {
    const repository = createLibraryRepository(await createTempDir());

    await repository.saveAll({
      roles: seedRoles,
      characters: seedCharacters,
      presets: seedPresets,
    });

    await expect(repository.loadAll()).resolves.toMatchObject({
      roles: [{ id: "werewolf" }],
      characters: [{ id: "qin_chuan" }],
      presets: [{ id: "six_player_standard" }],
    });
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

Run:

```bash
pnpm test src/server/__tests__/library-repository.test.ts
```

Expected: FAIL because repository does not exist.

- [ ] **Step 3: Implement repository**

Create `src/server/library-repository.ts`:

```ts
import { mkdir, readFile, writeFile } from "node:fs/promises";
import { join } from "node:path";
import type { CharacterDefinition } from "@/core/character-definition";
import type { GamePreset } from "@/core/game-preset";
import type { RoleDefinition } from "@/core/role-definition";

export type LibraryData = {
  readonly roles: readonly RoleDefinition[];
  readonly characters: readonly CharacterDefinition[];
  readonly presets: readonly GamePreset[];
};

export type LibraryRepository = ReturnType<typeof createLibraryRepository>;

const defaultDataDir = ".kiva-data";

export function createLibraryRepository(dataDir = defaultDataDir) {
  return {
    async loadAll(): Promise<LibraryData> {
      return {
        roles: await readJson<readonly RoleDefinition[]>(dataDir, "roles.json", []),
        characters: await readJson<readonly CharacterDefinition[]>(
          dataDir,
          "characters.json",
          [],
        ),
        presets: await readJson<readonly GamePreset[]>(dataDir, "presets.json", []),
      };
    },

    async saveAll(data: LibraryData): Promise<void> {
      await mkdir(dataDir, { recursive: true });
      await Promise.all([
        writeJson(dataDir, "roles.json", data.roles),
        writeJson(dataDir, "characters.json", data.characters),
        writeJson(dataDir, "presets.json", data.presets),
      ]);
    },
  };
}

async function readJson<T>(dir: string, file: string, fallback: T): Promise<T> {
  try {
    return JSON.parse(await readFile(join(dir, file), "utf8")) as T;
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code === "ENOENT") {
      return fallback;
    }
    throw error;
  }
}

async function writeJson(dir: string, file: string, value: unknown): Promise<void> {
  await writeFile(join(dir, file), `${JSON.stringify(value, null, 2)}\n`, "utf8");
}
```

- [ ] **Step 4: Add seed script and package scripts**

Create `src/scripts/seed-library.ts`:

```ts
import { seedCharacters } from "@/seeds/characters";
import { seedPresets } from "@/seeds/presets";
import { seedRoles } from "@/seeds/roles";
import { createLibraryRepository } from "@/server/library-repository";

await createLibraryRepository().saveAll({
  roles: seedRoles,
  characters: seedCharacters,
  presets: seedPresets,
});

console.log(
  `Seeded role library: ${seedRoles.length} roles, ${seedCharacters.length} characters, ${seedPresets.length} presets`,
);
```

Modify `package.json` scripts:

```json
{
  "seed:library": "tsx src/scripts/seed-library.ts",
  "seed:roles": "tsx src/scripts/seed-library.ts",
  "seed:characters": "tsx src/scripts/seed-library.ts",
  "seed:presets": "tsx src/scripts/seed-library.ts"
}
```

If `tsx` is not installed, add it:

```bash
pnpm add -D tsx
```

- [ ] **Step 5: Run tests and script**

Run:

```bash
pnpm test src/server/__tests__/library-repository.test.ts
pnpm seed:library
```

Expected: test PASS and script prints `Seeded role library: 4 roles, 6 characters, 1 presets`.

- [ ] **Step 6: Commit**

```bash
git add package.json pnpm-lock.yaml src/server/library-repository.ts src/server/__tests__/library-repository.test.ts src/scripts/seed-library.ts
git commit -m "feat: add local role library repository"
```

---

### Task 6: Extend PlayerSnapshot and Normalize Old Games

**Files:**
- Modify: `src/core/player.ts`
- Modify: `src/server/game-repository.ts`
- Modify: `src/core/__tests__/player.test.ts`
- Modify: `src/server/__tests__/game-repository.test.ts`

- [ ] **Step 1: Write failing tests for new snapshot fields and old normalization**

Add to `src/core/__tests__/player.test.ts`:

```ts
it("stores role and character source snapshots", () => {
  const snapshot = createPlayerSnapshot({
    playerId: "p1" as PlayerId,
    seatNo: 1,
    name: "秦川",
    gameRole: "werewolf",
    characterSourceId: "qin_chuan",
    roleSourceId: "werewolf",
    roleName: "狼人",
    team: "wolf",
    mechanicKey: "wolf_kill",
    characterSystemPromptSnapshot: "你是秦川。",
    roleSystemPromptSnapshot: "你是狼人。",
    roleActionPromptSnapshot: "夜晚刀人。",
  });

  expect(snapshot).toMatchObject({
    characterSourceId: "qin_chuan",
    roleSourceId: "werewolf",
    roleName: "狼人",
    team: "wolf",
    mechanicKey: "wolf_kill",
    characterSystemPromptSnapshot: "你是秦川。",
    roleSystemPromptSnapshot: "你是狼人。",
    roleActionPromptSnapshot: "夜晚刀人。",
  });
});
```

Add to `src/server/__tests__/game-repository.test.ts`:

```ts
it("normalizes old player snapshots without role-library fields", async () => {
  const repository = createGameRepository(await createTempDir());
  const created = createSeedGame({
    gameId,
    createdAt: "2026-06-26T00:00:00.000Z",
  });

  const rawRecord = {
    game: {
      ...created,
      players: created.players.map(
        ({
          characterSourceId,
          roleSourceId,
          roleName,
          team,
          mechanicKey,
          characterSystemPromptSnapshot,
          roleSystemPromptSnapshot,
          roleActionPromptSnapshot,
          ...player
        }) => player,
      ),
    },
    events: [],
    draft: null,
    generations: [],
  };

  await repository.save(rawRecord as typeof rawRecord & { game: typeof created });
  const loaded = await repository.get(gameId);

  expect(loaded?.game.players[0]).toMatchObject({
    characterSourceId: null,
    roleSourceId: "werewolf",
    roleName: "狼人",
    mechanicKey: "wolf_kill",
  });
});
```

- [ ] **Step 2: Run tests to verify they fail**

Run:

```bash
pnpm test src/core/__tests__/player.test.ts src/server/__tests__/game-repository.test.ts
```

Expected: FAIL because fields do not exist or normalization is missing.

- [ ] **Step 3: Extend player types and defaults**

Modify `src/core/player.ts`:

```ts
export type PlayerSnapshot = {
  readonly playerId: PlayerId;
  readonly seatNo: number;
  readonly name: string;
  readonly characterSourceId: string | null;
  readonly roleSourceId: string;
  readonly avatar: string | null;
  readonly persona: string;
  readonly speakingStyle: string;
  readonly reasoningStyle: string;
  readonly characterSystemPromptSnapshot: string;
  readonly roleSystemPromptSnapshot: string;
  readonly roleActionPromptSnapshot: string | null;
  readonly systemPrompt: string;
  readonly modelBindingSnapshot: ModelBindingSnapshot;
  readonly gameRole: GameRole;
  readonly roleName: string;
  readonly faction: Faction;
  readonly team: "wolf" | "god" | "villager";
  readonly mechanicKey: "wolf_kill" | "seer_check" | "witch_medicine" | "none";
  readonly initialPrivateKnowledge: readonly PrivateKnowledge[];
};
```

Update `CreatePlayerSnapshotInput` with optional versions of those fields. In `createPlayerSnapshot`, default missing values using helper functions:

```ts
roleSourceId: input.roleSourceId ?? input.gameRole,
characterSourceId: input.characterSourceId ?? null,
avatar: input.avatar ?? null,
roleName: input.roleName ?? roleLabel(input.gameRole),
team: input.team ?? teamForRole(input.gameRole),
mechanicKey: input.mechanicKey ?? mechanicKeyForRole(input.gameRole),
characterSystemPromptSnapshot: input.characterSystemPromptSnapshot ?? input.systemPrompt ?? "",
roleSystemPromptSnapshot: input.roleSystemPromptSnapshot ?? "",
roleActionPromptSnapshot: input.roleActionPromptSnapshot ?? null,
systemPrompt: input.systemPrompt ?? "",
```

Add helpers:

```ts
function roleLabel(role: GameRole): string {
  return {
    werewolf: "狼人",
    seer: "预言家",
    witch: "女巫",
    villager: "平民",
  }[role];
}

function teamForRole(role: GameRole) {
  if (role === "werewolf") return "wolf";
  if (role === "villager") return "villager";
  return "god";
}

function mechanicKeyForRole(role: GameRole) {
  return {
    werewolf: "wolf_kill",
    seer: "seer_check",
    witch: "witch_medicine",
    villager: "none",
  }[role];
}
```

- [ ] **Step 4: Normalize old records in game repository**

Modify `src/server/game-repository.ts` normalization so each loaded player is passed through `createPlayerSnapshot(player)`.

Expected pattern:

```ts
const normalizedGame = {
  ...record.game,
  players: record.game.players.map((player) => createPlayerSnapshot(player)),
};
```

Import `createPlayerSnapshot`.

- [ ] **Step 5: Run tests to verify they pass**

Run:

```bash
pnpm test src/core/__tests__/player.test.ts src/server/__tests__/game-repository.test.ts
```

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add src/core/player.ts src/core/__tests__/player.test.ts src/server/game-repository.ts src/server/__tests__/game-repository.test.ts
git commit -m "feat: snapshot role library fields on players"
```

---

### Task 7: Create Games from Preset Snapshots

**Files:**
- Modify: `src/core/game.ts`
- Modify: `src/core/__tests__/game.test.ts` or create `src/core/__tests__/game-creation.test.ts`
- Modify: `src/server/game-actions.ts`
- Modify: `src/server/__tests__/game-actions.test.ts`

- [ ] **Step 1: Write failing game creation tests**

Create `src/core/__tests__/game-creation.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import { createGameFromPreset } from "../game";
import { seedCharacters } from "@/seeds/characters";
import { seedPresets } from "@/seeds/presets";
import { seedRoles } from "@/seeds/roles";
import { createDefaultRuleset } from "../types";

describe("preset game creation", () => {
  it("creates player snapshots from the default preset", () => {
    const game = createGameFromPreset({
      gameId: "game_1" as never,
      title: "6人狼人杀试运行",
      createdAt: "2026-06-27T00:00:00.000Z",
      ruleset: createDefaultRuleset(),
      preset: seedPresets[0],
      roles: seedRoles,
      characters: seedCharacters,
    });

    expect(game.players.map((player) => [player.seatNo, player.name, player.gameRole])).toEqual([
      [1, "秦川", "werewolf"],
      [2, "林夏", "werewolf"],
      [3, "周知", "seer"],
      [4, "许棠", "witch"],
      [5, "陈墨", "villager"],
      [6, "沈岚", "villager"],
    ]);
    expect(game.players[0]).toMatchObject({
      characterSourceId: "qin_chuan",
      roleSourceId: "werewolf",
      roleSystemPromptSnapshot: expect.stringContaining("狼人"),
      characterSystemPromptSnapshot: expect.stringContaining("秦川"),
      mechanicKey: "wolf_kill",
    });
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

Run:

```bash
pnpm test src/core/__tests__/game-creation.test.ts
```

Expected: FAIL because `createGameFromPreset` does not exist.

- [ ] **Step 3: Implement `createGameFromPreset`**

Modify `src/core/game.ts`:

```ts
export function createGameFromPreset(input: {
  readonly gameId: GameId;
  readonly title: string;
  readonly createdAt: string;
  readonly ruleset: Ruleset;
  readonly preset: GamePreset;
  readonly roles: readonly RoleDefinition[];
  readonly characters: readonly CharacterDefinition[];
}): Game {
  const roleById = new Map(input.roles.map((role) => [role.id, role]));
  const characterById = new Map(
    input.characters.map((character) => [character.id, character]),
  );

  return {
    id: input.gameId,
    title: input.title,
    status: "drafting",
    ruleset: input.ruleset,
    players: input.preset.seatAssignments.map((seat) => {
      const role = roleById.get(seat.roleId);
      const character = characterById.get(seat.characterId);
      if (!role) throw new Error(`Missing role for preset seat: ${seat.roleId}`);
      if (!character) throw new Error(`Missing character for preset seat: ${seat.characterId}`);

      return createPlayerSnapshot({
        playerId: `${input.gameId}_p${seat.seatNo}` as PlayerId,
        seatNo: seat.seatNo,
        name: character.name,
        characterSourceId: character.id,
        roleSourceId: role.id,
        avatar: character.avatar,
        persona: character.persona,
        speakingStyle: character.speakingStyle,
        reasoningStyle: character.reasoningStyle,
        characterSystemPromptSnapshot: character.systemPrompt,
        roleSystemPromptSnapshot: role.systemPrompt,
        roleActionPromptSnapshot: role.actionPrompt,
        systemPrompt: [character.systemPrompt, role.systemPrompt].join("\n"),
        modelBindingSnapshot:
          seat.modelBinding ??
          character.defaultModelBinding ??
          role.defaultModelBinding ??
          undefined,
        gameRole: role.id as GameRole,
        roleName: role.name,
        faction: role.faction,
        team: role.team,
        mechanicKey: role.mechanicKey,
      });
    }),
    createdAt: input.createdAt,
    updatedAt: input.createdAt,
  };
}
```

Keep `createSeedGame` by calling `createGameFromPreset` with seed data. This minimizes test churn.

- [ ] **Step 4: Wire server game creation to library repository**

Modify `src/server/game-actions.ts`:

```ts
import { seedCharacters } from "@/seeds/characters";
import { seedPresets } from "@/seeds/presets";
import { seedRoles } from "@/seeds/roles";
import { createLibraryRepository, type LibraryRepository } from "./library-repository";
```

Extend options:

```ts
export type CreateGameActionsOptions = {
  readonly llmClient?: LlmClient;
  readonly libraryRepository?: LibraryRepository;
  readonly defaultPresetId?: string;
};
```

In `createGame`, load libraries:

```ts
const library = options.libraryRepository
  ? await options.libraryRepository.loadAll()
  : { roles: seedRoles, characters: seedCharacters, presets: seedPresets };
const presetId = options.defaultPresetId ?? "six_player_standard";
const preset = library.presets.find((candidate) => candidate.id === presetId);
if (!preset) throw new Error(`Game preset not found: ${presetId}`);
const game = createGameFromPreset({
  gameId: createGameId(),
  title: "6人狼人杀试运行",
  createdAt,
  ruleset: createDefaultRuleset(),
  preset,
  roles: library.roles,
  characters: library.characters,
});
```

- [ ] **Step 5: Run relevant tests**

Run:

```bash
pnpm test src/core/__tests__/game-creation.test.ts src/server/__tests__/game-actions.test.ts
```

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add src/core/game.ts src/core/__tests__/game-creation.test.ts src/server/game-actions.ts src/server/__tests__/game-actions.test.ts
git commit -m "feat: create games from role library presets"
```

---

### Task 8: Compose LLM Prompts from Character and Role Snapshots

**Files:**
- Modify: `src/core/player-context.ts`
- Modify: `src/core/prompt-builders.ts`
- Modify: `src/core/action-generation.ts`
- Modify: `src/core/__tests__/prompt-builders.test.ts`
- Modify: `src/core/__tests__/action-generation.test.ts`

- [ ] **Step 1: Write failing prompt tests**

Add to `src/core/__tests__/prompt-builders.test.ts`:

```ts
it("includes character and role prompt snapshots in speech prompts", () => {
  const game = createSeedGame({ gameId, createdAt });
  const player = game.players[2];
  const context = buildPlayerLlmContext({
    game: {
      ...game,
      players: game.players.map((candidate) =>
        candidate.playerId === player.playerId
          ? {
              ...candidate,
              characterSystemPromptSnapshot: "角色人格提示：周知很直接。",
              roleSystemPromptSnapshot: "身份提示：你是预言家。",
            }
          : candidate,
      ),
    },
    events: [],
    viewerPlayerId: player.playerId,
  });

  const prompt = buildSpeechPrompt({
    context,
    draft: daySpeechDraft(player.playerId),
  });

  expect(prompt.systemPrompt).toContain("角色人格提示：周知很直接。");
  expect(prompt.systemPrompt).toContain("身份提示：你是预言家。");
});
```

Add to `src/core/__tests__/action-generation.test.ts`:

```ts
expect(result.generation?.request?.systemPrompt).toContain("你本局身份");
```

- [ ] **Step 2: Run tests to verify they fail**

Run:

```bash
pnpm test src/core/__tests__/prompt-builders.test.ts src/core/__tests__/action-generation.test.ts
```

Expected: FAIL because prompt builders still rely mostly on `viewer.systemPrompt`.

- [ ] **Step 3: Extend player context viewer**

Modify `src/core/player-context.ts` viewer shape:

```ts
readonly characterSystemPromptSnapshot: string;
readonly roleSystemPromptSnapshot: string;
readonly roleActionPromptSnapshot: string | null;
readonly roleName: string;
readonly mechanicKey: PlayerSnapshot["mechanicKey"];
```

Populate from `viewer`.

- [ ] **Step 4: Update speech prompt composition**

Modify `src/core/prompt-builders.ts` system prompt:

```ts
systemPrompt: [
  input.context.viewer.characterSystemPromptSnapshot,
  input.context.viewer.roleSystemPromptSnapshot,
  input.context.viewer.systemPrompt,
  "你正在参与一局狼人杀内容创作。",
  "你只能依据用户消息中列出的可见信息发言。",
  "禁止引用、暗示或利用未出现在可见信息中的上帝视角事实。",
  '必须输出 JSON 对象，格式为 {"text":"你的发言","reasoning":"简短说明你为什么这样发言"}，不要输出 Markdown。',
].filter(Boolean).join("\n"),
```

- [ ] **Step 5: Update action prompt composition**

Modify `src/core/action-generation.ts` system prompt:

```ts
systemPrompt: [
  context.viewer.characterSystemPromptSnapshot,
  context.viewer.roleSystemPromptSnapshot,
  context.viewer.roleActionPromptSnapshot,
  context.viewer.systemPrompt,
  "你只能根据可见信息给出狼人杀行动建议。",
  "必须输出 JSON 对象，不要输出 Markdown。",
].filter(Boolean).join("\n"),
```

- [ ] **Step 6: Run tests**

Run:

```bash
pnpm test src/core/__tests__/prompt-builders.test.ts src/core/__tests__/action-generation.test.ts
```

Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add src/core/player-context.ts src/core/prompt-builders.ts src/core/action-generation.ts src/core/__tests__/prompt-builders.test.ts src/core/__tests__/action-generation.test.ts
git commit -m "feat: compose prompts from role library snapshots"
```

---

### Task 9: Add Mechanic Adapter Boundary

**Files:**
- Create: `src/core/role-mechanics.ts`
- Create: `src/core/__tests__/role-mechanics.test.ts`
- Modify: `src/core/action-generation.ts`

- [ ] **Step 1: Write failing mechanic adapter tests**

Create `src/core/__tests__/role-mechanics.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import { mechanicForDraftType } from "../role-mechanics";

describe("role mechanics", () => {
  it("maps current action drafts to mechanic keys", () => {
    expect(mechanicForDraftType("wolf_kill_selected")).toBe("wolf_kill");
    expect(mechanicForDraftType("seer_check_selected")).toBe("seer_check");
    expect(mechanicForDraftType("witch_antidote_decided")).toBe("witch_medicine");
    expect(mechanicForDraftType("witch_poison_decided")).toBe("witch_medicine");
    expect(mechanicForDraftType("vote_cast")).toBe("none");
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

Run:

```bash
pnpm test src/core/__tests__/role-mechanics.test.ts
```

Expected: FAIL because `role-mechanics.ts` does not exist.

- [ ] **Step 3: Implement adapter boundary**

Create `src/core/role-mechanics.ts`:

```ts
import type { DraftEvent } from "./drafts";
import type { RoleMechanicKey } from "./role-definition";

export type ActionDraftType = Extract<
  DraftEvent,
  | { type: "seer_check_selected" }
  | { type: "wolf_kill_selected" }
  | { type: "vote_cast" }
  | { type: "witch_antidote_decided" }
  | { type: "witch_poison_decided" }
>["type"];

export function mechanicForDraftType(type: ActionDraftType): RoleMechanicKey {
  switch (type) {
    case "wolf_kill_selected":
      return "wolf_kill";
    case "seer_check_selected":
      return "seer_check";
    case "witch_antidote_decided":
    case "witch_poison_decided":
      return "witch_medicine";
    case "vote_cast":
      return "none";
  }
}
```

- [ ] **Step 4: Use mechanic boundary in action generation**

Modify `src/core/action-generation.ts` to import `mechanicForDraftType` and include the mechanic in prompt:

```ts
const mechanicKey = mechanicForDraftType(input.draft.type);
...
`mechanic=${mechanicKey}`,
```

Keep current `legalTargetIdsForDraft` and `parseAndValidateActionEdit` unchanged for this phase. This task creates the adapter boundary without changing behavior.

- [ ] **Step 5: Run tests**

Run:

```bash
pnpm test src/core/__tests__/role-mechanics.test.ts src/core/__tests__/action-generation.test.ts
```

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add src/core/role-mechanics.ts src/core/__tests__/role-mechanics.test.ts src/core/action-generation.ts
git commit -m "feat: add role mechanic adapter boundary"
```

---

### Task 10: Full Regression and Local Data Compatibility

**Files:**
- Modify only files required by failing tests.

- [ ] **Step 1: Run full test suite**

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

Expected: no TypeScript errors.

- [ ] **Step 3: Run production build**

Run:

```bash
pnpm build
```

Expected: build succeeds and routes include:

```text
/games/[gameId]/editor
/games/[gameId]/preview
```

- [ ] **Step 4: Seed local library data**

Run:

```bash
pnpm seed:library
```

Expected:

```text
Seeded role library: 4 roles, 6 characters, 1 presets
```

- [ ] **Step 5: Smoke editor route**

Run:

```bash
curl -I --noproxy '*' http://127.0.0.1:9090/games/game_a8b96591-49cf-4f5a-b969-4b029fabba96/editor
```

Expected: `HTTP/1.1 200 OK`.

- [ ] **Step 6: Commit final fixes if any**

If any files changed during regression, stage the concrete changed files shown by `git status --short`. For example, if regression only required player normalization fixes:

```bash
git add src/core/player.ts src/server/game-repository.ts
git commit -m "fix: complete role library regression"
```

If no files changed, do not create an empty commit.

---

## Self-Review

Spec coverage:

- Role definitions: Task 1 and Task 4.
- Character definitions: Task 2 and Task 4.
- Game presets: Task 3 and Task 4.
- JSON library storage and scripts: Task 5.
- Preset-based game creation: Task 7.
- Player snapshots and old game compatibility: Task 6.
- LLM prompt composition: Task 8.
- Mechanic adapter boundary: Task 9.
- Full regression: Task 10.

Scope check:

- No UI management is included.
- No new roles are added.
- No replay/recording work is included.
- No remote database migration is included.

Type consistency:

- `RoleDefinition`, `CharacterDefinition`, `GamePreset`, and `PlayerSnapshot` fields are named consistently across tasks.
- `mechanicKey` values are limited to current mechanics: `wolf_kill`, `seer_check`, `witch_medicine`, `none`.
- Model binding priority is implemented at snapshot creation, not during prompt generation.
