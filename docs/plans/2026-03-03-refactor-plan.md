# 四方诛杀全量重构实现计划

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 将现有单机2人21点重构为三层解耦架构（逻辑/网络/渲染），支持局域网4人对战和AI填充。

**Architecture:** 逻辑层（`game/`）对渲染零感知，通过 Bevy Event 与渲染层通信；网络层（`net/`）只同步 `GameAction` 指令；渲染层（`plugins/`）维护 `PlayerEntityMap` 映射，监听事件驱动UI更新。

**Tech Stack:** Rust 2024, Bevy 0.16, rand 0.9, serde + bincode（网络序列化），标准库 UdpSocket（局域网通信）

---

## Task 1: 更新 Cargo.toml，添加新依赖

**Files:**
- Modify: `Cargo.toml`

**Step 1: 添加 serde 和 bincode 依赖**

将 `[dependencies]` 改为：

```toml
[dependencies]
bevy = { version = "0.16", features = ["dynamic_linking", "file_watcher"] }
rand = "0.9.1"
serde = { version = "1", features = ["derive"] }
bincode = "2"
```

**Step 2: 验证编译通过**

```bash
cargo check
```

期望输出：无错误（可能有 unused import 警告，忽略）

**Step 3: 提交**

```bash
git add Cargo.toml Cargo.lock
git commit -m "chore: add serde and bincode dependencies"
```

---

## Task 2: 新建 `src/states.rs`，迁移 WorldState

**Files:**
- Create: `src/states.rs`
- Modify: `src/main.rs`

**Step 1: 创建 `src/states.rs`**

```rust
use bevy::prelude::*;

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum WorldState {
    #[default]
    Menu,
    Lobby, // 局域网房间大厅
    Game,
}
```

**Step 2: 修改 `src/main.rs`，引用新的 states 模块**

删除 `main.rs` 中原有的 `WorldState` 枚举定义，改为：

```rust
mod plugins;
mod states;

use bevy::prelude::*;
use plugins::KivaPlugins;
use states::WorldState;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "四方诛杀 v0.2.0".into(),
                resolution: (1280., 720.).into(),
                resizable: false,
                decorations: true,
                position: WindowPosition::Centered(MonitorSelection::Primary),
                ..default()
            }),
            ..default()
        }))
        .init_state::<WorldState>()
        .add_systems(Startup, setup)
        .add_plugins(KivaPlugins)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}

pub fn despawn_screen<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    for entity in &to_despawn {
        commands.entity(entity).despawn();
    }
}
```

**Step 3: 验证编译**

```bash
cargo check
```

**Step 4: 提交**

```bash
git add src/states.rs src/main.rs
git commit -m "refactor: extract WorldState to states.rs, add Lobby state"
```

---

## Task 3: 新建逻辑层 `src/game/card.rs`

**Files:**
- Create: `src/game/card.rs`

**Step 1: 创建文件**

```rust
use rand::seq::SliceRandom;
use rand::rng;

/// 一张牌，只含点数，不含任何渲染信息
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Card {
    pub rank: u8, // 1-11
}

/// 牌组
pub struct Deck {
    cards: Vec<Card>,
}

impl Default for Deck {
    fn default() -> Self {
        Self::new()
    }
}

impl Deck {
    pub fn new() -> Self {
        let mut cards: Vec<Card> = (1..=11).map(|rank| Card { rank }).collect();
        let mut rng = rng();
        cards.shuffle(&mut rng);
        Deck { cards }
    }

    pub fn pop(&mut self) -> Option<Card> {
        self.cards.pop()
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }

    pub fn remaining(&self) -> usize {
        self.cards.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deck_has_11_cards() {
        let deck = Deck::new();
        assert_eq!(deck.cards.len(), 11);
    }

    #[test]
    fn deck_contains_ranks_1_to_11() {
        let deck = Deck::new();
        let mut ranks: Vec<u8> = deck.cards.iter().map(|c| c.rank).collect();
        ranks.sort();
        assert_eq!(ranks, (1u8..=11).collect::<Vec<_>>());
    }

    #[test]
    fn pop_reduces_count() {
        let mut deck = Deck::new();
        deck.pop();
        assert_eq!(deck.remaining(), 10);
    }

    #[test]
    fn reset_restores_11_cards() {
        let mut deck = Deck::new();
        deck.pop();
        deck.pop();
        deck.reset();
        assert_eq!(deck.remaining(), 11);
    }
}
```

**Step 2: 运行测试（此时会失败，因为 mod 还没注册）**

先创建 `src/game/mod.rs`（空文件占位）：

```rust
pub mod card;
```

然后在 `src/main.rs` 顶部加 `mod game;`

**Step 3: 运行测试**

```bash
cargo test game::card
```

期望：4个测试全部 PASS

**Step 4: 提交**

```bash
git add src/game/card.rs src/game/mod.rs src/main.rs
git commit -m "feat: add Card and Deck types with unit tests"
```

---

## Task 4: 新建 `src/game/player.rs`

**Files:**
- Create: `src/game/player.rs`
- Modify: `src/game/mod.rs`

**Step 1: 创建文件**

```rust
use crate::game::card::Card;

pub type PlayerId = u8; // 0-3

#[derive(Clone, Debug)]
pub struct Player {
    pub id: PlayerId,
    pub name: String,
    pub hand: Vec<Card>,
    pub is_ai: bool,
    pub is_stand: bool,
    pub is_connected: bool,
}

impl Player {
    pub fn new(id: PlayerId, name: impl Into<String>, is_ai: bool) -> Self {
        Player {
            id,
            name: name.into(),
            hand: vec![],
            is_ai,
            is_stand: false,
            is_connected: true,
        }
    }

    pub fn take_card(&mut self, card: Card) {
        self.hand.push(card);
    }

    pub fn clear_hand(&mut self) {
        self.hand.clear();
    }

    pub fn reset_for_new_round(&mut self) {
        self.hand.clear();
        self.is_stand = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_player_has_empty_hand() {
        let p = Player::new(0, "Alice", false);
        assert!(p.hand.is_empty());
        assert!(!p.is_stand);
    }

    #[test]
    fn take_card_adds_to_hand() {
        let mut p = Player::new(0, "Alice", false);
        p.take_card(Card { rank: 5 });
        assert_eq!(p.hand.len(), 1);
        assert_eq!(p.hand[0].rank, 5);
    }

    #[test]
    fn reset_clears_hand_and_stand() {
        let mut p = Player::new(0, "Alice", false);
        p.take_card(Card { rank: 7 });
        p.is_stand = true;
        p.reset_for_new_round();
        assert!(p.hand.is_empty());
        assert!(!p.is_stand);
    }
}
```

**Step 2: 注册模块**

在 `src/game/mod.rs` 中添加：

```rust
pub mod card;
pub mod player;
```

**Step 3: 运行测试**

```bash
cargo test game::player
```

期望：3个测试全部 PASS

**Step 4: 提交**

```bash
git add src/game/player.rs src/game/mod.rs
git commit -m "feat: add Player type with unit tests"
```

---

## Task 5: 新建 `src/game/rules.rs`（纯函数，可完整测试）

**Files:**
- Create: `src/game/rules.rs`
- Modify: `src/game/mod.rs`

**Step 1: 创建文件**

