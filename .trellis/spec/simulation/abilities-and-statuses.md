# 组合技能、状态与运动

## 1. 范围与触发

适用于新增技能投递、命中效果、控制状态、强制位移及它们和属性/生命周期的交互。
模块职责和规则表见[架构文档](../../../docs/architecture.md)；本文件约束具体接口和验证边界。

## 2. 签名与所有者

```text
AbilityDefinition { name, targeting, range, windup, recovery, cooldown, mana_cost, execution: AbilityExecution }
AbilityExecution::Impact(ImpactDefinition { delivery: Delivery, effects: Vec<Effect> })
AbilityExecution::Dash { speed: f32 }
AbilityExecution::Channel { duration: f32, interval: f32, pulse: ImpactDefinition }
Effect::Damage(DamageSpec)
Effect::ApplyStatus(StatusSpec)
Effect::Displace(DisplacementSpec)
StatusDuration::from_ticks(u32) -> Option<StatusDuration>
StatusDuration::from_millis(u64) -> Option<StatusDuration>
orders::resolve_target(&World, unit, &AbilityDefinition, AbilityTarget) -> Result<AbilityTarget, Rejection> // 内部解析
```

`effects.rs`保存不可变快照、投递和排序，不直接扣血/写位置/改最终属性。
`statuses.rs`拥有实例与权限归并，调用`actions`/`movement`内部入口执行中断，
同步复用属性模块的校验/提交操作。`combat`负责扣血和死亡编排；`movement`唯一写位置。
`UnitCommand::CastAbility { slot, target: AbilityTarget }`保留；装备使用
`UseEquipment { slot, instance, target }`，二者通过`CastSource`复用同一begin/commit，
不新增按英雄/装备ID的执行分支。装备实例与CD见[装备规范](equipment.md)。

只读接口：`UnitStatuses::iter/len/is_empty/airborne_height`；实例可读种类、来源生命、
技能（从内部完整origin派生）、实例ID、持续/剩余tick和秒、Slow强度。`ControlState`提供
`can_move/attack/cast/dash`、`forced_motion`、`permits(&AbilityDefinition)`。
公开意图接受Entity，已接受的锁定目标在内部转为`UnitLifeRef`。

## 3. 执行契约

- Enemy支持Direct、锁定生命的Projectile、冻结地点的Area；Direction支持首敌Projectile。
  Point支持Area或Dash；SelfCast支持敌方Area或Area脉冲Channel。
- Point Dash范围内到点击点，超距沿施法位置到原点击点的方向截到最大射程。
  begin仅验证并保存原目标，commit从当前UnitPose解析实际终点，再统一提交消耗/CD、AbilityCommitted.position和运动。
  方向/距离使用f64中间值，有限极大点击不能因f32求长溢出而丢失方向。
  角色/装备共用resolve_target；碰墙/地图边界仍由movement的sweep处理，不在客户端缩短命令目标。
  其它Point/Enemy执行保留严格射程校验；规则依据既有execution.is_dash()，不匹配技能ID。
- 效果列表非空且有序，整组快照验证先于消耗/冷却。命中后逐项执行，死亡后跳过目标其余效果。
- 直接命中也入统一末段队列；actor和命中包显式稳定排序。同一步双方已经提交的技能均保留。
- 旧状态在移动后、新命中前递减。末段新应用N tick，准确限制后续N个动作步；到期后不补跑动作。
- Stun/Root同来源生命+完整DamageOrigin+效果索引取较晚到期；不同来源独立。
  装备origin包含实例，同AbilityId的不同装备不合并。Slow独立实例取最强，Airborne独立实例取最大连续显示高度。
- Root禁走路和自主Dash，但允许普攻/普通施法/非移动引导；Stun/Airborne禁止全部自主动作。
  ForcedMotion独立锁，普通控制与Stop不取消它。Root/Stun会终止自主Dash。
- 新受限命令拒绝；原持续意图保留并在解除后重新规划；Stop清持续意图和普通路线。
  中断的一次性施法不重试，已提交效果/消耗/攻击间隔/冷却保留。
