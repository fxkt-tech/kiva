# AI Werewolf MVP Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a brand-new playable single-player 9-seat AI Werewolf MVP from a clean design.

**Architecture:** Treat the implementation as a greenfield Rust/Bevy app inside this repository. Keep rules, game session state, AI heuristics, and UI systems in separate modules. Do not copy, restore, or reference prior application code; only use the product documents as requirements.

**Tech Stack:** Rust 2024, Bevy 0.18.1, rand 0.9, Bevy UI, Rust unit tests.

---

## References

- Product rules: `docs/product/game-rules.md`
- Interaction and UI design: `docs/product/interaction-ui-design.md`
- Project manifest: `Cargo.toml`

## Constraints

- This is a fresh redesign. Do not reference old source files, old UI layouts, old module boundaries, or old implementation decisions.
- Do not restore deleted files as a way to recover behavior.
- Create only the files needed for the new MVP.
- Keep the first implementation offline and deterministic enough to test.
- Do not add LLM integration in the first pass; AI decisions should be heuristic placeholders.
- Use Chinese UI text.

---

### Task 1: Create Fresh App Skeleton

**Files:**
- Create: `src/main.rs`
- Create: `src/lib.rs`
- Create: `src/game/mod.rs`

**Step 1: Write minimal app entry**

Create `src/lib.rs`:

```rust
pub mod game;
```

Create `src/main.rs`:

```rust
use bevy::prelude::*;
use lrk::game::WerewolfGamePlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "AI 狼人杀".to_string(),
                resolution: (1280, 800).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(WerewolfGamePlugin)
        .run();
}
```

Create `src/game/mod.rs`:

```rust
use bevy::prelude::*;

pub mod domain;

pub struct WerewolfGamePlugin;

impl Plugin for WerewolfGamePlugin {
    fn build(&self, _app: &mut App) {}
}
```

**Step 2: Check compilation**

Run: `cargo check`

Expected: PASS.

**Step 3: Commit**

```bash
git add src/main.rs src/lib.rs src/game/mod.rs
git commit -m "chore: create fresh ai werewolf app skeleton"
```

---

### Task 2: Add Core Domain Types

**Files:**
- Create: `src/game/domain.rs`

**Step 1: Write tests for role counts**

Add unit tests in `src/game/domain.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nine_player_deck_has_expected_roles() {
        let roles = Role::nine_player_deck();
        assert_eq!(roles.len(), 9);
        assert_eq!(roles.iter().filter(|role| **role == Role::Werewolf).count(), 3);
        assert_eq!(roles.iter().filter(|role| **role == Role::Seer).count(), 1);
        assert_eq!(roles.iter().filter(|role| **role == Role::Witch).count(), 1);
        assert_eq!(roles.iter().filter(|role| **role == Role::Hunter).count(), 1);
        assert_eq!(roles.iter().filter(|role| **role == Role::Villager).count(), 3);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test game::domain::tests::nine_player_deck_has_expected_roles`

Expected: FAIL because `Role` is not implemented.

**Step 3: Implement domain types**

Add:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Camp {
    Good,
    Werewolf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Werewolf,
    Seer,
    Witch,
    Hunter,
    Villager,
}

impl Role {
    pub fn camp(self) -> Camp {
        match self {
            Role::Werewolf => Camp::Werewolf,
            Role::Seer | Role::Witch | Role::Hunter | Role::Villager => Camp::Good,
        }
    }

