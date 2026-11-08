# 架构与扩展约定

本项目用 Bevy ECS 承载一套权威游戏状态。`simulation` 是可独立运行的游戏规则模块，
`gameplay`组合地图、对局生命周期与自主决策，`client`适配输入、三维表现和浏览器。
依赖方向为 `client → gameplay → simulation`；客户端也直接读取模拟的公开视图。

## 模块接口

```text
src/lib.rs                    库入口
src/simulation/mod.rs         SimulationPlugin、spawn_unit、公开接口
src/simulation/model.rs       单位、指令、动作、只读视图、模拟事件
src/simulation/attributes.rs  基础/最终属性、来源修正、变更队列、普攻时序推导
src/simulation/damage.rs      伤害表达式、提交快照、类型与抗性计算
src/simulation/abilities.rs   目标、执行、投递、有序效果与四槽装配定义
src/simulation/mechanics_defs.rs 复合执行、单位类别与普攻特性的通用配置
src/simulation/mechanics.rs   护盾、强化、充能与临时属性的运行态/只读视图
src/simulation/attack_traits.rs 普攻特性、逐目标触发与攻击快照
src/simulation/compound.rs    复合施法与已提交运动的协调，不写位置/生命
src/simulation/battlefield.rs  军旗、光环、侦察、临时地形对象及寿命
src/simulation/vision.rs      团队可见性快照与共享查询
src/simulation/characters.rs  角色目录、身份、默认/显式装配解析与统一角色创建
src/simulation/equipment.rs   装备目录、六格实例、来源装卸、装备CD与被动收集
src/simulation/effects.rs     不可变命中快照、投递创建与稳定排序结算
src/simulation/facts.rs       已提交技能与真实命中/激活/终止的中性事实
src/simulation/statuses.rs    状态实例、计时、叠加、控制权限与减速归并
src/simulation/orders.rs      指令校验、持续指令、目标获取与追击
src/simulation/cast_orders.rs 单位目标的走近施法目的、复核和路线规划
src/simulation/navigation.rs  A*、静态/临时几何、解析扫掠与出生落点解析
src/simulation/movement.rs    移动、冲刺、位置重置
src/simulation/actions.rs     前摇、提交、收招、引导、冷却
src/simulation/combat.rs      弹丸、延迟范围效果、伤害、死亡、重生
src/simulation/recovery.rs    自然生命恢复与受限的友方结构恢复
src/gameplay/mod.rs           GameplayPlugin、调度接口与幂等准备
src/gameplay/content/         技能、角色、装备的不可变内容定义
src/gameplay/units.rs         端游距离/速度与世界单位的统一换算
src/gameplay/map.rs           地图、基地、出生和兵线路径的共同几何
src/gameplay/definition.rs    基地/兵种/波次/恢复的模式配置
src/gameplay/spawn.rs         原子开局、首波和重置
src/gameplay/match_state.rs   对局身份/阶段、单位角色/控制来源/波次组件
src/gameplay/lifecycle.rs     终局、后续波次、死兵回收和基地恢复
src/gameplay/ai/              小兵/英雄战术、观察、有限追击与技能语义
src/client/content.rs         客户端默认选择与玩法准备接线
src/client/session.rs         选角/开局阶段、选择校验与唯一活动玩家引用
src/client/equipment.rs       暂停装卸适配、严格请求/回执和只读装备投影
src/client/abilities.rs       输入与HUD共用的只读技能可用性和描述
src/client/ability_targeting.rs 控制与指示器共用的只读目标资格/解析意图
src/client/skill_assets/      本地原技能素材、来源元数据和共享加载句柄
src/client/skill_effects/     实际提交/载体/对象/附着事实驱动的技能表现
src/client/skill_indicators/  实际定义几何与当前相机下的瞄准指示器
src/client/controls.rs        玩家输入、施法预检、指令反馈
src/client/pointer.rs         Web输入帧指针镜像、Native绝对坐标与捕获边界
src/client/controls/tests.rs  同帧事件顺序、暂停与焦点恢复回归测试
src/client/camera.rs          统一取景参数、镜头控制与实际地面覆盖范围
src/client/picking.rs         实际模型部件屏幕命中与公共目标候选
src/client/scene.rs           场景、显示根插值和通用载体/单位标记回退
src/client/world_scene.rs     由模式地图派生的地面、中路和静态装饰
src/client/unit_appearance.rs 英雄/兵种/基地的模型、拾取和标签参数
src/client/presentation.rs    普通暂停与终局的统一表现采样帧
src/client/visibility.rs      模型/拾取/HUD共享的可见实体集合
src/client/battlefield.rs     军旗、墙、侦察与护盾的只读三维表现
src/client/match_view.rs      只读对局、波次与基地投影
src/client/character/rig.rs   内嵌骨骼动作校验、采样和父序变换
src/client/character/model.rs 分段模型、头眼与武器socket
src/client/character/weapon.rs 展示身份到剑/枪/弓几何，不参与战斗规则
src/client/character/poses.rs 源动作上的骨骼姿态与持武器手势
src/client/character/animation.rs 只读模拟到动作、过渡与生命缓存
src/client/imported_models/   本地模型目录、异步装配和显式源动作采样
src/client/audio.rs           已发生的模拟事实到有限、带生命身份的音频 cue
src/client/ui.rs              HUD 的只读状态投影
src/client/web.rs             浏览器交互适配
web/hud.js                    DOM 展示、暂停菜单、技能/被动说明与单位资源标签
web/hud.css                   游戏内布局、底栏/地图/浮层/标签与响应式
web/style.css                 基础主题、选角与装备管理菜单
web/match.css                 对局事实、结算与兵种样式
web/minimap.js                全图投影、逐帧视野矩形与小地图相机导航意图
web/startup.js                Trunk前同步捕获启动结果，向迟到HUD回放，无DOM依赖
web/equipment.js              首份只读目录缓存、六格HUD、装配面板和装备意图
web/match.js                  对局时钟、基地生命、波次和结算展示
web/pointer.js                Web实时CSS指针、正常位移校准、按下坐标与即时显示
web/audio.js                  唯一 WebAudio 上下文、手势解锁、播放预算和清理
tests/simulation.rs           既有属性、攻击、技能生命周期无窗口回归
tests/status_effects.rs       多效果、控制组合、生命身份和强制运动行为测试
tests/characters.rs           角色构造、装配、实例独立和重生保持测试
tests/champion_content.rs     内置皇子/寒冰真实内容配线与命令行为
tests/champion_mechanics.rs   通用被动/复合技能/视野/地形边界
tests/skill_facts.rs          真实提交/接触/空区域/单箭/独立来源事实
tests/equipment.rs            装卸/施法/被动/归因/生命周期公开行为测试
tests/reset.rs                整局实体/队列清理与静态资源/分配器保留测试
tests/combat_foundation.rs    结构受效、普攻投递、施法预检和恢复行为
tests/match_mode.rs           原子开局/波次、终局、恢复和整局重置
tests/minions.rs              真实兵线、普通避让、生命引用和长局运行
tests/hero_ai.rs              实际装配施法、战术、有限追击和攻守冲刺
```