- `AttributeModifier.move_speed_multiplier`默认1，有限[0,1]；最强Slow仅一份私有贡献。
  f64中乘最终系数后再饱和，0倍状态移除从基础重算，不能除旧系数恢复。
- 强制位移只在movement扫掠，下一移动步开始，替换和结束只管理自己运动的锁。
  方向由不可变上下文决定；拉拽不越过中心，零向量用提交朝向，碰墙/边界结束。
Direct/Projectile的中心是施法者提交位置，Area的中心是冻结的范围位置；AlongImpact使用载体命中方向。
- 目标生命引用在提交/命中时复核。死亡清状态/运动/临时属性，重生递增代次。
  来源销毁不撤销已提交效果；地面范围可命中当时的新生命。

## 4. 校验与错误矩阵

| 输入/边界 | 结果 |
|---|---|
| 空效果、不兼容目标/投递、NaN/Infinity参数、超出tick上限或无法扣除一步的f32时长 | 注册拒绝，catalog不改变 |
| StatusDuration为0或超出u32 tick | 构造None；60Hz的100ms是6tick |
| Slow强度非有限、≤0或>1 | 注册拒绝 |
| 位移距离/速度非有限或≤0 | 注册拒绝 |
| 速度虽为有限正数，但一步不能产生可表示位置变化 | 移动阶段结束运动并释放其锁；不能永久受控 |
| 有限超距Point Dash | 解析为最大距离终点，正常校验并提交 |
| 非Dash地点/单位目标超距 | OutOfRange，不部分扣费/启冷却 |
| 非有限/错误类型目标，或前摇结束权限/生命/资源失效 | 拒绝提交，不部分扣费/启冷却 |
| 合法定义但伤害快照溢出 | InvalidDamage，不部分扣费/启冷却 |
| 受限新移动/攻击/施法 | Controlled，原有效意图/动作保留 |
| 控制中Stop | 清意图；不清强制运动或已提交效果 |
| 旧生命锁定目标重生 | 旧引用失效；不能命中新生命 |

## 5. 正常、默认和错误案例

- 正常：Area按`[Damage, Root, Slow]`执行；当步HUD包含Slow和同步减速后的最终移速。
- 正常：基础移速10、+20%加速、30%Slow得到8.4；短50%Slow结束后回8.4，最后回12。
- 正常：原点、射程8的Dash点击(30,40)，提交终点(4.8,6.4)；点击(3,4)则到(3,4)，墙会提前截断。
- 默认：无状态、无强制运动时权限允许；不改变既有属性/普攻时序。
- 错误：每个Slow分别写负加算百分比，会让30%+50%变80%，且和加速混算错误。
- 错误：下一步命令前删除1tick状态，导致它从未限制过一次动作。

## 6. 必须验证

通过公开spawn/catalog/CommandQueue/AttributeChanges测试，不新增运行态修改接口。
既有`tests/simulation.rs`验证旧技能、普攻时序、快照和引导分数脉冲；
`tests/status_effects.rs`验证精确状态时钟、权限/中断/Stop、来源叠加、减速、位移几何和生命身份。
变更命中排序必须覆盖同tick互相施法；变更死亡必须覆盖有序致死前/后效果和来源移除。
Dash射程变更由simulation/equipment公开场景覆盖近远/斜向/极值、资源和事件终点、墙与非法目标保留；
browser-skills和browser-equipment使用真实鼠标点两倍射程，断言最大端点及对应来源CD。
执行`make check`；涉及客户端时按客户端规范运行WASM及浏览器验证。

## 7. 错误与正确写法

错误：在`advance_actions`遍历第一个单位时直接眩晕第二个，或某个Root结束直接设置`can_move=true`。

正确：提交不可变命中包到末段统一结算；权限由所有活动状态和运动锁重新派生。
新增技能通过内容装配复用相同执行/投递/效果，不复制扣费、扣血或中断逻辑。
