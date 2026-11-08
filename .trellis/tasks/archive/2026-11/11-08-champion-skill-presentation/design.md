# 技术方案

## 边界与分工

沿client→gameplay→simulation单向依赖。simulation只补已有结果的来源/目标/生命周期事实，不含纹理、粒子、颜色或皮肤；数值、碰撞、施法/命中顺序不变。
父任务直接实现模拟事实、client::skill_effects运行时、公共接线与联合验收；三个子任务分别交付skill_assets/离线资源、skill_indicators/共用目标意图、edge-camera-speed。
先完成真实资源/接口调研，再实现事实与资源，指示器和特效分别消费它们；root独占公共mod.rs、scene.rs、battlefield.rs、ui.rs和重型构建。工作子代理不得互相派生、并发Cargo/Trunk/GPU或操作服务/git。

## 事实接口

AbilityCommitted保留unit/ability/position原语义，补source_life、cast_source、origin、commit复核后的target、target_life、recast。中央helper仅在真实提交/重施处发布；方向正规化用现有宽中间值算法，取消和失败不发。

新增中性ImpactResolved结果，字段为source_life:UnitLifeRef、team:Team、origin:DamageOrigin、empower_origin:Option<DamageOrigin>、carrier:u64（提交组）、position:Vec2、direction:Vec2及kind：

- Contact { target:UnitLifeRef }：在真实payload结算首次生命/class验证后发，每payload/target一次；position取当前受效目标中心，包含纯控制和全盾吸收，不把空effects当命中。
- AreaActivated { carrier_entity:Option<Entity>, radius:f32 }：延迟区域真实到期或instant区域/Leap真实落地发一次，空场也有；独立于接触的每目标结果。
- ProjectileEnded { carrier_entity:Entity, reason:ProjectileEndReason }：真实碰撞、障碍、射程耗尽或锁定生命失效时发；位置取实际终点/接触点。原因Hit(UnitLifeRef)/Blocked/Expired/InvalidTarget。

Projectile/AreaEffect公开source_life()/origin()/carrier()/empower_origin()；BattlefieldObject存并只读公开origin、activates_at，Scout路径Reveal/到站Reveal与R圈/箭尾Reveal透传同一来源。
Shield实例记录origin，UnitMechanics::shields()返回不可写ShieldView {amount,origin,expires_at}；empower_origin()与强化状态同时set/expire/clear。snapshot_attack冻结强化来源，DamageOrigin仍BasicAttack，保证buff到期或来源消失后的在途强化箭外观正确。
不公开运行态写入口，不从client补算命中，不为配方增加模拟计时器。新事件按真实payload排序与旧结算共存，重开沿reset_simulation清消息。

## 本地资源与共享加载

采用research/league-vfx-assets.md的固定16.22 PNG、官方WAD SCB及已核对发射器参数。assets/vfx/保存原PNG、标准GLB、sources/catalog/参数依据；离线importer校验源及产物SHA，运行时不解析League二进制或联网。
client::skill_assets只拥有typed TextureKey/MeshKey/素材元数据、共享Image/Mesh/StandardMaterial句柄和ready/error；image/mesh/material/texture_spec/status为renderer共同入口。图集与静态UV atlas分开，SCB保留UV/顶点颜色并记录轴/bounds，外观范围按实际规则fit。原黑底全alpha指示图使用Add，不能用Blend贴成黑方块或将青色直接乘红色。
不造完整League粒子引擎；用原贴图/网格、已核对关键参数实现可识别的箭、尾迹、扫描、护盾、旗与岩墙。噪声/扭曲/层级发射器简化如实记录，不声称逐像素原生还原。

## 技能特效运行时

