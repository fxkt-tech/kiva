# 技能几何、提交事实与表现接缝调研

研究者：`check_battle_presentation`，2026-11-08。本记录只读当前代码、规范和任务，不修改产品、不运行 Cargo/Trunk/GPU、不接管服务或 Git。入口是本任务 PRD、implement/check JSONL、client 规范及 simulation 的 abilities-and-statuses/champion-primitives。数值以下均为项目当前真实定义，不用端游资料替换本项目规则。1 世界单位 = 100 端游距离单位。

## 当前八技能的准确映射

来源：`src/gameplay/content/champion_abilities.rs`，执行：`src/simulation/compound.rs`、`combat.rs`、`battlefield.rs`。

| 实际 AbilityId / 技能 | 当前权威几何与时序 | 指示器 | 已提交表现应依赖的事实 |
| --- | --- | --- | --- |
| 11 皇子 Q | Direction；范围7.85，线全宽1.36，前摇0.4秒。命中计算为线段最近点距离 ≤0.68+受击者半径，贯穿。联动另有 reach8.5、全宽3.0，选本生命已经激活的自有军旗，取方向上最远合法旗，真实运动速度18。 | 范围内的有宽线段/箭头；指向近点仍是完整7.85方向技能。不能把联动8.5画成Q伤害长度。联动路径可选显示，若做则与同一真实选旗规则一致；首版不必另复制一套选旗预测器。 | 提交时真实方向/起点；Contact表达各目标接触；联动尾迹跟真实UnitVisual/运动事实，不自行移动英雄。现有事件没有方向，见API缺口。 |
| 12 皇子 W | SelfCast；周围半径6.0减速，护盾4秒；技能选择即提交，无需左键。 | 不能新增二次瞄准；实际施放时地面冲击圈半径6.0，身上护罩另读实际盾实例。范围圈不是护罩球体半径。 | 提交事实驱动一次扩散；持续护罩读正amount的真实盾及其来源/到期，不能仅靠4秒客户端倒计时。 |
| 13 皇子 E | Point；最大范围8.6，超距沿方向截断；落地伤害半径2.0、延迟0.5秒；落旗后持续8秒，光环半径10、视野7。 | 主提示为实际解析落点的半径2.0落点圈/旗标。当前scene画的是10米光环，不能继续把它当伤害圈；光环只可用更弱的次级虚线表示。 | 提交→落旗的0.5秒视觉飞行；AreaActivated表达落地，即使没有敌人也落地；BattlefieldObject.active之前不显示已经站立的旗，之后跟真实对象生命周期。 |
| 14 皇子 R | Enemy且仅Actor/Hero；施法范围6.5，不额外加双方半径；超距指令尝试走近。提交时冻结目标位置，跃击0.35秒，落地伤害半径3.5；成功才生成半径3.5/厚0.45/持续3.5秒的真实墙。 | 模型候选与preview_cast一致；合法英雄圈和3.5落点圈，超距用“走近”色，兵/友军/隐藏目标不可绿。瞄准时随当前目标，提交后不可继续追踪移动目标的落点。 | 真正跃击用MotionView/UnitVisual；AreaActivated在实际落地触发，无目标也有效。墙仅在Terrain对象存在时显示；TerrainPlacementFailed仍可落地伤害，但不能显示假墙。重施只拆墙，不重播跃击/落地。 |
| 21 寒冰 Q | SelfCast；4层才能激活；6秒强化，实际每次普攻冻结5箭与1.1倍率，模拟仍只有一次普攻/命中触发。 | 无左键瞄准；选取可用状态继续读现有mechanics，不能自己数普攻/时间。 | 激活/持续光效读真实empower_origin与empowered状态；Q箭从普攻快照读强化来源/箭数，不能因发出后buff到期、源死亡或重生改回普通箭；不能新增5个伤害载体。 |
| 22 寒冰 W | Direction；7支真实Projectile，范围12，速度20，碰撞半径0.1。spec.angle=27.75°是**整个扇形**，每箭 `-angle/2 + angle*i/(count-1)`；最外为±13.875°。组内同目标只受效一次，但后续箭仍可撞上该目标并消失。 | 7条实际射线/扇形边界；全宽0.2的箭道。不能把27.75°当半角画成55.5°，不能按别处端游9箭资料改几何；近点仍沿方向发满12。 | 每支真实Projectile附着外观与尾迹；单箭终止用实体身份，受效火花按cast组+目标生命去重。不能仅从一条提交事件伪造7条客户端独立运动。 |
| 23 寒冰 E | Point；范围250，超距也在resolve_target中截断。速度14，视野半径10；沿途每移动1单位留下0.5秒Reveal，终点Reveal5秒。没有伤害。 | 真实解析落点+路径，终点视野圈半径10要与伤害圈有区别。当前abilities::clamps_point_target漏Scout，不能继续用该不完整帮助函数判断红绿；以preview_cast解析为准。 | 真实Scout对象插值；同一个Entity到站时kind从Scout变为Reveal，表现须换相位而非继续静止飞鸟。路径Reveal与终点Reveal均读真实对象；不自行铺永久视野或在预计抵达时间伪造到站。 |
| 24 寒冰 R | Direction；范围250，半径1.3即全宽2.6；初速15，按2/秒加速，最大21。仅敌方Hero拦截；爆炸半径4；眩晕按飞行距离28从1增至3.5秒。飞行提供半径3.5视野，结束另有1秒Reveal。 | 宽2.6的长箭方向通道；鼠标近点不是终点，不能在该点画承诺爆炸的落点圈。只按已知地图/viewport裁剪显示，不修改真实射程，不利用隐藏敌人预测碰撞。 | 真实Arrow Projectile及其实际终止事实；英雄命中才有伤害爆炸，撞静态边界/射程结束可淡出/碎裂但不可虚构AOE命中；末端Reveal读对象。 |