```rust
use crate::game::card::Card;
use crate::game::player::{Player, PlayerId};

pub fn calc_score(hand: &[Card]) -> u8 {
    hand.iter().map(|c| c.rank).sum()
}

pub fn is_bust(hand: &[Card]) -> bool {
    calc_score(hand) > 21
}

/// 返回胜者的 PlayerId 列表（可能平局多人）
/// 规则：爆牌者淘汰，剩余者中点数最高者胜，同分平局
pub fn judge_winner(seats: &[Option<Player>; 4]) -> Vec<PlayerId> {
    let scores: Vec<(PlayerId, u8)> = seats
        .iter()
        .enumerate()
        .filter_map(|(i, seat)| {
            seat.as_ref().map(|p| (p.id, calc_score(&p.hand)))
        })
        .filter(|(_, score)| *score <= 21)
        .collect();

    if scores.is_empty() {
        // 所有人都爆牌，返回所有参与者（平局）
        return seats
            .iter()
            .filter_map(|s| s.as_ref().map(|p| p.id))
            .collect();
    }

    let max_score = scores.iter().map(|(_, s)| *s).max().unwrap_or(0);
    scores
        .into_iter()
        .filter(|(_, s)| *s == max_score)
        .map(|(id, _)| id)
        .collect()
}

/// 游戏结束条件：所有在座玩家都 stand，或牌组耗尽
pub fn is_round_over(seats: &[Option<Player>; 4], deck_empty: bool) -> bool {
    if deck_empty {
        return true;
    }
    seats.iter().filter_map(|s| s.as_ref()).all(|p| p.is_stand)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::card::Card;
    use crate::game::player::Player;

    fn card(rank: u8) -> Card { Card { rank } }
    fn player_with_hand(id: u8, ranks: &[u8]) -> Player {
        let mut p = Player::new(id, "test", false);
        for &r in ranks { p.take_card(card(r)); }
        p
    }

    #[test]
    fn calc_score_sums_ranks() {
        assert_eq!(calc_score(&[card(5), card(7)]), 12);
    }

    #[test]
    fn is_bust_over_21() {
        assert!(is_bust(&[card(11), card(11)]));
        assert!(!is_bust(&[card(10), card(11)]));
    }

    #[test]
    fn judge_winner_highest_wins() {
        let mut seats: [Option<Player>; 4] = [None, None, None, None];
        seats[0] = Some(player_with_hand(0, &[10, 8])); // 18
        seats[1] = Some(player_with_hand(1, &[9, 9]));  // 18
        seats[2] = Some(player_with_hand(2, &[7, 7]));  // 14
        let winners = judge_winner(&seats);
        assert_eq!(winners.len(), 2); // 平局
        assert!(winners.contains(&0));
        assert!(winners.contains(&1));
    }

    #[test]
    fn judge_winner_bust_excluded() {
        let mut seats: [Option<Player>; 4] = [None, None, None, None];
        seats[0] = Some(player_with_hand(0, &[11, 11])); // 22 爆牌
        seats[1] = Some(player_with_hand(1, &[9, 8]));   // 17
        let winners = judge_winner(&seats);
        assert_eq!(winners, vec![1]);
    }

    #[test]
    fn judge_winner_all_bust_returns_all() {
        let mut seats: [Option<Player>; 4] = [None, None, None, None];
        seats[0] = Some(player_with_hand(0, &[11, 11]));
        seats[1] = Some(player_with_hand(1, &[11, 11]));
        let winners = judge_winner(&seats);
        assert_eq!(winners.len(), 2);
    }

    #[test]
    fn round_over_when_all_stand() {
        let mut seats: [Option<Player>; 4] = [None, None, None, None];
        let mut p0 = Player::new(0, "a", false);
        let mut p1 = Player::new(1, "b", false);
        p0.is_stand = true;
        p1.is_stand = true;
        seats[0] = Some(p0);
        seats[1] = Some(p1);
        assert!(is_round_over(&seats, false));
    }
}
```

**Step 2: 注册模块**

```rust
// src/game/mod.rs
pub mod card;
pub mod player;
pub mod rules;
```

**Step 3: 运行测试**

```bash
cargo test game::rules
```

期望：6个测试全部 PASS

**Step 4: 提交**

```bash
git add src/game/rules.rs src/game/mod.rs
git commit -m "feat: add pure game rules with unit tests"
```

---

## Task 6: 新建 `src/game/events.rs`

**Files:**
- Create: `src/game/events.rs`
- Modify: `src/game/mod.rs`

**Step 1: 创建文件**

```rust
use bevy::prelude::*;
use crate::game::card::Card;
use crate::game::player::PlayerId;

// ── 逻辑层 → 渲染层 / 网络层 ──────────────────────────────

/// 一张牌被发给某玩家
#[derive(Event, Clone, Debug)]
pub struct CardDealt {
    pub player_id: PlayerId,
    pub card: Card,
}

/// 回合切换
#[derive(Event, Clone, Debug)]
pub struct TurnChanged {
    pub player_id: PlayerId,
}

/// 某玩家选择不要牌
#[derive(Event, Clone, Debug)]
pub struct PlayerStood {
    pub player_id: PlayerId,
}

/// 本局游戏结束
#[derive(Event, Clone, Debug)]
pub struct RoundOver {
    pub winners: Vec<PlayerId>,
    pub scores: [u8; 4], // 每个座位的得分，空座位为 0
}

/// 新一局开始（牌已重置）
#[derive(Event, Clone, Debug)]
pub struct RoundReset;

// ── 渲染层 / 网络层 → 逻辑层 ──────────────────────────────

/// 玩家动作指令（人类输入或AI决策或网络收到）
#[derive(Event, Clone, Debug)]
pub enum GameAction {
    Draw  { player_id: PlayerId },
    Stand { player_id: PlayerId },
    Quit,
    StartGame, // 房主开始游戏
}
```

**Step 2: 注册模块**

```rust
// src/game/mod.rs
pub mod card;
pub mod events;
pub mod player;
pub mod rules;
```

**Step 3: 验证编译**

```bash
cargo check
```

**Step 4: 提交**

```bash
git add src/game/events.rs src/game/mod.rs
git commit -m "feat: define game event types for logic/render decoupling"
```

---

## Task 7: 新建 `src/game/room.rs`（核心逻辑 Resource）

**Files:**
- Create: `src/game/room.rs`
- Modify: `src/game/mod.rs`

**Step 1: 创建文件**

```rust
use bevy::prelude::*;
use crate::game::card::Deck;
use crate::game::player::{Player, PlayerId};

#[derive(Clone, PartialEq, Debug, Default)]
pub enum GamePhase {
    #[default]
    WaitingForPlayers,
    Playing,
    GameOver,
}

#[derive(Resource)]
pub struct Room {
    pub seats: [Option<Player>; 4],
    pub deck: Deck,
    pub current_turn: usize, // 当前回合的 seat index
    pub phase: GamePhase,
}

impl Default for Room {
    fn default() -> Self {
        Room {
            seats: [None, None, None, None],
            deck: Deck::new(),
            current_turn: 0,
            phase: GamePhase::WaitingForPlayers,
        }
    }
}

impl Room {
    /// 在指定座位坐下一个玩家
    pub fn seat_player(&mut self, seat: usize, player: Player) {
        self.seats[seat] = Some(player);
    }

    /// 返回当前回合玩家的可变引用
    pub fn current_player_mut(&mut self) -> Option<&mut Player> {
        self.seats[self.current_turn].as_mut()
    }

    /// 推进到下一个有人的座位
    /// 返回新的 current_turn（seat index）
    pub fn advance_turn(&mut self) -> PlayerId {
        let total = self.seats.len();
        for i in 1..=total {
            let next = (self.current_turn + i) % total;
            if self.seats[next].is_some() {
                self.current_turn = next;
                return next as PlayerId;
            }
        }
        self.current_turn as PlayerId
    }

    /// 重置为新一局（保留玩家，重置手牌和牌组）
    pub fn reset_round(&mut self) {
        self.deck.reset();
        for seat in self.seats.iter_mut().flatten() {
            seat.reset_for_new_round();
        }
        self.current_turn = self.first_occupied_seat();
        self.phase = GamePhase::Playing;
    }

    pub fn first_occupied_seat(&self) -> usize {
        self.seats.iter().position(|s| s.is_some()).unwrap_or(0)
    }

    pub fn active_player_count(&self) -> usize {
        self.seats.iter().filter(|s| s.is_some()).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::player::Player;

    #[test]
    fn seat_player_fills_slot() {
        let mut room = Room::default();
        room.seat_player(0, Player::new(0, "Alice", false));
        assert!(room.seats[0].is_some());
    }

    #[test]
    fn advance_turn_skips_empty_seats() {
        let mut room = Room::default();
        room.seat_player(0, Player::new(0, "Alice", false));
        room.seat_player(2, Player::new(2, "Bob", false));
        room.current_turn = 0;
        let next = room.advance_turn();
        assert_eq!(next, 2); // 跳过空的 seat 1
    }

    #[test]
    fn reset_round_clears_hands() {
        let mut room = Room::default();
        let mut p = Player::new(0, "Alice", false);
        p.take_card(crate::game::card::Card { rank: 5 });
        room.seat_player(0, p);
        room.reset_round();
        assert!(room.seats[0].as_ref().unwrap().hand.is_empty());
    }
}
```