client::skill_effects拥有SkillEffectsPlugin：FixedUpdate在完整SimulationSet::Step后、GameplaySet::Outcome前独立读取真实提交/ImpactResolved，立即捕获位置/生命/阵营并按观察过滤；有限队列/短命显示缓存不成为规则事实。
Update在统一PresentationFrame、单位/载体显示变换之后，采样实际Projectile、AreaEffect、BattlefieldObject、Shield/empower，绑定对应原素材。carriers按Entity跟随、group只用于施法分组；瞬时Contact/Area/终止各自按事实派生短命显示。origin到外观的AbilityId映射集中在表现catalog，不在controls/simulation增加具体ID分支。
源被删除或死亡不取消已提交独立弹丸/区域；attached buff按真实当前生命/来源/状态停止。可见性按每个fact实际位置与team过滤，不能统一检查caster当前存活或把隐藏目标接触点泄露。
时间只读SimulationTick+PresentationFrame；普通暂停精确保留，终局一次最终同步；资源迟到不补播过期闪光，暂停不替换已显示样式。整局session变化清所有瞬时缓存/显示实体；载体的子层随根清理，动态临时材质/mesh有界并回收。
不加PickGeometry，不遮挡HUD输入；已接原材质/mesh时移除相应旧通用几何，失败则保留现有fallback，不能双重显示或双写UnitVisual世界根。

## 指示器与目标意图

client::ability_targeting作为controls与指示器共用的只读目标意图入口，返回实际技能ID、原意图、preview解析目标和Ready/ApproachingEnemy/Invalid/NoTarget。仅Enemy的OutOfRange表示可尝试走近；按有序模型候选找到首个实际可用目标，R跳过不适用兵/基地，不能高亮英雄却点到前面的兵。Point保留原点提交，preview结果只用于绘图。
CastInfo增加实际ability:AbilityId；不把ID复制进AbilityDefinition。删除scene里旧技能瞄准段，由独立SkillIndicatorsPlugin替代；A攻击移动与普通单位/反馈标记保留。
几何从实际AbilityExecution推导：Line按range/width，Volley按总angle/count/radius，Arrow画全range及整宽2*radius，Standard/Scout按preview真实截断点和效果/视野半径，Leap按合法Enemy及真实落点半径，SelfCast/未知Impact/Channel/Dash和装备走通用回退。源配置只决定素材/样式，不覆盖项目规则值。
指示器PostUpdate在当前camera更新/Transform传播/CameraUpdate之后、VisibilityPropagate/CalculateBounds/CheckVisibility之前，以唯一CSS指针当帧重投影。少量无父根固定mesh通过自己的Transform/GlobalTransform显式更新，不改角色/骨骼或PointerState，不维护第二套权威坐标。起点使用已显示UnitVisual，资格检查用模拟只读事实。退出aim、UI、暂停、玩家/实例失效同帧隐藏。
IndicatorView与SkillEffectsView只投影最小只读可见诊断，供HUD/浏览器检查，无任何写World桥；截图必须实际查看材质、方向、透明与尺度，不能只看metadata。

## 相机

保留原键盘+边缘方向合并、相反抵消和斜向归一化；存在有效画布内边缘输入时选择31.5*zoom*dt，否则仍18*zoom*dt。原14CSS带、地图夹紧、Y/Space、小地图独占、UI/暂停/焦点协议保留。

## 验收与回退

simulation新公开行为测试覆盖提交方向/生命/重施、纯状态/全盾命中、空区域、单箭身份、终止原因、来源删除/新生命、强化在途、对象来源透传及重开。
client覆盖实际ID/混装、几何、共用目标资格、可见性、暂停/终局/新局、临时资源回收及源纹理/SCB结构/哈希。执行make check、WASM Clippy和现有Node。
有窗口独占Xvfb串行验证原素材实际渲染与黑底/Add、八技能、原指示器与实际鼠标点、pan/zoom当帧回投影、800/DPR2、过距点/方向/Enemy追近、原模型拾取、音频、重选/清理、smoke/minimap/pointer及原生时钟healthbars。不接管用户服务。

## 实拍收敛

首轮GPU已定位并修正：W nova漏源UV旋转形成亮楔、Standard旧VCP全零乘黑、来源根先删除后的重复despawn；原盾只在真实组成功后接管回退。W锚点沿世界Y+.8而非随billboard旋转，显示表面局部前移.5并保留普通深度测试，以实拍确认穿体遮挡改善。Terrain移除在固定步记录实际时刻，跨显示帧不补播过期坍塌。具体证据和最终接受状态见verification。
