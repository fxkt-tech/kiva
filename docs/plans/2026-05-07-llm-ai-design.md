# LLM AI Werewolf Design

## Goal

Build the AI layer as a long-term maintainable subsystem. Every player decision is driven by that player's own LLM configuration and personality prompt, while game rules remain deterministic and testable.

This design replaces heuristic AI with:

- Per-player AI configuration in a dedicated file.
- Typed game events as the source of truth.
- Role-aware visibility rules for prompt context.
- One LLM decision interface for day speech, wolf kill, witch medicine, hunter shot, seer check, and voting.
- Structured model output that is validated before rules are applied.

## Current State

The current project is a Rust 2024 / Bevy app. The game loop lives mostly in `src/game/ui.rs`, with simple domain and rule modules:

- `src/game/domain.rs`: roles, players, player AI config.
- `src/game/session.rs`: player list and winner checks.
- `src/game/rules.rs`: deterministic night, seer, and hunter resolution.
- `src/game/ai.rs`: heuristic target and speech helpers.
- `src/game/player_pool.rs`: hard-coded player names, personality preferences, and model placeholders.
- `src/game/app_state.rs`: Bevy resources including public string logs.

The existing `FlowState.public_records: Vec<String>` is useful for UI, but it is not sufficient as a long-term AI memory model because it cannot reliably express private wolf chat, hidden seer checks, witch decisions, or role-scoped visibility.

## Recommended Architecture

Use typed events as the authoritative game history. The UI can still render strings, but strings become a projection of typed events rather than the state model itself.

Recommended module layout:

- `src/game/player_config.rs`: default player definitions and per-player LLM config.
- `src/game/events.rs`: typed `GameEvent`, `EventVisibility`, and `EventLog`.
- `src/game/visibility.rs`: role-aware context filtering.
- `src/game/llm.rs`: provider-neutral LLM request/response types and client trait.
- `src/game/decision.rs`: AI action types, JSON schemas, validation, and fallback policy.
- `src/game/ai.rs`: orchestration that asks LLMs for decisions and converts valid responses into rule inputs.
- `src/game/rules.rs`: remains deterministic; it applies already chosen actions.
- `src/game/ui.rs`: eventually consumes event projections instead of building all records inline.

## Player Configuration

Each player profile should include both presentation and LLM fields:

```rust
pub struct PlayerAiConfig {
    pub base_url: String,
    pub api_key: String,
    pub model: String,
    pub system_prompt: String,
}

pub struct PlayerProfile {
    pub name: String,
    pub avatar: String,
    pub ai: PlayerAiConfig,
}
```

`base_url`, `api_key`, and `model` should be empty strings by default so they can be filled later:

```rust
PlayerAiConfig {
    base_url: "".to_string(),
    api_key: "".to_string(),
    model: "".to_string(),
    system_prompt: "你是林澈，冷静分析，偏好归纳投票逻辑。你会隐藏自己的真实身份，除非公开跳身份有收益。".to_string(),
}
```

The old `personality_preference` field should be removed or derived from `system_prompt`. Keeping both creates drift.

For the first implementation, keep this as a Rust source file. A later version can load RON/TOML/JSON if hot editing becomes important.

## Typed Events

Typed events should capture both facts and speech:

```rust
pub enum GameEvent {
    GameStarted { day: u32 },
    PhaseStarted { phase: FlowPhase, day: u32 },
    DaySpeech { day: u32, speaker: PlayerId, content: String },
    VoteCast { day: u32, voter: PlayerId, target: PlayerId, reason: String },
    PlayerExiled { day: u32, player: PlayerId },
    WolfChat { night: u32, speaker: PlayerId, content: String },
    WolfKillChosen { night: u32, actor: PlayerId, target: PlayerId, reason: String },
    SeerChecked { night: u32, seer: PlayerId, target: PlayerId, camp: Camp },
    WitchMedicineUsed { night: u32, witch: PlayerId, action: WitchAction, target: Option<PlayerId>, reason: String },
    NightDeaths { night: u32, deaths: Vec<Death> },
    HunterShot { day: u32, hunter: PlayerId, target: Option<PlayerId>, reason: String },
    GameEnded { winner: Winner },
}
```

Visibility should be explicit:

```rust
pub enum EventVisibility {
    Public,
    Wolves,
    ActorOnly(PlayerId),
    WerewolfAndActor(PlayerId),
    System,
}
```

Each appended event stores both the payload and visibility:

```rust
pub struct LoggedEvent {
    pub sequence: u64,
    pub event: GameEvent,
    pub visibility: EventVisibility,
}
```

The event log should provide:

- `append(event, visibility)`
- `visible_to(session, viewer)`
- `public_projection()`
- `llm_context_for(session, viewer, decision_kind)`

## Visibility Rules

Visibility must be deterministic and test-covered.

Baseline:

- Villager, hunter, witch, and seer can see public events.
- Werewolves can see public events plus all wolf chat and wolf kill choices.
- Seer can see their own check results.
- Witch can see the current wolf victim during the witch decision, plus their own medicine history.
- Hunter can see the death reason and public history when deciding whether to shoot.
- Dead players should not make new normal actions, except hunter shot if rules allow it.
- Observer UI may see everything because this app is currently an observer game.