**Step 2: 注册模块**

```rust
// src/game/mod.rs
pub mod card;
pub mod events;
pub mod player;
pub mod room;
pub mod rules;
```

**Step 3: 运行测试**

```bash
cargo test game::room
```

期望：3个测试全部 PASS

**Step 4: 提交**

```bash
git add src/game/room.rs src/game/mod.rs
git commit -m "feat: add Room resource with turn management and unit tests"
```

---

## Task 8: 新建 `src/game/mod.rs` 完整版（GameLogicPlugin）

**Files:**
- Modify: `src/game/mod.rs`

**Step 1: 完善 mod.rs，注册逻辑插件和所有事件**

```rust
pub mod ai;
pub mod card;
pub mod events;
pub mod player;
pub mod room;
pub mod rules;

use bevy::prelude::*;
use events::*;
use room::Room;

pub struct GameLogicPlugin;

impl Plugin for GameLogicPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<Room>()
            .add_event::<CardDealt>()
            .add_event::<TurnChanged>()
            .add_event::<PlayerStood>()
            .add_event::<RoundOver>()
            .add_event::<RoundReset>()
            .add_event::<GameAction>()
            .add_plugins(ai::AiPlugin)
            .add_systems(Update, process_game_actions);
    }
}

/// 核心逻辑系统：处理 GameAction，更新 Room，发出逻辑事件
fn process_game_actions(
    mut actions: EventReader<GameAction>,
    mut room: ResMut<Room>,
    mut card_dealt: EventWriter<CardDealt>,
    mut turn_changed: EventWriter<TurnChanged>,
    mut player_stood: EventWriter<PlayerStood>,
    mut round_over: EventWriter<RoundOver>,
    mut round_reset: EventWriter<RoundReset>,
) {
    use GameAction::*;
    use room::GamePhase;

    for action in actions.read() {
        if room.phase != GamePhase::Playing {
            continue;
        }
        match action {
            Draw { player_id } => {
                let pid = *player_id;
                // 验证是否轮到该玩家
                if room.current_turn != pid as usize { continue; }
                if let Some(card) = room.deck.pop() {
                    if let Some(player) = room.seats[pid as usize].as_mut() {
                        player.take_card(card);
                        card_dealt.write(CardDealt { player_id: pid, card });
                    }
                    // 检查是否游戏结束
                    if rules::is_round_over(&room.seats, room.deck.remaining() == 0) {
                        emit_round_over(&room, &mut round_over);
                        room.phase = GamePhase::GameOver;
                    } else {
                        let next = room.advance_turn();
                        turn_changed.write(TurnChanged { player_id: next });
                    }
                }
            }
            Stand { player_id } => {
                let pid = *player_id;
                if room.current_turn != pid as usize { continue; }
                if let Some(player) = room.seats[pid as usize].as_mut() {
                    player.is_stand = true;
                    player_stood.write(PlayerStood { player_id: pid });
                }
                if rules::is_round_over(&room.seats, room.deck.remaining() == 0) {
                    emit_round_over(&room, &mut round_over);
                    room.phase = GamePhase::GameOver;
                } else {
                    let next = room.advance_turn();
                    turn_changed.write(TurnChanged { player_id: next });
                }
            }
            StartGame => {
                room.reset_round();
                // 初始发两张牌给每个玩家
                let player_ids: Vec<u8> = room.seats.iter().enumerate()
                    .filter_map(|(i, s)| s.as_ref().map(|_| i as u8))
                    .collect();
                for _ in 0..2 {
                    for &pid in &player_ids {
                        if let Some(card) = room.deck.pop() {
                            if let Some(player) = room.seats[pid as usize].as_mut() {
                                player.take_card(card);
                                card_dealt.write(CardDealt { player_id: pid, card });
                            }
                        }
                    }
                }
                let first = room.first_occupied_seat() as u8;
                turn_changed.write(TurnChanged { player_id: first });
                round_reset.write(RoundReset);
            }
            Quit => {
                // 由渲染层处理 WorldState 切换，逻辑层不感知
            }
        }
    }
}

fn emit_round_over(room: &Room, writer: &mut EventWriter<RoundOver>) {
    let winners = rules::judge_winner(&room.seats);
    let mut scores = [0u8; 4];
    for (i, seat) in room.seats.iter().enumerate() {
        if let Some(p) = seat {
            scores[i] = rules::calc_score(&p.hand);
        }
    }
    writer.write(RoundOver { winners, scores });
}
```

**Step 2: 验证编译（ai 模块还不存在，先创建占位）**

创建 `src/game/ai.rs`（空插件占位，Task 9 完善）：

```rust
use bevy::prelude::*;
pub struct AiPlugin;
impl Plugin for AiPlugin {
    fn build(&self, _app: &mut App) {}
}
```

```bash
cargo check
```

**Step 3: 提交**

```bash
git add src/game/mod.rs src/game/ai.rs
git commit -m "feat: add GameLogicPlugin with core action processing system"
```

---

## Task 9: 完善 `src/game/ai.rs`（AI决策系统）

**Files:**
- Modify: `src/game/ai.rs`

**Step 1: 实现 AI 插件**

```rust
use bevy::prelude::*;
use crate::game::events::{GameAction, TurnChanged};
use crate::game::room::Room;
use crate::game::rules::calc_score;

pub struct AiPlugin;

impl Plugin for AiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, ai_decision);
    }
}

/// AI 决策系统：监听 TurnChanged，若当前玩家是 AI，自动产生 GameAction
/// 策略：分数 < 17 则抽牌，否则不要牌（与原版一致）
fn ai_decision(
    mut turn_ev: EventReader<TurnChanged>,
    room: Res<Room>,
    mut actions: EventWriter<GameAction>,
) {
    for ev in turn_ev.read() {
        let Some(player) = &room.seats[ev.player_id as usize] else { continue };
        if !player.is_ai { continue; }

        let score = calc_score(&player.hand);
        let action = if score < 17 {
            GameAction::Draw { player_id: ev.player_id }
        } else {
            GameAction::Stand { player_id: ev.player_id }
        };
        actions.write(action);
    }
}
```