Direction/Point规则在`orders::resolve_target`。Point Dash/Standard/Scout在**提交时当前施法位置**解析最大终点；其它Point依然严格OutOfRange。Enemy的preview是严格OutOfRange，实际命令才转为追近目的。指示器不得把Enemy追近应用到Direction，或把所有Point都改成最大距离语义。

范围250的圈大于整张64×64地图。表现可裁剪到当前画面/已知地图边界，防止无意义的跨屏线圈；这只是绘制裁剪。E的规则允许地图外有限目标，不能擅自把解析点改到地图边缘。不要为预测器公开整个动态导航几何再据此泄露隐藏障碍；当前临时R墙本来就不挡Projectile。

## 指示器与输入必须共享的只读接缝

`client::abilities::resolve`已统一实际技能槽/装备实例，不过其CastInfo只有definition/cooldown，缺实际AbilityId。建议增加`ability: AbilityId`；不要把id复制进AbilityDefinition，也不要从QWER槽或英雄默认技能反推。

建议新增小的`client::ability_targeting`模块，供controls和indicators共用：

```text
resolve_intent(world, player, CastSource, PointerState)
  -> { actual AbilityId, original AbilityTarget?, resolved target?, preview result }

preview result = Ready | ApproachingEnemy | Invalid(Rejection) | NoTarget
```

- Direction由当前ground与施法者UnitPose构成非零有限方向；Point保留原点击点提交，resolved只供预览，不把截断后的点反写命令。
- Enemy复用当前有序模型候选、alive/team/vision过滤和`preview_cast`。可以从合法可见候选中选首个Ok/OutOfRange；若保留旧的“首个通用敌人拒绝就停止”，指示器也必须同样选择，不能一边高亮英雄、一边左键提交前面的兵/基地。采用前者时补重叠候选回归。
- 只有Enemy的OutOfRange标记ApproachingEnemy，其余拒绝保留原原因；这表示“尝试走近”，不承诺导航一定存在路径。
- root/control/cooldown/mana/charges/focus/装备实例过期等权限都由现有preview/只读可用性检查决定，不能再写一套按距离判断的绿线。
- SelfCast和R重施保留现有直接选择提交路径；暂停、UI捕获、失锁、缺玩家/来源时不显示世界瞄准。
- 模型悬停/点击依旧使用同一个CSS指针和真实模型候选；血条不能参与选择。验证使用权威UnitPose，附着起点使用显示UnitVisual；不能拿骨骼/屏幕坐标写规则。