Important distinction: the observer UI seeing everything does not mean the LLM actor sees everything. Prompt context must always be generated from the actor's identity.

## LLM Decision Kinds

Use one orchestration path with different decision kinds:

```rust
pub enum AiDecisionKind {
    DaySpeech { day: u32 },
    WolfChat { night: u32 },
    WolfKill { night: u32 },
    WitchMedicine { night: u32, wolf_target: Option<PlayerId>, has_save: bool, has_poison: bool },
    HunterShot { day: u32, death_reason: DeathReason },
    SeerCheck { night: u32, checked: Vec<PlayerId> },
    Vote { day: u32 },
}
```

Each decision gets:

- Actor identity, role, alive/dead state.
- Actor `system_prompt`.
- Visible event summary.
- Legal target list.
- Required JSON output schema.
- A concise instruction to hide private information unless strategically revealed.

## Structured Output

LLM responses must be parsed and validated before applying rules.

Examples:

```json
{"speech":"我先听前置位发言，重点看谁急着归票。"}
```

```json
{"target":4,"reason":"4号白天发言试图压低狼坑，且不像神职。"}
```

```json
{"action":"save","target":6,"reason":"首夜救人保轮次。"}
```

```json
{"action":"skip","target":null,"reason":"信息不足，不开枪。"}
```

Validation should reject:

- Invalid JSON.
- Unknown fields if strict parsing is used.
- Illegal target IDs.
- Targeting dead players unless the action allows it.
- Werewolf targeting another living werewolf for wolf kill.
- Witch using a medicine already consumed.
- Seer checking self or repeating a checked target, unless no legal target remains.

When validation fails, the system should append an internal diagnostic event and use a deterministic fallback. Fallbacks preserve game progress but should be visible in tests.

## LLM Client Boundary

Define a trait so game logic can be tested without network:

```rust
pub trait LlmClient {
    fn complete(&self, request: LlmRequest) -> Result<LlmResponse, LlmError>;
}
```

`LlmRequest` should be provider-neutral:

- `base_url`
- `api_key`
- `model`
- `system`
- `user`
- `temperature`
- `response_format`

The first real client can use OpenAI-compatible chat completions because the user wants `base_url`, `api_key`, and `model` per player. The game should not assume one vendor beyond this wire format.

Because Bevy systems are synchronous today, implementation can start with blocking calls behind the trait. If latency becomes a UX problem, move decisions into async tasks later without changing the domain model.

## Flow Changes

Night should become explicit sub-actions:

1. Wolves discuss privately through `WolfChat` events.
2. A wolf LLM chooses `WolfKillChosen`.
3. Seer LLM chooses target; rules append `SeerChecked` visible only to seer.
4. Witch LLM receives wolf target and medicine state; rules append `WitchMedicineUsed`.
5. Rules resolve night deaths and append `NightDeaths`.
6. If hunter dies by a shootable reason, hunter LLM decides shot.

Day:

1. Each living player generates `DaySpeech`.
2. Each living player generates `VoteCast`.
3. Rules tally votes and append `PlayerExiled`.
4. If exiled hunter can shoot, hunter LLM decides shot.

The old `choose_group_vote_target` should be replaced by vote events and deterministic tallying.

## UI Projection

The UI should not manually push formatted strings during rule resolution. Instead:

- `EventLog.public_projection()` formats public events for the log panel.
- Observer mode can use `EventLog.observer_projection()` if we want to reveal private AI reasoning or all night actions.
- `FlowState.public_records` can be removed after UI is migrated.

This keeps rendering separate from game state.

## Error Handling

LLM failures should not crash the game loop.

Use this policy:

- Missing `base_url`, `api_key`, or `model`: return `LlmError::NotConfigured` and use fallback.
- HTTP/network failure: retry once, then fallback.
- Invalid JSON: append diagnostic and fallback.
- Valid JSON with illegal action: append diagnostic and fallback.

Diagnostics should not be visible to actor prompts unless intentionally added. Otherwise a model may react to implementation noise.

## Testing Strategy

Prioritize tests at the boundaries:

- Player configs include nine-plus players, empty connection fields, and non-empty `system_prompt`.
- Event visibility filters private events correctly for villager, werewolf, seer, witch, hunter, and observer.
- LLM context builder includes legal visible events and excludes hidden events.
- Decision validators reject illegal targets and invalid medicine use.
- Fake LLM responses drive every action kind without network.
- Night flow appends typed events in the expected order.
- Vote flow records individual votes and exiles the highest vote target.
- UI projection formats typed events into Chinese records.

## Migration Strategy

Implement in vertical slices:

1. Add player config and system prompts.
2. Add event log and public projection while keeping current UI usable.
3. Move day speech to typed events with fake LLM tests.
4. Move vote to typed vote events and tallying.
5. Move night actions to LLM decisions.
6. Add real OpenAI-compatible client.
7. Remove obsolete heuristic helpers and string-log mutation.

This sequence keeps the app playable between steps while steadily moving state ownership to the event log.
