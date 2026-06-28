# Character Personality Refresh Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Rewrite the six persisted character definitions so each player has a distinct Werewolf play personality while preserving names.

**Architecture:** This is a data-only change in `kivdb/characters.json`. Keep the existing character schema, preserve `name` and `id` values, and update only descriptive fields plus `updatedAt`.

**Tech Stack:** JSON data file, existing TypeScript character validation.

---

### Task 1: Refresh Character Data

**Files:**
- Modify: `kivdb/characters.json`

- [ ] **Step 1: Rewrite each character record**

Replace the repeated `tags`, `persona`, `speakingStyle`, `reasoningStyle`, `systemPrompt`, and `updatedAt` values with six distinct profiles:

- `qin_chuan`: strong agenda-setter who pushes votes from hard claims.
- `lin_xia`: warm mediator who tracks emotional changes and soft alliances.
- `zhou_zhi`: evidence auditor who builds chronological logic chains.
- `xu_tang`: intuitive pressure player who reads hesitation and overreaction.
- `chen_mo`: quiet observer who speaks late with concise contradictions.
- `shen_lan`: contrarian challenger who tests consensus and exposes weak logic.

- [ ] **Step 2: Validate JSON syntax**

Run:

```bash
node -e 'JSON.parse(require("fs").readFileSync("kivdb/characters.json", "utf8")); console.log("valid json")'
```

Expected: prints `valid json`.

- [ ] **Step 3: Validate character schema**

Run:

```bash
pnpm exec tsx -e 'import { readFileSync } from "node:fs"; import { validateCharacterDefinitions } from "./src/core/character-definition"; const data = JSON.parse(readFileSync("kivdb/characters.json", "utf8")); console.log(validateCharacterDefinitions(data).length);'
```

Expected: prints `6`.