**Step 2: 验证编译**

```bash
cargo check
```

**Step 3: 提交**

```bash
git add src/game/ai.rs
git commit -m "feat: implement AI decision system (draw < 17, stand >= 17)"
```

---

## Task 10: 重构 `src/plugins/` 目录结构

**Files:**
- Modify: `src/plugins/mod.rs`
- Create: `src/plugins/menu/mod.rs`（迁移原 menu.rs）
- Create: `src/plugins/game/mod.rs`（重写）
- Create: `src/plugins/game/table.rs`
- Create: `src/plugins/game/renderer.rs`
- Create: `src/plugins/game/hud.rs`
- Create: `src/plugins/game/actions.rs`
- Create: `src/plugins/game/popup.rs`
- Delete: `src/plugins/menu.rs`（内容迁移到 menu/mod.rs）

**Step 1: 更新 `src/plugins/mod.rs`**

```rust
pub mod game;
pub mod menu;

use bevy::prelude::*;
use crate::game::GameLogicPlugin;

#[derive(Default)]
pub struct KivaPlugins;

impl PluginGroup for KivaPlugins {
    fn build(self) -> bevy::app::PluginGroupBuilder {
        bevy::app::PluginGroupBuilder::start::<Self>()
            .add(GameLogicPlugin)
            .add(menu::MenuPlugin)
            .add(game::GamePlugin)
    }
}
```

**Step 2: 创建 `src/plugins/menu/mod.rs`**

将原 `src/plugins/menu.rs` 内容复制过来，只修改 `WorldState` 的引用路径和新增 Lobby 状态跳转：

```rust
use crate::states::WorldState;
use bevy::prelude::*;

const TEXT_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);
const RED_COLOR: Color = Color::srgb(1.0, 0.0, 0.0);
const BACKGROUND_COLOR: Color = Color::srgb(0.0, 0.3, 0.4);

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(WorldState::Menu), setup)
            .add_systems(Update, menu_action.run_if(in_state(WorldState::Menu)))
            .add_systems(OnExit(WorldState::Menu), cleanup);
    }
}

#[derive(Component)]
struct OnMenuScreen;

#[derive(Component)]
enum MenuButtonAction {
    Play,
    Quit,
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let default_font = asset_server.load("fonts/WenCangShuFang-2.ttf");
    let button_node = Node {
        width: Val::Px(300.0),
        height: Val::Px(65.0),
        margin: UiRect::all(Val::Px(20.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };
    let title_font = TextFont { font: default_font.clone(), font_size: 80.0, ..default() };
    let button_text_font = TextFont { font: default_font.clone(), font_size: 33.0, ..default() };

    commands.spawn((
        Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        BackgroundColor(BACKGROUND_COLOR),
        OnMenuScreen,
        children![
            (Text::new("四方诛杀"), title_font.clone(), TextColor(TEXT_COLOR)),
            (
                Button, button_node.clone(), BackgroundColor(RED_COLOR),
                MenuButtonAction::Play,
                children![(Text::new("开始游戏"), button_text_font.clone(), TextColor(TEXT_COLOR))]
            ),
            (
                Button, button_node.clone(), BackgroundColor(RED_COLOR),
                MenuButtonAction::Quit,
                children![(Text::new("退出"), button_text_font.clone(), TextColor(TEXT_COLOR))]
            ),
        ],
    ));
    commands.spawn((AudioPlayer::new(asset_server.load("sounds/menu_bgm.ogg")), OnMenuScreen));
}

fn menu_action(
    interaction_query: Query<(&Interaction, &MenuButtonAction), (Changed<Interaction>, With<Button>)>,
    mut app_exit_events: EventWriter<AppExit>,
    mut world_state: ResMut<NextState<WorldState>>,
) {
    for (interaction, action) in &interaction_query {
        if *interaction == Interaction::Pressed {
            match action {
                MenuButtonAction::Quit => { app_exit_events.write(AppExit::Success); }
                // 暂时直接跳 Game，Lobby 在 Task 14 实现
                MenuButtonAction::Play => { world_state.set(WorldState::Game); }
            }
        }
    }
}

fn cleanup(to_despawn: Query<Entity, With<OnMenuScreen>>, mut commands: Commands) {
    for entity in &to_despawn {
        commands.entity(entity).despawn();
    }
}
```

**Step 3: 删除旧文件**

```bash
rm src/plugins/menu.rs
rm src/plugins/game/core.rs
rm src/plugins/game/node.rs
```

**Step 4: 验证编译**

```bash
cargo check
```

（此时 game 插件还是空的，先让编译通过）

**Step 5: 提交**

```bash
git add -A
git commit -m "refactor: restructure plugins directory, migrate MenuPlugin"
```

---

## Task 11: 实现 `src/plugins/game/table.rs`（牌桌布局）

**Files:**
- Create: `src/plugins/game/table.rs`

**Step 1: 创建文件**

```rust
use bevy::prelude::*;
use crate::game::room::Room;

const TABLE_COLOR: Color = Color::srgb(0.05, 0.4, 0.3);
const SEAT_COLOR: Color = Color::srgba(1.0, 0.4, 0.3, 0.3);

/// 标记所有游戏界面实体，用于退出时批量 despawn
#[derive(Component)]
pub struct OnGameScreen;

/// 渲染层维护的 player_id → seat Entity 映射
#[derive(Resource, Default)]
pub struct PlayerEntityMap {
    pub seat_entities: [Option<Entity>; 4],
}

pub fn setup_table(
    mut commands: Commands,
    room: Res<Room>,
    mut map: ResMut<PlayerEntityMap>,
    asset_server: Res<AssetServer>,
) {
    let font = asset_server.load("fonts/WenCangShuFang-2.ttf");

    // 根节点：全屏绿色桌面，2x2 网格布局
    let root = commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            ..default()
        },
        BackgroundColor(TABLE_COLOR),
        OnGameScreen,
    )).id();

    // 上半行（seat 0 和 seat 1）
    let top_row = commands.spawn(Node {
        width: Val::Percent(100.0),
        height: Val::Percent(45.0),
        flex_direction: FlexDirection::Row,
        ..default()
    }).id();

    // 下半行（seat 2 和 seat 3）
    let bottom_row = commands.spawn(Node {
        width: Val::Percent(100.0),
        height: Val::Percent(45.0),
        flex_direction: FlexDirection::Row,
        ..default()
    }).id();

    commands.entity(root).add_children(&[top_row, bottom_row]);

    // 生成4个座位
    let seat_indices = [(0, top_row), (1, top_row), (2, bottom_row), (3, bottom_row)];
    for (seat_idx, parent_row) in seat_indices {
        let player_name = room.seats[seat_idx]
            .as_ref()
            .map(|p| p.name.clone())
            .unwrap_or_else(|| format!("空位 {}", seat_idx + 1));

        let seat_entity = commands.spawn((
            Node {
                width: Val::Percent(50.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                flex_wrap: FlexWrap::Wrap,
                padding: UiRect::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(SEAT_COLOR),
        )).with_children(|parent| {
            // 玩家名标签
            parent.spawn((
                Text::new(player_name),
                TextFont { font: font.clone(), font_size: 24.0, ..default() },
                TextColor(Color::WHITE),
                SeatLabel { seat: seat_idx },
            ));
        }).id();

        commands.entity(parent_row).add_children(&[seat_entity]);
        map.seat_entities[seat_idx] = Some(seat_entity);
    }
}

/// 标记座位名称文字，供 hud.rs 查询
#[derive(Component)]
pub struct SeatLabel {
    pub seat: usize,
}

pub fn cleanup_table(
    to_despawn: Query<Entity, With<OnGameScreen>>,
    mut commands: Commands,
    mut map: ResMut<PlayerEntityMap>,
) {
    for entity in &to_despawn {
        commands.entity(entity).despawn();
    }
    *map = PlayerEntityMap::default();
}
```