模块内部默认私有，通过各层`mod.rs`显式导出外部需要的接口。
`UnitPose`、`UnitState`、`AbilitySlots`、`EquipmentSlots` 的运行态字段不对外开放写权限。
客户端不添加修改这些字段的系统。游戏模拟不导入 `client` 或 Web 对象，
也不读取输入设备、模型、动画、相机、材质或 HUD。

目前保持一个 crate 和明确的模块接口。尚未引入插件注册框架、脚本语言、网络协议、
第二套实体世界或为单一实现准备的 trait。

## 时间与执行顺序

平面坐标使用 `Vec2(x, z)`，世界展示使用 `Vec3(x, y, z)`；距离按世界单位、时间按秒。
模拟固定运行在 60 Hz。修改频率时，须同时修改唯一的 `TICK_RATE`，
`Time<Fixed>` 与 `TICK_SECONDS` 均由它派生。

每个固定步按以下顺序执行：

玩法自主决策先在`GameplaySet::Decisions`运行，然后进入完整的`SimulationSet::Step`：

1. 推进模拟 tick。
2. 处理到期重生。
3. FIFO 消费属性修正请求，完整校验后原子更新来源集合与最终属性；推进增强/充能/对象寿命和光环，刷新团队视野。
4. FIFO 消费待处理玩家指令，每条消费一次。
5. 先推进走近施法目的，在范围内转入技能前摇；随后推进冷却与动作，提交已完成前摇的攻击和技能。两种开始入口共用同一步动作时钟，瞬发不能额外等待一个tick；装备CD在动作阶段开头对死亡单位也推进。
6. 解析其它持续指令，选择目标、规划路线或开始普攻前摇。
7. 统一处理普通移动、自主冲刺或强制运动；每步只使用一种运动预算。
8. 收集已提交路径运动的命中/落点，扫掠弹丸并推进延迟范围效果；全部只入队。
9. 推进并移除旧状态，同步更新控制权限和减速属性。
10. 稳定排序命中包，按定义顺序执行伤害、状态、位移请求；致死立即清理；重新发布可见性快照。

顺序由插件内 `.chain()` 声明，包括 Bevy 延迟实体操作的同步点。
因此同一步内，所有单位的动作提交先于控制、伤害与死亡：同一步被击杀或控制的单位，已提交效果仍有效。
状态在tick t末应用remaining=N，限制t+1至t+N的完整动作步，t+N移动后到期；
新状态不会在应用当步递减，到期后也不补跑动作。强制位移从下一个移动阶段开始。
此顺序是规则的一部分，调整时应补充相关行为测试。

完整模拟步后依次执行`Outcome → Waves → Cleanup → Recovery`。Outcome统一读取双方基地，
同tick双方归零判平局；阶段只终结一次，立即暂停并清命令。Waves在函数内再次检查状态，
终局当步不生成波次。Finished即使被误设为未暂停也不能再运行后续模拟步。
小兵根在完整结算后回收；已提交的独立效果继续存在。英雄恢复最后通过模拟的受限入口执行。

客户端在 `RunFixedMainLoopSystems::BeforeFixedMainLoop` 依次处理选角/开局请求、装备管理请求和采集输入。
会话与装备管理请求在模拟暂停时仍会消费；Web必须已绑定Active玩家且捕获/聚焦成功才允许固定步。
一次操作沿 Bevy 的统一 `WindowEvent` 消息流消费，保留键盘、鼠标和光标事件的先后顺序，
避免同一帧内“停止 → 选择技能”被聚合按键状态颠倒；持续相机操作读取按住状态。
浏览器通过Pointer Lock捕获鼠标，`web/pointer.js`唯一拥有画布内的实时CSS指针。
它从进入前正常指针的client/movement差值学习单位比例，保留系统加速；不能统一再除一次DPR。
捕获中按coalesced样本顺序移动/夹紧并在DOM事件内显示，不等待引擎帧，Rust不再重复积分或覆写DOM位置。
`GamePointer`仅镜像Rust输入帧坐标。滚屏、射线拾取、HUD和显示读取同一坐标来源；相机viewport逻辑单位
与画布CSS尺寸不同时仍由web适配双向换算。
锁定期间忽略系统固定不动的绝对坐标。进入或恢复捕获时居中，并丢弃前一会话的窗口事件及DOM技能请求。
HUD 按钮通过游戏光标命中后激活，技能请求在该次鼠标事件内消费，保持输入顺序。
浏览器按下队列保存每次点击的位置，Rust在对应MouseButtonInput消费，帧末再读取最新位置；
后续移动不能改写早先点击。无样本或按钮错配时拒绝并清队列，不回退到最新坐标。
移动事件只推进浏览器指针，Rust在点击及帧末拾取，避免高轮询鼠标逐raw样本重复射线查询。
即使这一帧没有固定步，指令仍保留；多个固定步不会重复执行一次按键。
相机和表现逐帧更新，人物根使用上一模拟位置与当前位置插值，并加上击飞表现高度。
`PresentationFrame`为显示根、骨骼、弹丸和兵种模型提供同一个采样分数；普通暂停保持最后显示帧，
首次观察终局允许同步最终状态一次，此后冻结，不能随继续变化的overstep抖动。
底座通过反向局部高度留在地面，模型子层只负责骨骼姿态。picking读取最后实际显示的模型子树：刚性部件使用Aabb与GlobalTransform，蒙皮部件使用逐关节动态边界及当前关节变换，逐部件投影为凸区域，提供6 CSS px容错；不读取未插值的战斗位置。
PickGeometry标记模型子树，技能特效/范围/弹丸不加入；UnitGroundBase装饰排除，基地平台仍是模型。根Pickable仅在模型边界未就绪时fallback，血条/名字不参与命中。公共候选不判断敌友或射程，controls按操作资格选目标，模拟继续复核攻击/技能规则，未来友方技能复用相同几何入口。
A+左键明确命中敌方模型时和右键一样发AttackTarget并更新selected；只有空地才发AttackMove自动索敌，不把旧selected当作新点击目标。攻击移动的模拟索敌规则保持在simulation。
HudView.hovered是只读悬停身份，selected仍为持久点击结果；逐帧kiva-camera.enemy_hovered驱动pointer所有者的箭头/剑形，UI手势、暂停、失锁和失焦清掉旧形状，不等待80ms完整快照。细节见[模型拾取规范](../.trellis/spec/client/unit-picking.md)。
浏览器 HUD 在相机投影和世界变换更新后读取数据，约每 80 ms 发布一次完整状态。
头顶血条的位置走独立的 `kiva-labels` 轻量通道，每个显示帧根据人物实际显示变换投影，
与插值后的模型和当前镜头保持同步。完整 HUD 快照不能覆盖血条坐标。
固定步长不承诺跨平台浮点确定性。

