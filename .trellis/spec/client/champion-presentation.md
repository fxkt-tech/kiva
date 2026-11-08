# 角色套件、观察与技能展示

## 1. 范围

适用于角色武器/动作、技能执行说明、层数/充能/重施、场上对象、护盾与团队视野显示。
模拟唯一拥有实际规则；客户端按通用execution、UnitMechanics和公开观察接口展示，不以技能ID执行规则。

## 2. 分工与公开投影

- `character/weapon.rs`独立选择Sword/Spear/Bow表现；角色身份1/2只在这一外观映射使用。
- `character/poses.rs`保留23骨骼/源动作，枪使用前刺、弓使用双臂拉弦；模型子树和武器手socket仍随根清理。
- `visibility.rs::VisibleWorld`按模拟`team_can_see/point_visible`采样，统一模型、轻标签、完整HUD和场上效果。
- `motion_view.rs`读取`is_dashing/leap_progress`，只转换为姿态/跃击显示高度，不写UnitPose。
- `battlefield.rs`只读旗、地形、鹰灵、揭示和护盾，实际寿命、光环、寻路障碍由模拟管理。
- `abilities.rs`按执行语义生成说明/可用提示，`web/skill-state.js`只格式化真实计数，不运行技能计时。

`UnitView.attributes`保留世界实际值；`display_attributes`只将移速/攻距经`gameplay::units::league_units`转换。
角色预览使用`initial_attributes_with_loadout`，因此已学军旗被动攻速与实际构造一致；base_attack_speed供基础值提示。
UnitView另有passives、shield、focus和empowered。所有值来自实际定义或运行态。

`AbilityView.range`保留world几何，`display_range`用于DOM；新增mechanics包含focus/required_focus、empowered、
charges/max_charges/recharge、recast/recast_ready。静态预览没有伪运行时计数。
`HudView.objects`只包含可见的Standard/Terrain/Scout/Reveal及position/radius/remaining/active，owner不可见时为null。

## 3. 输入与生命周期

键盘QWER与鼠标按钮共用select_cast。实际slot有recast时，无目标提交SelfCast；根据recast_ready启用，
不受普通长CD/二次费用阻挡。普通Enemy目标由preview_cast统一预检；只将OutOfRange放行成一次原Unit命令，由模拟追近，随后退出aim。错误类型、隐藏/无效候选仍保留aim。
只读order使用Approaching cast展示待放目的；Enemy提示说明超距自动走近，射程外预览用黄色，其它非法超距仍用红色。
Empower层不足不可释放，提示先普攻；Scout按实际charges/cooldown启用。JS不得补算层数或恢复充能。

地点截断预览使用`clamps_point_target`（Dash/Standard），实际原点击点仍交模拟解析；不在输入端写最终位置。
军旗只有active后才显示落地旗；护盾读取shield总量；圈墙几何来自真实对象，不能用一个装饰圈替代导航规则。
首个ProjectileVisual就按共享观察设置Visibility，并使用实际插值位置/方向/大小，不能等下一帧才藏敌箭。
TerrainPlacementFailed仅在unit为当前玩家时反馈「附近空间不足，围墙未生成」，不泄露隐藏敌方施法。

## 4. 视野与显示边界

所有单位观察复用模拟只读接口；独立无玩家/无视野Arena保持全视野。摄像机平移不提供视野。
隐藏敌人的模型、血条、HUD/unit JSON、拾取均不可见；输入直接复核当前团队观察，不能靠80ms旧HUD命中。
轻标签每帧只发送可见单位和最小标签元数据，显露当帧可创建标签，缺席的旧标签/目标面板当帧隐藏。
地图边界与基地静态坐标已知；不可见敌基地血量为null并显示「未知」，不能写0或给完整诊断后门。
不增加迷雾位图依赖；技能说明与界面明确未侦察敌人不可见，鹰灵提供真实团队侦察。

## 5. 正常与错误示例

正常：皇子属性显示移速340/攻距175，world字段为3.4/1.75；已学E实际攻速.7896，基础提示.658。
正常：寒冰Q初始0/4且禁用，E显示1/2；4次真实普攻后Q可激活。R圈生成后按钮显示重施并无目标拆除。
错误：Web自己计90秒恢复E、用相机位置揭露敌人、把旧普通CD当重施权限，或在动画命中帧扣血。

## 6. 验证

保留原generic输入/控制/装备回归；旧通用技能装配只放cfg(test)显式角色fixture，生产只有实际新套件。
新增客户端公开行为覆盖盾、E超距截断、英雄目标/重施、focus/charges、视野揭示/到期/暂停、隐藏目标拾取、
首帧隐藏投射物、枪弓骨长/有限姿态、原暂停/终局根和附着冻结。
`browser-champions`验证真实套件/对象/视野并保存有窗口截图；focus实战可穿真实纯防御样例装备，不能修改AI/血量。
`browser-skills`验证新套件键鼠槽位/取消/HUD隔离/暂停；characters/stats/reselection/equipment保留捕获、DPR、小窗口与旧请求边界。

实战装备夹具先完成伤害/状态断言，再使用真实护盾、装备冲刺和移动退出追击带；后续独立位移/引导在空旷侧翼进行，
不能把静止训练目标的生存假设带入英雄AI，也不能假设基地提供无敌保护。冲刺后等真实终点，再提交下一条移动命令。
失败日志收集全部console级别（tracing-wasm的Rust WARN/ERROR也走console.log），分别记录Date.now、真实tick及RAF/state/labels计数；
fixture的performance.now不是模拟已推进的时间，软件GPU与Time.max_delta会令二者不同。长对局同时限制宿主等待和模拟时长。

## 7. 约束

显示换算集中在Rust投影，装备flat移速/攻距同样换算，additive_percent保持百分比；范围说明不混world与端游数值。
规则继续留在simulation，战术/装配在gameplay，客户端只读事实与提交封闭意图，不公开World写桥或隐藏训练模式。