**Step 2: 验证编译**

```bash
cargo check
```

**Step 3: 提交**

```bash
git add src/plugins/game/table.rs
git commit -m "feat: implement 4-seat table layout with PlayerEntityMap"
```

---

## Task 12: 实现 `src/plugins/game/renderer.rs`（事件驱动卡牌渲染）

**Files:**
- Create: `src/plugins/game/renderer.rs`

**Step 1: 创建文件**

```rust
use bevy::prelude::*;
use crate::game::events::{CardDealt, RoundReset};
use crate::game::card::Card;
use super::table::PlayerEntityMap;

const CARD_COLOR: Color = Color::srgb(0.95, 0.95, 0.95);
const TEXT_COLOR: Color = Color::srgb(0.4, 0.4, 0.4);

/// 标记卡牌UI实体，存储所属座位，用于 RoundReset 时批量清除
#[derive(Component)]
pub struct CardUiMarker {
    pub seat: usize,
}

pub fn on_card_dealt(
    mut ev: EventReader<CardDealt>,
    map: Res<PlayerEntityMap>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    for ev in ev.read() {
        let seat = ev.player_id as usize;
        let Some(seat_entity) = map.seat_entities[seat] else { continue };
        commands.entity(seat_entity).with_children(|parent| {
            parent.spawn(card_bundle(ev.card, seat, &asset_server));
        });
    }
}

pub fn on_round_reset(
    mut ev: EventReader<RoundReset>,
    card_query: Query<Entity, With<CardUiMarker>>,
    mut commands: Commands,
) {
    for _ in ev.read() {
        for entity in &card_query {
            commands.entity(entity).despawn();
        }
    }
}

fn card_bundle(card: Card, seat: usize, asset_server: &AssetServer) -> impl Bundle {
    (
        Node {
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            margin: UiRect::all(Val::Px(4.0)),
            width: Val::Px(60.0),
            height: Val::Px(90.0),
            ..default()
        },
        BackgroundColor(CARD_COLOR),
        CardUiMarker { seat },
        children![(
            Text::new(card.rank.to_string()),
            TextFont {
                font: asset_server.load("fonts/WenCangShuFang-2.ttf"),
                font_size: 48.0,
                ..default()
            },
            TextColor(TEXT_COLOR),
        )],
    )
}
```

**Step 2: 验证编译**

```bash
cargo check
```

**Step 3: 提交**

```bash
git add src/plugins/game/renderer.rs
git commit -m "feat: implement event-driven card renderer"
```

---

## Task 13: 实现 `src/plugins/game/hud.rs`、`actions.rs`、`popup.rs`

**Files:**
- Create: `src/plugins/game/hud.rs`
- Create: `src/plugins/game/actions.rs`
- Create: `src/plugins/game/popup.rs`

**Step 1: 创建 `hud.rs`**

```rust
use bevy::prelude::*;
use crate::game::events::{TurnChanged, RoundOver};
use crate::game::room::Room;
use crate::game::rules::calc_score;

/// 分数标签组件，标记哪个座位
#[derive(Component)]
pub struct ScoreLabel(pub usize);

/// 回合指示器
#[derive(Component)]
pub struct TurnIndicator;

pub fn spawn_hud(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/WenCangShuFang-2.ttf");
    // HUD 覆盖层（绝对定位，叠在桌面上方）
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(10.0),
            left: Val::Px(10.0),
            flex_direction: FlexDirection::Column,
            ..default()
        },
        crate::plugins::game::table::OnGameScreen,
    )).with_children(|parent| {
        for seat in 0..4 {
            parent.spawn((
                Text::new(format!("P{}: 0分", seat + 1)),
                TextFont { font: font.clone(), font_size: 20.0, ..default() },
                TextColor(Color::WHITE),
                ScoreLabel(seat),
            ));
        }
        parent.spawn((
            Text::new("等待开始..."),
            TextFont { font: font.clone(), font_size: 24.0, ..default() },
            TextColor(Color::srgb(1.0, 1.0, 0.0)),
            TurnIndicator,
        ));
    });
}

pub fn update_scores(
    room: Res<Room>,
    mut query: Query<(&mut Text, &ScoreLabel)>,
) {
    if !room.is_changed() { return; }
    for (mut text, label) in &mut query {
        let score = room.seats[label.0]
            .as_ref()
            .map(|p| calc_score(&p.hand))
            .unwrap_or(0);
        let name = room.seats[label.0]
            .as_ref()
            .map(|p| p.name.clone())
            .unwrap_or_else(|| format!("P{}", label.0 + 1));
        *text = Text::new(format!("{}: {}分", name, score));
    }
}

pub fn on_turn_changed(
    mut ev: EventReader<TurnChanged>,
    room: Res<Room>,
    mut query: Query<&mut Text, With<TurnIndicator>>,
) {
    for ev in ev.read() {
        let name = room.seats[ev.player_id as usize]
            .as_ref()
            .map(|p| p.name.clone())
            .unwrap_or_default();
        for mut text in &mut query {
            *text = Text::new(format!("轮到 {} 出牌", name));
        }
    }
}
```

**Step 2: 创建 `actions.rs`**

```rust
use bevy::prelude::*;
use crate::game::events::GameAction;
use crate::states::WorldState;

#[derive(Component, Clone)]
pub enum ActionButton {
    Draw,
    Stand,
    Quit,
}

pub fn spawn_action_bar(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/WenCangShuFang-2.ttf");
    let btn_node = Node {
        width: Val::Px(160.0),
        height: Val::Px(50.0),
        margin: UiRect::all(Val::Px(5.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };
    let btn_color = Color::srgb(0.8, 0.8, 0.8);
    let text_color = Color::srgb(0.2, 0.2, 0.2);
    let text_font = TextFont { font: font.clone(), font_size: 28.0, ..default() };

    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(10.0),
            right: Val::Px(10.0),
            flex_direction: FlexDirection::Row,
            ..default()
        },
        crate::plugins::game::table::OnGameScreen,
        children![
            (Button, btn_node.clone(), BackgroundColor(btn_color), ActionButton::Draw,
             children![(Text::new("抽牌"), text_font.clone(), TextColor(text_color))]),
            (Button, btn_node.clone(), BackgroundColor(btn_color), ActionButton::Stand,
             children![(Text::new("不要牌"), text_font.clone(), TextColor(text_color))]),
            (Button, btn_node.clone(), BackgroundColor(btn_color), ActionButton::Quit,
             children![(Text::new("退出"), text_font.clone(), TextColor(text_color))]),
        ],
    ));
}

/// 将按钮点击转换为 GameAction 事件（player_id 固定为 0，即本地玩家）
/// 多人联网时此处改为从 NetRole resource 读取本地玩家 ID
pub fn handle_button_input(
    query: Query<(&Interaction, &ActionButton), (Changed<Interaction>, With<Button>)>,
    mut actions: EventWriter<GameAction>,
    mut world_state: ResMut<NextState<WorldState>>,
) {
    for (interaction, button) in &query {
        if *interaction != Interaction::Pressed { continue; }
        match button {
            ActionButton::Draw  => { actions.write(GameAction::Draw { player_id: 0 }); }
            ActionButton::Stand => { actions.write(GameAction::Stand { player_id: 0 }); }
            ActionButton::Quit  => { world_state.set(WorldState::Menu); }
        }
    }
}
```