对局HUD按属性/角色标记/技能与资源/六格装备划分，资源条位于技能下方；七个中文短属性仍不显示单位。
游戏内布局统一在hud.css，约110px高的底栏和右下小地图均距底8px，尺寸变量协调二者空间；800×600也不隐藏地图或将它抬入主画面。
目标信息在左上，比赛与操作收紧到右上，Active隐藏大品牌；基础菜单和比赛事实样式分别留在style.css/match.css。
所有原稳定选择器、data-ui隔离及暂停/捕获/装备/会话协议保留。英雄头顶能量条只读完整快照，坐标仍走逐帧标签。

CameraRig默认zoom=0.5、最近0.4、最远1.5，保持原视角/FOV；键盘18*zoom平移，有效画布内边缘输入31.5*zoom（原1.75倍），均乘dt；初始化/逐帧更新复用同一变换构造。
以近景取得约2倍英雄屏幕比例，不缩放模拟单位或独立挪动拾取/血条。Y/Space、自由平移、首次开局/重开居中和普通恢复语义保持。
HudView.camera_footprint为null或按左上/右上/右下/左下排列的4个world[x,z]：由实际逻辑viewport角点射线与地面求交，包含DPR/viewport偏移约定。
minimap.js固定投影完整地图，8 CSS px内边距以内显示全图边界/中路/基地/当前可见单位，后备画布跟随CSS尺寸与DPR。
视野矩形使用真实地面四角的轴对齐外接范围并裁到地图边界；透视轮廓仍是梯形，不为矩形改主相机投影。独立kiva-camera每显示帧发布{center,footprint,enemy_hovered}，更新框与光标不等待约80ms完整快照，也不重绘底图。
小地图使用kivaPointer的事件时CSS位置进行左键点击/偏移拖动，只提交最新data-camera-request {player_id,center}及拖动状态；camera统一复核玩家身份、暂停和有限坐标，夹紧既有边界，退出锁定但保持zoom。暂停/失焦/失锁/重选清旧拖动与意图，拖动中屏蔽其它平移。既有拾取、kiva-labels数组、view_projection与viewport保持，CSS不裁切游戏canvas。
完整布局/投影契约和验证入口见[HUD与取景规范](../.trellis/spec/client/hud-layout.md)。

## 技能原素材与只读表现

`skill_assets`集中本地原PNG/标准GLB和共享模板；离线importer记录版本、来源、像素/产物哈希、图集、轴与bounds。
运行时不解析League私有格式、不联网。`skill_effects`在完整固定步后捕获真实AbilityCommitted/ImpactResolved及对象过渡，
再按统一PresentationFrame采样真实弹丸、区域、旗、墙、侦察、盾与强化。瞬时表现有界且及时丢弃旧事实，
独立载体不依赖来源当前生命，附着效果核对原source_life；暂停冻结，重开清队列、显示组和临时材质。
普通普攻的DamageOrigin仍是BasicAttack，强化来源另外冻结在投递快照中。

`ability_targeting`让输入与指示器共用实际装配及preview_cast资格，按模型候选选择首个合法目标；
Point原点击点和Enemy追近语义仍交模拟处理。`skill_indicators`在当前相机及传播之后更新独立显示根，
按实际range/width/angle/count/radius绘图，不写PointerState或人物根。成功创建原效果后才移除通用回退，
透明层不投影、不加入PickGeometry，共享素材模板不随单次技能销毁。
已接入皇子原皮与源计划寒冰的关键素材和配方；噪声/扭曲/层级发射器有简化，不是完整League粒子引擎。
接口、观察/时间/资源所有权及验收见[技能表现](../.trellis/spec/client/skill-presentation.md)和
[模拟事实](../.trellis/spec/simulation/skill-facts.md)。

## 单中路模式与自主决策

`MapDefinition`是导航、出生、三维地面、小地图和浏览器投影共同的地图来源。
默认边长64、中路宽7，基地在Ally(-8,24)/Enemy(8,-24)，沿默认观察方向为左下/右上。
地图大小不改变镜头玩法，仍保留滚屏、缩放、Y与空格跟随。

`prepare_world`幂等准备内容和导航；`start_match`只接受Waiting与零tick，预检全部16个根的定义、出生及路线，
只构造两基地、双英雄和双方首只近战兵，失败回滚本次新根，成功才发布单调`MatchId`和Running。
`MatchDefinition`在开局时快照，
`MatchState`只保存模式阶段、英雄/基地引用、波次与终局，不复制HP/CD等模拟事实。
`UnitRole`表达英雄/近战兵/远程兵/基地；`Controller`独立表达人类/英雄AI/小兵AI。

首波在0秒开始，每方先3近战、后3远程，slot0..5分别在0/30/60/90/120/150tick生成。
以后每3600tick开始下一波，波次起点不随本波出齐时间漂移；默认近战465生命、远程284生命。
`MatchRecord`私有待发状态只保存波次、下一slot和tick，暂停不推进，终局/失败/reset清除。
每波先全量预检，实际每次同时原子生成双方同slot的一对兵；失败回滚本次新增根并冻结，
已参与战斗的早先兵保留。不能隐瞒漏兵、跳波或静默限制活兵数量。
单兵间隔必须为正，出齐时长小于波间隔且整数运算不溢出。
基地3000生命、20护甲，Structure不移动、不施法、不复活；技能、装备主动、普攻附属效果和控制不作用于Structure。
普攻本体可伤基地，攻击属性加成正常计入。免疫结构不挡住伤害Actor的技能弹丸。
结构分类与受效判定集中在模拟层，不能只靠UI禁选或AI不施法实现免疫。
英雄继续3秒复活；友方基地2.5半径内脱战120tick后，每秒恢复最大HP和最大法力的各20%（每0.5秒各10%）。
范围、当前生命、脱战、上限和每tick最多一次由`restore_in_zone`裁决，不开放任意回血接口。

`AiPlugin`独立于模式生命周期，5Hz决策、首次/新生命延迟12tick，每次至多一条指令。
Brain只拥有当前生命目标、命令去重和战术记忆，移动/攻击/技能都经过相同模拟入口。
小兵沿中路三列推进，优先敌兵，再获取附近英雄，清空后打基地；稳定目标和前摇避免频繁重发指令。
英雄可推进、交战、撤退、恢复和守家；技能候选从自己的实际四槽与装备实例读取，按目标/投递/效果语义评分，
不按内容ID或QWER分支。方向弹丸按当前速度有限前导并估计首撞，范围技能考虑延迟与覆盖，
引导要求当前范围有敌且风险许可，冲刺要求可达和威胁许可。预检复用`preview_cast`，估伤复用`estimate_damage`。
追击英雄共用有限资格：12视距、中路带、起追位置6距离、离攻距最多240tick，放弃后180tick回线，
所有进攻技能和普攻共用该限制。AI不读取敌方输入/CD或未来路径，也不跳过法力、控制和冷却。

