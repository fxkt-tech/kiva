# Werewolf Director Design

Date: 2026-06-26

## Goal

Build a local web tool for creating, replaying, and recording complete Werewolf games.

This is not a multiplayer game. The user acts as the host/director. The host advances the game step by step, reviews generated drafts, edits or regenerates them, and confirms only approved content into the official game log. Confirmed games can be replayed and recorded as horizontal video.

## Product Principle

The product is a Werewolf match director and recording tool.

The core value is not player interaction. The core value is producing a believable complete Werewolf match with strict information boundaries, a durable event log, replay, and video recording.

## Hard Rules

1. The official event log is the source of truth.
2. Character models must never read global truth.
3. A character model may only receive that character's visible event log, role knowledge, profile snapshot, public rules, and current legal actions.
4. Host tools, rule settlement, and debug views may read global truth.
5. Drafts, LLM traces, and debug data are not facts.
6. Playback and recording read only confirmed active events.
7. Editor UI and playback UI are separate products.

## First Version Scope

The first version uses a fixed 6-player board:

- 2 Werewolves
- 1 Seer
- 1 Witch
- 2 Villagers

Rules should be complete for this board:

- Role assignment
- Werewolf night chat
- Werewolf kill
- Seer check
- Witch antidote and poison
- Night settlement
- Public death announcement
- Last words
- Daytime speeches
- Voting
- Tie handling
- PK speeches
- Exile
- Win condition checks
- Endgame reveal

Configurable rule defaults:

- Win condition: configurable, default `slaughter_side`.
- Witch first-night self-save: configurable.
- Witch same-night antidote and poison: configurable, default disabled.
- Vote result visibility: default reveal full vote table after all votes.
- Tie handling: tied players enter PK speeches, then revote; if still tied, no exile.
- Dead player role reveal: default no reveal until game end.
- PK voters: default tied PK players do not vote in the PK revote.

The output format is horizontal 16:9 video, target canvas 1920x1080.

## Architecture

### Core Modules

`rules-engine`

- Derives current game state from active official events.
- Owns phase transitions, legal action generation, rule validation, death settlement, and win condition checks.

`event-store`

- Stores active and superseded official events.
- Stores draft events.
- Stores LLM traces.
- Supports rollback by marking later active events as superseded.

`visibility-projector`

- Filters the official event log into one player's visible event log.
- This is a hard security boundary for all character LLM calls.

`advance-planner`

- Runs when the host clicks Continue.
- Reads derived state.
- Decides the next required task.
- Requests legal actions from the rules engine.
- Calls role models through the LLM orchestrator when needed.
- Produces a draft.

`llm-orchestrator`

- Calls the model bound to the acting character snapshot.
- Requires strict structured JSON output.
- Records prompts, visible event IDs, raw responses, parsed actions, and validation failures.
- Retries once on parse or validation failure.

`playback-compiler`

- Converts active official events into playback timelines.
- Supports public-view timeline and god-view timeline.

`recorder`

- Runs in the browser.
- Records the playback page with `MediaRecorder`.
- First version exports browser-supported video, such as WebM. MP4 conversion can be added later.

## Pages

### Editor

The editor is the host's creation workspace.

It can show:

- Global game state
- Seats, roles, factions, alive/dead state, witch resources
- Official event log
- Draft candidate
- LLM trace summaries
- Character visible logs
- Prompt and model debug information
- Embedded preview

The editor supports:

- Continue
- Regenerate draft
- Edit draft
- Confirm draft
- View character perspective
- Roll back to an earlier event
- Preview active events
- Temporarily preview active events plus the current draft

Draft preview must be visibly marked as draft preview.

### Playback

The playback page is the recording surface.

It must not show:

- Editor controls
- Debug information
- LLM traces
- Global truth unless the selected playback mode intentionally compiles it into god view

It supports:

- Play
- Pause
- Restart
- Speed control
- Jump to event
- Hide controls while recording
- Public-view playback
- God-view playback

Recording uses this page, not the editor.

## Data Model

### CharacterProfile

Long-lived character asset.

Fields:

- `id`
- `name`
- `avatar`
- `tags`
- `enabled`
- `persona`
- `speakingStyle`
- `reasoningStyle`
- `roleAdaptation`
- `systemPrompt`
- `modelBinding`
- `createdAt`
- `updatedAt`

Character profiles are not game roles. The same profile can be assigned any game role in different games.

### PlayerSnapshot

Per-game copy of a selected character profile.

Fields:

- `playerId`
- `seatNo`
- `profileSourceId`
- `name`
- `avatar`
- `persona`
- `speakingStyle`
- `reasoningStyle`
- `systemPrompt`
- `modelBindingSnapshot`
- `gameRole`
- `faction`
- `initialPrivateKnowledge`

Snapshots preserve old games even if the character library changes later.

### Game

Fields:

- `id`
- `title`
- `status`
- `ruleset`
- `players`
- `currentPhaseCache`
- `createdAt`
- `updatedAt`

Current state is derived from active official events. Caches may exist for performance but are not the source of truth.

