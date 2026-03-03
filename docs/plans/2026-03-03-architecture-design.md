# 四方诛杀 — 大型重构架构设计

**日期：** 2026-03-03
**目标：** 局域网4人21点，支持AI填充空位，一次性全量重构

---

## 核心设计原则

**游戏逻辑层对渲染层零感知。** `game/` 目录下的所有代码不得 `use bevy::render` 或持有任何 `Entity`。逻辑层通过发出 Bevy `Event` 通知渲染层，渲染层通过发出 `GameAction` 事件通知逻辑层。网络层只同步 `GameAction`，不同步 Entity。

---

## 目录结构

```
src/
├── main.rs                  # App 入口：窗口、DefaultPlugins、KivaPlugins、全局 State
├── states.rs                # WorldState 枚举（全局状态机）
│
├── game/                    # 纯游戏逻辑层（禁止引入任何渲染相关依赖）
│   ├── mod.rs               # pub use 导出，GameLogicPlugin（只注册逻辑系统）
│   ├── card.rs              # Card（rank: u8）、Deck（shuffle、pop、reset）
│   ├── player.rs            # Player（id、hand、is_ai、is_stand）
│   ├── room.rs              # Room Resource：seats[4]、deck、current_turn、phase
│   ├── rules.rs             # 纯函数：calc_score、judge_winner、is_bust
│   ├── ai.rs                # AiPlugin：监听 AiTurnReady 事件，自动产生 GameAction
│   └── events.rs            # 逻辑层对外广播的事件（CardDealt、TurnChanged、GameOver 等）
│
├── net/                     # 局域网通信层
│   ├── mod.rs               # NetPlugin
│   ├── protocol.rs          # 消息协议：NetMessage 枚举（序列化用 serde + bincode）
│   ├── host.rs              # HostPlugin：UDP广播、接收客户端 Action、分发给逻辑层
│   └── client.rs            # ClientPlugin：发送本地 Action 给 Host、接收 GameSnapshot
│
└── plugins/                 # Bevy 渲染 + 输入插件层
    ├── mod.rs               # KivaPlugins 插件组
    ├── menu/
    │   ├── mod.rs           # MenuPlugin
    │   └── lobby.rs         # LobbyPlugin：局域网房间大厅（扫描、创建、加入房间）
    └── game/
        ├── mod.rs           # GamePlugin（组合下面所有子插件）
        ├── table.rs         # 牌桌布局：OnEnter 时 spawn 4个座位框架
        ├── renderer.rs      # 监听逻辑事件 → spawn/despawn 卡牌 UI 实体
        ├── hud.rs           # 监听逻辑事件 → 更新分数、回合指示、玩家名
        ├── actions.rs       # 按钮输入 → 发出 GameAction 事件
        └── popup.rs         # 监听 GameOver 事件 → 显示结算弹窗
```

---

## 模块职责详解

### `game/` — 逻辑层

#### `card.rs`
```rust
#[derive(Clone, Copy, Debug)]
pub struct Card { pub rank: u8 }  // 不含 Entity

pub struct Deck { cards: Vec<Card> }
impl Deck {
    pub fn new() -> Self { ... }   // 创建并洗牌
    pub fn pop(&mut self) -> Option<Card> { ... }
    pub fn reset(&mut self) { ... }
}
```

#### `player.rs`
```rust
#[derive(Clone, Debug)]
pub struct Player {
    pub id: PlayerId,          // u8: 0-3
    pub name: String,
    pub hand: Vec<Card>,
    pub is_ai: bool,
    pub is_stand: bool,
    pub is_connected: bool,    // 局域网玩家是否在线
}
```

#### `room.rs`
```rust
#[derive(Resource)]
pub struct Room {
    pub seats: [Option<Player>; 4],  // 固定4个座位，None表示空位
    pub deck: Deck,
    pub current_turn: usize,         // 当前回合玩家 index
    pub phase: GamePhase,
}

#[derive(Clone, PartialEq)]
pub enum GamePhase {
    WaitingForPlayers,   // 大厅等待阶段
    Dealing,             // 初始发牌
    Playing,             // 游戏进行中
    GameOver,            // 结算
}
```

#### `rules.rs`（纯函数，无副作用，可单元测试）
```rust
pub fn calc_score(hand: &[Card]) -> u8 { ... }
pub fn is_bust(hand: &[Card]) -> bool { calc_score(hand) > 21 }
pub fn judge_winner(seats: &[Option<Player>; 4]) -> Vec<PlayerId> { ... }
pub fn is_game_over(seats: &[Option<Player>; 4]) -> bool { ... }
```

#### `events.rs` — 逻辑层对外广播的事件
```rust
// 逻辑层 → 渲染层 / 网络层
#[derive(Event)] pub struct CardDealt   { pub player_id: PlayerId, pub card: Card }
#[derive(Event)] pub struct TurnChanged { pub player_id: PlayerId }
#[derive(Event)] pub struct PlayerStood { pub player_id: PlayerId }
#[derive(Event)] pub struct GameOver    { pub winners: Vec<PlayerId>, pub scores: [u8; 4] }
#[derive(Event)] pub struct GameReset;

// 渲染层 / 网络层 → 逻辑层
#[derive(Event, Clone)]
pub enum GameAction {
    Draw  { player_id: PlayerId },
    Stand { player_id: PlayerId },
    Quit  { player_id: PlayerId },
}
```

