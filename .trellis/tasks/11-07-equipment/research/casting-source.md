# 装备主动能力复用施法流程

调研基线：`8f7107c`。本文件只记录源码调查和推荐设计，未修改产品代码、未运行测试、未启动服务。
关联：`prd.md`，`research/inventory-and-passives.md`；后者负责装备目录、实例与被动。

## 1. 结论

保留四槽`AbilitySlots`和既有`UnitCommand::CastAbility`，新增带装备实例身份的主动命令。
二者在orders入口归一成封闭`CastSource`，复用同一个开始、前摇复核、消耗、冷却提交和执行流程。
装备目录的主动能力引用已有`AbilityId`，不复制`AbilityDefinition`或另写一套物品投递/伤害系统。

推荐最小改造：

1. `Casting.slot`改为`Casting.source`，仍保存已接受的`ability`和目标生命。
2. 单一`begin_cast`/`commit_cast`通过来源解析器读写技能槽或装备实例自己的冷却。
3. `DamageOrigin`增加装备主动与被动来源，包含实例身份；状态刷新键使用完整来源。
4. `Channeling`已有不可变`pulse.origin`，可由此识别装备实例，无需再存一份相同归属。
5. 卸下取消该实例未提交的前摇和未来引导脉冲；已经发出的弹丸/区域继续使用提交快照。

这只为已有的两种施法来源建立实际接口，不引入trait、插件注册、脚本或任意技能槽解释器。

## 2. 当前代码事实

| 位置 | 当前行为 | 扩展影响 |
|---|---|---|
| `simulation/orders.rs:149–206` | CastAbility直接读取AbilitySlots；校验冷却、权限、资源、Busy、目标后写Casting | 不可复制整段用于物品；只将来源解析与开始统一 |
| `simulation/actions.rs:26–50` | 跳过死亡单位，随后更新普攻间隔、能量和四槽冷却 | 装备时钟生命周期必须显式决定，不能靠复用四槽数组暗自继承 |
| `simulation/actions.rs:102–149` | Casting保存slot/ability；Channeling保存pulse并逐次emit | Casting必须捕获实例；Channeling可以从pulse.origin识别归属 |
| `simulation/actions.rs:201–301` | 单一提交点重新验证目标生命/控制/能量，生成整组快照后才扣费和提交冷却，随后Impact/Dash/Channel | 保持这个顺序，装备不能绕过控制、Busy或快照溢出检查 |
| `simulation/actions.rs:9–23` | Root仅检查Casting.ability的执行是否Dash；Stun/Airborne取消自主动作 | 保留ability字段后这一控制逻辑可原样复用 |
| `simulation/effects.rs:85–130` | 快照保存UnitLifeRef、DamageOrigin、定义效果及位置/朝向 | 装备出处应在这里冻结，命中时不再找当前装备 |
| `simulation/statuses.rs:231–242` | Stun/Root按来源生命、Option<AbilityId>、效果索引刷新 | 单纯把装备映射None或同AbilityId会错误合并不同装备实例 |
| `simulation/combat.rs:158–210` | 死亡清动作/状态/运动；重生只将四技能冷却归零 | 装备保留与冷却策略需要自己的明确处理 |
| `client/controls.rs:121–168,304–332` | 查询四槽并按Targeting选择/提交；SelfCast直接释放 | 共用目标转换，来源只负责给出定义、冷却、稳定实例 |
| `client/scene.rs:443–489` | Aim::Ability按AbilityCatalog绘制范围/区域 | 只扩展只读来源解析，不增加按装备ID的绘制分支 |
| `client/abilities.rs:41–90` | 展示和不可用原因目前直接依赖AbilitySlots | 提取接受cooldown数值的共用展示函数 |
| `client/web.rs:193–205` | DOM技能请求为一次性data-skill | 装备请求也须在暂停/just_captured时排空，不能跨菜单重放 |

依据规范：`.trellis/spec/simulation/attributes-and-damage.md`、`abilities-and-statuses.md`、
`runtime.md`、`.trellis/spec/client/browser.md`和`character-selection.md`。

## 3. 建议类型和接口

以下为设计示意，最终命名由主方案统一：

```rust
pub enum CastSource {
    Ability { slot: usize },
    Equipment { slot: usize, instance: EquipmentInstanceId },
}

// 既有命令完全保留，新增独立来源的命令。
UnitCommand::CastAbility { slot, target }
UnitCommand::CastEquipment { slot, instance, target }

UnitAction::Casting {
    source: CastSource,
    ability: AbilityId,
    target: AbilityTarget,
    target_life: Option<UnitLifeRef>,
    remaining: f32,
}

EquipmentDefinition { /* 内容与属性等 */ active: Option<AbilityId> }
```