纯几何构造器从AbilityExecution派生Circle/Segment/Fan/Point/Enemy形状；外观catalog只选颜色/贴图/网格。未知技能、通用Impact/Channel/Dash和装备主动按执行语义回退，不按角色或槽位回退。

## 当帧相机与贴地指示器调度

现PointerState.ground在BeforeFixedMainLoop按**上一显示Camera**计算，camera::update却会在PostUpdate滚屏/跟随/缩放。若新指示器继续直接使用该ground，当前相机中的鼠标落点会落后一个显示帧。

新`SkillIndicatorsPlugin`建议在PostUpdate执行：

```text
camera::update
  -> TransformSystems::Propagate + CameraUpdateSystems
  -> 指示器：当前Camera/GlobalTransform + 唯一CSS PointerState.position重投影
  -> VisibilitySystems::{VisibilityPropagate, CalculateBounds}
  -> VisibilitySystems::CheckVisibility
```

- 使用现有web::css_to_viewport转换，再由当前Camera.viewport_to_world与地面Y=0求交。只在当前渲染帧临时计算，不覆写PointerState、不维护第二份权威鼠标/目标位置。
- 起点锚定已显示的UnitVisual GlobalTransform；规则/范围与`preview_cast`读取实际AbilityCatalog/UnitPose，不改变UnitVisual、模拟位置或点击命令。方向类一律按归一方向画完整range，不按近鼠标点截短。
- Enemy若相机刚移动，应从当前显示Camera/模型重新取得局部候选，仍调用与controls同一候选资格/preview helper；不能一个用旧ground、另一个用新camera。原controls输入仍按最后真正显示帧判定点击，不能为绘图移动其时序。
- 少量贴地indicator mesh采用无父子的平面根，可在此阶段同时写**自己的**Transform/GlobalTransform，避免等下一次传播。不能给角色或骨骼GlobalTransform增加第二写者。引用Unity/League中的视觉offset也只限这些展示根。
- 仅before CheckVisibility不够：此时修改Visibility可能晚于VisibilityPropagate，导致首次显隐慢一帧；动态mesh还可能错过CalculateBounds。需显式在这两个集合之前，或对这些独立根提供等价的可见性/边界处理。优先共享固定单位mesh并用Transform缩放，避免每帧新建Mesh/Material。
- 所有特效根不加PickGeometry/Pickable；结束瞄准、暂停、缺玩家、装备instance失效或UI捕获时同帧隐藏并清空IndicatorView。
- 原reticle若是“黑底、alpha全1”的加色贴图，材质应unlit+AlphaMode::Add，不能普通Blend画黑底矩形；图片有效透明通道才按原素材选择Blend。纹理只提供轮廓，碰撞宽度/落点半径仍来自定义。

约定外部接口为`SkillIndicatorsPlugin`及只读`IndicatorView`（Resource、Clone/Serialize）。可投影actual ability_id、shape/status、实际显示origin/target、目标id及range/width/radius/rays/angle等最小几何用于HUD/browser断言，缺少瞄准时为None。它是渲染诊断，不参与提交验证，不接受JS反写；目标信息仍限当前可见候选。公共mod/scene/ui接线由主代理负责，新实现独占`client/skill_indicators/`。

## 原始事实目前不足的证据