**Step 3: 创建 `popup.rs`**

```rust
use bevy::prelude::*;
use crate::game::events::{GameAction, RoundOver};
use super::table::OnGameScreen;

#[derive(Component)]
struct GameOverPopup;

pub fn on_round_over(
    mut ev: EventReader<RoundOver>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    for ev in ev.read() {
        let font = asset_server.load("fonts/WenCangShuFang-2.ttf");
        let result_text = if ev.winners.len() > 1 {
            format!("平局！胜者：{:?}", ev.winners)
        } else if let Some(&w) = ev.winners.first() {
            format!("P{} 获胜！", w + 1)
        } else {
            "游戏结束".to_string()
        };

        commands.spawn((
            GameOverPopup,
            OnGameScreen,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
            children![(
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    padding: UiRect::all(Val::Px(40.0)),
                    ..default()
                },
                children![
                    (
                        Text::new(result_text),
                        TextFont { font: font.clone(), font_size: 60.0, ..default() },
                        TextColor(Color::WHITE),
                    ),
                    (
                        Button,
                        Node {
                            width: Val::Px(200.0), height: Val::Px(55.0),
                            margin: UiRect::top(Val::Px(30.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.8, 0.8, 0.8)),
                        ContinueButton,
                        children![(
                            Text::new("继续"),
                            TextFont { font: font.clone(), font_size: 33.0, ..default() },
                            TextColor(Color::srgb(0.2, 0.2, 0.2)),
                        )],
                    ),
                ],
            )],
        ));
    }
}

#[derive(Component)]
struct ContinueButton;

pub fn handle_continue(
    query: Query<&Interaction, (Changed<Interaction>, With<ContinueButton>)>,
    popup_query: Query<Entity, With<GameOverPopup>>,
    mut commands: Commands,
    mut actions: EventWriter<GameAction>,
) {
    for interaction in &query {
        if *interaction == Interaction::Pressed {
            for entity in &popup_query {
                commands.entity(entity).despawn();
            }
            actions.write(GameAction::StartGame);
        }
    }
}
```

**Step 4: 验证编译**

```bash
cargo check
```

**Step 5: 提交**

```bash
git add src/plugins/game/hud.rs src/plugins/game/actions.rs src/plugins/game/popup.rs
git commit -m "feat: implement HUD, action bar, and game-over popup"
```

---

## Task 14: 组装 `src/plugins/game/mod.rs`（GamePlugin）

**Files:**
- Create: `src/plugins/game/mod.rs`

**Step 1: 创建文件**

```rust
pub mod actions;
pub mod hud;
pub mod popup;
pub mod renderer;
pub mod table;

use bevy::prelude::*;
use crate::states::WorldState;
use table::{PlayerEntityMap, setup_table, cleanup_table};

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<PlayerEntityMap>()
            // 进入游戏时
            .add_systems(OnEnter(WorldState::Game), (
                setup_table,
                hud::spawn_hud,
                actions::spawn_action_bar,
                start_game_immediately, // 单机模式直接开始
            ).chain())
            // 游戏进行中
            .add_systems(Update, (
                renderer::on_card_dealt,
                renderer::on_round_reset,
                hud::update_scores,
                hud::on_turn_changed,
                actions::handle_button_input,
                popup::on_round_over,
                popup::handle_continue,
            ).run_if(in_state(WorldState::Game)))
            // 退出游戏时
            .add_systems(OnExit(WorldState::Game), cleanup_table);
    }
}

/// 单机模式：进入游戏后立即填充玩家并开始
fn start_game_immediately(
    mut room: ResMut<crate::game::room::Room>,
    mut actions: EventWriter<crate::game::events::GameAction>,
) {
    use crate::game::player::Player;
    // seat 0: 本地玩家
    room.seat_player(0, Player::new(0, "玩家", false));
    // seat 1-3: AI
    room.seat_player(1, Player::new(1, "AI甲", true));
    room.seat_player(2, Player::new(2, "AI乙", true));
    room.seat_player(3, Player::new(3, "AI丙", true));

    actions.write(crate::game::events::GameAction::StartGame);
}
```

**Step 2: 在 `src/main.rs` 中注册 game 模块**

确认 `src/main.rs` 顶部有：

```rust
mod game;
mod plugins;
mod states;
```

**Step 3: 完整编译并运行**

```bash
cargo run
```

期望：游戏窗口打开，主菜单显示，点击"开始游戏"进入4人牌桌，AI自动出牌，游戏可以完整运行一局。

**Step 4: 提交**

```bash
git add src/plugins/game/mod.rs src/main.rs
git commit -m "feat: assemble GamePlugin, single-player 4-seat mode working"
```

---

## Task 15: 实现网络协议层 `src/net/protocol.rs`

**Files:**
- Create: `src/net/mod.rs`
- Create: `src/net/protocol.rs`

**Step 1: 创建 `src/net/mod.rs`**

```rust
pub mod protocol;
pub mod host;
pub mod client;
```

**Step 2: 创建 `src/net/protocol.rs`**

```rust
use serde::{Deserialize, Serialize};
use crate::game::card::Card;
use crate::game::player::PlayerId;
use crate::game::room::GamePhase;

pub const NET_PORT: u16 = 7777;
pub const BEACON_PORT: u16 = 7778;

/// 玩家快照（可序列化，不含 Entity）
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PlayerSnapshot {
    pub id: PlayerId,
    pub name: String,
    pub hand: Vec<Card>,
    pub is_ai: bool,
    pub is_stand: bool,
}

/// 房间快照（全量状态，用于新客户端加入时同步）
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RoomSnapshot {
    pub seats: [Option<PlayerSnapshot>; 4],
    pub deck_remaining: usize,
    pub current_turn: usize,
    pub phase: GamePhase,
}

/// 局域网发现 beacon（Host 广播）
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RoomBeacon {
    pub room_name: String,
    pub host_addr: String, // "192.168.x.x:7777"
    pub player_count: usize,
    pub max_players: usize,
}

/// 网络消息枚举
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum NetMessage {
    // 客户端 → Host
    JoinRequest { name: String },
    PlayerAction { player_id: PlayerId, action_type: NetAction },

    // Host → 客户端
    JoinAccepted { assigned_id: PlayerId, snapshot: RoomSnapshot },
    JoinRejected { reason: String },
    ActionBroadcast { player_id: PlayerId, action_type: NetAction },
    FullSync { snapshot: RoomSnapshot },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum NetAction {
    Draw,
    Stand,
}

/// 序列化消息为字节
pub fn encode(msg: &NetMessage) -> Vec<u8> {
    bincode::serde::encode_to_vec(msg, bincode::config::standard()).unwrap_or_default()
}

/// 反序列化字节为消息
pub fn decode(bytes: &[u8]) -> Option<NetMessage> {
    bincode::serde::decode_from_slice(bytes, bincode::config::standard())
        .ok()
        .map(|(msg, _)| msg)
}

// serde 需要 Card 和 GamePhase 实现 Serialize/Deserialize
// 在各自文件中添加 derive 即可（见下一步）
```

**Step 3: 为 Card 和 GamePhase 添加 serde derive**

在 `src/game/card.rs` 的 `Card` 和 `Deck` 上添加：

```rust
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Card { pub rank: u8 }
```

在 `src/game/room.rs` 的 `GamePhase` 上添加：

```rust
#[derive(Clone, PartialEq, Debug, Default, Serialize, Deserialize)]
pub enum GamePhase { ... }
```

**Step 4: 在 `src/main.rs` 注册 net 模块**

