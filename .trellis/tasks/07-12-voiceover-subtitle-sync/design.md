# 配音与字幕同步：技术设计

## 1. Design principles

1. 音频事实驱动时间线：音频、字幕和镜头不各自估时。
2. Game 聚合自身产物：结构化记录与二进制 voice 文件共享生命周期。
3. TTS 是 adapter：领域层不泄漏 Edge 包格式。
4. 成功产物不可变：可复现性优先于重新生成便利。
5. Preview 与导出同源：只编译一次确定性 composition input。

## 2. Storage layout

废弃 `kivdb/games/<gameId>.json`，新格式为：

```text
kivdb/games/<gameId>/
├── record.json
├── voice/
│   └── <eventId>.mp3
└── voice-jobs/
    └── <jobId>.json
```

主理人属于 Library，不属于 Game：

```text
kivdb/presenters/<presenterId>/
├── manifest.json
└── voice/
    └── <clipId>.mp3
```

`GameRepository` 只暴露 Game 聚合操作，内部负责 record path、voice path、原子 JSON、临时文件和递归删除。实现不保留旧单文件读取分支。

## 3. Domain contracts

### 3.1 Voice configuration snapshot

```ts
type VoiceProfileSnapshot = {
  provider: "edge";
  adapterVersion: 1;
  voice: string;
  lang: string;
  pitch: string;
  rate: string;
  volume: string;
};
```

`CharacterDefinition` 持有配置；`PlayerSnapshot` 在创建 Game 时复制。Presenter Definition 持有独立配置，但 Game Presenter 不复制预制音频，只引用带版本的 presenter manifest。

MVP 的 voice allowlist 固定为三种已实测可用值：Presenter 使用
`zh-CN-YunjianNeural`，男性 Player 使用 `zh-CN-YunxiNeural`，女性
Player 使用 `zh-CN-XiaoxiaoNeural`。所有 profile 固定 `rate: "+20%"`
（1.2 倍速），同类角色只通过 pitch/volume 保留轻微表达差异。更换 profile 后，Presenter 构建不得复用旧 profile
生成的 clip；已有 Game 的旧玩家音频需要整体清空后重新生成。

### 3.2 Player voice artifact

```ts
type SpeechCue = {
  text: string;
  startMs: number;
  endMs: number;
};

type PlayerVoiceArtifact = {
  eventId: EventId;
  playerId: PlayerId;
  sourceTextHash: string;
  synthesizedText: string;
  voiceProfile: VoiceProfileSnapshot;
  generator: { adapter: "node-edge-tts"; packageVersion: string };
  audio: { file: string; durationMs: number };
  cues: readonly SpeechCue[];
  createdAt: string;
};
```

成功 artifacts 存在 `record.json` 的 `voiceArtifactsByEventId`。失败信息属于 job item，不创建 artifact。

### 3.3 TTS adapter

```ts
type VoiceSynthesisRequest = {
  sourceText: string;
  profile: VoiceProfileSnapshot;
};

type VoiceSynthesisResult = {
  synthesizedText: string;
  audioFile: string; // adapter-owned temporary path
  durationMs: number;
  wordBoundaries: readonly WordBoundary[];
  generator: PlayerVoiceArtifact["generator"];
};
```

Edge-specific preprocessing, package invocation and subtitle JSON decoding live inside the adapter. A separate domain service validates text coverage and converts word boundaries to `SpeechCue[]`.

## 4. Atomic generation flow

```text
Confirmed player event
  → derive speaker + source text
  → acquire Game lock
  → skip if artifact already exists
  → mark job item generating
  → Edge adapter writes unique temp MP3/JSON
  → decode + validate + aggregate cues
  → verify duration and non-empty audio
  → rename temp MP3 to voice/<eventId>.mp3
  → append immutable artifact to record.json
  → mark job item completed
```

If any step fails, remove temporary files, preserve no artifact, and store a sanitized error in the job. Locking prevents two jobs from generating the same event. The post-lock existence check supplies idempotency.

## 5. Voice job state machine

Reuse the semantics and shared primitives of video export jobs rather than copy-pasting them.

```text
queued → preparing → generating → completed
                     ↘ failed
queued/preparing/generating → canceled | interrupted
```

The job snapshots target event IDs at creation time. New events confirmed later are not silently added; the next “generate all” job picks them up. Progress is completed items divided by total items. Only one voice job drains at a time; event synthesis is serial for MVP.

On service initialization, active persisted jobs become `interrupted`. Retry creates a new job referencing the old job, rescans the original target IDs, and filters already successful artifacts.

## 6. Presenter manifest and clip planning

The presenter build script compiles the finite copy model into a versioned manifest:

