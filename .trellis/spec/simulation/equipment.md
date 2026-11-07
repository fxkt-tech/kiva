# 六格装备契约

## 1. 适用范围

适用于装备注册、装卸、属性、主动、普攻命中被动及死亡/暂停/重开。
`equipment.rs`拥有定义目录、六格实际实例、装备冷却和装卸；属性、施法、结算仍由原模块拥有。
它不拥有商店、金币或合成。客户端管理协议见[装备交互](../client/equipment.md)。

## 2. 接口

```text
EQUIPMENT_SLOTS = 6
EquipmentId(pub u16)
EquipmentInstanceId(private ModifierSourceId): Copy + Eq + Display
EquipmentDefinition { name, description, flat: AttributeDelta,
  additive_percent: AttributeDelta, passives: Vec<EquipmentPassive>, active: Option<AbilityId> }
EquipmentPassive::OnBasicAttackHit { name: String, effects: Vec<Effect> }
EquipmentCatalog::register(id, definition, &AbilityCatalog) -> Result<(), EquipmentError>
EquipmentCatalog::get(id) / iter() // 只读，按ID排序
EquipmentSlots::item(slot) -> Option<&EquippedItem>
EquippedItem::equipment() / instance() / cooldown()
equip_equipment(&mut World, unit, slot, EquipmentId) -> Result<EquipmentInstanceId, EquipmentError>
unequip_equipment(&mut World, unit, slot, expected: EquipmentInstanceId) -> Result<(), EquipmentError>
UnitCommand::UseEquipment { slot, instance: EquipmentInstanceId, target: AbilityTarget }
CastSource::AbilitySlot(usize) | EquipmentSlot { slot, instance: EquipmentInstanceId }
```

`spawn_unit`统一附加空的`EquipmentSlots`，不在`UnitSpec`或角色定义复制库存。
实例ID内部复用属性来源分配器，整个World中不复用，包括整局重开后；DOM只使用Display字符串。
不提供从外部字符串构造实例ID的入口。

## 3. 时序和所有权契约

- 目录注册先验证加成、被动有序效果和主动引用，成功后不可改写定义。
- 装卸先完成所有边界校验，来源分配是穿戴的最后一个可失败步骤。
  通过属性模块内部`set_immediate`同步重算；暂停和死亡期间同样生效。
  每实例贡献一个`Explicit`来源，移速乘数固定1；卸装只删除该来源。
- 占用格不隐式替换；同种装备可重复，属性、被动和主动冷却按实例独立。
  训练场每次穿戴创建新实例且初始CD为0；没有唯一组或共享冷却规则。
- `CastAbility`和`UseEquipment`统一进入`actions::begin_cast/commit_cast`。
  目标、生命、控制、忙碌、能量、快照、前摇/恢复、冲刺/引导走同一流程。
  提交前再次解析来源、复核实例，成功才扣能量并启动该来源冷却。
  `AbilityCommitted`继续报告所复用的技能；装备归因由完整`DamageOrigin`保存。
- 卸装只中断本实例未提交的Casting及未来Channel脉冲；不撤销已提交弹丸、区域、自主冲刺或附属被动快照。
  Channel来源取不可变`pulse.origin`，不另存一份来源字段。
- 普攻出手前按格/被动索引快照全部被动，全部成功才提交攻击间隔和弹丸。
  命中先结算普攻基础包；只有实际正生命损失且目标仍为活的同一生命，才执行附属被动。
  被动通过既有payload结算，不重新发现触发器；技能/主动/被动伤害不会递归触发。
  已提交快照不查询实时库存或来源是否仍存活，致死后跳过目标其余效果。
- `DamageOrigin::EquipmentActive { equipment, instance, ability }`和
  `EquipmentPassive { equipment, instance, passive }`保留实例与内容身份。
  Stun/Root刷新键使用来源生命+完整origin+效果索引，同技能的不同装备不会合并。
- 装备CD在`actions::advance_actions`开头、存活actor筛选之前推进，实际仍在命令消费之后。
  死亡保留装备和属性，装备CD在死亡期间继续推进，重生不刷新；暂停冻结全部固定步。
  `reset_simulation`随Unit移除装备，保留目录和单调分配器；新局空六格。

## 4. 校验与错误矩阵

| 情形 | 结果 |
|---|---|
| 重复目录ID | `DuplicateId`，原定义保留 |
| 空名称、非有限加成、空/非法被动效果 | `InvalidDefinition`，目录不改变 |
| 主动引用未注册技能 | `UnknownActive`，目录不改变 |
| 无效单位、越界格、未知装备 | `InvalidUnit` / `InvalidSlot` / `UnknownEquipment` |
| 穿戴到占用格、卸空格、实例不匹配 | `SlotOccupied` / `EmptySlot` / `StaleInstance` |
| 来源耗尽 | `SourceExhausted`；没有半件装备或部分属性 |
| 装备主动空格/无主动/旧实例 | 沿既有指令拒绝通道返回`NoAbility`，保留原动作 |
| 开始命令时目标/资源/权限/忙碌/冷却不满足 | 复用技能拒绝原因，不扣费、不重置原有效动作 |
| 前摇末段提交复核失败 | 结束本次Casting并回Ready，不扣费、不启冷却 |
| 合法定义的攻方伤害快照溢出 | `InvalidDamage`；主动不扣费/CD，普攻不提交间隔 |

装备管理失败不部分改变库存、来源、最终属性和其它动作。属性极值聚合沿既有有限饱和规则。

## 5. 正常、默认和错误案例

- 默认：所有raw单位和角色初始空六格；原四技能、初始属性和战斗节奏不变。
- 正常：40攻击穿两件+20锋刃得到80，卸一件得到60；两实例身份不同。
- 正常：角色Q和两件引用同技能的装备有三份冷却、三份完整状态来源。
- 正常：普攻弹丸出手后卸下霜纹护符，已出手仍附伤/减速，下一次攻击不再附带。
- 错误：命中时读取当前背包、监听所有Damaged再触发、重生时把所有CD归零。

## 6. 必须验证

`tests/equipment.rs`通过公开目录、spawn、装卸、指令和事件覆盖：注册失败、原子六格、重复与属性来源、
共享施法/独立冷却/状态来源、拒绝保留、未提交取消和旧实例、已提交载体/冲刺保留、引导取消、
被动快照与顺序/无递归/正损血/目标生命、快照溢出、死亡/暂停/重开。
同时执行原simulation/status/characters/reset测试和`make check`，防止来源改造改变原技能行为。

## 7. 错误与正确

错误：给装备复制一套施法系统，或把装备塞进四个`AbilitySlots`；卸装时直接减最终属性。

正确：orders将两种命令归一为`CastSource`并进入统一提交，装备模块拥有实例冷却，属性模块从来源集合重算。

错误：被动命中后再遍历当前背包，为被动伤害再次派发普攻触发。

正确：普攻出手附带不可变被动快照，正损血/生命门控后只解析一层既有效果。