#### `ai.rs`
```rust
// AI 系统：监听 TurnChanged，如果当前玩家是 AI，自动产生 GameAction
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

---

### `net/` — 网络层

#### `protocol.rs`
```rust
#[derive(Serialize, Deserialize)]
pub enum NetMessage {
    // 客户端 → 主机
    JoinRequest { name: String },
    PlayerAction { action: GameAction },

    // 主机 → 客户端
    JoinAccepted { assigned_id: PlayerId },
    GameSnapshot { room_state: RoomSnapshot },  // 全量状态同步
    ActionBroadcast { action: GameAction },      // 增量动作广播
}

// RoomSnapshot：Room 的可序列化快照（不含 Entity）
#[derive(Serialize, Deserialize, Clone)]
pub struct RoomSnapshot {
    pub seats: [Option<PlayerSnapshot>; 4],
    pub deck_remaining: usize,
    pub current_turn: usize,
    pub phase: GamePhase,
}
```

#### 网络拓扑：星形（Host-Client）
```
[Client 0] ──┐
[Client 1] ──┤── UDP ──► [Host] ──► 逻辑层 Room
[Client 2] ──┤                │
[AI      ] ──┘                └──► 广播 ActionBroadcast 给所有客户端
```

- **Host** 是权威端，所有 `GameAction` 必须经过 Host 的逻辑层验证后才生效
- **Client** 本地不运行逻辑，只渲染 Host 广播过来的状态
- **AI** 在 Host 本地运行，不走网络

#### 局域网发现（`lobby.rs`）
- Host 在局域网广播 UDP beacon（包含房间名、当前人数）
- Client 扫描 beacon，列出可加入的房间
- 使用固定端口（如 `7777`），无需中心服务器

---

### `plugins/game/` — 渲染层

#### 渲染层的核心：`PlayerEntityMap` Resource
```rust
// 渲染层维护 player_id → Entity 的映射，彻底替代 Option<Entity> 存在数据层
#[derive(Resource, Default)]
pub struct PlayerEntityMap {
    pub seat_entities: [Option<Entity>; 4],  // 每个座位的根 Entity
}
```

#### `renderer.rs` — 事件驱动更新
```rust
fn on_card_dealt(
    mut ev: EventReader<CardDealt>,
    map: Res<PlayerEntityMap>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    for ev in ev.read() {
        let Some(entity) = map.seat_entities[ev.player_id as usize] else { continue };
        commands.entity(entity).with_children(|parent| {
            parent.spawn(card_bundle(&ev.card, &asset_server));
        });
    }
}
```

#### `hud.rs` — 监听事件更新文字
```rust
fn on_turn_changed(mut ev: EventReader<TurnChanged>, mut query: Query<(&mut Text, &TurnIndicator)>) { ... }
fn on_score_changed(room: Res<Room>, mut query: Query<(&mut Text, &ScoreLabel)>) { ... }
```

---

## 数据流总览

```
【本地玩家点击按钮】
    actions.rs → GameAction::Draw { player_id: 0 }
        │
        ├─► [Host模式] game/mod.rs 逻辑系统处理
        │       Room.deck.pop() → CardDealt 事件
        │           ├─► renderer.rs → spawn 卡牌UI
        │           ├─► hud.rs → 更新分数
        │           └─► net/host.rs → 广播给其他客户端
        │
        └─► [Client模式] net/client.rs → UDP发送给Host
                Host处理后广播 ActionBroadcast
                Client收到 → 本地发出对应事件 → 渲染更新

【AI回合】
    TurnChanged { player_id: 2 } → ai.rs 检测到是AI
        → GameAction::Stand { player_id: 2 }
        → 走同样的逻辑处理路径（AI和人类完全一致）
```

---

## 状态机设计

```
WorldState::Menu
    └─► [开始游戏] → WorldState::Lobby
            └─► [房间满员/房主开始] → WorldState::Game
                    └─► [退出/结束] → WorldState::Menu

GamePhase（Room内部状态，不是 Bevy State）
    WaitingForPlayers → Dealing → Playing → GameOver → Playing（下一局）
```

`WorldState` 控制哪些 Plugin 系统激活，`GamePhase` 控制游戏内部流程，两者分离。

---

## 重构步骤（建议顺序）

1. 新建 `states.rs`，迁移 `WorldState`
2. 新建 `game/` 目录，实现 `card.rs`、`player.rs`、`room.rs`、`rules.rs`、`events.rs`（全部无渲染依赖，可先写单元测试）
3. 实现 `game/ai.rs`（单机AI先跑通）
4. 重构 `plugins/game/`：`table.rs` 负责布局，`renderer.rs` 监听事件，`hud.rs` 更新显示，`actions.rs` 处理输入，`popup.rs` 处理结算
5. 新建 `net/protocol.rs`，定义消息结构
6. 实现 `net/host.rs` 和 `net/client.rs`
7. 新建 `plugins/menu/lobby.rs`，实现局域网房间大厅

---

## 关键依赖

在 `Cargo.toml` 中需要新增：

```toml
[dependencies]
serde = { version = "1", features = ["derive"] }
bincode = "2"                    # 二进制序列化，用于网络传输
tokio = { version = "1", features = ["net", "rt"] }  # 异步UDP
# 或者使用 bevy 生态的网络库：
# bevy_quinnet = "0.x"          # QUIC协议，比UDP更可靠
```