    pub fn nine_player_deck() -> Vec<Role> {
        vec![
            Role::Werewolf,
            Role::Werewolf,
            Role::Werewolf,
            Role::Seer,
            Role::Witch,
            Role::Hunter,
            Role::Villager,
            Role::Villager,
            Role::Villager,
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PlayerId(pub usize);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlayerKind {
    Human,
    Ai,
}

#[derive(Debug, Clone)]
pub struct Player {
    pub id: PlayerId,
    pub name: String,
    pub role: Role,
    pub kind: PlayerKind,
    pub alive: bool,
}
```

**Step 4: Run tests**

Run: `cargo test game::domain`

Expected: PASS.

**Step 5: Commit**

```bash
git add src/game/domain.rs
git commit -m "feat: add fresh werewolf domain types"
```

---

### Task 3: Implement Fresh Session Model

**Files:**
- Create: `src/game/session.rs`
- Modify: `src/game/mod.rs`

**Step 1: Write tests**

Create `src/game/session.rs` with tests for:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::domain::{PlayerKind, Role};

    #[test]
    fn session_creates_nine_players_with_one_human() {
        let session = GameSession::new_with_roles(Role::nine_player_deck(), 1);
        assert_eq!(session.players.len(), 9);
        assert_eq!(
            session.players.iter().filter(|player| player.kind == PlayerKind::Human).count(),
            1
        );
    }

    #[test]
    fn good_wins_when_all_wolves_are_dead() {
        let mut session = GameSession::new_with_roles(Role::nine_player_deck(), 1);
        for player in &mut session.players {
            if player.role == Role::Werewolf {
                player.alive = false;
            }
        }
        assert_eq!(session.winner(), Some(Winner::Good));
    }

    #[test]
    fn wolves_win_when_wolves_equal_good_count() {
        let mut session = GameSession::new_with_roles(Role::nine_player_deck(), 1);
        for player in &mut session.players {
            player.alive = matches!(player.role, Role::Werewolf | Role::Villager);
        }
        assert_eq!(session.winner(), Some(Winner::Werewolf));
    }
}
```

**Step 2: Run tests to verify failure**

Run: `cargo test game::session`

Expected: FAIL because `GameSession` is not implemented.

**Step 3: Implement session**

Implement:

- `Winner`
- `GameSession`
- `new_with_roles`
- `alive_players`
- `winner`

Names should default to `你`, `2 号`, ..., `9 号`.

**Step 4: Export module**

Modify `src/game/mod.rs`:

```rust
pub mod session;
```

**Step 5: Run tests**

Run: `cargo test game::session`

Expected: PASS.

**Step 6: Commit**

```bash
git add src/game/session.rs src/game/mod.rs
git commit -m "feat: add fresh session model"
```

---

### Task 4: Implement Rules and Phase Engine

**Files:**
- Create: `src/game/phase.rs`
- Create: `src/game/rules.rs`
- Modify: `src/game/mod.rs`
- Modify: `src/game/session.rs`

**Step 1: Write tests**

Cover:

- Night kill kills target unless witch saves.
- Seer check returns target camp.
- Witch poison kills target.
- Hunter can shoot after exile or wolf kill.
- Hunter cannot shoot after poison.

**Step 2: Run tests to verify failure**

Run: `cargo test game::rules`

Expected: FAIL because rule resolution is not implemented.

**Step 3: Implement phase model**

Create phase types:

```rust
pub enum GamePhase {
    Start,
    RoleReveal,
    Night(u32),
    Dawn(u32),
    DaySpeech(u32),
    Vote(u32),
    HunterShot(u32),
    Review,
}
```

Create records for:

- wolf target
- seer check
- witch save
- witch poison
- deaths
- death reasons
- vote result

**Step 4: Implement rule functions**

Implement pure functions for:

- resolving night actions
- resolving exile
- resolving hunter shot
- checking whether hunter may shoot
- computing winner

**Step 5: Run tests**

Run: `cargo test game::rules game::phase`

Expected: PASS.

**Step 6: Commit**

```bash
git add src/game/phase.rs src/game/rules.rs src/game/session.rs src/game/mod.rs
git commit -m "feat: add fresh phase and rule engine"
```

---

### Task 5: Implement Local AI Heuristics

**Files:**
- Create: `src/game/ai.rs`
- Modify: `src/game/mod.rs`

**Step 1: Write tests**

Cover:

- AI wolf chooses a living non-wolf target.
- AI seer chooses a living unchecked target.
- AI vote chooses a living target and never votes self.
- AI speech does not reveal hidden roles unless the AI should know them.

**Step 2: Run tests to verify failure**

Run: `cargo test game::ai`

Expected: FAIL because AI module is not implemented.

**Step 3: Implement AI helpers**

Implement deterministic or seeded helpers:

- `choose_wolf_target`
- `choose_seer_target`
- `choose_witch_action`
- `choose_vote_target`
- `generate_speech`

Keep speeches short and Chinese.

**Step 4: Run tests**

Run: `cargo test game::ai`

Expected: PASS.

**Step 5: Commit**

```bash
git add src/game/ai.rs src/game/mod.rs
git commit -m "feat: add local ai heuristics"
```

---

### Task 6: Add Bevy App State

**Files:**
- Create: `src/game/app_state.rs`
- Modify: `src/game/mod.rs`

**Step 1: Define states and resources**

Create app states:

```rust
#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AppScreen {
    #[default]
    Start,
    RoleReveal,
    Game,
    Review,
}
```

Create resources:

- current `GameSession`
- selected player
- active information tab
- trust marks
- notes
- pending player input

**Step 2: Register state**

Modify `WerewolfGamePlugin` to add Bevy state and initialize resources.

**Step 3: Check compilation**

Run: `cargo check`

Expected: PASS.

**Step 4: Commit**

```bash
git add src/game/app_state.rs src/game/mod.rs
git commit -m "feat: add fresh app state"
```

---

### Task 7: Build Start and Role Reveal Screens

**Files:**
- Create: `src/game/ui.rs`
- Modify: `src/game/mod.rs`

**Step 1: Implement start screen**

Create Bevy UI for:

- title
- 9-player board configuration
- start button
- rules button

**Step 2: Implement role reveal screen**

On start:

- initialize `GameSession`
- randomly assign roles
- show player role
- show wolf teammates if player is wolf
- provide “入夜” button

**Step 3: Check manually**

Run: `cargo run`

Expected:

- Window opens.
- Start screen appears.
- Clicking start moves to role reveal.
- Clicking “入夜” moves to game screen placeholder.

**Step 4: Commit**

```bash
git add src/game/ui.rs src/game/mod.rs
git commit -m "feat: add fresh start and role reveal screens"
```

---

### Task 8: Build Night Round Table UI

**Files:**
- Modify: `src/game/ui.rs`

**Step 1: Implement top judge bar**

Show:

- phase
- judge hint
- alive count
- progress

**Step 2: Implement 9-seat round table**

Show:

- player number
- name
- alive/dead
- current speaker
- selected target
- human role on own seat
- trust mark

**Step 3: Implement information drawer**

Show tabs:

- public records
- my clues
- my marks

**Step 4: Implement bottom action console**

Render action controls based on phase and human role.

**Step 5: Check manually**

Run: `cargo run`

Expected:

- Game screen displays all 9 seats in a new round-table layout.
- Current phase and judge hint are visible.
- Seat selection works visually.

**Step 6: Commit**

```bash
git add src/game/ui.rs
git commit -m "feat: add night round table ui"
```

---

### Task 9: Wire Full Game Flow

**Files:**
- Modify: `src/game/ui.rs`
- Modify: `src/game/session.rs`
- Modify: `src/game/rules.rs`

**Step 1: Wire night actions**

Connect buttons to:

- wolf kill
- seer check
- witch save or poison
- continue for inactive roles

**Step 2: Wire AI actions**

When advancing phases, resolve AI night actions, AI speeches, and AI votes.

**Step 3: Wire player speech**

Allow user text input and append it to public speech records.

**Step 4: Wire voting**

Allow vote target or abstain, then resolve AI votes and exile.

**Step 5: Wire review**

After winner is found, transition to review screen and reveal full records.

**Step 6: Manual end-to-end check**

Run: `cargo run`

Expected:

- A full game can progress from start to victory.
- No blocked state leaves the user without an available action.

**Step 7: Commit**

```bash
git add src/game/ui.rs src/game/session.rs src/game/rules.rs
git commit -m "feat: wire fresh ai werewolf game flow"
```

---

### Task 10: Polish New Visual Direction and Verify

**Files:**
- Modify: `src/game/ui.rs`
- Optional Create: `assets/fonts/`

**Step 1: Apply black-night round table treatment**

Add:

- near-black blue-gray background
- deep wood round table
- red danger accents for wolves and deaths
- cold gold accents for special roles
- selected target highlight
- current speaker spotlight
- private clue styling

**Step 2: Check Chinese font rendering**

If Chinese text renders poorly, add a local font asset and configure Bevy text styles to use it.

**Step 3: Run verification**

Run:

```bash
cargo test
cargo check
cargo run
```

Expected:

- Tests pass.
- Project compiles.
- App opens and the MVP can be played manually.

**Step 4: Commit**

```bash
git add src/game/ui.rs assets/fonts
git commit -m "polish: style black-night round table ui"
```

---

## Final Verification

Run:

```bash
cargo test
cargo check
git status --short
```

Expected:

- Tests pass.
- Check passes.
- Only intended files are modified.