```rust
mod game;
mod net;
mod plugins;
mod states;
```

**Step 5: 验证编译**

```bash
cargo check
```

**Step 6: 提交**

```bash
git add src/net/ src/game/card.rs src/game/room.rs src/main.rs
git commit -m "feat: define network protocol with serde serialization"
```

---

## Task 16: 实现 `src/net/host.rs`（Host 端 UDP 服务）

**Files:**
- Create: `src/net/host.rs`

**Step 1: 创建文件**

```rust
use std::net::UdpSocket;
use std::sync::{Arc, Mutex};
use bevy::prelude::*;
use crate::game::events::GameAction;
use crate::game::player::PlayerId;
use crate::net::protocol::{self, NetMessage, NetAction, NET_PORT, BEACON_PORT};
use crate::states::WorldState;

/// Host 模式下的网络资源
#[derive(Resource)]
pub struct HostNet {
    socket: Arc<UdpSocket>,
    /// 已连接客户端的地址列表（seat_index → addr）
    pub clients: Arc<Mutex<Vec<Option<String>>>>,
}

pub struct HostPlugin;

impl Plugin for HostPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(WorldState::Lobby), start_host)
            .add_systems(Update, (
                receive_client_actions,
                broadcast_beacon,
            ).run_if(resource_exists::<HostNet>))
            .add_systems(OnExit(WorldState::Game), stop_host);
    }
}

fn start_host(mut commands: Commands) {
    let addr = format!("0.0.0.0:{}", NET_PORT);
    match UdpSocket::bind(&addr) {
        Ok(socket) => {
            socket.set_nonblocking(true).ok();
            info!("Host listening on {}", addr);
            commands.insert_resource(HostNet {
                socket: Arc::new(socket),
                clients: Arc::new(Mutex::new(vec![None; 4])),
            });
        }
        Err(e) => error!("Failed to bind host socket: {}", e),
    }
}

fn stop_host(mut commands: Commands) {
    commands.remove_resource::<HostNet>();
}

fn receive_client_actions(
    host: Res<HostNet>,
    mut actions: EventWriter<GameAction>,
) {
    let mut buf = [0u8; 1024];
    loop {
        match host.socket.recv_from(&mut buf) {
            Ok((len, src)) => {
                if let Some(msg) = protocol::decode(&buf[..len]) {
                    match msg {
                        NetMessage::JoinRequest { name } => {
                            info!("Join request from {} ({})", name, src);
                            // TODO: 分配 seat，发送 JoinAccepted
                        }
                        NetMessage::PlayerAction { player_id, action_type } => {
                            let action = match action_type {
                                NetAction::Draw  => GameAction::Draw { player_id },
                                NetAction::Stand => GameAction::Stand { player_id },
                            };
                            actions.write(action);
                        }
                        _ => {}
                    }
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
            Err(e) => { error!("Host recv error: {}", e); break; }
        }
    }
}

/// 广播 beacon 让客户端发现房间（每秒一次）
fn broadcast_beacon(
    host: Res<HostNet>,
    time: Res<Time>,
    mut timer: Local<f32>,
) {
    *timer += time.delta_secs();
    if *timer < 1.0 { return; }
    *timer = 0.0;

    let beacon = protocol::RoomBeacon {
        room_name: "四方诛杀".to_string(),
        host_addr: format!("0.0.0.0:{}", NET_PORT),
        player_count: 1,
        max_players: 4,
    };
    let bytes = protocol::encode(&NetMessage::JoinRequest { name: "beacon".to_string() });
    let broadcast_addr = format!("255.255.255.255:{}", BEACON_PORT);
    host.socket.send_to(&bytes, &broadcast_addr).ok();
}

/// 广播 GameAction 给所有已连接客户端（供逻辑系统调用）
pub fn broadcast_action(host: &HostNet, player_id: PlayerId, action: &NetAction) {
    let msg = NetMessage::ActionBroadcast { player_id, action_type: action.clone() };
    let bytes = protocol::encode(&msg);
    if let Ok(clients) = host.clients.lock() {
        for addr in clients.iter().flatten() {
            host.socket.send_to(&bytes, addr).ok();
        }
    }
}
```

**Step 2: 创建占位 `src/net/client.rs`**

```rust
// TODO: Task 17 实现
use bevy::prelude::*;
pub struct ClientPlugin;
impl Plugin for ClientPlugin {
    fn build(&self, _app: &mut App) {}
}
```

**Step 3: 验证编译**

```bash
cargo check
```

**Step 4: 提交**

```bash
git add src/net/host.rs src/net/client.rs
git commit -m "feat: implement Host UDP server for LAN discovery and action relay"
```

---

## Task 17: 实现 `src/net/client.rs`（Client 端）

**Files:**
- Modify: `src/net/client.rs`

**Step 1: 实现文件**

```rust
use std::net::UdpSocket;
use bevy::prelude::*;
use crate::game::events::GameAction;
use crate::game::player::PlayerId;
use crate::net::protocol::{self, NetMessage, NetAction, NET_PORT, BEACON_PORT};

#[derive(Resource)]
pub struct ClientNet {
    socket: UdpSocket,
    pub host_addr: String,
    pub local_player_id: PlayerId,
}

pub struct ClientPlugin;

impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, (
                receive_from_host,
                scan_for_rooms,
            ).run_if(resource_exists::<ClientNet>));
    }
}

pub fn connect_to_host(commands: &mut Commands, host_addr: &str, player_name: &str) {
    let socket = UdpSocket::bind("0.0.0.0:0").expect("bind client socket");
    socket.set_nonblocking(true).ok();

    let join_msg = NetMessage::JoinRequest { name: player_name.to_string() };
    let bytes = protocol::encode(&join_msg);
    socket.send_to(&bytes, host_addr).ok();

    commands.insert_resource(ClientNet {
        socket,
        host_addr: host_addr.to_string(),
        local_player_id: 0, // 由 JoinAccepted 更新
    });
}

fn receive_from_host(
    mut client: ResMut<ClientNet>,
    mut actions: EventWriter<GameAction>,
) {
    let mut buf = [0u8; 4096];
    loop {
        match client.socket.recv(&mut buf) {
            Ok(len) => {
                if let Some(msg) = protocol::decode(&buf[..len]) {
                    match msg {
                        NetMessage::JoinAccepted { assigned_id, .. } => {
                            client.local_player_id = assigned_id;
                            info!("Joined as player {}", assigned_id);
                        }
                        NetMessage::ActionBroadcast { player_id, action_type } => {
                            let action = match action_type {
                                NetAction::Draw  => GameAction::Draw { player_id },
                                NetAction::Stand => GameAction::Stand { player_id },
                            };
                            actions.write(action);
                        }
                        _ => {}
                    }
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
            Err(e) => { error!("Client recv error: {}", e); break; }
        }
    }
}

/// 扫描局域网 beacon（监听广播端口）
fn scan_for_rooms(
    // 此处简化：实际应用中用独立线程监听 BEACON_PORT
    // 这里只是占位，Lobby UI 在 Task 18 实现
) {}

/// Client 发送本地玩家动作给 Host
pub fn send_action(client: &ClientNet, action: &NetAction) {
    let msg = NetMessage::PlayerAction {
        player_id: client.local_player_id,
        action_type: action.clone(),
    };
    let bytes = protocol::encode(&msg);
    client.socket.send_to(&bytes, &client.host_addr).ok();
}
```

**Step 2: 验证编译**

```bash
cargo check
```

**Step 3: 提交**

```bash
git add src/net/client.rs
git commit -m "feat: implement Client UDP connection and action forwarding"
```

---