详细接口、错误矩阵和长局验证见[玩法规范](../.trellis/spec/gameplay/index.md)。

## 复合技能与团队观察

角色模板把通用 `AttackPassive` 置于 `UnitStats`，通过实际 loadout 推导已学技能被动；
`CharacterDefinition::initial_attributes_with_loadout`在不创建实体的情况下预览同一份贡献。
`UnitKind`表达通用Hero/Minion/Other目标类别，与模式UnitRole独立，技能过滤不导入gameplay。

复合执行由Line/Shield/Standard/Leap/Empower/Volley/Scout/Arrow配置组合几何、运动与有序效果。
它们仍经过统一preview、前摇复核、伤害快照和消耗/CD提交；效果不能另开扣血或位置写入口。
护盾吸收抗性后的伤害，实际生命损失仍由combat发布；盾全吸收的有效普攻仍可触发一次命中被动。
战争律动用命中前当前生命，按来源生命/被动索引/目标生命独立计时；结构不接受附伤。
强化普攻将倍率和箭数一并冻结，五箭表现只触发一次装备命中效果；军旗光环对友方英雄取最强一份。

场上对象保存来源生命、阵营、激活/到期tick和位置；地图只保存其临时墙几何句柄。
movement先在包含全部旧墙与新圈的导航副本中为所有受影响Actor规划局部排离，全部成功才提交；
径向受阻会近邻搜索，仍无解则整圈不创建并发布TerrainPlacementFailed，伤害/消耗保留，客户端仅向施法玩家反馈。
普通移动路径与sweep共用墙；弹丸与侦察不受临时墙阻挡。
跃击按冻结落点运动，来源死亡不落地结算；已布置独立旗帜/侦察/地形按各自寿命存续。
二次释放通过当前来源的recast权限走统一施法入口，不重复扣法力或刷新原CD。
构造/复活复用NavigationMap::resolve_placement：静态非法仍拒绝，临时墙遮挡原点时只局部排离，原home不变；
无安全复活点则保留dead life待下tick重试。模式预检使用当前实际静态几何path_static，不能因R圈暂时封路冻结兵波。

`TeamVision`由存活单位和已激活临时视野源构成；构造、reset以及固定步边界刷新。
没有视野源的独立Arena保留全视野。方向/地点技能允许盲打，敌方单位锁定和持续攻击只使用当前可见信息；
已提交载体的命中按几何和生命身份裁决，不因之后丢视野而撤销。
客户端VisibleWorld集合将同一事实供模型、逐帧血条、HUD和拾取使用；AI也按自身阵营过滤观察。

## 模型、动作与生命周期表现

生产对局使用本地 GLB：皇子原皮、源计划寒冰，以及蓝/红双方近战和远程兵。
`imported_models::catalog`集中映射角色/兵种、模型比例、源动作名、技能图标和头像；
`runtime`共享资源句柄，为每单位创建独立骨骼和播放器，完整就绪后原子替换程序模型。
加载失败保留回退并提供只读错误，不能出现两套身体。地图原纹理铺在既有模式几何上，
地图边界、导航和取景仍由原所有者决定。资源来源、哈希与离线重建见[模型资源](../assets/models/README.md)。

客户端以模拟 tick 和统一表现分数显式采样待机、真实移动、普攻前后摇、实际技能和死亡动作。
普攻准备缓存首次观察到的前摇长度，后摇读取冻结的 duration；真实提交事件识别瞬发技能。
源动画不驱动伤害、冷却、移动或复活；UnitVisual 继续唯一拥有世界位置与跃击高度，
源 root motion 在导入/动画后处理中去重。生命或动作阶段切换先恢复绑定姿态，避免源 clip
省略轨道时继承旧尸体动作。普通暂停保持已显示姿态，终局首次同步最终步后冻结。
蒙皮拾取使用实际显示的逐关节边界，血条不参与。详见[导入模型规范](../.trellis/spec/client/imported-models.md)。

`client/character`保留 act 分支的23骨骼刚性模型，承担加载回退和无模式测试演员。
原内嵌动作、枪弓 socket 和规则只读边界仍成立，详见[回退骨骼规范](../.trellis/spec/client/character-animation.md)。
基地继续使用低台与阵营水晶；`unit_appearance`统一角色类型、标签高度和死亡可见性，
模型层不写生命或胜负。

致死由模拟记录 `UnitState::death_tick()`，`respawn_remaining()`只读返回真实剩余时间。
死英雄停留在死亡位置，倒地后保留尸体，地面标记隐藏；到真实复活时同一实体恢复新生命并重置动作。
阻塞复活点时保持0秒并等待安全落点，HUD不自行启动计时器或假定复活。
小兵仍立即回收，基地保留原终局外观；整局重开统一随单位根清理模型子树。

## 战斗音频

`CombatAudioPlugin`在每个完整模拟步后、玩法回收前捕获英雄的实际攻击、技能提交、死亡和复活，
并立即快照生命身份、位置和团队可见性。有限 cue 队列在显示帧派发；客户端按相机距离/方向调整听感，
过旧、旧局、旧生命或不可见来源不能迟到补播。表现随机数独立于模拟。

`web/audio.js`是唯一播放后端，提前同步注册目录监听；首次可信用户手势解锁同一个 WebAudio 上下文。
音量、静音和声道预算只影响播放。暂停、失焦、失锁或重开立即停止节点，恢复不补播旧声音。
源计划寒冰和皇子原皮使用各自实际 SFX，全部 OGG 本地发布；来源、提取和解码复现见
[音频资源](../assets/audio/README.md)，跨层字段与边界见[音频规范](../.trellis/spec/client/combat-audio.md)。


## 指令与动作

`UnitCommand` 是一次意图，可被拒绝；`SimulationEvent` 表示已发生的结果。
事件主要供界面与表现读取，游戏规则不依赖表现层是否读取事件。

持续指令 `UnitOrder` 表达目的：移动到某地、持续攻击目标、攻击移动、一次走近后施法、空闲。
瞬时动作 `UnitAction` 表达正在做的事：普攻前摇、普攻后摇、施法、引导、技能收招、就绪。
范围内技能暂时占用动作，保留原移动/攻击目的；结束后可继续。超距Enemy请求则以CastTarget替换当前目的，开始前摇前清掉该一次性目的，结束后不自动普攻或重复施法。
显式移动和停止会取消当前可中断动作。Stop始终可以清理持续意图和普通路线，
但不会取消已提交的自主冲刺、外来强制位移或弹丸。其它新指令不能覆盖自主冲刺。
死亡清理所有运动。重复攻击同一目标不会不断重置攻击前摇。