1. `SimulationEvent::AbilityCommitted {unit, ability, position}`对Direction只存原点。提交并不转动UnitPose.facing（只有movement写朝向），故皇子Q方向无法事后从人物朝向可靠恢复。瞬发、同帧多技能或取消不能通过监视旧UnitAction补齐。
2. Projectile/AreaEffect的payload已冻结source life、DamageOrigin与组序号，但公开getter只有几何/阵营。按arrows、crystal标志或角色猜具体技能会破坏混装。
3. BattlefieldObject只保留owner，未保留实际技能/装备origin。Ashe E和R都会生成Reveal，不能按“owner是Ashe”或半径猜来源；Scout到站还会原Entity变kind。
4. Damaged仅表示HP实际下降。全盾吸收、纯减速/控制、没有敌人的落地和载体到边界都没有足够结果事实；不能用它当完整命中/结束通知。
5. ShieldInstance只有amount/until，empowered只有状态/截止时间。混装、多盾及buff到期后的在途强化普攻都不能从角色或当前总盾/状态可靠归因。
6. Volley各Projectile的`carrier`均取payload.sequence，同一施法7箭共用组ID。它不是单箭唯一身份；生命周期必须保留实际Projectile Entity。

## 最小可实施模拟只读接口

以下是中性战斗事实，不包含贴图、粒子、颜色、皮肤或“播放某特效”指令。既有规则/伤害/调度不变，客户端不增加第二套战斗状态。

### 提交快照

扩充既有AbilityCommitted，保留旧unit/ability/position不改语义：

```text
unit: Entity                         // legacy
ability: AbilityId                   // legacy
position: Vec2                       // legacy：Point/Enemy解析位置，Direction/Self为原点
source_life: UnitLifeRef
cast_source: CastSource               // 实际角色槽或装备实例
origin: Vec2                         // 提交起点
target: AbilityTarget                // commit复核后的解析目标
target_life: Option<UnitLifeRef>       // Enemy锁定生命；不能靠将来的同Entity复活重建
recast: bool
```

由actions中的中央helper在实际commit/recast处发布，确保来源/目标/旧position一致。Direction可保留已校验的原向量并按已有宽中间值正规化；不要读取后来的facing。只在所有预检、快照、消耗提交成功后发布；取消/非法命令不发。recast=true让音画适配能区分R拆墙，不把它当第二次跃击。

### 载体与对象

```text
Projectile::source_life() -> UnitLifeRef
Projectile::origin() -> DamageOrigin
Projectile::carrier() -> u64           // 是组，不承诺唯一Projectile
Projectile::empower_origin() -> Option<DamageOrigin>
AreaEffect::source_life()/origin()/carrier()
BattlefieldObject::origin() -> DamageOrigin
```

这些getter直接读已有payload/新增来源字段，不复制位置/伤害/计时。BattlefieldObject创建时透传origin；军旗、Scout、沿途/到站Reveal、Arrow尾端Reveal、Leap墙全部保留实际归因，不能等显示时查已卸装槽位。

ShieldInstance增加实际DamageOrigin，提供只读迭代amount/origin/remaining_seconds(tick)，过滤已吸空的0量盾。UnitMechanics增加`empower_origin() -> Option<DamageOrigin>`，与真实empowered_until同时设置、到期/死亡同时清除。`snapshot_attack`冻结当前empower_origin到ImpactSnapshot，Projectile和命中事实从快照读；**DamageOrigin仍为BasicAttack**，不能为了特效改成Ability(21)，否则装备触发/单位特性会改变。箭数仍使用既有不可变attack_arrows。

### 中性结果事件

主代理已确认采用一个ImpactResolved，形状为：

```text
common:
  source_life: UnitLifeRef
  team: Team                          // 来源可能已删除，不能迟读Unit猜阵营
  origin: DamageOrigin
  empower_origin: Option<DamageOrigin>
  position: Vec2
  direction: Vec2
  carrier: u64                       // 投递/施法组

kind:
  Contact { target: UnitLifeRef }
  AreaActivated { carrier_entity: Option<Entity>, radius: f32 }
  ProjectileEnded { carrier_entity: Entity, reason: ProjectileEndReason }
```

发出点和语义：

