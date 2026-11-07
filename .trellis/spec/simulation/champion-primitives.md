# 普攻特性、复合技能、场上对象与视野

适用于`mechanics_defs/mechanics/attack_traits/compound/battlefield/vision`及其在已有动作、结算和移动中的接线。
英雄内容与标定见[角色数值](../../../docs/champions.md)，通用构造/指令不得依赖具体角色或技能ID。

## 定义与事实所有者

- `UnitStats.kind: UnitKind`为Hero/Minion/Other，独立于Actor/Structure受效分类及gameplay的UnitRole。
- `health_regen`为每秒自然生命恢复，默认0；恢复写入通过recovery内部入口，伤害/死亡仍由combat裁决。
- `attack_passives`为不可变普攻特性：CurrentHealth附伤、Frost减速；不把装备被动重复写入这里。
- `UnitMechanics`保存护盾、共享攻击专注/强化、角色槽充能、重施和临时属性来源。字段不对外写，只读getter供AI/客户端。
- `BattlefieldObject`保存旗、临时墙、鹰灵和短暂侦察；具有来源生命、阵营、激活/到期tick及显示位置，不伪装成可选敌军单位。
- `TeamVision`缓存团队视野圈，`point_visible/team_can_see`是规则、AI、HUD和拾取共用查询。

## 执行组合

`AbilityExecution`在保留Impact/Dash/Channel的基础上提供Line/Shield/Standard/Leap/Empower/Volley/Scout/Arrow。
定义负责目标、几何、效果、运动/对象参数；compound只协调原有模块，不另开扣血、最终属性或单位位置写入口。
所有执行仍走preview→前摇→提交复核→快照验证→一次扣费/CD；注册时拒绝非有限、非法时间/几何/组合。
单位普攻特性的冷却/持续时间复用技能的可表示时长校验；Volley/Scout还须验证射程与速度推导的最长飞行寿命，
不能把溢出的浮点时长饱和转换为tick后再相加。Arrow的初速和最大速度均须有限且为正。

`BonusAttackDamage`在提交时快照最终AD与裸基础AD差值；不能将总AD错作额外AD。
`ArmorReduction`在有序效果列表对应位置生效，以私有属性来源维护期限；同一来源/生命/技能重命中刷新，死亡清除。
`DecayingSlow`定义总时长、起止强度与衰减时长；每tick更新最强Slow贡献，复用现有控制/属性生命周期。

## 普攻与护盾

普通攻击冻结倍率、箭数、单位特性与装备命中快照；强化多箭仍只有一次普通命中/装备触发发现，不递归。
CurrentHealth按命中前HP计算，来源生命+特性索引+目标生命独立冷却；主伤致死后不继续附伤，结构不接受附伤。
护盾吸收抗性后的伤害，剩余才裁剪到HP；不能先按目标HP裁剪整次伤害，再让盾吸收。
`Damaged.amount`仍是实际生命损失；被盾全吸收的正伤害命中仍能触发附属效果，并记录受击脱战时间。
死亡清当前增强和临时来源，已发射攻击的倍率/箭数不因来源后来变化而重新计算。
多个护盾的只读总量以f64聚合并饱和到有限f32，独立吸收/到期实例保持不变。

## 对象、移动与临时地形

