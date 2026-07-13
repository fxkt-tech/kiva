# 移除历史兼容并收敛最新数据模型

## Goal

Remove historical-data and rollback compatibility now that all persisted game
records will be deleted. Keep one authoritative current contract per workflow
so new development does not carry schema-v1 branches, legacy prompt builders,
or old-record normalization that can no longer be exercised in production.

The cleanup must distinguish historical compatibility from legitimate current
fallback behavior. Provider protocol names, presenter fallback copy, missing
media placeholders, retry behavior, and resources whose latest schema happens
to be version 1 are not obsolete merely because they contain words such as
`compatible`, `fallback`, or `v1`.

## Background

- The just-completed episode authoring flow writes schema v2 but still decodes
  and executes schema v1, carries a legacy input hash, shows a v1 Review UI
  message, and retains v1 tests.
- Prompt generation still exposes an explicit v1 builder mode and keeps
  `prompt-builders-v1.ts`; Local Heuristic understands both v1 and v2 prompt
  schema names.
- Game repository loading still fills missing run mode, Game Script, player
  snapshot fields, ruleset fields, generation metadata, and other fields for
  historical records.
- Video composition currently writes schema v4 but decodes schema v3 by
  injecting the legacy Game Script presentation.
- Several current contracts legitimately use a schema/version value of 1
  because no newer version exists; those are not compatibility branches.
- The working tree contains unrelated Preview/spec and generated framework
  changes. They must not be edited or folded into this task. Persisted game,
  voice, and export data is explicitly in scope for the final deletion step.

## Requirements

- Inventory every code, test, and spec branch whose purpose is reading,
  executing, rendering, or regenerating an obsolete persisted format.
- Remove selected historical branches end-to-end: types, decoders, hashes,
  factory aliases/default injection, UI messages, prompt modes, local test
  handlers, tests, and specs.
- New records and requests must use only the latest authoritative schema and
  prompt path.
- Do not confuse current operational fallbacks with historical compatibility.
- Do not add a migration. This task deletes incompatible persisted
  game/voice/video data after the new code contract has passed its quality
  gate.
- Apply the cleanup repository-wide, not only to episode-script schema v1.
- After the latest-only code passes its quality gate, delete persisted data
  that does not satisfy the resulting contracts. Code changes and validation
  must happen before any destructive data operation.
- Preserve unrelated dirty Preview/spec and generated framework work exactly
  as found.

## Acceptance Criteria

- [x] A repository-wide compatibility inventory classifies every candidate as
      obsolete historical support, current required fallback, or current schema
      whose version number merely equals 1.
- [x] Every approved obsolete branch is removed from production code, tests,
      and project specs with no old-mode selector left reachable.
- [x] Newly created games, episode scripts, prompts, compositions, voice jobs,
      and exports still pass their latest-version workflows.
- [x] Full tests, typecheck, production build, and diff checks pass.
- [x] Only after the code quality gate passes, incompatible persisted game,
      voice, composition/export, and related derived data is deleted from the
      workspace; reusable current library definitions are retained when they
      satisfy the latest schema.

## Product Decision

- “Only the latest version” applies to all historical persisted-data and
  rollback paths in the repository. A compatibility branch does not survive
  merely because it is outside the immediately preceding episode task.