- Contact：在resolve_payload开始通过当前目标生命与UnitClass接受来源检查后，每payload/target一次，先于逐项effects。全盾吸收、零HP损失、纯状态仍有接触；无效旧生命/结构拒绝不发。空effects的内部自盾payload可不发，避免纯自盾虚构周围受击。不能在每个Damage或Status效果各发一次。**position取合法目标当前UnitPose位置，不直接取HitPacket.center**：Direct/Projectile的center常为施法原点，AOE则是区域中心，拿它当受击点既画错也会错滤隐藏受击者。
- AreaActivated：advance_areas真正到期时发一次，包含真实AreaEffect Entity/中心/半径；compound::queue_area发无实体的即时区域事实，覆盖Shield脉冲、Leap实际落地。没有受击者也发。位置是实际区域中心，不靠“预计0.35秒/0.5秒后”生成。
- ProjectileEnded：advance_projectiles在实际命中、静态阻挡/边界、射程耗尽、追踪目标失效时发一次，携带实际Projectile Entity。命中时明确取实际碰撞点或目标/爆炸中心；不能等despawn后回查。当前代码会将Projectile.position写到该步end而不是首个fraction触点，故应在碰撞分支就保存正确结束点。Arrow的AOE中心当前是被命中Hero位置，爆炸不能与地图边界的普通淡出混同。
- 同一个Arrow命中可以同时产生ProjectileEnded、Area/Contact事实，视觉catalog按kind分工：结束负责末端爆发/尾迹释放，每目标Contact负责小受击火花，不把每个AOE目标都生成一个主爆炸。
- cast组/target life可以用于受效去重；单箭尾迹/结束按carrier_entity，不能一个Volley箭结束就删除全部七支。多payload附属效果如装备被动保留各自origin，不改变原排序或重新发现触发。

现有is_dashing/leap_progress/MotionView可继续支持世界根与高度。若需要精确区分EQ链接与其它装备Dash，只增加对当前CompoundMotion的只读来源/类型/目的地视图；不新建运动组件或用VFX计时维持冲刺。不是本轮基础Q线/R跃击的硬性前提。

## 客户端模块拆分建议

```text
client/ability_targeting.rs          共享目标意图/预检，controls与指示器读取
client/skill_presentation/mod.rs     plugin与明确调度
  catalog.rs                        实际AbilityId/强化origin -> 本地素材/视觉样式
  indicators.rs                     纯执行几何与绘制；无战斗计时
  capture.rs                        每完整固定步后的独立事件读者、位置/视野/生命快照
  projectiles.rs                    真实Projectile Entity的外观/尾迹
  objects.rs                        Standard/Scout/Reveal/Terrain与盾/强化持续表现
  bursts.rs                         已捕获瞬时事实的有限寿命贴图/网格表现
  tests.rs                          只读投影与真实公开指令行为回归
```

也可在规模较小时合并运行文件，但不能把所有八技能渲染和事件处理继续塞scene.rs。scene保留UnitVisual/选择/反馈根职责；原通用projectile/area/field visuals迁移或由同一个所有者决定原皮/通用回退，不允许两套插件同时挂两个身体或两圈墙。特效绝不标记PickGeometry。

素材/网格/材质句柄共享，创建与销毁有界；不能在每显示帧给同一个持续光环添加新Mesh/Material。贴图失效回退通用语义，不修改规则。已验证资源和参数只作为外观，不引入整套League粒子引擎。

## 生命周期与观察风险

- 捕获沿用audio模式：每个完整SimulationSet::Step之后、Gameplay Outcome/Cleanup之前。仅Update消费会跨多固定步丢失短暂载体；不能为补救而回放过期施法。独立cursor不抢其它读者。
- 以MatchId+玩家会话区分重选/Active→Active重开，清待表现队列和独立world特效根。实体上的模型子树随真实载体清理，但不是所有transient都挂在模拟根，reset_simulation不会自动替客户端清孤立根。
- Attached盾/强化跟源当前life，到期/吸空/死亡/换life立即清理；**已提交Projectile、Area、旗和墙不因施法者死亡或装备卸下就全部删除**。来源旧生命与当前源生命不同不等于载体失效。
- 所有表现时间用SimulationTick + PresentationFrame，普通暂停精确冻结，Finished首次同步最后步一次。不得用Time.elapsed/宿主RAF给技能VFX或尾迹独立续时。缓冲生存期是表现缓存，不反向延长真实载体。
- Imported AnimationPlayer虽然暂停仍每帧写骨骼；前轮已修的Root高度后处理必须保留，不能在新特效插件中改UnitVisual运动或把源骨骼位移再叠一次。
- 持续实体复用VisibleWorld；瞬时事实按自身team与**实际事件位置**过滤，不统一按来源Entity当前是否可见/存活过滤。隐藏caster射来的箭进入己方视野时应能显示，反之看见源不代表可以显示远处隐藏受击者。Contact使用受击点，Area使用区域中心，长beam/ribbon必要时按可见段裁剪。
- 原始全量模拟事件只在Rust内部消费，不直接序列化为浏览器诊断。新增只读诊断也只能包含当前可观察效果，隐藏source/target身份不得旁路现有UnitView过滤。