军旗落地延迟结束才激活光环/视野/连接资格。光环仅友方Hero，按最强旗效果聚合，不多旗叠加；已学被动独立来源。
已学攻速被动按不同技能定义去重，以f64聚合后饱和为有限来源值；初始选角预览与实际创建都通过属性模块的同一修正计算，
不能先在f32基础攻速上相乘，或用预览中的另一套技术攻速限制。
军旗联动使用实际来源生命和配置几何；沿途击飞按目标生命去重。`DashLinkSpec.traverse_terrain`允许越墙，
`can_cast_during_motion`允许途中施法；Move/Attack不能替换已提交运动。Root允许旗拉，Stun/Airborne可打断链接；
被打断且处于墙体中的单位由movement排离。`requires_mobility()`只对Dash/Leap要求can_dash，不把Line联动误判成自主Dash。
位移始终由movement执行，不直接写UnitPose。
Leap冻结落点，来源死亡不落地结算；地形对象按寿命保留，重施移除自己所有的墙句柄。
临时圆环同时参与path/walkable/sweep，生成重叠排离也由movement执行。创建前在全部既有几何加候选新圈下预计算每个受影响Actor的合法位置；
扫掠用f64求单位半径膨胀后的内/外圆首次交点，不能用固定空间采样跳过合法薄圈或短暂擦边交段。
优先径向排离，受阻时只搜索原位置附近（最多圈半径+厚度+单位半径），并要求旧几何下的扫掠可达，不能越过旧墙或瞬移整图。
全部位置可行后一次提交地形与移动；任一Actor局部无安全出口则不创建整圈、不移动任何Actor，发出
`SimulationEvent::TerrainPlacementFailed { unit, position }`。跃击已提交的伤害/消耗/冷却仍保留，不开放重施。
projectile_sweep忽略临时墙，不能把墙变成挡箭技能。
reset移除所有对象、临时墙、待命中和机制记忆，保留原单调身份分配器；不能让旧局墙或侦察进入新局。

## 视野与目标

构造和reset即时刷新，固定步在指令前及结算后刷新；暂停开局第一帧不能默认全图显示。
没有视野源的独立Arena保持原全视野。单位、旗、鹰灵、大箭提供友方视野；不读取玩家输入或敌方未来路径。
Enemy技能、普攻新锁定和持续追踪只读当前可见敌；方向/地点允许盲打，已提交弹丸按几何和生命身份继续。
Volley共享施法序号对同目标去重，但后续箭仍被该目标挡下；Arrow只由Hero拦截，小兵仍可受到爆炸伤害。
鹰灵路径按距离留下短时视野，终点独立计时，不把整条路径永久暴露。

## 明确组合限制

专注是单位共享攻击资源：相同Empower配置重复槽合法，不同Empower配置混装在构造/角色注册时拒绝，不能隐式取首槽。
充能和对象重施当前属于角色四槽；EquipmentCatalog注册时拒绝Empower、Scout和带地形重施的Leap主动，避免注册成功后永久不可用。
需要扩展这些组合时先修改状态来源模型、校验、预览、提交、卸装/死亡和HUD全链，不能只解除注册限制。

## 验证

`tests/champion_content.rs`走真实builtin内容与公开命令，验证套件配线；`tests/champion_mechanics.rs`验证通用边界；
`tests/terrain_placement.rs`覆盖受阻排离、多圈与原子拒绝；`tests/definition_validation.rs`覆盖非法定义的原子拒绝、盾总量和被动属性预览的有限边界。
原simulation/status/equipment/reset套件继续覆盖旧语义；新源/状态/对象必须检查暂停、生命代次与整局重置。
`tests/terrain_placement.rs`通过实际跃击覆盖径向落点被静态障碍阻挡、双圈重叠与独立拆墙，以及封闭腔室的整体拒绝/结果事件和已提交伤害保留。
角色/几何/输入变化按client规范执行浏览器；make check与WASM Clippy通过后再记录验收。

## 出生、复活与临时地形

`NavigationMap::resolve_placement(preferred, radius)`是构造/复活共用的只读解析：静态非法位置仍拒绝；
仅被临时圈阻挡时在局部范围确定性找全几何合法点，并要求静态直线可达，不穿静态墙。
`validate_unit_spec`与`spawn_unit`共享解析，初始previous/position均为解析点，`Unit.home`始终保存原请求点。
复活没有安全点时保留旧dead life，下tick重试；只有确定位置后才补满资源、递增life并发布Respawned。
临时墙消失后的再次复活仍回原home，不能累计偏移。
模式配置可达性用当前实际NavigationMap::path_static校验，临时圈造成的暂时无路是运行状态，不能成为wave_error。
实际移动仍用完整几何；圈内可走点的新兵可以暂时等待墙到期，出兵tick/双方原子pair/3+3不改变。
公开回归见tests/placement_lifecycle.rs与tests/match_mode.rs。
