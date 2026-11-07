# 属性与伤害契约

## 1. 适用范围

适用于构造单位、修改属性、定义伤害、普攻时序和命中结算。实现位于
`src/simulation/attributes.rs`、`damage.rs`、`actions.rs`、`combat.rs`。
本规范描述基础属性与伤害实现；状态计时和控制由[状态规范](abilities-and-statuses.md)约束，
装卸、持久来源及被动快照由[装备规范](equipment.md)约束。

## 2. 接口

```rust
Unit::definition(&self) -> &UnitStats
Unit::attributes(&self) -> &EffectiveAttributes
AttributeChanges::allocate_source(&mut self) -> Option<ModifierSourceId>
AttributeChanges::upsert(&mut self, unit: Entity, source: ModifierSourceId, modifier: AttributeModifier)
AttributeChanges::remove(&mut self, unit: Entity, source: ModifierSourceId)
```

`UnitStats` 含 `base_attributes: BaseAttributes`、静态生命/法力/碰撞参数、`attack_windup_fraction`，
以及默认保持旧英雄行为的`class`/`attack_delivery`，详见[战斗基础](combat-foundation.md)。
最终值通过七个 getter 读取：`attack_damage()`、`ability_power()`、`armor()`、
`magic_resistance()`、`move_speed()`、`attack_range()`、`attack_speed()`。
不能从客户端写入运行态，也不能将 `definition().base_attributes` 当作当前生效值。

```rust
pub struct AttributeModifier {
    pub flat: AttributeDelta,
    pub additive_percent: AttributeDelta,
    pub removal: ModifierRemoval,
    pub move_speed_multiplier: f32, // 默认1，有限[0,1]
}
pub struct DamageSpec {
    pub kind: DamageType,
    pub base: f32,
    pub ad_ratio: f32,
    pub ap_ratio: f32,
}
```

`AttributeDelta` 七项字段均为有符号 f32，默认全零；`BaseAttributes` 有玩法默认值，二者不可互换。
`DamageType` 为 `Physical`、`Magical`、`True`。`DamageSpec::physical(base)` /
`magical(base)` 创建零加成表达式；需要加成时显式设置对应系数。

## 3. 数据与时序契约

- 来源 ID 在当前 World 内分配且不复用；耗尽返回 None。调用方保留句柄，对同一来源 upsert 为替换。
- 属性队列在重生之后、玩家指令之前按 FIFO 消费；暂停时保留请求。
- 计算为 `(base + sum(flat)) * max(0, 1 + sum(additive_percent))`，0.2 表示 +20%。
- f64 聚合，最终饱和为有限值；护甲/魔抗可负，其余不小于 0。
- 正攻速限制在整数时序可表达范围，0 表示不开始新普攻。极小正结果应在转 f32 前处理下限。
- 普攻周期与技术上下限由 `TICK_RATE` 推导，避免先使用舍入后的 f32 秒长再向上取整，
  将应为两 tick 的最大攻速误算成三 tick。
- 每次攻击开始快照整数前摇和剩余间隔，成功出手后提交间隔；改攻速只影响下一次新攻击。
- 成功出手进入 `UnitAction::AttackBackswing { remaining: u32, duration: u32 }`，两个计时值均来自本次剩余间隔。
  后摇自然结束不额外增加攻击周期；移动/停止/换目标/技能可以取消，独立攻击间隔不变。
  同目标重复指令和无效请求保留后摇，暂停冻结、死亡清理；技能 `Recovery` 保持独立。
- `AttributeModifier.move_speed_multiplier`默认1且有限[0,1]，仅对最终移速乘算；在f64中乘完再做最终饱和。
- Slow模块聚合最强强度，只维护一个私有来源：20%加速+30%Slow是1.2×0.7=0.84倍。
- 状态末段变更同步复用属性内部校验/提交，不能等下一步公开队列才更新最终值。
- 装卸同样复用内部同步入口，暂停期间也即时更新；实例贡献一个Explicit来源，不直接加减最终值。
- `Explicit` 来源仅显式移除；`OnDeath` 在死亡时清理。状态计时不属于属性模块。
- 伤害提交快照 `base + ad_ratio * AD + ap_ratio * AP`；命中时读取目标对应最终抗性。
- 来源死亡/移除不改变已提交效果；引导后续脉冲复用同一攻方快照。
- 抗性非负倍率 `100 / (100 + r)`，负抗性倍率 `2 - 100 / (100 - r)`，真实伤害忽略抗性。
- `Damaged { source, target, amount, kind, origin }` 的 amount 为实际生命损失；0 实际损失不发事件。
- 抗性后伤害与护盾吸收保持f64，直接用f64剩余伤害扣除HP后再转回f32；amount从前后HP差计算。
  不先将剩余伤害转f32再扣HP，否则负抗性等场景与estimate_damage产生双重舍入差异。
  盾全吸收的正伤害仍作为有效命中触发普攻附属效果，amount继续为0且不发Damaged。