无效指令不替换已有指令，不取消已有施法，不扣资源。Enemy技能超距保存CastSource、实际AbilityId与目标UnitLifeRef，寻路进入施法距离后才开始前摇；英雄和装备共用此入口。preview_cast仍严格表示当前可否立即施放，供AI预检；Direction只使用方向，不触发走近，Point原严格射程/截断规则不变。
追近每tick复核来源、生命和可见性，移动目标节流重规划；目标中心不可达但施法区域可达时仍可施法。目标失效/丢视野、来源失效或无路即取消，Stun/Airborne/技能权限失效取消，单纯移动限制暂停走近。
追近期间不扣资源/CD，新合法移动/攻击/Stop/技能替换，非法输入保留。已开始的前摇仍走原提交复核，出范围或被打断不恢复旧待放目的。
Point Dash允许超距点击：范围内到点击点，范围外沿方向到最大距离，碰墙/边界仍由统一扫掠提前停止。
普攻会追踪目标；停止后不自动获取附近敌人。
攻击移动会选择自身获取范围内最近的敌人，距离相同时按实体 ID 排序，目标消失后继续原路。

## 状态所有权

- `movement.rs` 负责位置写入及重生位置重置；寻路和技能只提出移动需求。
- `effects.rs` 负责不可变效果快照、有序命中包和路由；不直接写位置或最终属性。
- `combat.rs` 负责命中几何、扣血、死亡和重生；所有投递复用同一命中效果结算通道。
- `statuses.rs` 负责状态实例与权限归并，通过动作/移动模块的内部入口中断，不自行写位置。
- `actions.rs` 负责动作时钟、能量恢复、技能消耗与冷却提交。
- `cast_orders.rs`只写一次性走近施法目的/普通路线并调用动作入口，不能写单位位置或绕过原子提交。
- `equipment.rs`负责装备目录/六格实例、装卸和装备CD，主动复用actions，属性复用attributes。
- `attributes.rs` 负责最终属性的唯一计算和来源修正生命周期；客户端只读最终结果。
- `damage.rs` 负责伤害表达式、快照与抗性计算，生命写入仍由 `combat.rs` 完成。
- `recovery.rs`拥有受限恢复写入口，统一检查源/目标当前生命、友方、范围、脱战和同tick去重。
- `gameplay`只写模式阶段、波次与Brain记忆，战术通过公开指令、构造和恢复接口进入模拟。
- 构造状态只通过 `spawn_unit`；错误的出生位置、属性和技能装配会被拒绝。
- `Transform` 属于显示，不作为战斗位置来源。动画和粒子不决定命中或伤害。

单位碰撞半径用于障碍膨胀，路径平滑、普通移动、冲刺和弹丸共用静态地图几何。
扫掠采用保守的膨胀矩形，转角可能比圆形角色真实轮廓稍宽。地图是平面、障碍是静态的。
单位没有硬阻挡；显式标记`SoftAvoidance`的小兵只在普通沿路线行走时做有界侧向分离。
邻居位置在写位置前统一快照；侧移撞墙时回退原有效路线，不能因此清掉可行窄路。
静止/前摇单位不被挤动，英雄未默认启用此标记，Dash和强制位移沿用原扫掠及终点。

## 属性与来源修正

`UnitStats` 是构造时的静态定义：包含七项 `BaseAttributes`、生命/法力参数、碰撞半径、普攻前摇比例、
`UnitClass`和`BasicAttackDelivery`。默认Actor保留原弹丸普攻；近战可选择Direct投递。
`Unit::definition()` 读取静态定义，`Unit::attributes()` 读取模拟维护的 `EffectiveAttributes`。
移动、普攻射程、范围圈、技能伤害和 HUD 都读取最终值；生命/法力上限、碰撞半径本期仍为静态参数。

七项属性是攻击力、法术强度、护甲、魔抗、移动速度、攻击距离、攻击速度。
移速使用世界单位/秒，攻击距离使用双方碰撞边缘之间的世界距离，攻速使用次/秒。
默认值为 40、0、0、0、5.5、5.5、1.25；默认普攻前摇比例为 0.275。

规则调用方通过 `AttributeChanges` 分配类型化来源 ID，并入队 `upsert` 或 `remove`。
同一单位同一来源替换现有修正，不会重复叠加；不同来源共同生效。
每项结果从基础值与当前来源集合重算：

```text
final = (base + sum(flat)) * max(0, 1 + sum(additive_percent))
```

移速在上述结果后再乘所有来源的 `move_speed_multiplier`，默认1，必须有限且在[0,1]。
乘算使用f64并在最终f32饱和之前完成；其余六项不增加乘法层。状态模块只维护一份私有来源，
写入 `1 - 最强活动Slow强度`，所以20%加速与30%减速得到0.84倍基础移速。
移除来源从基础值重算，0倍减速也能正确恢复，不做逆向除法。

百分比使用小数，例如 0.2 表示 +20%。攻击力、法强、移速、距离最终不小于 0；护甲和魔抗允许负值。
聚合使用 f64 中间值并饱和到有限的输出范围，避免来源移除或死亡清理因剩余加成极端而失败。
非法输入整条拒绝，保留原属性、动作和持续指令，结果通过 `AttributeChangeRejected` 观察。

`Explicit` 来源保留到显式移除；`OnDeath` 来源在死亡时移除。
死亡期间允许更新持久来源，拒绝添加死亡即失效的临时来源；复活不重新叠加已有来源。
公开属性变更阶段在玩家指令和动作之前，之前入队的变化同一步生效；暂停时请求留存，恢复后消费一次。
命中末段的Slow和暂停装卸使用属性模块内部的同步校验入口，和公开队列共用更新实现，
确保状态/装备与最终属性同步。来源不带状态计时器；状态模块维护自己的实例和持续时间。

## 普攻时序

攻速是唯一攻击频率来源，不再同时保存可独立配置的攻击周期和绝对前摇时间。
每次新攻击开始时，将最终攻速与前摇比例推导为整数 tick 的前摇和剩余间隔，并保存本次时序。
周期向上取整，保证至少一 tick 前摇、一 tick 后续间隔，正攻速限制在整数计时可表达的范围。
0 攻速不开始新普攻；0 移速保留路径和移动意图，但普通移动不产生位移。技能冲刺使用其独立速度。

