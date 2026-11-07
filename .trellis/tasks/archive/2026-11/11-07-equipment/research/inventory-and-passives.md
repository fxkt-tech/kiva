# 六格装备实例、属性来源与命中被动调查

状态：规划研究，不是已实现规范。基线`8f7107c`。本调查没有修改产品代码、运行测试或启停服务。
相关现有规范：`simulation/runtime.md`、`attributes-and-damage.md`、`abilities-and-statuses.md`、`characters.md`及跨层复用指南。

## 1. 当前代码事实

- `src/simulation/mod.rs:103`的`spawn_unit`统一创建单位运行态，`UnitSpec`目前只含角色技能四槽。
  新增默认空的装备组件即可覆盖NPC/raw spawn/角色创建，不应给`UnitSpec`增加必填字段迫使旧构造器更改。
- `src/simulation/attributes.rs:156`的`ModifierSourceId`是私有构造的单调ID；`AttributeChanges::allocate_source`在184行分配，
  `clear_pending`只清请求队列，不回收ID。装备可以复用这一来源身份，不必另建随机或全局实例ID资源。
- 属性模块在`attributes.rs:257`使用候选集合校验并重算最终值；`set_immediate`在296行复用同一实现。
  `statuses.rs:207`已通过该内部入口同步最强Slow，证明暂停中合法同步属性变更可继续保持属性模块唯一计算。
- 七项属性属于`AttributeDelta/EffectiveAttributes`；`max_health/max_mana/mana_regen`仍是`model.rs:17`的静态UnitStats。
  首批装备应只增加现有七项，不引入生命上限变化、当前生命调整或额外资源公式。
- `actions.rs:155`在普攻前摇结束后复核目标距离/遮挡，再于187行快照基础攻击、提交冷却并发射弹丸。
  当前普攻只有一个物理Damage，伤害快照读取提交时最终AD。
- `effects.rs:95`为整组效果生成不可变快照；`ImpactSnapshot`保存来源生命、origin、攻方位置/朝向及效果。
  弹丸和区域直接携带此快照，命中时不重新查询施法者属性或定义。
- `effects.rs:193`按`stage/sequence/carrier/target Entity`稳定排序，只取出当前命中包一次。
  `combat.rs:58/110`分别把弹丸/范围放入阶段1/2，Direct为阶段0。
- `combat.rs:123`统一算实际损血，并在非零损血时发Damaged；它当前返回`()`。致死随后执行状态/运动/OnDeath来源清理。
- `statuses.rs:231–242`目前把来源效果压成`Option<AbilityId>`，Stun/Root按来源生命+ability+效果索引刷新。
  直接把所有装备效果映成None会错误合并同一施法者的不同装备实例。
- `combat.rs:170`重生保留Entity/Unit和Explicit来源，只重置角色技能冷却；没有通用“复制角色配置”的重建。
- `mod.rs:84`整场reset移除所有Unit/Projectile/AreaEffect及linked children，保留地图/目录/身份计数器。
  装备运行态放在Unit组件中，即可随现有reset自然清理。

## 2. 推荐的模块Interface

建议一个`simulation/equipment.rs`拥有装备目录、六格实例及其属性来源；不引入Trait或通用物品继承树。
主动施法来源归并由另一调查`casting-source.md`给出，这里只规定装备实例可提供定义和冷却的只读入口。

```text
EquipmentId(u16)
EquipmentInstanceId(private ModifierSourceId)
EquipmentDefinition {
  name, description,
  flat: AttributeDelta,
  additive_percent: AttributeDelta,
  passives: Vec<EquipmentPassive>,
  active: Option<AbilityId>,
}
EquipmentPassive::OnBasicAttackHit { effects: Vec<Effect> }
EquipmentCatalog::register(id, definition, &AbilityCatalog) -> Result
EquipmentCatalog::get/iter
EquipmentSlots::get/iter (always six positions)
EquipmentInstance::{id, definition_id, active_cooldown} readonly getters
equip_equipment(&mut World, unit, slot, definition_id) -> Result<EquipmentInstanceId, EquipmentError>
unequip_equipment(&mut World, unit, slot, expected_instance) -> Result<EquipmentId, EquipmentError>
```

名字供主方案统一。推荐显式slot参数、占用时拒绝，不做隐式替换；UI可从只读六格找到第一个空位。
这样一件新装备注册/装配失败，不会先把旧装备卸掉。以后需要“换装”可另设计真正原子的替换，不先实现两步假替换。