## 必须补的回归断言

1. **提交与目标**：非默认朝向释放Q，提交方向与真实命中线一致；取消前摇/非法目标零新特效事实。Point E超距预览终点与提交点一致；极大有限点击保持方向；Enemy R远英雄是Approaching，兵/基地/友方/隐藏/旧life不成为Ready。
2. **几何**：W恰好7射线，总角27.75°、左右±13.875°，宽0.2；R宽2.6，鼠标点近仍显示方向远射，命中圈半径4只在真实结果；E主落点2与次级光环10分离；Q全宽1.36与联动全宽3不混淆。
3. **点击一致**：同一PointerState/模型候选下，renderer标出的目标与controls提交的Unit一致；重叠英雄/小兵、AttackMove与普通技能、装备旧instance、UI隔离分别覆盖。不能只检查最终有人掉血。
4. **真实结果**：盾全吸收仍有一次Contact、Damaged可为零；纯状态也有Contact；零敌人的E落地/R落地/通用Area到期有一次AreaActivated；目标失效、阻挡、边界、射程耗尽有各自ProjectileEnded。接触不按effects数量重复，附属快照和原结算排序不变。
5. **多箭身份**：同一W组7个Projectile Entity各自结束；单箭命中不会清其它箭，组内同目标仍只一次实际受效。强化AA飞行期间buff到期、源死/复活/销毁后仍保留冻结origin/箭数，装备只触发一次。
6. **对象来源**：混装Ashe E与R的Reveal能按origin区别，不按owner/半径猜；Scout同Entity到站确实变终点表现；E落地前无假旗，R地形创建失败无假墙，重施不生成第二次跃击/爆炸。多盾各自到期/吸空时样式正确。
7. **生命周期/可见性**：一显示帧跨提交→命中→删除或死亡→复活，仍只消费准确事实；暂停后Transform/UV帧/尾迹/剩余寿命保持；相机平移不揭示隐藏效果，隐藏caster可见箭与可见caster隐藏受击点分别验证；重选后旧队列/独立world根为零。
8. **当帧指示器**：保持CSS鼠标不动，逐帧pan/zoom/跟随后，用同帧相机把IndicatorView落点回投影，应仍落在鼠标处；原地进入/取消aim的首帧Visibility和GlobalTransform均更新。按DPR2/缩放验证css_to_viewport，不用等两帧掩盖时序错误。
9. **最终Web**：真实键鼠八技能、混装/装备回退、模型剑光标/标签负例、HUD隔离、800×600/常规/DPR2，保存实际纹理/尾迹/落点/墙截图；原生帧钟healthbars与pointer-response保留。虚拟帧钟可稳定短动作交互，不能声称帧率改善。

## 附：边缘滚屏接缝

当前camera把按键/边缘合并后clamp_length_max(1)，再乘18×zoom×dt；只在edge向量里乘1.75会被最后clamp吃掉。主代理已交独立实现：保留合并方向，有有效edge时选1.75倍最终速度，只有键盘时维持原速。保持现有锁定/Space、小地图导航优先、UI隔离、失焦/暂停/坐标边界与地图clamp，不能通过改鼠标增益实现滚屏提速。