前摇结束时重新检查目标和最终射程，成功出手后提交保存的剩余间隔，
并进入独立的 `AttackBackswing { remaining, duration }`，后摇长度使用本次攻击快照的剩余间隔。
后摇自然结束与攻击间隔同步，不额外延后下一次攻击；移动、停止、换目标或施法可取消后摇。
相同目标的重复攻击和无效请求不重启或取消后摇，暂停冻结计时，死亡清理动作。
失败或取消前摇不提交新间隔；已经提交的间隔不能被取消后摇、停止、移动、换目标、重复指令或属性变化重置。
攻速变化只影响下一次新攻击，本次伤害则在出手时读取最终攻击力。
固定步量化使周期最多多出不足一个 tick，默认 1.25 次/秒对应 48 tick。

客户端将后摇显示为“攻击后摇”，并根据模拟中的剩余 tick 对模型子层做收招姿态插值。
身体、头部和武器位于 `CharacterModel` 骨骼子树，角色根位置、底座、模拟碰撞和血条锚点不参与该姿态变换；客户端命中读取已显示的各模型部件，随实际姿态变化。
动画只读取规则，不能决定命中、冷却或动作结束。

## 统一伤害

`DamageSpec` 定义伤害类型、基础值、攻击力系数和法强系数：

```text
raw = base + ad_ratio * final_attack_damage + ap_ratio * final_ability_power
```

定义参数必须有限且非负。技能提交前校验组合结果，结果无法表示时不扣资源、不进冷却。
普攻在出手成功时、技能在统一提交点生成 `DamageSnapshot`。
弹丸、延迟范围和引导脉冲使用此快照；来源之后属性变化、死亡或移除都不改动已提交的伤害。
快照携带 `DamageType` 与 `DamageOrigin`，区分普攻、具体技能、含实例的装备主动/被动，供观察和来源归并。

命中时读取目标的最终抗性：物理伤害使用护甲，魔法伤害使用魔抗，真实伤害忽略两者。

```text
resistance >= 0: multiplier = 100 / (100 + resistance)
resistance <  0: multiplier = 2 - 100 / (100 - resistance)
```

统一结算扣血并截断到 0，不提前四舍五入。`Damaged.amount` 是实际生命损失，0 实际损失不发受伤事件。
目标命中前的抗性变化生效；同一次引导的不同脉冲可以受到目标抗性变化影响。
普攻是物理伤害，具体技能按实际内容定义的伤害类型与加成执行；旧演示定义仅保留给装备/独立测试，不再是英雄默认装配。
默认R只有击退与击飞，组合效果不要求包含伤害。
这些示例系数属于内容配置，不进入输入、移动或伤害算法中的技能 ID 分支。

## 技能生命周期

技能 ID 标识内容，技能槽位标识角色的装配位置。Q/W/E/R 只映射槽位，
模拟不包含具体按键，输入和镜头不匹配具体技能 ID。
装备主动以`CastSource::EquipmentSlot { slot, instance }`进入同一begin/commit流程，
角色技能以`AbilitySlot`进入；两种来源共用目标/权限/快照/消耗/中断，只将冷却提交到各自所有者。

技能定义与运行态分离。定义包括目标类型、射程、前摇、收招、冷却、消耗、效果参数；
运行态包括施法目标、当前阶段、计时和每槽冷却。
注册时校验数值、目标与效果的组合，避免运行时拼出不合法状态。
引导脉冲间隔至少一个模拟步，支持与模拟步不整除的持续时间和间隔。
秒制时钟注册时同时验证tick数量上限和非零f32能被一步实际递减，避免巨大有限时钟永久停滞。

所有技能在**前摇结束**时再次校验目标和资源，再原子提交消耗、冷却与效果。
内部`resolve_target`返回实际执行目标：Dash保留原始点击意图到提交，使用当前位置和f64方向/距离中间值截取射程；
`AbilityCommitted.position`与运动使用同一解析结果，角色/装备共用。客户端只读is_dash提供可释放预览及说明。
这是当前统一的提交策略：

- 前摇期间取消：不扣能量、不进入冷却。
- 前摇结束时目标死亡、被移除或离开射程：拒绝提交。
- 提交后取消或施法者死亡：不退款，已发射弹丸、已布置范围效果继续存在。
- 引导被移动/停止中断：停止后续脉冲，已提交消耗和冷却保留。
- 冷却独立于动作，移动与走砍不能重置冷却。

增加由现有效果表达的技能时，在 `gameplay/content/` 注册新定义并装配到单位即可。
增加一种全新效果时，扩展 `Effect`、注册校验、快照与对应规则所有者；
新增生命周期机制应先用无窗口测试描述规则。不要在输入、相机、伤害系统中增加
“如果技能 ID 等于某个英雄技能”的分支。需要新的显示效果时，再添加对应的表现逻辑。

## 组合效果、状态和运动

`AbilityDefinition.execution` 使用封闭枚举：`Impact(ImpactDefinition)`、`Dash { speed }`、
`Channel { duration, interval, pulse }`。`ImpactDefinition`包含一个投递和非空的有序效果列表：

- `Delivery`：Direct、Projectile、Area；执行时创建载体或排队，均不直接打断其它actor。
- `Effect`：Damage、BonusAttackDamage、ArmorReduction、ApplyStatus、Displace。伤害参数在提交时快照；引导所有脉冲复用同一快照。
- Enemy可以直接命中、锁定生命跟踪弹丸、提交时冻结地点的范围；Direction使用首个敌人弹丸。
  Point使用范围或自主Dash，SelfCast使用敌方范围或范围引导；其它组合注册时拒绝。

提交前验证整组快照，成功才一次扣费并进入冷却；命中后按列表顺序逐项执行，不回滚已发生效果。
`[致死伤害,状态]`跳过后续状态，`[状态,致死伤害]`应用后由死亡清理；0伤害不阻断其它效果。
命中包按投递阶段、提交序号、载体序号、目标实体排序，actor提交遍历也按实体排序，
多次位移的覆盖顺序不依赖偶然的ECS查询顺序。

`StatusDuration::from_ticks(u32)`和`from_millis(u64)`返回`Option`，只接受正的有界时长；
60Hz下100ms为6步。`StatusSpec`区分Stun、Root、Slow和Airborne，Slow强度有限且在(0,1]。
`UnitStatuses::iter()`提供只读实例视图及来源/技能/剩余时间；`ControlState`提供
`can_move/attack/cast/dash()`、`forced_motion()`及`permits(&AbilityDefinition)`。
状态与运动锁共同派生权限，结束一个实例不会直接把所有权限设回true。

| 规则来源 | 自主走路 | 普攻/普通施法 | 自主Dash | 中断 |
|---|---|---|---|---|
| Slow | 按最终移速 | 允许 | 独立速度 | 无 |
| Root | 禁止 | 允许 | 禁止 | 终止自主Dash/移动类前摇，保留普通施法和引导 |
| Stun / Airborne | 禁止 | 禁止 | 禁止 | 中断普攻前后摇、施法、引导、自主Dash |
| ForcedMotion | 禁止 | 禁止 | 禁止 | 玩家输入和普通控制均不能取消当前强制运动 |

