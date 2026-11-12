# Boss AI 改用 boss_01 行为树

前置：`11-12-boss-own-clips`（Boss 播放自己的原作片段、按片段烘焙判定）。

## Goal

用 `~/go/src/github.com/fxkt-tech/mysrc/boss_01/reparsed/ai`（仓库外）的原作行为树重写愁铃客的决策，去掉旧的“三招一轮、停 3 秒”节奏。

## 来源

| 树 | 用途 | 转写到 |
|---|---|---|
| `51000025_BOSS01_fight_P1` / `51000031_BOSS01_fight_P2` | 优先级：连招 > 格挡反制 > 计数内选招 > 游走清零；开场；距离池；P2 终结技 | `boss/source.rs` `Mind::choose`、`Boss::decide` |
| `51000025_BOSS01_fight_Action` | 57 个技能槽、冷却、哪些动作计入 `attack_count`；动作 1/2/3 的游走、逼近距离 | `SLOTS`、`COOLDOWNS`、`counts_as_attack`、`Boss::begin_action` |
| `51000025_BOSS01_P1_ComboCondition` / `51000031_BOSS01_P2_ComboCondition` | 技能标签（槽位末三位）+ 距离 → `BOSS01_Combo` | `p1_tag`、`p2_tag` |
| 主树中的 `BOSS01_Combo` 分支 | 每个连招号的后续选招 | `p1_combo`、`p2_combo` |

## Requirements

| ID | 需求 |
|---|---|
| R1 | 决策顺序与原 `ConditionPriorityNode` 一致；`attack_count` 上限 P1 3、P2 5，满后游走 1–2s 再清零 |
| R2 | 动作号、权重、距离、计时、`PassNode` 一次性分支、`different_with_last` 交替与原树一致 |
| R3 | 招式最后一个有效窗口过后检查连招，成立则不收招直接接下一招 |
| R4 | 玩家格挡（`Blocked`）了 Boss 的一击后，下次非连招决策出 6/7 交替 |
| R5 | 每个动作 4..=60 都映射到一个已烘焙招式（`STRIKE_OF`，Kiva 适配） |
| R6 | 旧的 `ATTACKS_PER_BURST`、`BURST_REST`、固定距离池常量删除 |

## Kiva 适配（不是原作数据）

- 冷却 × `COOLDOWN_SCALE`（0.25）。原值下近距离池约四招后耗尽，一分钟里约 23s 在游走；原战斗靠玩家移动、受击和事件维持节奏，Kiva 没有。改为 1.0 即恢复原节奏。
- 动作 → 17 招的映射：槽位 → 片段未恢复，P1 动作用 P1 片段，P2 动作用 P2 片段，反制类（4、6–9、34、47）用最短的招。
- 伤害、两条血、转阶段沿用原 Kiva 设定。
- 游走速度沿用原实现（0.7 m/s）。

## 未实现（依赖 Kiva 没有的系统）

- 自身增益 450079 分支、受击计数与金身（`BOSS01_behit`、`behit_gold` → 34）、真气虚弱（`BOSS01_weak`）。
- 锁定 / 闪避 / 80 / 81 事件标签（`lock_behit`、`dodgetandao` → 4、6–9）。
- 成就增益 450100 与吸魂 27、`CheckObstacle` 障碍探测。
- P1 柱子特殊招、分身、死亡树。

## Acceptance Criteria

- [x] `boss/source.rs` 单元测试：开场、距离池、计数与冷却、连招标签、P2 终结技计时、格挡反制交替
- [x] 实战测试：P1 先游走 2s 再出 45 并只命中一次；连招无收招直接接招；无连招的招收招到时间轴结束；满一轮后游走并清零；格挡置 `GedangCondition`
- [x] 每个动作 1..=60 移动或挥出已烘焙招式，17 招都有动作使用
- [x] 正反防御、闪避、弹反收步、墙体、暂停/HitStop、阶段、死亡重置回归通过
- [x] `make check`、WASM Clippy 通过；`make bake-hits` 可复现
- [ ] 用户在运行中的游戏里验收手感

## 补充：背后转身（2026-11-13）

原作只在动作 1/2/3 和打满一轮后的游走开头检查目标是否在背后（135°–225°），是则先 `TurningAction` 原地转身；出招和开场游走不检查。已按此实现为 `BossState::Turning`。转速原作取转身动画速度（`yaw_speed=-1`），未恢复，Kiva 用 180°/s；3s 超时取原值。

## 验证记录（2026-11-13）

- `make check`：Rust 347 个（1 个忽略）、Node 108 个通过；WASM Clippy 通过；`make bake-hits` 两次逐字节相同。
- 站桩一分钟无窗口回放（玩家不动、不防御）：P1 出招 19 次、命中 18 次、游走约 6s，序列以 45→43→20 连招开场；P2 出招 19 次、命中 19 次，21→22 开场。原冷却下 P1 同一分钟有约 23s 游走，据此引入冷却倍率。
- 墙体测试的墙从 8m 加宽到 60m：新 AI 会游走，能从 8m 墙的端头绕过去，这是新行为，不是穿墙。
