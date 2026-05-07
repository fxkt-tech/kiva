# LLM AI Werewolf Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace heuristic Werewolf AI with per-player LLM-driven decisions backed by typed events, role-aware visibility, structured validation, and maintainable player configuration.

**Architecture:** Add typed event history as the authoritative game memory, then migrate each AI action to a unified decision pipeline. Keep game rules deterministic, test LLM behavior through a fake client, and expose UI logs as projections from typed events rather than hand-built strings.

**Tech Stack:** Rust 2024, Bevy 0.18, rand 0.9, serde/serde_json, reqwest or ureq for an OpenAI-compatible HTTP client, Rust unit tests.

---

## References

- Design doc: `docs/plans/2026-05-07-llm-ai-design.md`
- Current heuristic AI: `src/game/ai.rs`
- Current game loop: `src/game/ui.rs`
- Current state resources: `src/game/app_state.rs`
- Current player pool: `src/game/player_pool.rs`
- Current rules: `src/game/rules.rs`

## Constraints

- Use TDD for every production code change.
- Preserve observer-mode playability during migration.
- Do not hard-code real API keys.
- Default `base_url`, `api_key`, and `model` to empty strings.
- Every actor prompt must be built from that actor's visible events, not observer-visible state.
- LLM output must be parsed and validated before rules are applied.
- Network failures and invalid model output must fall back deterministically.

---

### Task 1: Add Player System Prompts

**Files:**
- Modify: `src/game/domain.rs`
- Create: `src/game/player_config.rs`
- Modify: `src/game/player_pool.rs`
- Modify: `src/game/mod.rs`

**Step 1: Write failing tests**

Add tests proving player profiles carry non-empty prompts and empty connection fields:

```rust
#[test]
fn default_ai_configs_are_unconfigured_but_have_system_prompts() {
    let pool = PlayerPool::default();
    let selected = pool.draw_nine(&mut rand::rng()).unwrap();

    for profile in selected {
        assert!(profile.ai.base_url.is_empty());
        assert!(profile.ai.api_key.is_empty());
        assert!(profile.ai.model.is_empty());
        assert!(!profile.ai.system_prompt.trim().is_empty());
    }
}
```

Run: `cargo test game::player_pool::tests::default_ai_configs_are_unconfigured_but_have_system_prompts`

Expected: FAIL because `system_prompt` does not exist and defaults are not empty.

**Step 2: Implement minimal domain/config changes**

Add `system_prompt` to `PlayerAiConfig`. Move hard-coded default player definitions to `src/game/player_config.rs` with helper constructors. Update `player_pool.rs` to consume those configs.

**Step 3: Verify**

Run: `cargo test game::player_pool`

Expected: PASS.

**Step 4: Commit**

```bash
git add src/game/domain.rs src/game/player_config.rs src/game/player_pool.rs src/game/mod.rs
git commit -m "feat: add per-player ai system prompts"
```

---

### Task 2: Introduce Typed Event Log

**Files:**
- Create: `src/game/events.rs`
- Modify: `src/game/mod.rs`
- Modify: `src/game/app_state.rs`

**Step 1: Write failing tests**

Create tests for append order and public projection:

```rust
#[test]
fn event_log_assigns_monotonic_sequence_numbers() {
    let mut log = EventLog::default();

    log.append(GameEvent::GameStarted { day: 1 }, EventVisibility::Public);
    log.append(GameEvent::PhaseStarted { phase: FlowPhase::Night, day: 1 }, EventVisibility::Public);

    assert_eq!(log.events()[0].sequence, 1);
    assert_eq!(log.events()[1].sequence, 2);
}

#[test]
fn public_projection_formats_day_speech() {
    let mut log = EventLog::default();
    log.append(
        GameEvent::DaySpeech {
            day: 1,
            speaker: PlayerId(3),
            content: "我先听后置位。".to_string(),
        },
        EventVisibility::Public,
    );

    assert_eq!(log.public_projection(), vec!["第 1 天：3 号发言：我先听后置位。"]);
}
```

Run: `cargo test game::events`

Expected: FAIL because `events` does not exist.

**Step 2: Implement minimal event types**