新受限命令拒绝并反馈，不排队；原持续移动/追击意图在解控后重新验证并重规划。
Root下可以下达AttackTarget，射程外等待；原AttackMove仍可执行允许的普攻部分。
中断技能不自动重放，不重置已提交攻击间隔、不退款，冷却和能量回复在受控期间继续推进。

Stun/Root的同来源生命+完整DamageOrigin+效果索引刷新到较晚期限，不加总时间；不同来源独立。
装备origin包含实例，因此同AbilityId的角色与多件装备不会错误合并状态。
Slow每次命中独立计时，只取当前最强，强短结束后弱长接替。
Airborne保留独立实例，显示取连续抛物线高度最大值，不因重新施加跳回地面。
显示高度不影响Vec2战斗距离、导航、命中或可选中性；暂停保留实际已显示的根和模型变换。

`DisplacementSpec`支持AlongImpact、AwayFromCenter、TowardCenter，距离/速度为有限正数。
方向取提交/命中上下文快照，来源消失也能继续；零向量使用快照朝向，拉拽不超过到中心距离。
Direct/Projectile的中心是施法者提交位置，Area的中心是冻结的范围位置；AlongImpact使用载体命中方向。
移动模块使用相同地图扫掠处理墙和边界，强制运动不叠加普通步行预算。
后一个请求从当前位置替换旧请求；运动结束只释放自己的锁，仍存在的控制继续限制动作。

`UnitLifeRef(Entity,epoch)`区分同一实体的两次生命。接受指向意图时固定目标生命，提交和跟踪命中复核；
旧锁定效果不能击中新生命，地面范围按当前存活目标处理。来源身份只用于归因，不要求仍存在。
死亡统一清理状态、减速、动作、意图和运动；重生增加代次并重置前后位置和显示高度。

HUD只读投影包含状态种类/剩余时间、控制权限、技能目标方式/描述/可用性，JS不计算规则。
逐帧标签与完整快照都投影实际显示根；`body_screen`是同一拾取偏移的可见屏幕坐标。
Enemy技能点击空地/友军保持瞄准；合法候选提交后退出，后续拒绝仅反馈，不自动重试。
默认装配Q点选眩晕、W范围伤害+禁锢/减速、E自主Dash、R方向击退/击飞。
原飞矢和引导仍在内容中注册，未引入具体技能ID规则分支。

## 角色、实际装配与开局

`CharacterCatalog`拥有不可变的角色名称/描述、UnitStats和默认技能ID列表，先注册技能再注册角色。
`CharacterDefinition::initial_attributes()`与单位构造复用属性模块的纯计算，包含攻速归一化；
预览不创建单位、不分配修正来源，也不在JS复制属性公式。

`spawn_character(world, CharacterSpawnSpec)`解析角色及完整装配后复用`spawn_unit`，成功时附加
只读`CharacterIdentity`。角色身份只存角色ID；实际HP、属性、状态和技能仍由原组件唯一拥有。
未知角色/技能或非法出生点等正常失败不产生半成品或玩家绑定。NPC仍可直接spawn_unit且没有角色身份。

`LoadoutSelection::CharacterDefault`复制默认四槽；`Explicit(AbilityLoadout)`完整替换，None就是空槽。
重复技能ID保持合法、冷却按槽独立。`AbilitySlots`是实际装配唯一事实，不回写模板；同角色多个实例彼此独立。
死亡重生保留角色ID、实例stats和实际技能ID，沿用原生命代次/清理/冷却规则，不重新套默认技能。
未来随机模式可先生成Explicit配置再创建，本期没有局内换技能或随机生成器。

`client/session.rs`拥有Selecting/Active两个阶段和唯一活动玩家Entity；`PlayerController`只保存选中目标和瞄准。
Web启动准备目录和地图并暂停，模式Waiting且没有单位，默认预选皇子。只读HUD此时player_id为null、
运行时abilities为空，选角使用独立预览；camera/scene/controls都按合法无玩家阶段处理。

`startup.js`同步位于Trunk模块之前，唯一保存loading/failed/ready，并向HUD订阅者立即回放当前状态。
WASM快速下载失败可能早于defer HUD执行，不能只在HUD末尾监听错误；启动适配不依赖DOM，
HUD就绪后再呈现失败/重试。实际ready为终态，早先或迟到错误不能把已经进入的游戏改为启动失败。
启动回归用请求屏障保证真实WASM拒绝早于HUD下载完成，不靠延时或碰运气复现。

角色卡片提交带request_id的SelectCharacter；每次接受都推进Rust选择修订并回显回执。
失败选择保留旧预览但撤销开始资格；旧Launch不能通过。点击进入冻结最新已确认角色/修订，
沿用按钮点击内的Pointer Lock请求；同一成功捕获回调先发送Launch再改变DOM暂停展示。
Rust固定步前复核阶段、开始资格、角色/版本、实际捕获和焦点，通过`start_match`原子创建两基地、
双英雄和双方首只近战兵，成功后绑定`MatchHandle`中的玩家和对局身份；后续出兵只由玩法固定步推进。
镜头模块观察到新的玩家实体时定位一次并保持自由模式，普通暂停/恢复保留原镜头；Selecting复位默认。
捕获或创建失败保留有效选择供重试；结构错误批次整体拒绝，结构有效的语义意图按FIFO独立处理。

浏览器实际锁定状态可能早于pointerlockchange事件。每次申请先清除旧事件确认，事件回调再确认捕获；
会话不释放尚在等待通知的锁，控制层也不提前启用输入或消耗光标居中边界。
只有事件已确认、请求处理后仍无有效玩家的捕获才作为无效开局释放。
Web指针在暂停时立即隐藏并清按下快照；GamePointer清捕获边界，恢复确认后重新居中并丢弃旧窗口/技能请求。

Active恢复仅重新捕获，不发初次Launch，不创建新玩家或重置对局。死亡仍为Active，局内选角/重复启动不会改绑定。
Selecting始终冻结模拟，排空菜单阶段游戏输入和命令，不向假Entity发Stop；Active暂停保留原一次Stop语义。
`SimulationPlugin`默认暂停值不改变；Native通过同一创建路径自动选择皇子，不等待Web事件。

暂停菜单与结算页均有“重新选择角色”，明确结束本局并重开整场对局。ReturnToSelection携带当前玩家Entity字符串，
会话核对身份和已暂停/实际释放捕获后，调用玩法层`reset_match`，回到无单位的Waiting。
该入口清除模拟实体及子层、技能载体、待执行命令/属性/命中包/消息，暂停并将SimulationTick归零；
地图、目录、静态场景和身份分配器继续保留。属性来源与效果/状态序号在同World内仍单调分配。
controls在无玩家边界清反馈和输入，camera复位原CameraRig；Brain随单位删除，不重建应用墙钟或相机实体。

