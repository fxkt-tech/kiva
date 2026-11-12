# 设计

## 模块

`src/simulation/combat/`：

| 文件 | 内容 |
|---|---|
| `mod.rs` | 子模块与再导出；对外接口与原 `combat.rs` 相同 |
| `skills.rs` | 数据：`Slot`、`Control`、`Shape`、`SkillSpec`、`ActionKind::spec()`、`WeaponPair` 槽位到动作的映射、锁定常量 |
| `runtime.rs` | `ActionRuntime`（从 `abilities.rs` 移入，只有战斗使用）：标记去重、取消、序列快照 |
| `combatant.rs` | `CombatIntent`、`CombatStatus`、`MeleeChain`、`Combatant` |
| `health.rs` | `Health`、`SpawnPoint` |
| `feedback.rs` | `CombatEffect`、`CombatEvent`、`CombatFeedback`、`ActionIds` |
| `hit.rs` | `HitContext`、`HitOutcome`、`apply_hit`、视线查询、`hit_enemy`、查询类型别名 |
| `availability.rs` | `availability_for`、`window_valid`、`publish_availability` |
| `prepare.rs` | `tick_slows`、`prepare` 及其具名步骤 |
| `resolve.rs` | `resolve_actions`（玩家倒地复活 + 按形态结算标记）、`recover_stamina` |
| `enemy.rs` | `Enemy`、`advance_enemies` |
| `companion.rs` | `ensure_companions`、出场位置、`publish_companions` |

## 数据表

四种形态覆盖现有七个动作：

- `Arc`：普攻，数值继续来自 `MeleeProfile`。
- `Area { time, centre: Actor|Target, radius, damage, control, effect, window }`：锤 E、链斧 E。
- `Shot { depart, impact, damage, stun, window }`：鸟铳 Q、链斧弓 Q（`depart` 是链斧离手的时刻）。
- `Volley { times, range, damage, sniper, pull, return_from }`：两种联动。

`SkillSpec { duration, cooldown, cast_range, shape }`。准入用的施放距离和结算用的射程读同一个字段。加一个已有形态的技能：加 `ActionKind` 变体、一行 `spec`、一处槽位映射；新形态仍需要写结算函数。

联动窗口的保持条件（锤鸟要求目标浮空、链斧弓要求被本次动作僵直）仍按武器组判断，没有进表：只有两种，进表需要给 `ComboWindow` 加字段。

## 调度

原：`prepare`（含减速计时）…… `advance`（玩家 + 普通敌人）→ Boss → `publish_availability` → `finish_resources`。

新：`tick_slows` → `prepare` …… `resolve_actions` → `advance_enemies` → Boss → `publish_availability` → `recover_stamina` → `publish_companions`。相邻、同一门控、无延迟命令，顺序等价。

## 风险

事件顺序和取消时序是测试断言的对象。逐段移植，不改语句顺序；非联动动作取消时把 elapsed 提到 0.65 的既有行为保留并注明。