Add `GameEvent`, `EventVisibility`, `LoggedEvent`, and `EventLog`. Keep projection focused on events currently needed by UI, then extend in later tasks.

**Step 3: Verify**

Run: `cargo test game::events`

Expected: PASS.

**Step 4: Commit**

```bash
git add src/game/events.rs src/game/mod.rs src/game/app_state.rs
git commit -m "feat: add typed game event log"
```

---

### Task 3: Add Role-Aware Visibility Filtering

**Files:**
- Create: `src/game/visibility.rs`
- Modify: `src/game/events.rs`
- Modify: `src/game/mod.rs`

**Step 1: Write failing tests**

Test that hidden events are visible only to legal actors:

```rust
#[test]
fn werewolf_sees_wolf_chat_but_villager_does_not() {
    let session = GameSession::new_with_roles(Role::nine_player_deck());
    let mut log = EventLog::default();
    log.append(
        GameEvent::WolfChat {
            night: 1,
            speaker: PlayerId(1),
            content: "刀 4。".to_string(),
        },
        EventVisibility::Wolves,
    );

    assert_eq!(visible_events(&session, PlayerId(1), &log).len(), 1);
    assert_eq!(visible_events(&session, PlayerId(7), &log).len(), 0);
}

#[test]
fn seer_only_sees_own_check_result() {
    let session = GameSession::new_with_roles(Role::nine_player_deck());
    let mut log = EventLog::default();
    log.append(
        GameEvent::SeerChecked {
            night: 1,
            seer: PlayerId(4),
            target: PlayerId(1),
            camp: Camp::Werewolf,
        },
        EventVisibility::ActorOnly(PlayerId(4)),
    );

    assert_eq!(visible_events(&session, PlayerId(4), &log).len(), 1);
    assert_eq!(visible_events(&session, PlayerId(5), &log).len(), 0);
}
```

Run: `cargo test game::visibility`

Expected: FAIL because `visibility` does not exist.

**Step 2: Implement visibility rules**

Implement `visible_events(session, viewer, log)` and helper predicates for `Public`, `Wolves`, `ActorOnly`, `WerewolfAndActor`, and `System`.

**Step 3: Verify**

Run: `cargo test game::visibility`

Expected: PASS.

**Step 4: Commit**

```bash
git add src/game/visibility.rs src/game/events.rs src/game/mod.rs
git commit -m "feat: add role-aware event visibility"
```

---

### Task 4: Define LLM Request Boundary

**Files:**
- Create: `src/game/llm.rs`
- Modify: `src/game/mod.rs`
- Modify: `Cargo.toml`

**Step 1: Write failing tests**

Test that missing connection fields fail before network:

```rust
#[test]
fn request_from_unconfigured_player_returns_not_configured() {
    let config = PlayerAiConfig {
        base_url: "".to_string(),
        api_key: "".to_string(),
        model: "".to_string(),
        system_prompt: "你是测试玩家。".to_string(),
    };

    let err = LlmRequest::new(&config, "user prompt".to_string()).unwrap_err();

    assert_eq!(err, LlmError::NotConfigured);
}
```

Run: `cargo test game::llm`

Expected: FAIL because `llm` does not exist.

**Step 2: Implement provider-neutral types**

Add `LlmClient`, `LlmRequest`, `LlmResponse`, and `LlmError`. Add `serde` and `serde_json` only if needed by response parsing in this task; otherwise defer.

**Step 3: Verify**

Run: `cargo test game::llm`

Expected: PASS.

**Step 4: Commit**

```bash
git add Cargo.toml Cargo.lock src/game/llm.rs src/game/mod.rs
git commit -m "feat: define llm client boundary"
```

---

### Task 5: Add AI Decision Schemas and Validators

**Files:**
- Create: `src/game/decision.rs`
- Modify: `src/game/mod.rs`
- Modify: `Cargo.toml`

**Step 1: Write failing tests**

Cover speech, target, and witch medicine validation:

```rust
#[test]
fn wolf_kill_rejects_werewolf_target() {
    let session = GameSession::new_with_roles(Role::nine_player_deck());
    let decision = TargetDecision {
        target: PlayerId(2),
        reason: "测试".to_string(),
    };

    assert!(validate_wolf_kill(&session, PlayerId(1), &decision).is_err());
}

#[test]
fn witch_rejects_save_when_no_wolf_target() {
    let decision = WitchDecision {
        action: WitchActionDecision::Save,
        target: None,
        reason: "测试".to_string(),
    };

    assert!(validate_witch_decision(None, true, true, &decision).is_err());
}
```

Run: `cargo test game::decision`

Expected: FAIL because `decision` does not exist.

**Step 2: Implement decision types**

Add `AiDecisionKind`, `SpeechDecision`, `TargetDecision`, `WitchDecision`, `HunterDecision`, parse helpers, and validators. Use serde for JSON parsing.

**Step 3: Verify**

Run: `cargo test game::decision`

Expected: PASS.

**Step 4: Commit**

```bash
git add Cargo.toml Cargo.lock src/game/decision.rs src/game/mod.rs
git commit -m "feat: validate structured ai decisions"
```

---

### Task 6: Build Prompt Context From Visible Events

**Files:**
- Create: `src/game/prompt.rs`
- Modify: `src/game/mod.rs`

**Step 1: Write failing tests**

Test that prompts include visible records and exclude hidden records:

```rust
#[test]
fn villager_prompt_excludes_wolf_chat() {
    let session = GameSession::new_with_roles(Role::nine_player_deck());
    let mut log = EventLog::default();
    log.append(
        GameEvent::DaySpeech { day: 1, speaker: PlayerId(7), content: "公开发言".to_string() },
        EventVisibility::Public,
    );
    log.append(
        GameEvent::WolfChat { night: 1, speaker: PlayerId(1), content: "私聊刀人".to_string() },
        EventVisibility::Wolves,
    );

    let prompt = build_prompt(&session, PlayerId(7), AiDecisionKind::DaySpeech { day: 1 }, &log);

    assert!(prompt.user.contains("公开发言"));
    assert!(!prompt.user.contains("私聊刀人"));
}
```

Run: `cargo test game::prompt`

Expected: FAIL because `prompt` does not exist.

**Step 2: Implement prompt builder**

Build `PromptParts { system, user }` from player system prompt, visible event summaries, role, decision kind, and legal output schema.

**Step 3: Verify**

Run: `cargo test game::prompt`

Expected: PASS.

**Step 4: Commit**

```bash
git add src/game/prompt.rs src/game/mod.rs
git commit -m "feat: build role-aware llm prompts"
```

---

### Task 7: Replace Day Speech With LLM Decision Pipeline

**Files:**
- Modify: `src/game/ai.rs`
- Modify: `src/game/ui.rs`
- Modify: `src/game/app_state.rs`
- Test: `src/game/ai.rs`

**Step 1: Write failing test with fake LLM**

```rust
#[test]
fn day_speech_uses_fake_llm_and_appends_typed_event() {
    let session = GameSession::new_with_roles(Role::nine_player_deck());
    let mut log = EventLog::default();
    let client = FakeLlmClient::replying(r#"{"speech":"我先听后置位。"}"#);

    generate_day_speech(&client, &session, &mut log, PlayerId(1), 1).unwrap();

    assert!(matches!(
        &log.events()[0].event,
        GameEvent::DaySpeech { speaker: PlayerId(1), content, .. } if content == "我先听后置位。"
    ));
}
```

Run: `cargo test game::ai::tests::day_speech_uses_fake_llm_and_appends_typed_event`

Expected: FAIL because the pipeline does not exist.

**Step 2: Implement speech orchestration**

Call prompt builder, fake/real client trait, parse `SpeechDecision`, validate non-empty speech, append `GameEvent::DaySpeech`.

**Step 3: Migrate UI playback**

Update `speech_playback_system` to append typed speech events and derive displayed records from event projection.

**Step 4: Verify**

Run:

```bash
cargo test game::ai game::events game::visibility game::prompt
cargo check
```

Expected: PASS.

**Step 5: Commit**

```bash
git add src/game/ai.rs src/game/ui.rs src/game/app_state.rs
git commit -m "feat: drive day speech through llm decisions"
```

