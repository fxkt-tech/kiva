# 最新版单轨数据契约设计

## 1. 设计结论

Kiva 不再读取、补全、升级或执行任何历史持久化格式。每条工作流只有一个当前写入格式和一个当前读取入口；读取到缺字段、旧 schema 或旧 prompt 记录时直接报错。因为现有对局数据会在代码验收后整体删除，本任务不提供迁移器，也不保留 rollback 开关。

“最新版”针对历史数据兼容和显式回滚路径，不等于删除所有带 `v1`、`fallback` 或 `compatible` 的名称。当前唯一版本恰好为 1 的 Voice Job、Export Job、Presenter Voice Manifest、Edge Voice Adapter 和 Episode Compiler 继续保留；OpenAI-compatible 是供应商协议名称；模型故障切换、主持人口播兜底、缺失头像提示和作业重试是当前运行行为，也继续保留。

## 2. 单轨数据流

```text
Current libraries
  roles + characters + preset + presenter + script
                         │
                         ▼
                GameRecord schema v1
                (all fields required)
                         │
          ┌──────────────┼──────────────┐
          ▼              ▼              ▼
   Prompt v2 only   Episode schema 2   Playback current shape
          │              │              │
          └──────────────┴───────┬──────┘
                                 ▼
                      Composition schema 4
                                 │
                                 ▼
                         Voice / Export jobs
```

Repository reads are validation, not normalization. Factories may still own intentional defaults used to create a new current object, but unknown persisted JSON is never passed through a factory to manufacture missing fields.

## 3. 当前契约

### 3.1 GameRecord

- Add `GAME_RECORD_SCHEMA_VERSION = 1` and require `GameRecord.schemaVersion` on every save/read.
- Require `game`, `events`, `draft`, `generations`, `voiceArtifactsByEventId`, and `episodeScript`; no optional top-level compatibility fields.
- Require an explicit valid `game.runMode`, full current ruleset, presenter snapshot, Game Script snapshot, and Player snapshots.
- A game-mode record must store `episodeScript: null`; a scripted record must store a valid explicit state (`idle`, `generating`, `review`, `approved`, or `failed`). Missing scripted state is invalid rather than becoming `idle`.
- Validate current records on both read and save. Loading never mutates or rewrites the JSON.
- `GenerationRecord.request` is required. Provider token usage may still be `null`; repair `attempts` remains optional because zero-repair output is current behavior.

### 3.2 Player snapshot and rules

- Remove persisted aliases `profileSourceId` and duplicate `systemPrompt`.
- `characterSourceId` and `characterSystemPromptSnapshot` are the only current character-source fields; prompts use structured `persona`, `speakingStyle`, and `reasoningStyle` directly.
- Keep `createPlayerSnapshot` as a new-object factory, but remove legacy alias fallback. Add a strict Player snapshot validator for repository input.
- Remove the explicit six-player compatibility ruleset, `SixPlayerRoleCounts`, and `validateSixPlayerBoard` alias. `createDefaultRuleset` directly owns the current 12-player standard rules.
- Missing persisted run mode is invalid. New-object convenience APIs may choose `game` explicitly, but persistence never infers it.

### 3.3 Episode and Prompt

- `EpisodeScriptSnapshot.schemaVersion` is exactly 2.
- Remove schema-v1 decoding, field injection, legacy input hash, schema-conditional dramaturgy, old Review copy, tests, and Local Heuristic handlers.
- Every current script requires cast directions and current speech-beat direction fields. Game matching always uses the character-driven input hash and always validates dramaturgy.
- Delete `prompt-builders-v1.ts`, `LlmPromptMode`, `KIVA_LLM_PROMPT_VERSION`, and all mode propagation. Speech/action generation always builds Prompt v2.
- Local Heuristic accepts only the current speech/action/episode schema names.
- Remove fallback from structured character data to legacy system-prompt fields. This makes the fixed actor profiles, rather than an old opaque prompt, authoritative.
- LLM details render only `decisionSummary`; old `reasoning`/`reason`/`thought`/`analysis` aliases and null-request UI are removed.

### 3.4 Game Script and Preview/Export

- The only Game Script visual style is `midnight_archive_v1`, which is the current style identifier. Remove `legacy_v1`, `legacyGameScriptSnapshot`, preview-background acceptance, and default-script injection.
- `createCompositionInput` and `createHtmlFrameViewModel` require a Game Script snapshot.
- `VideoCompositionInput` accepts exactly schema 4; schema 3 is rejected without upgrade.
- Current `PlaybackItem` carries explicit `playerVoice`, `presenterVoiceClips`, `presenterSourceId`, and `stage` fields. Absence is an old shape; current “no value” is represented by `null` or an empty array.
- Composition day/night background URLs are required current script assets. Export snapshotting rejects unsupported paths instead of falling back to old Preview backgrounds.
- Remove obsolete tracked Preview day/night background assets and their HTTP allow-list entries. The Preview font remains current and stays in that asset directory.

## 4. 兼容清单

| Candidate | Classification | Decision |
|---|---|---|
| Episode schema 1, legacy hash and UI note | Historical | Remove |
| Prompt v1 builder, env selector and local handlers | Rollback path | Remove |
| Missing Game fields normalized on repository load | Historical | Replace with strict current validation |
| Player `profileSourceId` / duplicate `systemPrompt` | Historical aliases | Remove |
| Explicit six-player ruleset and validator alias | Historical ruleset support | Remove |
| Game Script `legacy_v1` and Preview backgrounds | Historical presentation | Remove |
| Composition schema 3 upgrade and missing stage fields | Historical composition | Remove |
| Null Generation request and reasoning-key aliases | Historical debugging records | Remove |
| Export/Voice/manifest/adapter schema 1 | Current only schema | Keep and validate exactly |
| `midnight_archive_v1` asset/style IDs | Current immutable IDs | Keep |
| OpenAI-compatible provider naming | Current external protocol | Keep |
| Model fallback, retry, interrupted-job recovery | Current operational resilience | Keep |
| Presenter fallback copy and missing-avatar warnings | Current rendering behavior | Keep |
| Game event rollback action | Current user-facing editing feature | Keep |

## 5. 数据删除

Deletion is a final gated operation:

1. finish code changes and focused tests;
2. pass typecheck, full tests, production build, and `git diff --check`;
3. delete every directory under `kivdb/games/`, including tracked records, voice jobs, and audio artifacts;
4. delete `kivdb/exports/` if it exists;
5. verify no game/export records remain and rerun a non-mutating smoke check.

Reusable current libraries and assets under `kivdb/{roles,characters,presets,presenters,scripts}.json`, `kivdb/assets/{characters,presenters,scripts}`, presenter voice manifests, and the Preview font are retained. Worker PID files are operational state rather than incompatible game data and are not deleted by this task.

## 6. Dirty-work isolation

Before each broad edit and final handoff, compare `git status` and preserve the pre-existing unrelated changes in:

- `.trellis/spec/frontend/programmatic-video.md` outside compatibility-specific lines;
- `src/components/preview-v2/preview-v2-studio.tsx`;
- `src/components/preview-v2/stage/html-playback-stage.test.ts`;
- `src/components/preview-v2/stage/stage-event-visual.tsx`;
- `next-env.d.ts`.

The programmatic-video spec itself must be updated where it currently promises old composition/stage compatibility, but unrelated edits in the same file are retained.

## 7. Rollback shape

There is no production compatibility rollback. Before the destructive data gate, code changes remain reversible by normal Git history. After data deletion, restoring old games would require restoring files from Git/backup together with the old code; the new application intentionally refuses them.
