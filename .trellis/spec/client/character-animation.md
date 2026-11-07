# 骨骼角色与动作表现

## 1. 范围与触发

修改角色几何、骨骼资源、动画、普攻/施法姿态、暂停或生命周期表现时阅读。
`src/client/character/`恢复act分支的刚性骨骼火柴人：17骨段、14关节、头眼和骨骼武器（皇子枪、寒冰弓，独立Arena保留默认剑）。
双方英雄共用骨骼与动作适配，外观映射在weapon模块选择枪/弓，不按具体技能或装备ID判断规则。小兵/基地按`UnitRole`使用独立轻量表现，不挂英雄骨架。

## 2. 接口与职责

- `rig::AnimationLibrary`：内嵌KIVAAN01骨骼/局部动作数据，`sample(Clip, seconds, looping)`插值，`model_pose(local)`计算父序模型变换。
- `model::spawn_model(commands, owner, assets, library, pose, body, weapon)`：在单位下创建一个`CharacterModel`容器与所有几何子实体。
- `animation::RigPose::new(library, life, now, weapon)`、`model()`：本次生命的表现缓存及只读初始模型姿态。
- `record_events`：独立`MessageReader<SimulationEvent>`消费已发生的提交、移动、死亡和重生通知。
- `sync_poses`：读取权威动作、步位移、控制和目录，更新局部/模型骨骼及过渡。
- `sync_attachments`：从模型骨骼更新`BoneVisual`的局部Transform，武器socket附着hand_r，weapon模块提供几何与双臂持握姿态。

依赖方向为`client::character → simulation`；本模块不写模拟字段、命令、伤害或时钟。

## 3. 数据与时间契约

资源由`include_bytes!`同时嵌入原生/WASM。23根骨骼、19段30fps源轨道保留原始局部变换，
只统一源+Z至游戏-Z和1.1比例。根平移为零，不重复应用位移；来源、CC0许可和重建方式见
[动画资源](../../../assets/animations/README.md)。

`UnitVisual`继续唯一拥有世界平面插值、朝向和Airborne/公开leap_progress派生高度；底座抵消高度，模型容器本身不改变世界根。
拾取与标签读取同一实际显示根，骨骼手脚摆动不能移动拾取框、血条或模拟位置。

播放时间取`(SimulationTick + PresentationFrame.fraction()) / TICK_RATE`，暂停不累计宿主时间。
`presentation::prepare`为根、骨骼、附着和特效提供同一个采样分数；正常运行使用overstep，
普通暂停保留实际姿态。Finished首次观察使用1.0同步最后权威步一次，之后冻结；不可每帧重新采样宿主overstep。
移动姿态由`UnitPose::position - interpolated(0)`的真实步位移决定，步态速度按实际移动距离校准，
不能看到MoveTo命令就走路。普通过渡为0.12秒，普攻前摇/后摇直接使用对应阶段进度，短后摇不会被过渡吞掉。

普攻前摇缓存本次首次观察到的remaining作为准备阶段长度；它不使用可能已经变化的当前攻速重算。
低帧率跳过的准备帧不会补播。前摇前65%准备，后35%接近CONTACT；实际出手和命中仍只由模拟推进。
后摇使用自身duration；源Attack必须采样到完整duration后再接Recover首帧，两个接缝都要保持骨骼和socket连续。
`Casting`用实际AbilityDefinition.windup；引导和恢复跟随UnitAction，恢复可读取已提交技能的recovery。

瞬发/装备冲刺也可直接从`AbilityCommitted`被识别；不要求显示帧曾看到Casting。
已提交冲刺记忆与实际步位移组合显示冲刺，可持续超过Recovery；实际停止、后续非Idle指令、
新移动通知、已经越过目标的步、强制运动或硬控制清除记忆。它不驱动/延长冲刺。
同帧的提交→死亡→重生按事件顺序清理，`life_ref`变化也清理旧记忆。

普通暂停时只消费通知和记下待重置标记，不改已显示的局部/模型骨骼、附着Transform、过渡和播放时钟；
恢复后才应用待重置。死亡沿用隐藏根，重生重置骨骼姿态；整场重开通过根的linked children清理全模型。

## 4. 校验与错误矩阵

| 情形 | 行为 |
| --- | --- |
| 资源截断/错误magic/非23骨骼/非父序层级 | 启动校验失败，禁止显示损坏数据 |
| 非有限TRS、非单位四元数、非正scale、无效时长/帧数、重复名称、根平移或尾部数据 | 启动校验失败 |
| 缺少实际使用的clip或bone | 构造/装配失败，不静默回退 |
| 无动作、零位移 | 待机，不由命令文字推断运动 |
| 普攻取消/技能切换 | 跟随新的权威动作，无额外命中或冷却变更 |
| 暂停期间收到提交或生命通知 | 保留显示姿态，安全消费通知，恢复再更新 |
| 单位死亡/重生/整场重开 | 隐藏/重置姿态/递归清理，无孤立武器或眼睛 |

## 5. 正常、默认与错误案例

正常：瞬发冲刺只显示到Ready仍持续移动，从提交通知保留冲刺姿态，停止后回待机。
默认：新单位按武器采用中立guard，友敌材质不同；只有weapon模块映射角色外观，动作不执行角色或技能规则。
错误：在动画出剑某帧扣血、靠播放完clip解除后摇，或把动画root motion加到UnitPose上。

## 6. 验证与断言点

`cargo test --locked --lib client::character`覆盖全部源帧有限/归一/无根平移、循环采样、骨长和剑socket，
所有无模式测试演员/英雄单模型、真实对局仅英雄有骨架、小兵与基地的拾取/标签描述一致、终局最终根和骨骼同步后冻结、实际移动与Root停止、前摇→后摇/后摇内部源clip接缝连续性、最大攻速一tick后摇的0.9tick暂停/恢复/取消、
暂停前瞬发通知、施法/引导/恢复、冲刺超出Recovery、控制/击飞、同帧死亡重生与整场清理。
原`scene`击飞根、底座、暂停、重生测试继续保留。

执行`make check`和WASM Clippy；临时Trunk构建后运行smoke、skills、healthbars与reselection，截图使用有窗口浏览器。
截图检查关节分段、头眼朝向、武器握持、弓弦姿态和实际动作，不能仅凭HUD的action文字判定骨骼正确。

## 7. 错误与正确

错误：复制act::Fighter状态机，按具体E技能识别冲刺；暂停后把alpha置0重采样；将剑建在世界根外。

正确：复用act的独立资源与附着计算，读当前模拟动作/结果；暂停保留实际姿态；全部几何在单位子树内。
