# 配音与字幕同步：实施计划

## 1. Storage and domain foundation

- [ ] Convert `GameRepository` from `<gameId>.json` to `<gameId>/record.json`, including atomic save, locking, listing and recursive delete.
- [ ] Regenerate/move Game fixtures; do not add legacy compatibility.
- [ ] Add and validate `VoiceProfileSnapshot` on Character Definition and Player snapshot; snapshot it during Game creation.
- [ ] Add `PlayerVoiceArtifact` and `voiceArtifactsByEventId` to `GameRecord`, with a single decoder/normalizer owner.
- [ ] Add repository helpers for safe temp paths, artifact publication and event audio reads.
- [ ] Tests: round trip, invalid paths, immutable artifact collision, recursive deletion, role-library snapshot isolation.

Rollback point: revert storage/domain commits before any UI depends on the new layout.

## 2. Edge adapter and cue alignment

- [ ] Pin `node-edge-tts` and record the exact package version in generated artifacts.
- [ ] Implement the provider-neutral synthesis interface and a fake adapter for tests.
- [ ] Implement Edge-only stage-direction preprocessing and return `synthesizedText`.
- [ ] Decode word subtitle JSON, verify lossless normalized coverage, calculate true duration and aggregate short sentence cues.
- [ ] Publish MP3 only after all validation succeeds; clean temporary files on every failure path.
- [ ] Add opt-in live smoke command for voice listing and one Chinese synthesis; keep it out of normal tests.
- [ ] Tests: punctuation, Chinese without spaces, stage directions, pauses, metadata gaps, mismatch rejection, empty audio, timeout.

Validation: `pnpm typecheck && pnpm test` plus explicit opt-in Edge smoke.

## 3. Presenter prebuild pipeline

- [ ] Replace old presenter `voiceFile/status` data with a presenter voice profile and semantic clip-plan model.
- [ ] Implement `pnpm voice:build-presenter` using the Edge adapter.
- [x] Generate fixed semantic clips and 12 complete utterances for every single-seat prompt in `kivdb/presenters/<id>/`.
- [ ] Validate manifest completeness and reject missing/duplicate/mismatched clips.
- [ ] Replace runtime presenter resolution with clip plans; remove old MP3 assets, routes, duration loader and compatibility code.
- [ ] Tests: all copy keys resolve, all legal seats resolve, dynamic lists produce correct clip order/display text, no player name is spoken.

Rollback point: presenter manifest builder and resolver are one atomic slice; do not leave mixed old/new mappings.

## 4. Persistent voice jobs

- [ ] Extract reusable persisted-job/state-transition primitives from video export where practical.
- [ ] Implement voice job repository, service, serial queue, initialization reconciliation, cancellation and retry.
- [ ] Snapshot eligible confirmed player event IDs at job creation using the shared playback projection.
- [ ] Recheck artifact existence under Game lock before synthesis; skip immutable successes.
- [ ] Add routes and typed public projections with sanitized errors.
- [ ] Tests: eligibility including private wolf speech, leave-page independence, duplicate job race, partial success, restart interruption, retry idempotency.

Validation: server integration tests with fake adapter; no network calls.

## 5. Canonical voice-aware playback

- [ ] Introduce ordered playback segments and centralized timing constants: 250ms handoff, 350ms tail, 80ms presenter clip gap.
- [ ] Compile presenter clip plans, silence and player artifact cues into one absolute timeline.
- [ ] Preserve an explicit silent fallback projection for incomplete Preview only.
- [ ] Project audio cues and frame view state from the same timeline; remove character-count authority for voiced scenes.
- [ ] Replace progress-based subtitle windows with interval lookup.
- [ ] Update composition schema and decoder; bump schema version.
- [ ] Tests: no overlaps, exact boundaries, rounding at 30fps, missing voice fallback, presenter/player visual handoff, final cue extension.

Validation: `pnpm typecheck && pnpm test` and deterministic composition snapshots.

## 6. Preview UI and asset serving

- [ ] Add a right-panel voice generation section with completeness counts, create/cancel/retry controls and per-event errors.
- [ ] Poll active jobs and reload the composition when artifacts complete without regenerating successes.
- [ ] Add safe Game voice asset route backed by declared artifacts.
- [ ] Keep Preview usable with silent fallback and full static text for missing artifacts.
- [ ] Replace export boolean with readiness blockers and actionable messages.
- [ ] Tests: UI states, polling lifecycle, refresh recovery, blocked export, failed item retry, completed item immutability.

## 7. Export integration

- [ ] Validate presenter and player voice completeness before creating an export job.
- [ ] Copy Game voice and presenter clips into export snapshot assets.
- [ ] Ensure Remotion composition uses only snapshot asset URLs and the canonical timeline.
- [ ] Verify exported MP4 contains audio and its measured duration matches the compiled timeline within frame tolerance.
- [ ] Update video smoke script for a fully voiced fixture.

Validation: `pnpm video:smoke`, one-frame visual checks at presenter/player/cue boundaries, and one short MP4 render.

## 8. Final cleanup and review gate

- [ ] Remove all old audio files, `voiceFile/status` schemas, system voice routes, duration loaders and dead tests.
- [ ] Search repository for old paths and mappings; no compatibility branch remains.
- [ ] Run full `pnpm typecheck`, `pnpm test`, `pnpm build`, `pnpm video:smoke`.
- [ ] Review data flow end-to-end: Character → Game snapshot → voice job → artifact → playback timeline → Preview → export snapshot → Remotion.
- [ ] Review every filesystem boundary for path validation, atomicity and deletion semantics.
- [ ] Update `.trellis/spec/frontend/programmatic-video.md` with the new canonical audio/subtitle timeline contract after implementation proves it.

## 9. Isolated workers

- [x] Make API services persist-only; do not start Edge TTS or Remotion from Next.js.
- [x] Add serial filesystem-polling Voice and Video Worker entry points with per-kind process locks.
- [x] Preserve queued jobs across restart and interrupt only jobs that had started executing.
- [x] Make cancellation observable across processes through persisted job state.
- [x] Add `worker:voice`, `worker:video`, and `dev:all` scripts.
- [x] Add regressions proving API-created jobs stay queued until a Worker claims them.

## Manual acceptance

- [ ] Build a presenter voice library from scratch.
- [ ] Create a Game and confirm multiple public/private player speech events.
- [ ] Start generation, leave Preview, return and observe continuing/completed progress.
- [ ] Restart the server during a job, verify `interrupted`, retry and confirm completed events are skipped.
- [ ] Watch presenter prompt → 250ms → player speech → 350ms with no overlap.
- [ ] Verify player Galgame text switches by actual short-sentence timing and never reveals later text early.
- [ ] Confirm export is blocked before completeness and succeeds afterward with matching audio/subtitles.