### GameEvent

Official fact.

Fields:

- `id`
- `gameId`
- `index`
- `status`: `active` or `superseded`
- `type`
- `phase`
- `actorPlayerId`
- `targetPlayerIds`
- `visibility`
- `payload`
- `display`
- `createdFromDraftId`
- `createdAt`

Visibility must be explicit:

- `public`
- `host_only`
- `player_private`
- `faction_private`
- `custom`

### DraftEvent

Unconfirmed candidate.

Fields:

- `id`
- `gameId`
- `baseEventIndex`
- `draftType`
- `candidateEvents`
- `llmTraceIds`
- `validationStatus`
- `editableFields`
- `createdAt`

Drafts do not affect official state until confirmed.

### LLMTrace

Debug and audit data.

Fields:

- `id`
- `gameId`
- `playerId`
- `provider`
- `model`
- `visibleEventIds`
- `promptSnapshot`
- `rawResponse`
- `parsedAction`
- `validationErrors`
- `createdAt`

LLM traces are visible in the editor only.

### ViewpointLog

Derived player perspective.

Input:

- Active official events
- Player ID

Output:

- Events visible to that player
- Public state summary visible to that player

It can be computed on demand or cached. The cache is not authoritative.

### PlaybackTimeline

Derived display timeline.

It compiles active official events into visual segments. Multiple low-level events can become one playback segment, such as vote animation or night announcement.

## Character Library

The character library needs a minimal management page in the first version:

- List characters
- Create character
- Edit character
- Disable character
- Configure model binding
- Test prompt

System-generated character creation does not need a page in the first version. Bulk character generation can be handled by import scripts.

Character model binding is per character:

- `provider`
- `model`
- `temperature`
- `maxTokens`
- `responseFormat`
- `fallbackModel`

At game creation, the model binding is copied into the player snapshot.

## Game Creation Flow

1. Host creates a new game.
2. Host selects 6 enabled character profiles.
3. System randomly assigns seats by default; host may adjust.
4. System randomly assigns roles by default; host may adjust.
5. System validates board composition.
6. System creates player snapshots.
7. System writes initial official events:
   - `game_created`
   - `ruleset_selected`
   - `player_snapshots_created`
   - `seat_assigned`
   - `role_assigned`
   - `initial_private_knowledge_given`
   - `phase_started`
8. Editor opens on Night 1.

Initial private knowledge:

- Werewolves know their own role and wolf teammates.
- Seer knows only their own role.
- Witch knows only their own role and medicine resources.
- Villagers know only their own role.
- Host knows all roles and state.

## Continue Flow

When the host clicks Continue:

1. Read active official events.
2. Derive current state.
3. Determine the next required task.
4. Generate legal actions.
5. Build character-specific visible context when a role model is needed.
6. Call the character's bound model.
7. Parse strict JSON.
8. Validate the action.
9. Retry once if parsing or validation fails.
10. Create a draft candidate.
11. Show draft to host.
12. Host confirms, edits, or regenerates.
13. Confirmed draft becomes active official events.

The LLM can influence drafts. Only validated and host-confirmed drafts become facts.

## Night Flow

Default order:

1. Werewolf night chat
2. Werewolf kill
3. Seer check
4. Witch decision
5. Night settlement
6. Daybreak announcement

### Werewolf Chat

Werewolf chat is visible only to wolves and host.

Events:

- `wolf_chat_message`
- `wolf_kill_discussed`

The chat should influence kill selection.

### Werewolf Kill

Legal targets: living non-wolves by default.

Events:

- `wolf_kill_selected`

Visible to wolves and host.

### Seer Check

Legal targets: living players except self. Repeated checks can be allowed by rules but should be discouraged by generation prompts.

Events:

- `seer_check_selected`
- `seer_check_result`

The result reveals alignment only, not exact role.

Visible to seer and host.

### Witch Decision

The witch sees the night death information.

The witch may:

- Use antidote if available and allowed.
- Use poison if available and allowed.
- Use neither.

Default: antidote and poison cannot both be used in the same night.

Events:

- `witch_death_info_shown`
- `witch_antidote_decided`
- `witch_poison_decided`

Visible to witch and host.

### Night Settlement

Rules engine resolves deaths.

Events:

- `night_resolved`
- `phase_started(day_n)`
- `peaceful_night_announced` or `death_announced`

Public announcement reveals deaths but not hidden causes.

## Day Speech Flow

The speech queue is derived from rules:

- Living players speak in order.
- Last words are separate from normal speeches.
- First-day direction can be random.
- Later direction follows rules based on prior death or exile position.

For each speech:

Model input:

- Player snapshot
- Own game role
- Visible event log
- Public state
- Legal expression boundaries
- Current speech objective

Model output:

- `private_analysis`
- `speech_intent`
- `public_text`
- `vote_leaning`
- `risk_flags`

Editor displays:

- Public speech draft
- Character visible event summary
- Private analysis
- Risk flags
- Possible information-boundary issues

Confirmed events:

