# 玩法层规范

适用于`src/gameplay/`及`tests/{match_mode,minions,hero_ai}.rs`。
本层组合通用模拟，不依赖窗口、客户端、浏览器或模型。

## 阅读入口

- [match.md](match.md)：地图、原子构造、阶段、兵波、终局与恢复。
- [ai.md](ai.md)：兵线与英雄战术、观察边界、技能语义、有限追击和普通避让。
- [模拟基础](../simulation/combat-foundation.md)：结构受效、普攻投递、施法预检、估伤和受限恢复。
- [客户端对局](../client/lane-match.md)：会话、投影、模型和结算交互。

## Pre-Development Checklist

1. 阅读本入口和目标模块规范；内容定义集中在`content/`，模式配置集中在`MatchDefinition`。
2. 明确事实所有者：模式写阶段/波次，AI写Brain记忆，模拟写战斗状态。
3. 使用真实生命引用和实际装配，复用构造、指令、预检及恢复入口。
4. 时序基于SimulationTick，终局先于兵波；暂停/新局不得遗留决策和旧身份。

## Quality Check

- `make check`与WASM Clippy通过；公开行为测试不篡改私有运行态。
- 涉及波次、恢复、终局或重置时运行`tests/match_mode.rs`。
- 涉及自主决策或普通避让时运行`tests/minions.rs`与`tests/hero_ai.rs`；长局变化显式运行ai规范中的30分钟场景。
- 地图/投影/会话联动须运行对应浏览器回归，包括完整对局和重选。
- 无客户端依赖、隐形活兵上限、墙钟波次或AI战斗写入口。
