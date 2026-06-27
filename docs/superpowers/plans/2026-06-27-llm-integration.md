# LLM Integration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a rules-bound, visibility-bound, editable LLM generation pipeline for Werewolf speeches and actions.

**Architecture:** Core modules own prompt construction, model adapter interfaces, structured parsing, and rule validation. Server actions orchestrate game persistence, draft updates, and generation records. UI exposes regeneration and debug summaries without letting model output bypass structured draft payloads.

**Tech Stack:** Next.js server actions, TypeScript core modules, Vitest, Node `fetch` for OpenAI-compatible chat completions, local JSON persistence.

---

## File Structure

- Create `src/core/llm.ts`: model message types, `LlmClient`, JSON parsing, mock client, OpenAI-compatible fetch adapter.
- Create `src/core/generation-record.ts`: generation record types and helpers.
- Create `src/core/prompt-builders.ts`: prompt builders for speech and action drafts from `PlayerLlmContext`.
- Create `src/core/speech-generation.ts`: generate text for `day_speech_given`, `last_words_given`, `pk_speech_given`.
- Create `src/core/action-generation.ts`: generate and validate target/action payloads for seer, wolf, vote, witch.
- Modify `src/server/game-repository.ts`: add `generations` to `GameRecord`, default old records to `[]`.
- Modify `src/server/game-actions.ts`: inject `LlmClient`, generate speech on continue, add regenerate action, later add action suggestions.
- Modify `src/app/actions.ts`: add regenerate server action.
- Modify `src/components/editor/draft-panel.tsx`: add regenerate controls and generation summary.
- Add tests beside each module in `src/core/__tests__`, `src/server/__tests__`, and component tests.

## Task 1: LLM Client Boundary

**Purpose:** Define the only interface business logic may use to call a model.

- [ ] Add failing tests in `src/core/__tests__/llm.test.ts` for:
  - `MockLlmClient` returns queued JSON.
  - `parseLlmJsonObject` accepts fenced JSON and rejects non-object JSON.
  - `OpenAICompatibleLlmClient` sends OpenAI-compatible payload shape through injected fetch.
- [ ] Run `pnpm test src/core/__tests__/llm.test.ts`; expect module-not-found failure.
- [ ] Implement `src/core/llm.ts`.
- [ ] Run targeted test; expect pass.
- [ ] Run `pnpm test && pnpm typecheck`.
- [ ] Commit: `feat: add llm client boundary`.

## Task 2: Generation Records

**Purpose:** Persist model call evidence separately from official game events.

- [ ] Add failing tests in `src/core/__tests__/generation-record.test.ts` for creating success/failure records with prompt version, model binding, raw output, parsed output, and draft id.
- [ ] Add failing repository/action tests proving `GameRecord.generations` survives save/load and old records default to `[]`.
- [ ] Implement `src/core/generation-record.ts`.
- [ ] Modify `GameRecord` and repository JSON loading to normalize missing `generations`.
- [ ] Run targeted tests, then `pnpm test && pnpm typecheck`.
- [ ] Commit: `feat: record llm generations`.

## Task 3: Prompt Builders

**Purpose:** Build prompts only from player-visible context, never from host truth.

- [ ] Add failing tests in `src/core/__tests__/prompt-builders.test.ts` proving speech prompts include viewer persona, visible timeline, safe roster, draft intent, and do not include hidden role names.
- [ ] Implement `src/core/prompt-builders.ts` with `buildSpeechPrompt` and prompt version constants.
- [ ] Run targeted tests, then `pnpm test && pnpm typecheck`.
- [ ] Commit: `feat: build visibility safe llm prompts`.

## Task 4: Speech Generation Service

**Purpose:** Generate draft speech text from LLM JSON and write it into draft payload only.

- [ ] Add failing tests in `src/core/__tests__/speech-generation.test.ts` for supported speech drafts, unsupported drafts, malformed JSON, and context visibility.
- [ ] Implement `src/core/speech-generation.ts`.
- [ ] Wire `continueGame` to enrich newly planned speech drafts using injected `LlmClient`, falling back to original draft on generation failure while recording failure.
- [ ] Add server tests for generated speech draft and failure fallback.
- [ ] Run targeted tests, then full verification.
- [ ] Commit: `feat: generate speech drafts with llm`.

## Task 5: Regenerate Speech UI

**Purpose:** Let the host retry current speech draft generation without advancing the game.

- [ ] Add server action tests for `regenerateDraftAction`.
- [ ] Add component tests for DraftPanel showing regenerate button only on LLM-supported drafts and showing latest generation status.
- [ ] Implement server action, game action, and UI button.
- [ ] Run targeted tests, then full verification.
- [ ] Commit: `feat: add draft regeneration control`.

## Task 6: OpenAI-Compatible Runtime Configuration

**Purpose:** Enable real model calls without hard-coding provider secrets into core logic.

- [ ] Add tests for resolving provider config from environment variables.
- [ ] Implement server-side LLM factory using `OPENAI_COMPATIBLE_BASE_URL`, `OPENAI_COMPATIBLE_API_KEY`, and player `modelBindingSnapshot`.
- [ ] Keep mock client as default when env is absent.
- [ ] Run full verification.
- [ ] Commit: `feat: add openai compatible llm runtime`.

## Task 7: Action Prompt and Legal Validation

**Purpose:** Make LLM action output advisory; system rules decide whether payload is accepted.

- [ ] Add failing tests for seer target, wolf kill target, vote target, and witch medicine output.
- [ ] Implement `src/core/action-generation.ts` with legal target validation using existing rules helpers.
- [ ] On invalid output, record failure and preserve deterministic draft.
- [ ] Run targeted tests, then full verification.
- [ ] Commit: `feat: generate legal action suggestions`.

## Task 8: Wire Action Suggestions

**Purpose:** Automatically enrich non-speech actionable drafts after planning.

- [ ] Add server tests for seer, wolf, vote, and witch draft enrichment.
- [ ] Modify `continueGame` and regenerate action to call action generation for supported action drafts.
- [ ] Keep role assignment, phase, night resolution, death announcement, exile resolution, and game end deterministic.
- [ ] Run full verification.
- [ ] Commit: `feat: wire llm action suggestions`.

## Task 9: Host Debug Surface

**Purpose:** Give host enough visibility to trust and debug generation without exposing irrelevant raw internals.

- [ ] Add component tests for latest generation summary: status, provider/model, prompt version, error.
- [ ] Update DraftPanel to display latest generation summary and a compact “visible events used” count.
- [ ] Run full verification.
- [ ] Commit: `feat: show llm generation summary`.

## Task 10: Final Verification and Hardening

**Purpose:** Prove the full game loop works with mock LLM, persistence, regeneration, and preview.

- [ ] Add integration test that advances through setup into generated speech and confirms preview text comes from structured draft payload.
- [ ] Run `pnpm test`, `pnpm typecheck`, `pnpm build`.
- [ ] Smoke test active dev server route.
- [ ] Commit final fixes if needed.

## Self-Review

- Spec coverage: all six approved stages map to tasks: client boundary, prompts, speech, generation records, actions, UI controls, runtime adapter.
- No hidden-info shortcut: prompt builders consume `PlayerLlmContext`; tests must prove hidden role names do not leak.
- Event log boundary: generated model process is stored in `generations`; official `events` remain structured game facts.
- Scope: recording/video export is intentionally excluded because it depends on content generation being valuable first.
