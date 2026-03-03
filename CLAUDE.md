# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**kiva** is a Rust/Bevy game project — a four-player Blackjack (21点) card game titled "四方诛杀". Supports local human player, AI-filled seats, and LAN multiplayer (up to 4 players). The architecture is fully decoupled into three layers: game logic, network, and rendering.

## Commands

```bash
# Run the game (dev mode, fast iteration)
cargo run

# Type-check without compiling
cargo check

# Lint
cargo clippy

# Format code
cargo fmt

# Run unit tests (16 tests in game logic layer)
cargo test

# Optimized release build
cargo build --release

# WebAssembly size-optimized build
cargo build --profile wasm-release --target wasm32-unknown-unknown
```

## Architecture

The project uses a **strict three-layer architecture** enforced by module boundaries:

```
src/
├── main.rs              # App entry: window, DefaultPlugins, KivaPlugins, global State
├── states.rs            # WorldState enum (Menu → Lobby → Game)
│
├── game/                # LOGIC LAYER — zero rendering dependency
│   ├── mod.rs           # GameLogicPlugin: registers Room, all Events, AiPlugin, process_game_actions
│   ├── card.rs          # Card { rank: u8 }, Deck (shuffle/pop/reset) — no Entity
│   ├── player.rs        # Player { id, name, hand, is_ai, is_stand, is_connected }
│   ├── room.rs          # Room Resource: seats[4], deck, current_turn, GamePhase
│   ├── rules.rs         # Pure functions: calc_score, is_bust, judge_winner, is_round_over
│   ├── ai.rs            # AiPlugin: listens TurnChanged → emits GameAction (Draw/Stand)
│   └── events.rs        # All event types (see Event Flow below)
│
├── net/                 # NETWORK LAYER — LAN UDP communication
│   ├── mod.rs
│   ├── protocol.rs      # NetMessage enum, encode/decode (bincode+serde), NET_PORT=7777
│   ├── host.rs          # HostPlugin: UDP server, receives client actions, beacon broadcast
│   └── client.rs        # ClientPlugin: connects to host, forwards local actions
│
└── plugins/             # RENDER LAYER — Bevy UI + input only
    ├── mod.rs           # KivaPlugins group: GameLogicPlugin + MenuPlugin + GamePlugin
    ├── menu/
    │   ├── mod.rs       # MenuPlugin: main menu UI, audio
    │   └── lobby.rs     # LobbyPlugin: LAN room lobby (host/join/add AI/start)
    └── game/
        ├── mod.rs       # GamePlugin: orchestrates all render sub-systems
        ├── table.rs     # 4-seat 2×2 grid layout, PlayerEntityMap Resource
        ├── renderer.rs  # Listens CardDealt/RoundReset → spawn/despawn card UI entities
        ├── hud.rs       # Listens TurnChanged/RoundOver → update score/turn text
        ├── actions.rs   # Button input → GameAction events (local player_id = 0)
        └── popup.rs     # Listens RoundOver → game-over popup, Continue button
```

## Key Design Rules

### Layer Isolation
- `game/` must **never** import `bevy::render` or hold `Entity`
- `net/` only imports from `game/events` and `game/player`
- `plugins/` is the only layer allowed to use full Bevy APIs

### Event Flow (the core decoupling mechanism)

```
Button click → actions.rs → GameAction::Draw { player_id }
                                    ↓
                          game/mod.rs: process_game_actions
                          Room.deck.pop() → CardDealt { player_id, card }
                                    ↓
                    ┌───────────────┼───────────────┐
                    ↓               ↓               ↓
             renderer.rs       hud.rs         net/host.rs
           spawn card UI    update score    broadcast to clients

AI turn: TurnChanged { player_id } → ai.rs detects is_ai=true → GameAction::Stand/Draw
         (AI and human players use identical GameAction interface)
```

### PlayerEntityMap (replaces old Option\<Entity\> in data)
The render layer maintains its own `player_id → Entity` mapping in `PlayerEntityMap` Resource (`plugins/game/table.rs`). Game logic structs (`Card`, `Player`) contain **no** Entity references.

### State Machine
```
WorldState::Menu → WorldState::Lobby → WorldState::Game → WorldState::Menu
GamePhase (inside Room): WaitingForPlayers → Playing → GameOver → Playing
```
`WorldState` controls which plugin systems are active. `GamePhase` controls in-game flow. They are independent.

## Network (LAN)
- **Topology**: Star (Host-Client). Host is authoritative; clients only render.
- **Protocol**: UDP, port 7777 (NET_PORT), beacon on 7778 (BEACON_PORT)
- **Serialization**: `bincode` v2 with `serde` feature + `serde` derives on `Card` and `GamePhase`
- **AI players** run on Host only, never cross the network

## Assets
- `assets/fonts/WenCangShuFang-2.ttf` — Chinese font used throughout
- `assets/sounds/menu_bgm.ogg` / `table_bgm.ogg` — per-state background music

## Build Profiles

| Profile | Use case |
|---|---|
| `dev` | Default; deps at `opt-level=3` for fast iteration |
| `release` | Optimized binary with thin LTO |
| `wasm-release` | Size-optimized WebAssembly |