装备格保存实际实例与其冷却，不把六件主动映射成技能槽4–9。
同一AbilityId出现在角色技能和两个装备实例时，必须有三个独立冷却所有者。
装备实例身份从实际装配只读getter取得；对外命令不能只带可重用的slot。

可在一个私有`casting.rs`中集中开始/提交的复杂流程，也可暂留actions中的提交点并提取小型来源解析函数。
优先目标是orders不再复制权限/消耗流程、两种来源只有一处提交，而不是为了文件数量引入多层转发。

```text
begin_cast(world, unit, source, target) -> Result<(), Rejection>
commit_cast(world, unit, source, expected_ability, target, target_life) -> UnitAction
cancel_equipment_cast(world, unit, instance) // 内部，卸下生命周期使用
```

来源解析器只需封闭match两个真实所有者，负责：

- 读取当前定义引用和冷却。
- 校验slot、实例和已接受的AbilityId仍匹配。
- 在提交成功后写回同一个来源的冷却。
- 生成不可变DamageOrigin；装备类型/实例不能在命中时从易变槽位反查。

`EquipmentInstanceId`可按另一份调研建议私有包装`ModifierSourceId`，共享已有单调分配器。
若JSON需要字符串，可在attributes所有者提供仅模拟内部的数值读取，并给装备ID实现Display。
不要用Debug格式作稳定协议，也不必公开任意ID构造/FromStr；客户端收到字符串后先与实际格实例比对，
通过后使用该格现有typed handle向核心提交。

## 4. 开始与提交的同一条路径

### 开始

1. 确认单位存在、存活、未被自主Dash占用；保留orders现有通用前置检查。
2. 解析source，确认实例存在、有active且绑定正确。
3. 读取该来源冷却，调用既有`ControlState::permits`，检查能量和Busy。
4. 复用`validate_target`，将Enemy锁为UnitLifeRef。
5. 保存Casting的source/id/target/remaining并清普通路线。

开始失败必须继续保留原动作、持续指令和路线；物品主动也不能因为按键而先取消后摇。

### 提交

1. 再次验证source仍是原实例与原AbilityId；卸下后换进同定义的新实例不能接管旧前摇。
2. 复核当前目标、锁定生命、控制和能量；可共用来源冷却复核，避免将来新增变更入口时失效。
3. 生成全部影响快照；伤害不可表示时拒绝，不能只丢某个效果。
4. 仅在全部通过后扣费，写来源冷却，发对应提交事件，进入既有Impact/Dash/Channel实现。

物品常见mana_cost=0是内容配置，不能在装备路径中跳过通用资源检查。
即时物品也仍受同一个自主动作Busy规则约束；“可在其它施法中额外按下”的离体主动不是本期默认能力。

## 5. 取消、移除与生命周期

| 情形 | 推荐行为 |
|---|---|
| 选中装备后、命令受理前卸下/替换 | 身份校验拒绝；不使用槽位里的新装备 |
| 前摇中卸下对应实例 | 取消该次Casting，不扣费、不开始冷却；保留原持续UnitOrder |
| 卸下其它实例或普通加属性装备 | 不取消无关角色技能/装备动作；后续提交读取当前属性 |
| 弹丸/延迟区域已经提交后卸下 | 载体和快照继续；卸下去掉属性不会逆改已提交伤害 |
| 装备引导已经提交后卸下 | 取消未来Channeling脉冲；已emit的区域保留，不退款 |
| 装备Dash提交后卸下 | 已提交Movement.dash继续；只能由既有运动/控制规则中断 |
| 施法者死亡 | 既有combat清动作，已提交弹丸/区域保留；不新增物品专属死亡结算 |
| 普通暂停/恢复 | 装配、实例与冷却保持，时钟冻结，不重建装备 |
| 明确整场重开 | 随Unit及载体清理；新实例身份不得复用，目录保留 |

Channeling的归属不必双存：`pulse.origin`扩展后已包含EquipmentInstanceId。
`cancel_equipment_cast`匹配Casting.source或Channeling.pulse.origin中的该实例即可，
不能调用通用Stop清掉无关持续指令，也不能删除已经存在的Projectile/AreaEffect。

需要主方案明确、不能由代码偶然决定的两项：

- 装备冷却是否跨死亡保留。建议保留并在模拟tick中继续倒计时；这需要装备时钟在alive guard外推进，
  不改变既有QWER重生归零。若选择沿用训练场技能重生归零，也应明确写入规范并由装备生命周期处理。