- `DamageOrigin`区分普攻、技能、含实例的装备主动/被动；不可把装备归因为同一个技能ID。

## 4. 校验与错误矩阵

| 输入/情形 | 结果 |
|---|---|
| 非有限基础值、负攻击力/法强/移速/距离/攻速 | 构造拒绝 |
| 有限负护甲/魔抗 | 合法，增加对应类型受到的伤害 |
| 非有限修正值或非法最终移速系数 | `AttributeChangeRejected` / `InvalidModifier`，原集合和最终值保留 |
| 无效目标或来源 | `InvalidUnit` / `InvalidSource`，不变更状态 |
| 死亡单位添加 OnDeath 来源 | `Dead`，不带入重生 |
| 死亡单位更新 Explicit 来源 | 可更新，重生不重复叠加 |
| 移除不存在的来源 | 已存在单位上的幂等成功 |
| 有限极值来源叠加或移除 | 有限饱和值，不阻止死亡清理 |
| 负数/非有限伤害表达式参数 | 技能注册拒绝 |
| 合法表达式与攻方属性组合超出伤害表示范围 | `InvalidDamage`，提交不扣费、不进入冷却 |
| 来源在提交后被销毁 | 效果保留快照继续结算 |
| 后摇时移动、停止、换目标或合法施法 | 取消后摇，已提交攻击和攻击间隔保留 |
| 后摇时重复同目标攻击或提交无效命令 | 保留并正常推进后摇 |
| 出手复核失败 | 不进入后摇，不提交攻击间隔 |

每条属性请求独立原子执行，失败不撤销之前成功的其它请求，也不改动玩家动作和持续指令。

## 5. 正常、默认和错误示例

- 正常：攻击力 40，来源 A +10，B +20%，C +30%，结果 75。移除 B 后 65，替换 A 为 +20 后 78。
- 默认：40 AD、0 AP、0 护甲/魔抗、5.5 移速/距离、1.25 次/秒；无修正时等于基础值。
- 正常：100 物理伤害遇到 100 护甲扣 50 血，-50 护甲约扣 133.333 血；法术使用魔抗。
- 错误：固定值里写 NaN、在客户端加减最终值、命中时重新查询来源攻击力，都违反契约。

## 6. 必须验证的行为

通过公开构造、属性队列与命令验证来源替换/移除、原子失败、暂停、死亡/重生、
零/极值属性、类型化抗性、提交/命中时间点和来源移除后的效果存续。
用 `AttackReleased` 的模拟 tick 序列验证 1/2 次每秒和技术最大攻速，不用墙钟等待验证节奏。
移动/停止/重复攻击以及中途修改攻速均不能多打一次或刷新已提交间隔。
新增后摇时检查自然完成、取消、重复/非法请求、暂停、死亡、攻速变化与出手失败场景。
执行 `cargo test --locked --test simulation` 与 `make check`。

## 7. 错误与正确用法

错误：装备先 `final_attack_damage += 10`，卸下时 `-= 10`；来源替换和百分比叠加后容易残留。

正确：通过`equip_equipment/unequip_equipment`管理实例，入口在属性模块同步提交来源并从基础值重算。
其它公开属性调用方继续使用`AttributeChanges`队列，不自行改写最终值。
输入和 UI 读取 `unit.attributes().attack_damage()`，定义配置读取 `unit.definition()`。

错误：弹丸命中时查询施法者当前 AP；施法者已销毁时丢弃整个效果。

正确：提交时生成 `DamageSnapshot`，命中时只读取目标抗性与生命，来源 ID 作为已发生事实保留。

错误：用普攻后摇状态的剩余时间代替独立攻击间隔，取消后摇时一起清零，导致走砍多打。

正确：后摇是可取消动作，攻击间隔是已提交时钟；结束或取消后摇都不能更改该时钟。