Active保留开局选择版本，返回时递增并预选上局角色；旧版本Launch不能在下一局重新合法。
成功返回即结束当前请求批次，剩余旧局意图丢弃；旧玩家返回不能重置新玩家。DOM等Rust投影再进入选角，
清本地选择关联/捕获尝试，下次成功开局创建完整新阵容；普通继续仍恢复同一Entity。

结算页“重新开始”在真实按钮点击内先申请捕获，成功回调发送`RestartMatch { match_id, player_id }`。
仅当前Finished对局、当前玩家且已聚焦捕获的请求能重开；重置后以同角色原子开局并重新绑定。
Active到Active同样是完整会话边界，旧批次尾部和旧游戏输入丢弃；重开失败清绑定回Selecting。
`match_view`只读发布`match`与`map`，包括对局身份、阶段、结果、波次/倒计时、双方基地HP和共同几何。
`UnitView`显式投影role/wave/formation_slot，JS不按名称猜兵种或推导对局结果。
对敌军的单位/模型/标签/拾取投影共用团队可见性，未知敌基地生命为null，静态地图几何仍公开。

默认两名角色为德玛西亚皇子与寒冰射手，各自装配独立被动与完整QWER。一级基础值、技能与换算口径见[角色数值](champions.md)。
角色变化由内容和实际装配驱动，没有在输入/战斗模块增加角色ID条件。Web主入口支持两角色预览、键盘选择、捕获失败重试及响应式布局。
完整的核心接口与选角协议分别见Trellis模拟/客户端规范。

## 六格装备与触发边界

`EquipmentCatalog`存不可变定义：名称/说明、七属性flat/加算百分比、被动列表、可选主动AbilityId。
先注册技能再注册装备；定义注册完成完整校验后插入。`EquipmentSlots`保存每单位最多六件实际实例和各自CD，
所有单位初始为空，不在角色模板中复制库存。重复装备合法，属性、被动、冷却按实例独立。

`equip_equipment/unequip_equipment`是唯一装卸入口：先校验单位、格、定义和预期实例，再同步提交属性来源。
来源ID复用`AttributeChanges`分配器，装备实例封装该句柄，整个World中不回收；卸装不能误删后来换入的装备。
每实例贡献一个Explicit来源，属性模块从基础值重算；不直接加减最终值、不等待暂停期间不运行的固定步。
占用格拒绝穿戴，要求先明确卸装；失败不改变库存/属性/其它动作。

主动不占用四技能，以`UseEquipment { slot, instance, target }`进入统一施法。
卸装取消该实例未提交前摇与未来引导脉冲；已提交弹丸/区域/自主冲刺及普攻附属快照继续存在。
装备CD由固定步推进，死亡保留实例/加成且CD继续，重生不刷新装备CD；暂停冻结。
整局重开随Unit删除库存，保留装备目录与来源分配器，新局空六格。

首版被动仅`OnBasicAttackHit`。普攻出手前按格/被动索引快照全部附属效果，成功才提交攻击间隔。
命中先结算基础普攻，抗性后为正伤害且目标仍是活的同一生命，才解析附属快照（包含盾全吸收的命中）。
复用既有Damage/Status/Displace结算，只发现一层触发，不监听所有Damaged再递归派发；技能和装备主动不会触发。
命中不查实时背包，卸装或来源死亡不改变已提交快照；目标死亡则跳过其后续效果。

Web首版允许直接装卸：P或暂停菜单打开装配面板，暂停并释放捕获，关闭后仍暂停。
`client/equipment.rs`只接受当前Active玩家的封闭Equip/Unequip请求，严格整批解析后逐条执行，回显成功/错误。
六格HUD用1–6或真实鼠标释放主动，与角色技能共用瞄准/范围/说明，输入携带实例并在捕获边界丢弃旧意图。
静态目录仅首份完整HUD发送；equipment模块在Rust之前同步注册一次目录监听，缓存后移除，
其顶层不访问DOM，迟到HUD创建工厂时仍能取得目录。正常早工厂继续由render首份状态初始化；
JS缓存只读说明，动态属性/CD/可用性始终来自Rust，
原abilities四格与aim协议保持；装备使用独立equipment/equipment_aim字段和DOM选择器。

9件示例分别覆盖攻击、法强、双抗、移速、攻速/攻距、命中附伤/减速及普通/冲刺/引导主动。
演示数值与引用位于`gameplay/content/`，不在输入或结算中按装备ID分支。经济、合成、唯一组和共享CD属于后续规则，
当前每次穿戴创建新实例且主动CD为0。具体接口和错误矩阵见
[模拟装备契约](../.trellis/spec/simulation/equipment.md)和[客户端装备契约](../.trellis/spec/client/equipment.md)。

## 验证与维护

`make check` 执行格式检查、Clippy、完整 Rust 测试与 WASM 类型检查，CI 执行同一入口。
核心行为测试只安装 `MinimalPlugins` 与 `SimulationPlugin`；玩法测试按范围增加`GameplayPlugin`与独立`AiPlugin`。
通过 `CommandQueue` 下指令，
读取单位状态和模拟事件验证结果。测试不创建相机或加载模型，也不直接篡改运行态。
内部函数和 ECS 调度可以重构，而无需改写这些行为场景。

浏览器测试验证真实 WebGPU 渲染、鼠标拾取、UI 隔离、镜头、技能和暂停。
Linux 的截图验收使用带窗口的 Chromium（可运行在 Xvfb 中），避免无头 WebGPU 画布未进入截图合成的情况。
诊断快照 `window.__kivaState` 是只读投影，不提供修改游戏状态的测试后门。
浏览器DOM只能提交暂停、槽位/实例选择、封闭的会话和装备管理意图；不能传属性或实际技能数组。
技能是否合法由模拟验证，角色选择与开局由客户端会话结合玩法原子构造入口校验，装备管理通过核心装卸入口校验。
Web 端使用整个文档的焦点判断是否离开游戏，而非画布焦点。
技能按钮等 HUD 控件获取焦点不暂停模拟；切出页面仍暂停并清理待执行输入。
进入浏览器游戏需要成功捕获鼠标；Esc、切出窗口或捕获丢失会释放鼠标并暂停。
浏览器原生光标被隐藏，HTML光标由Web指针所有者直接显示；模拟层不感知捕获API。

后续新增装备触发类型、经济、多人同步等能力时，应先明确状态所有权、执行顺序、
指令中断规则和可观测结果，再实现扩展。更改既有规则时同步更新本文和行为测试。