---

### Task 8: Replace Voting With Individual LLM Votes

**Files:**
- Modify: `src/game/ai.rs`
- Modify: `src/game/rules.rs`
- Modify: `src/game/ui.rs`
- Modify: `src/game/events.rs`

**Step 1: Write failing tests**

Test vote recording and tally:

```rust
#[test]
fn vote_tally_exiles_highest_vote_target() {
    let votes = vec![
        VoteCast { voter: PlayerId(1), target: PlayerId(4), reason: "a".to_string() },
        VoteCast { voter: PlayerId(2), target: PlayerId(4), reason: "b".to_string() },
        VoteCast { voter: PlayerId(3), target: PlayerId(5), reason: "c".to_string() },
    ];

    assert_eq!(tally_votes(&votes), Some(PlayerId(4)));
}
```

Run: `cargo test game::rules::tests::vote_tally_exiles_highest_vote_target`

Expected: FAIL.

**Step 2: Implement vote decisions**

Each living player gets a `Vote` decision. Append `VoteCast` events. Tally deterministically, using lowest seat number as tie-breaker.

**Step 3: Verify**

Run:

```bash
cargo test game::rules game::ai
cargo check
```

Expected: PASS.

**Step 4: Commit**

```bash
git add src/game/ai.rs src/game/rules.rs src/game/ui.rs src/game/events.rs
git commit -m "feat: drive voting through llm decisions"
```

---

### Task 9: Replace Night Actions With LLM Decisions

**Files:**
- Modify: `src/game/ai.rs`
- Modify: `src/game/rules.rs`
- Modify: `src/game/ui.rs`
- Modify: `src/game/events.rs`
- Modify: `src/game/app_state.rs`

**Step 1: Write failing tests**

Cover wolf chat, wolf kill, seer check, and witch medicine with fake LLM responses.

```rust
#[test]
fn night_flow_appends_private_wolf_chat_and_public_deaths() {
    let mut session = GameSession::new_with_roles(Role::nine_player_deck());
    let mut log = EventLog::default();
    let client = FakeLlmClient::scripted(vec![
        r#"{"speech":"建议刀4。"}"#,
        r#"{"target":4,"reason":"疑似预言家。"}"#,
        r#"{"target":1,"reason":"先验前置位。"}"#,
        r#"{"action":"skip","target":null,"reason":"首夜不用药。"}"#,
    ]);

    run_night_ai(&client, &mut session, &mut log, 1).unwrap();

    assert!(log.events().iter().any(|e| matches!(e.event, GameEvent::WolfChat { .. })));
    assert!(log.events().iter().any(|e| matches!(e.event, GameEvent::NightDeaths { .. })));
}
```

Run: `cargo test game::ai::tests::night_flow_appends_private_wolf_chat_and_public_deaths`

Expected: FAIL.

**Step 2: Implement night orchestration**

Run wolf chat, wolf kill, seer check, witch medicine, deterministic night resolution, and hunter pending state if needed.

**Step 3: Verify visibility**

Run:

```bash
cargo test game::ai game::visibility game::rules
cargo check
```

Expected: PASS.

**Step 4: Commit**

```bash
git add src/game/ai.rs src/game/rules.rs src/game/ui.rs src/game/events.rs src/game/app_state.rs
git commit -m "feat: drive night actions through llm decisions"
```

---

### Task 10: Replace Hunter Shot With LLM Decision

**Files:**
- Modify: `src/game/ai.rs`
- Modify: `src/game/ui.rs`
- Modify: `src/game/events.rs`

**Step 1: Write failing tests**

```rust
#[test]
fn hunter_can_choose_to_skip_shot() {
    let mut session = GameSession::new_with_roles(Role::nine_player_deck());
    let mut log = EventLog::default();
    let client = FakeLlmClient::replying(r#"{"action":"skip","target":null,"reason":"没有确定狼坑。"}"#);

    resolve_hunter_ai(&client, &mut session, &mut log, PlayerId(6), 1, DeathReason::Exile).unwrap();

    assert!(session.player(PlayerId(1)).unwrap().alive);
    assert!(log.events().iter().any(|e| matches!(e.event, GameEvent::HunterShot { target: None, .. })));
}
```

