# 近战、结构与受限资源恢复

## 1. 范围与触发

修改普攻投递、建筑规则、AI只读预检、固定步模式组合或基地恢复时阅读。
这些是通用模拟能力，不依赖地图、兵线、玩家、DOM或某个基地ID。模式只通过公开构造、查询和指令使用它们。

## 2. 签名与所有者

```text
UnitClass = Actor | Structure
BasicAttackDelivery = Direct | Projectile { speed, radius }
UnitStats { class, attack_delivery, ... }
Unit::class() -> UnitClass
validate_unit_spec(&World, &UnitSpec) -> Result<(), &'static str>
SimulationSet::Step
preview_cast(&World, Entity, CastSource, AbilityTarget) -> Result<AbilityTarget, Rejection>
is_dashing(&World, Entity) -> bool
DamageSpec::estimate_damage(self, source_attrs, target_attrs, health) -> Option<f32>
RecoverySpec { radius, health_fraction_per_second, mana_fraction_per_second, out_of_combat_ticks }
restore_in_zone(&mut World, source: UnitLifeRef, target: UnitLifeRef, RecoverySpec)
    -> Result<RecoveryAmount { health, mana }, RecoveryError>
UnitState::last_damaged_tick() -> Option<u64>
```

`model`拥有不可变分类/投递；`actions`仍拥有出手与施法；`combat`拥有伤害和生命转换；
`recovery`拥有结构区域恢复入口；正常法力回复仍由`actions`推进，重生资源重置仍由`combat`负责。
`statuses`将永久结构约束与临时控制归并到同一个`ControlState`。

## 3. 数据与时序契约

- 默认`Actor`和`Projectile { speed:22, radius:0.12 }`保持原英雄普攻。Direct也在既有前摇提交点入命中队列，
  继续共用攻击间隔、后摇、目标生命复核、伤害快照和装备被动，不能在AI中立即扣血。
- 构造预检包含原有定义/导航/装配校验，不创建实体或分配来源。`spawn_unit`复用同一入口，模式可先验证多份spec，
  但实际创建仍可能因分配失败而需要由模式回滚本次新增根；不得重置单调身份分配器。
- Structure不能移动、施法、自主Dash、受控制或被强制位移；构造不允许技能槽和复活延迟。
  分类本身不禁止普攻，基地主动性由模式配置的0攻速/无控制器保证，不以地图ID约束通用结构。
- 结构只接受`DamageOrigin::BasicAttack`本体。装备属性进入普通攻击快照；技能、装备主动及命中被动附加效果不作用于结构。
  指向技能在目标校验时拒绝结构，区域在共享命中边界跳过，方向技能弹丸不会被免疫结构拦截。
  状态/强制位移入口也检查结构，不能只让AI不选基地。
- `preview_cast`与实际开始复用生命、Dash忙碌、当前来源/实例、CD、权限、法力、动作和目标校验。
  返回当前姿态下解析的目标（含Dash最大射程），不入队、不分配命中序号、不扣费。结果不是预授权，
  实际开始和提交仍重新校验；提交时的伤害快照失败同样可以拒绝。AI的动作承诺策略不属于此合法性查询。
- `estimate_damage`只复用快照与抗性数学，返回当前生命可损失值；不选择单位或豁免结构目标策略。
  非法health、表达式或快照溢出返回None，不修改任何运行态。
- `SimulationSet::Step`包含完整原战斗链。模式可在前后排序和附加运行条件；单独安装SimulationPlugin仍按原暂停规则运行。
- 恢复一次最多应用`maximum * fraction / TICK_RATE`，返回实际增量。源须为存活同一生命的Structure，目标须为存活友方Actor，
  两者中心距离不超radius，且最近一次抗性后正伤害命中已过去至少out_of_combat_ticks（含被盾全吸收）。抗性后0伤害不刷新脱战时间。
- 恢复拒绝暂停，同一目标生命每SimulationTick最多一次（不同来源也不能叠加）。恢复钳制上限，不复活死亡目标；
  新生命清除旧受伤/恢复tick。模式在完整战斗步之后选择需要恢复的英雄，不由AI或客户端写HP/法力。

## 4. 校验与错误矩阵

| 情形 | 行为 |
| --- | --- |
| 非有限或非正弹丸速度/半径 | 构造预检及spawn拒绝，原World不增加单位 |
| Structure配置技能或复活 | 构造拒绝 |
| 结构移动/合法装备主动请求 | Controlled，不改变位置/动作/资源 |
| 对结构施放Enemy技能 | InvalidTarget，不消耗或替换有效动作 |
| 区域/方向/被动命中结构 | 按来源规则过滤，无伤害、控制或位移 |
| 恢复半径非法，速率非有限/不在[0,1]，两速率均0 | InvalidDefinition |
| 暂停/源死亡或旧生命/目标死亡或旧生命 | Paused / InvalidSource / InvalidTarget |
| 恢复阵营不符、超距、仍在脱战间隔 | WrongTeam / OutOfRange / InCombat，无部分恢复 |
| 目标本tick已经恢复 | AlreadyRestored，不重复增加资源 |

## 5. 正常、默认与错误案例

默认：旧英雄不指定新增字段仍为Actor与原弹丸，已有173项测试保持原行为。

正常：Direct近战在出手tick命中，慢弹丸随后命中；两者都进入一份后摇和攻击间隔，停止不撤销已发弹丸。

正常：60AD攻击20护甲结构损失50生命，附属100真伤/击飞被动不进入结构；同一装备对Actor仍完整生效。

正常：600最大生命、0.08每秒恢复率每tick最多增加0.8；受伤tick1且脱战120tick时，tick120仍拒绝，tick121可恢复。

## 6. 必测断言

`tests/combat_foundation.rs`通过公开接口验证：无分配预检、非法投递/结构构造、近战与弹丸时序/取消、
结构本体普攻/装备属性与被动区分、角色与装备的区域/方向过滤、永久结构权限、预检无副作用及真实忙碌/CD/资源/旧实例、
恢复全部拒绝边界/暂停/精确脱战tick/每tick去重/上限/旧生命、估伤与真实命中相等、外部Step集合完整门控。

执行定向测试、make check和WASM Clippy。模式集成仍要额外验证终局门控、显示冻结和整场清理。

## 7. 错误与正确

错误：AI不对基地放技能就认定基地免控，或客户端自己减基地HP。

正确：共享目标/命中边界识别不可变Structure分类，所有投递和来源统一裁决，HP只有模拟写。

错误：每个显示帧回家时加一次HP，或把旧preview_cast结果当成可执行许可。

正确：模式按模拟tick调用受限恢复，重复调用不会叠加；AI提交意图，开始/提交继续复核当前状态。