```ts
type PresenterVoiceManifest = {
  schemaVersion: 1;
  presenterId: string;
  profile: VoiceProfileSnapshot;
  clips: Record<string, { file: string; text: string; durationMs: number }>;
};
```

Clip filenames include a content fingerprint derived from the full profile and
text. This makes `immutable` caching truthful after speaker/rate/copy changes.
All MP3-serving routes implement byte ranges so Remotion Player can seek after
timeline jumps; Range requests must return 206 rather than a chunked 200.

At runtime, presenter resolution returns a `PresenterClipPlan` with one or more clip IDs and a single display text. Every template with exactly one scalar seat variable (`seatNo`, `player`, `target`, or `voter`) prebuilds 12 complete utterances and resolves to exactly one clip, so the seat number and surrounding sentence remain continuous. Templates with two player variables, a role plus player, or an arbitrary player list may expand to reusable `seat.1`…`seat.12`/role clips; names are not spoken. Multi-clip plans retain an 80ms gap. The display text is visible from the first clip start through the last clip end.

## 7. Canonical playback timeline

Replace scene-level “one duration + optional presenter cue” with ordered voice-aware segments:

```ts
type PlaybackSegment =
  | PresenterVoiceSegment
  | PlayerVoiceSegment
  | SilenceSegment;
```

Each segment has absolute `startsAtMs`, `durationMs`, speaker identity, display cue and optional audio source. A player speech scene compiles to:

```text
[presenter clips] [250ms silence] [player audio + sentence cues] [350ms silence]
```

For incomplete Preview, missing player audio becomes one silent fallback segment using the current estimated scene duration and full static text. That fallback is never accepted by the export readiness validator.

`createCompositionInput()` consumes the compiled timeline and projects both frame view data and audio cues. Remotion `Sequence` placement and HTML Preview therefore share the same absolute times.

## 8. Subtitle projection

- Presenter: one cue spanning the full presenter clip plan.
- Player: adapter word boundaries are aligned against `synthesizedText`, then grouped by punctuation and visual length into non-overlapping sentence cues.
- At any frame, select the cue whose half-open interval contains local audio time: `[startMs, endMs)`.
- If the audio begins before the first spoken word, keep the dialogue box identity visible but content empty until the first cue.
- The final cue may extend to audio duration to avoid a premature blank caused by trailing metadata rounding.

The old `progress * windowCount` algorithm is removed.

## 9. API and UI

Server routes mirror export jobs:

- `GET/POST /api/games/:gameId/voice-jobs`
- `GET /api/voice-jobs/:jobId`
- `POST /api/voice-jobs/:jobId/cancel`
- `POST /api/voice-jobs/:jobId/retry`
- `GET /api/games/:gameId/voice/:eventId`

Preview right panel lists completeness, active job progress and per-event failures. It polls only while a job is active. The export panel receives a readiness projection with explicit blockers rather than a single `canExport` derived only from event count.

## 10. Export snapshot

Before render, the export snapshot builder:

1. validates presenter manifest completeness;
2. validates every target player event has an artifact;
3. copies presenter clips and Game voice MP3 files into the immutable export job assets directory;
4. builds composition input using only copied snapshot URLs.

This prevents a Game deletion or library change during rendering from affecting an active export.

## 11. Risks and rollback

- Edge endpoint instability: isolate adapter, persist successful output, expose retry, and keep a future provider seam.
- Subtitle mismatch: fail closed; never publish audio without validated cues.
- Partial filesystem writes: temp files + atomic rename + Game lock.
- Directory migration: no compatibility path by decision; seed/test fixtures must be regenerated together.
- Rollback point: storage migration and voice domain contracts land before UI and renderer changes; each stage remains testable without external Edge calls through a fake adapter.

## 12. Security and operational notes

- Validate all IDs before constructing paths; serve only artifacts declared by the Game record.
- Never expose proxy credentials or provider internals to the browser.
- Unit/integration tests use fake synthesis; live Edge calls belong only to an opt-in smoke script.
- Pin `node-edge-tts`; commercial release requires separate review of the underlying Edge service terms.

## 13. Worker process boundary

`pnpm dev` only serves Next.js. `pnpm worker:voice` and
`pnpm worker:video` are independent long-running processes; `pnpm dev:all`
starts all three for local development. API services persist `queued` jobs but
never enqueue in memory. Workers poll the filesystem, claim the oldest queued
job serially, and use one process lock per worker kind to prevent duplicate
local consumers. Restart reconciliation leaves `queued` intact and converts
only execution states to `interrupted`. Cancellation is communicated through
the persisted job record so it crosses process boundaries.

Game mutation locks use PID/token ownership and stale-owner recovery. Edge TTS
network work is deliberately outside the Game lock; the Worker locks only the
final re-read, idempotency check, MP3 publication, and record update. This keeps
Editor `continueGame()` independent of voice-generation latency.