Run: `cargo test game::ai::tests::hunter_can_choose_to_skip_shot`

Expected: FAIL.

**Step 2: Implement hunter decision**

Call LLM only when `hunter_can_shoot(reason)` is true. Validate target or skip, append `HunterShot`, then apply rule if target exists.

**Step 3: Verify**

Run:

```bash
cargo test game::ai game::rules
cargo check
```

Expected: PASS.

**Step 4: Commit**

```bash
git add src/game/ai.rs src/game/ui.rs src/game/events.rs
git commit -m "feat: drive hunter shot through llm decisions"
```

---

### Task 11: Add Real OpenAI-Compatible Client

**Files:**
- Modify: `src/game/llm.rs`
- Modify: `Cargo.toml`
- Modify: `Cargo.lock`

**Step 1: Write failing tests**

Test request serialization without network:

```rust
#[test]
fn chat_request_serializes_system_and_user_messages() {
    let request = LlmRequest {
        base_url: "https://example.test/v1".to_string(),
        api_key: "key".to_string(),
        model: "model".to_string(),
        system: "system".to_string(),
        user: "user".to_string(),
        temperature: 0.7,
    };

    let body = serialize_chat_request(&request).unwrap();

    assert!(body.contains("\"model\":\"model\""));
    assert!(body.contains("\"role\":\"system\""));
    assert!(body.contains("\"role\":\"user\""));
}
```

Run: `cargo test game::llm::tests::chat_request_serializes_system_and_user_messages`

Expected: FAIL.

**Step 2: Implement HTTP client**

Use OpenAI-compatible `/chat/completions` requests. Keep the trait boundary so tests use fake clients and the app can fallback when configs are empty.

**Step 3: Verify**

Run:

```bash
cargo test game::llm
cargo check
```

Expected: PASS.

**Step 4: Commit**

```bash
git add Cargo.toml Cargo.lock src/game/llm.rs
git commit -m "feat: add openai-compatible llm client"
```

---

### Task 12: Remove Obsolete String-Log and Heuristic AI Paths

**Files:**
- Modify: `src/game/ai.rs`
- Modify: `src/game/app_state.rs`
- Modify: `src/game/ui.rs`
- Modify: `src/game/events.rs`

**Step 1: Write failing regression tests**

Add tests proving UI log data comes from event projection:

```rust
#[test]
fn public_records_are_projected_from_event_log() {
    let mut flow = FlowState::default();
    flow.event_log.append(
        GameEvent::PlayerExiled { day: 1, player: PlayerId(2) },
        EventVisibility::Public,
    );

    assert_eq!(flow.public_records(), vec!["第 1 天：2 号被放逐。"]);
}
```

Run: `cargo test game::app_state::tests::public_records_are_projected_from_event_log`

Expected: FAIL.

**Step 2: Remove obsolete state**

Remove `FlowState.public_records` storage after all callers use `flow.public_records()` or event projection. Delete unused heuristic helpers once no longer referenced.

**Step 3: Verify full suite**

Run:

```bash
cargo test
cargo check
```

Expected: PASS.

**Step 4: Commit**

```bash
git add src/game/ai.rs src/game/app_state.rs src/game/ui.rs src/game/events.rs
git commit -m "refactor: project ui logs from typed events"
```

---

## Final Verification

Run:

```bash
cargo test
cargo check
```

Manual smoke test:

```bash
cargo run
```

Expected behavior:

- Start game creates nine AI players with system prompts.
- Empty LLM config does not crash; deterministic fallbacks keep the game moving.
- Public UI log is generated from typed events.
- Wolf chat and private role information are not included in non-authorized actor prompts.
- Each phase uses the same LLM decision pipeline with action-specific schemas.

Plan complete and saved to `docs/plans/2026-05-07-llm-ai-implementation.md`. Two execution options after design approval:

**1. Subagent-Driven (this session)** - Dispatch fresh subagent per task, review between tasks, fast iteration.

**2. Parallel Session (separate)** - Open a new session with executing-plans, batch execution with checkpoints.