## Task 18: 实现大厅界面 `src/plugins/menu/lobby.rs`

**Files:**
- Create: `src/plugins/menu/lobby.rs`
- Modify: `src/plugins/menu/mod.rs`

**Step 1: 创建 `lobby.rs`**

```rust
use bevy::prelude::*;
use crate::states::WorldState;
use crate::game::room::Room;
use crate::game::player::Player;
use crate::game::events::GameAction;

const BG_COLOR: Color = Color::srgb(0.05, 0.1, 0.2);
const BTN_COLOR: Color = Color::srgb(0.2, 0.5, 0.8);
const TEXT_COLOR: Color = Color::WHITE;

pub struct LobbyPlugin;

impl Plugin for LobbyPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(WorldState::Lobby), setup_lobby)
            .add_systems(Update, lobby_action.run_if(in_state(WorldState::Lobby)))
            .add_systems(OnExit(WorldState::Lobby), cleanup_lobby);
    }
}

#[derive(Component)]
struct OnLobbyScreen;

#[derive(Component)]
enum LobbyAction {
    HostGame,
    JoinGame,
    AddAi,
    StartGame,
    Back,
}

fn setup_lobby(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/WenCangShuFang-2.ttf");
    let btn = Node {
        width: Val::Px(280.0), height: Val::Px(60.0),
        margin: UiRect::all(Val::Px(10.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };
    let tf = TextFont { font: font.clone(), font_size: 30.0, ..default() };

    commands.spawn((
        Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        BackgroundColor(BG_COLOR),
        OnLobbyScreen,
        children![
            (Text::new("游戏大厅"), TextFont { font: font.clone(), font_size: 60.0, ..default() }, TextColor(TEXT_COLOR)),
            (Button, btn.clone(), BackgroundColor(BTN_COLOR), LobbyAction::HostGame,
             children![(Text::new("创建房间（主机）"), tf.clone(), TextColor(TEXT_COLOR))]),
            (Button, btn.clone(), BackgroundColor(BTN_COLOR), LobbyAction::JoinGame,
             children![(Text::new("加入房间"), tf.clone(), TextColor(TEXT_COLOR))]),
            (Button, btn.clone(), BackgroundColor(BTN_COLOR), LobbyAction::AddAi,
             children![(Text::new("添加AI玩家"), tf.clone(), TextColor(TEXT_COLOR))]),
            (Button, btn.clone(), BackgroundColor(Color::srgb(0.3, 0.7, 0.3)), LobbyAction::StartGame,
             children![(Text::new("开始游戏"), tf.clone(), TextColor(TEXT_COLOR))]),
            (Button, btn.clone(), BackgroundColor(Color::srgb(0.5, 0.5, 0.5)), LobbyAction::Back,
             children![(Text::new("返回"), tf.clone(), TextColor(TEXT_COLOR))]),
        ],
    ));
}

fn lobby_action(
    query: Query<(&Interaction, &LobbyAction), (Changed<Interaction>, With<Button>)>,
    mut world_state: ResMut<NextState<WorldState>>,
    mut room: ResMut<Room>,
    mut actions: EventWriter<GameAction>,
) {
    for (interaction, action) in &query {
        if *interaction != Interaction::Pressed { continue; }
        match action {
            LobbyAction::HostGame => {
                // 本机作为 Host，seat 0 给本地玩家
                room.seat_player(0, Player::new(0, "玩家(主机)", false));
                info!("Created room as host");
            }
            LobbyAction::AddAi => {
                // 找第一个空位填入AI
                for i in 0..4 {
                    if room.seats[i].is_none() {
                        room.seat_player(i, Player::new(i as u8, format!("AI {}", i), true));
                        break;
                    }
                }
            }
            LobbyAction::StartGame => {
                // 至少要有一个玩家
                if room.active_player_count() > 0 {
                    world_state.set(WorldState::Game);
                    actions.write(GameAction::StartGame);
                }
            }
            LobbyAction::JoinGame => {
                // TODO: 显示房间列表，选择后调用 net::client::connect_to_host
                info!("Join game - not yet implemented");
            }
            LobbyAction::Back => {
                world_state.set(WorldState::Menu);
            }
        }
    }
}

fn cleanup_lobby(query: Query<Entity, With<OnLobbyScreen>>, mut commands: Commands) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
```

**Step 2: 更新 `src/plugins/menu/mod.rs`**

将 `MenuButtonAction::Play` 的跳转目标改为 `WorldState::Lobby`：

```rust
MenuButtonAction::Play => { world_state.set(WorldState::Lobby); }
```

并在 `MenuPlugin::build` 中注册 `LobbyPlugin`：

```rust
pub mod lobby;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(lobby::LobbyPlugin)  // 新增
            .add_systems(OnEnter(WorldState::Menu), setup)
            ...
    }
}
```

**Step 3: 完整运行验证**

```bash
cargo run
```

期望流程：主菜单 → 点击"开始游戏" → 进入大厅 → 点击"添加AI玩家"（可多次）→ 点击"开始游戏" → 进入4人牌桌 → 游戏正常运行

**Step 4: 提交**

```bash
git add src/plugins/menu/lobby.rs src/plugins/menu/mod.rs
git commit -m "feat: implement lobby UI with AI slot filling and game start"
```

---

## Task 19: 清理旧代码，更新 CLAUDE.md

**Files:**
- Delete: `src/plugins/game/` 旧文件（deck.rs, player.rs, room.rs, node.rs 如还存在）
- Modify: `CLAUDE.md`

**Step 1: 删除所有旧的 plugins/game 子文件（如果还存在）**

```bash
# 确认旧文件已不存在（应在 Task 10 已删除）
ls src/plugins/game/
```

期望只看到：`mod.rs table.rs renderer.rs hud.rs actions.rs popup.rs`

**Step 2: 运行所有测试**

```bash
cargo test
```

期望：所有单元测试通过（card、player、rules、room 共 16 个测试）

**Step 3: 完整构建**

```bash
cargo build --release
```

**Step 4: 更新 CLAUDE.md**

将 CLAUDE.md 中的架构部分更新为新的三层结构（参考设计文档）。

**Step 5: 最终提交**

```bash
git add -A
git commit -m "refactor: complete architecture overhaul - logic/net/render decoupled"
```

---

## 重构完成后的目录结构验证

```
src/
├── main.rs          ← App 入口
├── states.rs        ← WorldState
├── game/            ← 纯逻辑层（无渲染依赖）
│   ├── mod.rs       ← GameLogicPlugin
│   ├── ai.rs        ← AiPlugin
│   ├── card.rs      ← Card, Deck（含单元测试）
│   ├── events.rs    ← 所有事件类型
│   ├── player.rs    ← Player（含单元测试）
│   ├── room.rs      ← Room Resource（含单元测试）
│   └── rules.rs     ← 纯函数（含单元测试）
├── net/             ← 网络层
│   ├── mod.rs
│   ├── protocol.rs  ← NetMessage, encode/decode
│   ├── host.rs      ← HostPlugin
│   └── client.rs    ← ClientPlugin
└── plugins/         ← 渲染层
    ├── mod.rs        ← KivaPlugins
    ├── menu/
    │   ├── mod.rs    ← MenuPlugin
    │   └── lobby.rs  ← LobbyPlugin
    └── game/
        ├── mod.rs    ← GamePlugin
        ├── table.rs  ← 布局 + PlayerEntityMap
        ├── renderer.rs ← 卡牌渲染
        ├── hud.rs    ← 分数/回合显示
        ├── actions.rs ← 按钮输入
        └── popup.rs  ← 结算弹窗
```