- `speech_turn_started`
- `public_speech_spoken`
- `speech_turn_ended`

Speech intent must be adjustable before regeneration.

## Voting, PK, Exile, and Last Words

### Voting

After speeches:

- Generate `vote_started`.
- Each living player votes based on that player's visible information.
- Votes are generated independently from speech `vote_leaning`.
- Host reviews the full vote draft.

Confirmed events:

- `vote_cast`
- `vote_tallied`

Default: reveal the full vote table after all votes are confirmed.

### Tie and PK

If top votes tie:

1. Generate `tie_detected`.
2. Start PK.
3. Tied players speak.
4. Non-PK living players revote by default.
5. If still tied, no exile.

Events:

- `pk_started`
- `pk_speech_spoken`
- `pk_vote_started`
- `pk_vote_cast`
- `pk_vote_tallied`
- `no_exile_announced` or `exile_announced`

### Exile

The exiled player dies.

Events:

- `player_exiled`
- `death_state_changed`
- `exile_announced`

Default: role is not revealed until game end.

### Last Words

Last words are generated using only the dead player's visible information up to the moment of death or exile, plus the public death or exile event.

Events:

- `last_words_started`
- `last_words_spoken`
- `last_words_ended`

Death does not grant hidden truth.

## Win Conditions

Check after every death, exile, no-exile settlement, and night settlement.

Default slaughter-side rules:

- All wolves dead: good team wins.
- All gods dead: wolves win.
- All villagers dead: wolves win.

If no side wins, continue to the next phase.

If the game ends:

- Write `game_ended`.
- Reveal roles.
- Generate endgame summary.

## Playback

Playback supports two modes:

### Public View

Audience sees only public information, similar to a normal player experience.

Hidden night actions are not shown until endgame reveal.

### God View

Audience sees hidden information:

- Wolf night chat
- Wolf kill target
- Seer checks
- Witch decisions
- Hidden causes of death

The same official event log can produce both timelines.

### Visual Layout

Target: 1920x1080, 16:9.

First version layout:

- Center: table and six seats
- Top: day/night, phase, alive count
- Bottom: subtitle and speech text
- Side: public event hints
- Night: dark overlay
- Speech: active speaker highlighted
- Vote: vote animation or vote table
- End: faction result, role reveal, key recap

Playback timeline segments have estimated durations:

- Phase transition: 2-3 seconds
- Death announcement: 3-5 seconds
- Speech: estimated from text length
- Vote animation: 5-8 seconds
- End reveal: 8-12 seconds

Text events should reserve future fields for audio/TTS duration, but first version does not need TTS.

## Recording

First version recording:

- Browser `MediaRecorder`
- Records playback page only
- Hides control layer during recording
- Does not call LLM
- Does not mutate game data
- Exports browser-supported video, initially WebM

Later improvements:

- MP4 conversion through ffmpeg
- Server-side rendering or frame-based video generation
- TTS and audio synchronization

## Rollback

Rollback keeps history.

First version approach:

- `GameEvent.status` is `active` or `superseded`.
- Rolling back to event index N marks active events after N as superseded.
- New confirmed drafts create new active events after N.
- Playback reads only active events.
- Editor can show superseded history by request.

This preserves debugging value without polluting playback.

## Error Handling

Draft generation failures:

- Retry once on LLM failure, JSON parse failure, or validation failure.
- If still failing, create a `manual_required` draft.
- Host can manually fill the action or speech.
- Failure details go to `LLMTrace`.

Draft edit validation:

- Action type must match current phase.
- Targets must be legal.
- Visibility must be valid.
- Required text cannot be empty.
- Edited events cannot break event order.

Derived state errors:

- Stop advancement.
- Show conflicting event indexes.
- Do not continue generating on inconsistent state.

Examples:

- Dead player speaks.
- Witch medicine goes negative.
- Duplicate settlement for one phase.
- Vote count does not match eligible voters.

## Technology

Use one Next.js project.

Stack:

- Next.js
- TypeScript
- Tailwind CSS
- shadcn/ui
- Radix Primitives
- lucide-react
- SQLite
- Drizzle or Prisma
- Server-side LLM provider adapters
- Browser MediaRecorder

Next.js unifies the project, but boundaries remain explicit:

- Pages and components handle UI.
- Server routes or server actions handle persistence and LLM calls.
- Rules, events, visibility, planner, and playback compiler are pure modules.
- Playback rendering is client-side and recordable.

Avoid scattering rules or event mutation logic inside pages.

## Out of Scope for First Version

- Multiplayer live play
- Online accounts
- Cloud collaboration
- Server-side MP4 generation
- TTS generation
- Full custom board builder
- In-page automatic character generation
- Mobile-first vertical video
- Real-time human player input

## Open Extension Points

- More roles: Hunter, Guard, Idiot, Sheriff, Cupid, etc.
- Board presets: 9-player and 12-player games.
- TTS per character.
- Video export worker.
- Advanced timeline editing.
- Scripted character import and batch generation.
- Multiple visual themes for playback.
- Public-view and god-view export presets.