每个实例的属性来源和实例身份可以是同一份已分配ModifierSourceId的私有包装，避免两个计数器同步。
装备模块不公开其可写来源句柄。若HUD需要整数/字符串身份，可给InstanceId一个只读值getter；
属性来源的原始值只需`pub(super)`读取桥梁，无需开放任意来源构造。
浏览器回显字符串后，客户端与当前实例的只读值比较，并使用已解析的真实类型提交指令。

定义使用两份AttributeDelta而非任意AttributeModifier，内部统一构造Explicit、move_speed_multiplier=1。
装备作者不应意外把永久装备写成OnDeath来源，也不应混入Slow使用的最终移速系数。
若最终方案选择直接存AttributeModifier，至少在注册时明确校验或规范化这些生命周期字段。

### 定义与装配校验

1. 目录拒绝重复ID、空白名称、非有限属性、空/非法被动效果和未知主动技能ID。
2. 被动只允许已经支持的Damage/ApplyStatus/Displace列表，使用共同的效果验证实现；不要复制公式和控制校验。
   当前`ImpactDefinition::valid`在`abilities.rs:87`私有，可提升为模拟内部可用入口，
   用Direct/Enemy约束校验被动的绑定目标效果；不必为被动虚构一个AbilityId。
3. 装配先验证unit/state/slot/空格/定义，再分配来源；创建Explicit来源成功后才发布实例。
   在已验证unit和定义下，同步属性提交应没有正常失败路径；仍保持Result，不先写槽再尝试属性。
4. 卸下检查当前slot的expected instance，移除该来源后再移走实例；无效/OOB/陈旧实例不改变属性或动作。
5. 装卸通过模块同步入口使暂停面板立即看到真实最终属性，不借JS重算，也不等待暂停不会执行的固定步。
6. 装卸本身不实现金币规则。训练场Adapter决定何时允许操作，模拟Interface负责所有权、容量和数值合法性。

## 3. 重复装备与生命周期建议

首期建议明确允许重复定义，属性和无内部冷却的命中被动按实例叠加，主动冷却按实例独立。
不默认引入“唯一被动组”或按同名去重；那需要独立产品规则，不能从某个商业游戏常识推导。

| 时机 | 建议规则 |
|---|---|
| 新实例装配 | 分配新身份，主动冷却初始0；角色QWER不受影响 |
| 普通暂停 | 槽位/属性保留；固定步冷却冻结 |
| 普通死亡/重生 | 装备实例和Explicit属性保留，不重复加属性；不重新查角色默认装备 |
| 装备主动冷却 | 建议死亡期间继续按固定步减少，重生不重置，避免死亡刷新；需在alive早退前单独推进 |
| 明确卸下 | 从该次之后的新攻击/新装备施法中移除；陈旧实例请求失效 |
| 重装 | 新实例、新来源；训练场可得到新实例初始冷却，这是无经济模式的明示基线，不承诺购买/出售平衡 |
| 整场重开 | Unit上的实例随旧局删除；目录和既有分配器保留，新单位初始空六格 |

核心可允许死亡单位装卸，因为Explicit来源本来支持死亡更新；训练场仍只在暂停管理面板操作。
卸下正在前摇/引导的主动如何取消，应使用共同施法来源来辨认实例，参见casting调查；
不要从装备模块取消所有普通角色动作，也不要删除已提交的弹丸/区域。

## 4. 真正的被动MVP

推荐首期只有`OnBasicAttackHit`一个封闭触发种类，不做泛事件订阅器、递归触发链或未使用的内部CD。
示例可为普攻附加魔法伤害，或普攻命中施加短Slow。它们有可观察战斗效果，足以证明被动不是另一份静态属性。

### 比较

| 方案 | 优点 | 与现有规则的冲突/代价 |
|---|---|---|
| 命中后读当前装备，监听Damaged | 触发实现表面短 | 当前Damaged仅source Entity，会混入新生命/新装备；装备在弹丸飞行中变更会改写已提交攻击；来源销毁丢被动；监听自身伤害还需防递归 |
| 普攻出手提交时快照被动，随载体落地 | 来源/属性/效果均不可变，来源移除后仍成立，排序可显式定义 | 需给普攻payload附带有界附属快照，在同一resolve路径多处理一层 |

推荐第二项。被动目录/槽位读取只发生在**普攻出手提交点**，不是普攻前摇开始，也不是命中时。

### 有界解析流程