- 训练场卸下再装是否创建全新实例/冷却。无背包/经济时可以如此定义，但必须公开语义；
  将来若需要保留离身物品的冷却，应保留同一个实例而不是重新构造，不应把冷却硬绑AbilityId。

## 6. 伤害出处与状态刷新

保留既有`BasicAttack`和`Ability(AbilityId)`两个DamageOrigin变体以保持现有事件匹配。
新增装备出处可采用：

```text
EquipmentActive { equipment, instance, ability }
EquipmentPassive { equipment, instance, passive_index }
```

其中实例用于归属，equipment/ability用于移除后仍可解释的已提交事实；不以slot作出处身份。
被动出处与BasicAttack区分，也为“被动伤害不会再次触发普攻命中被动”提供明确判断。

`StatusView`增加私有完整origin并在Stun/Root刷新键中比较它，保留当前source生命和effect_index条件。
已有`ability()`getter可保留为展示兼容：角色技能及装备主动返回UnderlyingAbilityId，被动可返回None。
不能仅在新match分支返回None而仍比较ability，否则两件物品会共享一个状态刷新槽。

提交事件推荐角色技能仍发原`AbilityCommitted`，装备发单独`EquipmentActivated`或同义事件。
当前客户端没有依赖AbilityCommitted渲染；既有测试只统计角色技能时，避免把装备额外塞入该旧事件导致含义漂移。
不建议同时发两个等价“已提交”事件供被动消费，否则容易重复触发。

## 7. 客户端迁移与兼容

- PlayerController仍只保存一份aim。可保留`Aim::Ability(usize)`并加装备实例变体，通过共用source/definition解析
  和target转换复用同一选择/提交代码；也可统一为Cast(CastSource)，但后者需要机械更新多处已有unit断言。
- Targeting的Direction/Point/Enemy/SelfCast逻辑复用；鼠标和1–6热键只标识来源，不判断具体装备ID。
- refresh_aim还须检查装备实例仍存在且可用；控场、卸下和暂停不能留下错误瞄准。
- AbilityMetadata已经是静态通用投影，装备active可直接复用；将不可用原因提取为
  `state/control/cooldown/definition`函数，避免复制控制/能量文案与规则。
- 既有`HudView.aim:Option<usize>`和abilities数组可继续代表四技能，再增加独立equipment_aim/equipment数据；
  两个JSON瞄准字段必须来自同一个enum投影，不能再创建第二份权威aim状态。
- scene的范围/目标绘制通过source解析同一个AbilityDefinition，不复制装备特有几何分支。
- 新DOM请求要和旧技能请求一样排空、在just_captured当帧忽略；虚拟鼠标点击HUD后必须即时消费，保留窗口事件顺序。
  如果统一成闭合CastIntent通道，须同步迁移旧data-skill的捕获边界回归，不能仅让新装备通路漏过该清理。
- 装备DOM选择器使用独立data-equipment-slot等标识，不能扩大旧`[data-slot]`选中范围，避免旧四技能HUD/测试误选六件装备。
- Native至少保持默认角色和编译路径；新输入处理不能无条件依赖DOM。

## 8. 兼容性与迁移顺序

当前公开场景中Casting匹配位于`tests/simulation.rs:1028,1813`，均为`{..}`；
Channeling匹配位于`tests/reset.rs:199`，同样是`{..}`。没有外部测试构造Casting/Channeling或读取slot字段。
因此改Casting来源可局部迁移，而无需重写既有行为测试。
DamageOrigin::Ability与BasicAttack在simulation/status测试中有具体事件断言，必须保留。

建议先做不改变行为的共同施法改造，通过138项后再加装备入口：

1. 内部CastSource/共同begin与commit，只接既有CastAbility；旧语义全部通过。
2. 加装备实例解析和独立冷却，沿同一条路径进入现有执行。
3. 扩展origin、状态刷新键及卸下取消归属，再接客户端选择/瞄准/HUD。

最少新增公开行为场景：角色技能与装备共用同AbilityId却独立冷却；两个相同装备独立；
空槽/无主动/非法slot/旧实例拒绝；前摇中卸下再装同ID不能提交；控制/Busy/资源/目标失败无扣费；
根禁Dash但不禁普通物品；已提交效果移除后保留快照；引导卸下取消未来脉冲但不抹掉旧区域；
死亡/重生和整局重开；装备来源控制不互相覆盖；鼠标/热键与捕获清理、普通恢复及小窗口/DPR2可达。

以上均经已有公开构造和命令接口验证，不添加写运行态的浏览器后门。