1. `release_attack`通过已有出手复核后，生成基础普攻快照与按slot、passive index排序的装备被动快照。
2. 所有被动的伤害表达式都成功快照后才提交攻击间隔/AttackReleased/弹丸。快照失败走InvalidDamage，
   不应把现有“finite basic attack”的expect扩大到可能溢出的新被动表达式。
3. 普攻payload保存这些只读附属payload。建议单独私有`AttackPassiveSnapshot`或类似容器，
   不把装备槽组件引用、可写指针、回调函数放进载体。
4. 原命中包仍按现有stage/sequence/carrier/target排序。
5. 对每个基础普攻包，先用同一个`resolve_effects` helper解析基础效果；`combat::apply_damage`可改为内部返回实际损血量/成功事实。
6. 只有基础普攻实际损血>0且目标生命仍有效时，依固定slot/index顺序，用相同target、carrier、center/direction解析其被动payload。
   致死基础攻击不对已经死亡目标施加后续状态，符合当前逐效果生命复核规则。
7. 这个helper不再递归启动被动；只有最外层BasicAttack payload可携带附属列表，EquipmentPassive来源没有附属列表。

这种局部顺序是“每个正常命中包→其附属被动→下一个正常包”，不需要增加一个全局被动阶段，
也不改变旧包的相对顺序。若改成统一stage3再解析全部被动，后续其它技能可能先杀死目标，
导致早先已经命中的普攻被动不触发；可以选择，但必须明确，不建议无需求引入这种语义差异。

### 来源与状态身份

DamageOrigin建议增加独立EquipmentActive与EquipmentPassive来源，至少包含实例ID，
被动另带passive index；可加静态EquipmentId以便诊断。普通Ability与BasicAttack保留现有含义。
statuses的Stun/Root刷新键应使用完整效果origin（加来源生命+effect index），
而不继续把装备都变成ability=None；否则两件相同/不同装备的控制错误地共用一个实例。
已有`StatusView::ability()`可以保留角色技能兼容，只作显示，不再充当完整来源唯一键。

### 对装备移除和来源死亡的含义

- 卸下发生在出手前：该次出手不快照其被动，最终AD也已经移除装备加成。
- 卸下/替换发生在出手后：已提交基础攻击和附属被动保留原属性/来源，落地后照常结算。
- 装上发生在出手后：不能追加入飞行中的旧攻击。
- 施法者死亡/销毁：不取消已提交基础攻击及其被动，不查询当前装备。
- 锁定目标换生命：整个旧锁定弹丸无效，基础和被动都不能命中新生命。
- 首期无被动内部CD，不产生“出手消耗还是命中消耗”的隐性预留状态。以后若要求ICD，需要重新设计提交/触发时机，不能随手加一个计时字段。

## 5. 必测行为

新增公开`tests/equipment.rs`，沿用MinimalPlugins、SimulationPlugin、spawn及命令/事件观察：

1. 登记重复/空名/NaN/非法效果/未知主动失败，原目录保持且ID排序稳定。
2. 六格上限、占用格拒绝、OOB、未知装备/单位/陈旧instance装卸失败无部分变更。
3. 两件相同装备分别加成，卸下一件只移除自身来源；重复卸下/旧请求不能卸掉新实例。
4. 暂停中装卸的真实最终属性立即更新，恢复后普攻/技能使用同一最终属性，无HUD公式。
5. 角色技能四槽保持，装备主动独立；两个同定义实例冷却独立。
6. 普攻命中附伤/Slow真实生效，普通技能和被动自身伤害不能再次触发。
7. 单次基础攻击实际损血为0、未命中、目标死亡、目标换生命时，不产生不合法被动。
8. 普攻出手后卸下/装上/改属性/来源死亡，基础与被动快照保持；下一次攻击按新装备状态。
9. 多实例多个被动按固定slot/index顺序，独立来源控制不被错误合并，致死后跳过后续目标效果。
10. 普攻被动快照数值溢出不进入冷却/不发射部分弹丸，且无panic。
11. 死亡重生不重复加属性，实例身份保持；主动冷却按明确死亡规则；显式重开空装备且来源ID不复用。
12. 原138项保留，尤其类型化伤害、同tick互相施法、有序致死、快照存续、精确控制时钟和reset场景。

## 6. 决策边界

以上是最小技术推荐，仍需主方案统一名字、重复规则、死亡冷却策略和训练场获取方式。
没有从仓库推导金币/商店/唯一被动组、生命上限装备或完整商业平衡；这些需要明确范围后另做。
