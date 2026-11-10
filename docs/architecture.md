# Kiva 竹林演武架构

本项目是单 crate 的 Bevy / Avian 三维物理游戏。依赖保持 `client → gameplay → simulation`，客户端可以直接读取模拟状态。
旧 MOBA 实现、资源和活跃规范已移出当前版本，完整历史在 `lol` 分支；Trellis 归档只作历史参考。

## 状态所有权

| 模块 | 唯一负责的状态 / 行为 |
|---|---|
| `simulation/physics.rs` | Avian 权威三维姿态、角色控制、接触支撑、道具快照与瞬移清理 |
| `simulation/movement.rs` | `Body` 只读三维脚底快照与平面投影；`MoveIntent` 世界 x/z 移动意图 |
| `simulation/actions.rs` | `ActionIntent`、`Facing`、`MotionState`；动作计时、跳跃额度及接触结果 |
| `simulation/combat.rs` | 分离 E／Q 意图、显式目标、动作准入、共同命中、生命、敌人攻击、复活与有界反馈 |
| `simulation/abilities.rs` | 动作／联动时间线快照、来源窗口与逻辑妹妹状态 |
| `simulation/defense.rs`、`resources.rs` | 有序右键、拆招／防御／弹反、耐力与减速的唯一状态 |
| `simulation/boss.rs` | 单愁铃客的原作决策池/权重/行动冷却，以及Kiva适配的阶段、预警、命中和复活 |
| `simulation/impact.rs` | 狙击命中停顿预算、固定步门控与恢复 |
| `simulation/weapons.rs` | 锤鸟／链斧弓绑定组与哥哥近战普攻数据；近远枚举只作握柄类型 |
| `simulation/geometry.rs` | 旧无窗口夹具的几何描述适配，产品碰撞由 Avian 结算 |
| `gameplay/mod.rs`、`gameplay/terrain.rs` | `Level`、`Prop`、`Boundary`、`Player`、`EnemyMode`、共享地形几何及实体装配 |
| `client/controls.rs` | `ViewYaw`，设备输入转世界方向；捕获失效时停止意图 |
| `client/enemies.rs`、`web/enemies.js` | 进入卡片 / F2 的敌人模式请求及模式显示，调用玩法装配接口 |
| `client/appearance.rs`、`web/appearance.js` | 进入 / 暂停卡片和 F3 的玩家外观选择，只替换显示资产，不改变玩法武器套装 |
| `client/scene.rs` | 玩家显示根和插值后的显示 `Transform` |
| `client/stickman.rs` | 角色构造兼容入口及动态右手挂点 |
| `client/avatar.rs` | 两套随包角色 / 近战外观、各自蒙皮与材质缓存、结构动作适配；只写显示骨骼 |
| `client/environment.rs`、`environment/ground_cover.rs` | 玩法同源竹干/地形、共享草叶装饰、雾和灯光 |
| `client/rendering.rs`、`client/texture_mips.rs` | 相机 deferred/SSR/FXAA pass；重复纹理的线性颜色与单位法线 mip |
| `client/source_animation.rs` | 986节点原主骨架与35个具名原作片段的有界解码、稀疏采样和层级求值 |
| `client/forest_material.rs`、`assets/shaders/forest.wesl` | 标准PBR地表分层与干湿响应、共享环境材质和近叶可见/深度裁切，不改变模拟碰撞 |
| `client/skeleton.rs` | 随包 CC0 动画的解码、局部采样、父子骨骼求值 |
| `client/animation.rs`、`animation/poses.rs` | 根据只读模拟快照与明确 actor 的事件选择 / 混合身体姿态 |
| `client/camera.rs` | 从相同显示位置与朝向派生跟随镜头 |
| `client/combat.rs` | 只读敌人 / 武器 / 标记 / 特效，按反馈序号消费并回收临时实体 |
| `client/companion.rs`、`weapons.rs`、`weapon_models.rs` | 独立白灵淼、伏浪锚／谪星摧山／引冰落／羿之影共享源模型、链斧离手／回收及绳索表现 |
| `client/impact.rs` | 狙击枪口、弹道、空中命中火花和镜头后坐的共享资产及显示时钟 |
| `client/audio.rs`、`web/audio.js` | 实际战斗反馈的一次消费、短音效与暂停同步的音乐循环 |
| `client/hud.rs`、`web/hud.js` | 把玩家与当前选中敌人的真实快照显示为血条、人格 / 武器、技能可用性、冷却和联动提示 |
| `web/input.js` | 浏览器锁定确认、焦点、鼠标位移单位与一次性消费 |
| `web/startup.js` | 在 WASM 加载前记录成功 / 失败，给晚订阅者重放 |

Avian 的 Position / Rotation / 速度是唯一运动权威；`Body` 和 `PhysicsPose` 只保存物理结果快照。
客户端使用 `ActorView { owner }`、`PlayerView` 和 `PropView { owner }` 的独立显示根，不能用模型 `Transform` 反写物理实体。
角色物理中心与脚底相差当前胶囊半高；显示和相机使用脚底锚点，箱子显示使用中心位置与完整转动。
`Level.props` 装配真实三维道具；`Level.ground/ramp/forest_trunks/forest_rocks` 原始顶点 / 三角形同时生成 Collider 和 Mesh，围墙使用共享 Boundary 尺寸。

## 一帧中的顺序

1. Startup 顺序执行：玩法 / 物理场景装配 → 独立三维显示根 → 战斗表现 → 相机 → 通知 Web 引擎就绪。
2. Bevy 输入更新后，在 `RunFixedMainLoopSystems::BeforeFixedMainLoop` 采样控制。
   鼠标位移直接改 `ViewYaw`；提交世界方向 `MoveIntent` / `Facing`、持续动作和锁存的离散指令。
3. `FixedUpdate` 以 60Hz 执行动作／防御准备 → motor → Avian 求解 → 当前空间查询、接触与快照 → 玩家战斗／普通敌人 → Boss → 耐力恢复／妹妹快照 → 瞬移清理。
   离散指令在实际固定步消费，显示帧无固定步时保留、多固定步时不重复；支撑发布排在当前空间查询刷新之后。
   `SimulationActive` 共同门控物理 StepSimulation 和全部规则，菜单期间角色、箱体、冷却及 AI 一起停止。
   `HitStop` 在步头锁定同一步的执行决定，狙击命中后停顿后续 3 个固定步（约 50ms），同时冻结动作 / 物理 / 战斗而保留速度；失锁保留预算，恢复无补算。
4. Update 先回收失效敌人的显示树和血条、补齐新敌人表现，再同步人物 / 箱体 / 敌人显示根、武器显隐、身体姿态、蒙皮骨骼、骨骼附件、HUD、相机。三维脚底与箱体中心姿态按 fixed overstep 插值；
   暂停或命中停顿时统一取当前状态，避免 fixed overstep 持续循环导致模型 / 镜头往复抖动；命中停顿冻结骨骼时钟。
   专用冲击表现及音效消费在骨骼附件后、HUD / 相机前执行；冲击粒子和后坐可在命中停顿中继续，失锁时冻结。
   相机读取同一个显示根位置，朝向使用当前 `ViewYaw`，不追加平滑延迟。
5. Bevy 完成全局变换传播和渲染。

## 控制与构图

移动意图用 `Vec2(x,z)`，实际位置 / 速度为 `Vec3(x,y,z)`；向前为局部 `-Z`，`Quat::from_rotation_y(yaw)` 决定平面意图方向。
鼠标右移令 yaw 减小，以 `0.0025 rad / CSS px` 直接累计，绝不再乘帧时间。
移动意图长度不超过 1，步行目标速度为 6m/s。人物使用半径 0.35m、站立总高 1.73m 的直立动态胶囊，
通过有限加速度追踪支撑面切向目标；碰撞、重力和冲量统一由 Avian 求解，高速 CCD 包含动态箱体。
接地依据脚底接触、法线、距离与相对速度，允许站在箱顶 / 坡面，包含休眠和移动支撑；离开支撑后自然下落。
边界使用真实 0.7m 围墙而非无限高平面钳制，跌出场景后通过复活流程恢复。

固定相机距离：相对人物身体中心后方 4.2m、上方 6m，垂直 FOV 50°。
后退距离固定为高度的 0.7 倍，调整高度时同比缩放，保持镜头俯角。
人物身体中心为显示根上方 0.9m，实际投影固定在水平 50%、从顶部算垂直 2/3 的下三分点。
俯角由相机偏移、FOV 和目标屏幕位置推导，不通过偏移人物模型来伪造构图。
联动读取模拟 SequenceSnapshot：锤鸟三枪使用低位背后仰视，0.20s 入场、0.30／0.42／0.54s 发射、0.68–0.96s 回切；链斧弓拉近使用人物胸口附近高度的平视，0.65–0.93s 回切。
镜头与身体共用显示位置和固定步时钟，不另建射击时钟。角色仍参与真实物理；镜头用 0.15m 球体扫掠限制遮挡、留 0.03m 净空。
联动期间不累计鼠标转向；取消、倒地或 F2 替换清理动作归属的旧后坐，暂停也不能留下永久偏移。

## 场景与地形

场景为确定性 50×50m 竹林重建，包含中央战斗空地、曲径、密集竹海、岩石与可见低岸；不声称已恢复原版关卡实例布局。竹干和岩石使用玩法层的共享三角几何生成碰撞及显示，枝叶、竹节、落叶与草丛只作装饰，环境合并为 20 个网格 / 7 个扩展材质，12组植被随模拟暂停风动。地表用原土壤/草苔主色与法线分层，原草叶atlas构造不规则草丛；局部积水为非金属低粗糙度表面，deferred/DepthPrepass/SSR提供屏幕内反射，alpha透光植被保留前向路径。近相机叶片裁切同步到深度prepass；原叶片阴影保留。旧箱子/石柱不再装配到默认地图，通用箱体控制和道具姿态接口仍由物理层提供。
西侧坡道宽 3m、坡度约 20°，连接 1.5m 高台；东侧小山顶约在 `(16.5,2.6,8.5)`，西侧缓坡可登顶，东侧较陡用于测试行走上限。
地面与小山由一份连续三角网格构成，客户端 Mesh 与物理 Collider 使用同一原始顶点和索引，不创建共面的第二块碰撞地板。
坡道与高台为连续闭合几何。接缝、法线、出生净空和缓坡 / 陡坡由几何与真实控制器回归共同验证。

## 动作与联动

Shift 参考 `origin/act`（`bd3e344`）：按下立即沿当前移动方向闪避，静止时向 Facing 的反方向后闪。
短按闪避持续 0.42s，以 smoothstep 提供累积 2.4m 的运动目标；实际位移受碰撞限制。0.06–0.28s 为无敌窗口，冷却仍为 0.8s。
按住 Shift 满 0.16s 且有物理方向键按住时锁定疾跑，初始闪避在 0.26s 提前结束后衔接。
疾跑为步行的 1.6 倍，松开 Shift 不停止，松开所有方向键才解除；W+S 等抵消为零不算松开方向键。
失焦 / 暂停通过清理动作意图请求取消疾跑锁存，在恢复的首个固定步执行，客户端不直接写 MotionState。
Space 参考 `act` 的两次跳跃：每次把竖直速度设为 6.4m/s，空中可再次起跳，第三次无效，落地恢复次数。
持续按住不重复起跳，暂停保留已用次数，死亡 / 复活重置；进攻动作期间的新跳跃／闪避拒绝，不延迟重放。
动画读取已用次数，二段跳重启原作共用跳跃阶段；远程/缺源动作回退到CC0的NinjaJump等姿态，不额外叠加模型高度。
右键按下开启 0.20s 拆招窗口，持续达到 0.20s 进入防御，从有效防御松开开启 0.18s 弹反窗；各自再触发冷却 0.65s。拆招免伤，弹反免伤并使攻击者僵直 0.80s。
防御正面有效命中减伤 50%、消耗 20 耐力；耐力上限 100、不足时正常受伤，安静 1s 后每秒恢复 15。无三次限制、破防或额外反伤。防御状态独占于 DefenseState，不再保存在 MotionState。

C 点按切换蹲伏 / 站立，松开保持，蹲行固定 1.6m/s；疾跑按 C 启动锁定方向的 0.55s 滑铲，
初始目标速度 5m/s 线性衰减，理想运动目标总长 1.375m；实际位移由接触求解限制。
滑铲结束后保持蹲姿，C 可提前起身结束。蹲姿兼容普攻和防御；滑铲中不能起手攻击 / 技能 / 联动。
跳跃、有效闪避、击飞、死亡复活和暂停清理退出蹲伏；空中或闪避中按 C 不排队到落地后执行。
蹲伏胶囊总高为 1.40m，改变姿态时保住脚底；起身前检查净空。重力统一为沿 -Y 的 18m/s²，
跳跃与击飞提供速度，落地依赖实际支撑，不强制回到世界 y=0。箱子为可转动动态刚体，石块 / 石柱 / 地形为固定刚体。
愁铃客敌人与玩家可选身体使用随包蒙皮模型、原始 UV/四权重、主色与法线贴图。`assets/choulingke/character.bin` 由离线工具生成，记录原始 GUID、材质槽和来源哈希；不在运行时读取游戏目录。13 个穿戴部件与配件共用网格、纹理和不可变受击/蓄力材质变体。敌人保留原刀外观，哥哥当前手持由锤鸟／链斧弓绑定组决定；刀鞘和腰刀仍为身体配件。原主骨架986节点已恢复，原mix图经DXIL家族证据转换成线性ORM；布料/发丝物理及完整原BRDF尚未移植，使用Standard PBR近似。
哥哥默认李火旺，可在进入／暂停卡片或 F3 选择愁铃客／李火旺身体和服饰，敌人仍为愁铃客。选择属于 AppearanceSelection，reconcile 在动画前只替换玩家身体／调色板，保留物理 actor、显示根、RigPose、动作和当前武器。
妹妹使用独立 CompanionView／RigPose 和固定 Bailingmiao 外观，不含 PlayerView／ActorView，不能被 F3 换成哥哥。身体资产分别缓存，重复技能／换装不增长资产。

`scripts/import-prepared-models.py` 从冻结的 `assets/prepared/naraka` 离线生成白灵淼 KIVAAV03 与引冰落／羿之影 KIVAWP01，无新增运行时 GLTF／JSON 依赖。白灵淼衣装、莲母和女性基脸保留女性自身骨序／逆绑定／当前 rest，以自身 23 结构参考接收既有动作；双手握持使用女性臂长 IK。远程武器保留真实当前 LBS 后烘焙为刚性几何，局部发射源0／朝向 -Z 对齐模拟；弓附件按明确的 Kiva 适配挂在 spine_03。专属捏脸、原发丝／布料 shader 和原作开火／拉弓动画未恢复，来源与适配见 [白灵淼](../assets/bailingmiao/README.md)、[引冰落](../assets/yinbingluo/README.md)、[羿之影](../assets/yizhiying/README.md) 与 [模型规范](../.trellis/spec/client/model-assets.md)。

`scripts/import-melee-models.py` 单独生成 [伏浪锚](../assets/fulangmao/README.md) 和 [谪星摧山](../assets/zhexingcuishan/README.md)，复用同一 KIVAWP01 材质／几何格式，旧五组资产与转换器保持稳定。四件武器由 `WeaponModel` 以 pair＋mode 缓存；近战握柄0／主轴 +Y，手持和 FlyingAxe 共享同一网格／材质。谪星的主体与四个非 FX 输入根据原控制器证据作固定 Kiva 静态装配，不声称恢复原随机星格、Float 形变、溶解／灵光／cubemap。绳索在本帧手腕附件同步后连接真实握柄和飞斧根，物理轨迹与动作标记仍由模拟决定。

李火旺资源由 `scripts/import-naraka.py` 生成 `assets/lihuowang/character.bin`，包括衣装、心素发型、惘心笠、通用基础脸和原刀鞘，烛泪以刚性当前姿态适配既有握柄轴。惘心笠为单独原头饰3411276，跟随原头骨绑定，不改变已归一化角色比例或碰撞高度；原帽下限发协议未完整恢复，不凭隐藏mask删掉整套发型。KIVAAV03 在现 KIVAAV02 上增加材质双面标记及每骨的源模型空间姿态，保留原逆绑定矩阵；源导入姿态不等于绑定姿态时不强行抵消为单位矩阵。该外观用自身 23 个结构锚点接收既有动作增量，附件与身体读取同一适配手腕，不直接索引燕云 986 节点或使用另一角色的绝对手位置。具体来源、重建挂接、通用脸与材质适配限制见[资源说明](../assets/lihuowang/README.md)及[接入说明](plans/naraka-appearances.md)。
`avatar::sync` 位于动画与挂点同步之间，用源绑定矩阵和既有动作增量更新独立显示骨骼；武器位置使用适配后的手腕，不能反写 Body。角色标准高度 1.73m，靶子由 `Body::scaled(1.45)` 同时放大显示和胶囊；胶囊半径、站立/蹲伏高、瞬移和复活使用实际尺寸。
RigPose沿现有状态优先级与只读反馈选择原作长刀/卸势/跳跃/闪避/受击片段；source_animation解码35个具名clip，完整原层级求值并按骨名驱动调色板。共享动作保留愁铃客rest骨长，未恢复原技能槽和命中帧，裁剪/时长属于Kiva适配。移动、蹲滑、枪姿、旋转与倒地等继续用CC0的23骨骼/19片段和现有IK；两路平滑混合，独立附件缺失部分保留结构映射。
资源嵌入WASM/原生二进制，运行时不读取游戏目录或联网加载。原动作平面根位移剔除，地面保留腰身竖直起伏，空中根高归模拟；原手腕相对rest的旋转增量用于握柄校正，避免将源轴约定直接套到旧枪轴。
动画只写 RigPose，附件同步只写肢体与武器 Transform，不写人物显示根、Body 或相机。
蹲伏复用 act 的模型空间腿部 IK：骨盆平滑下移 0.33m，保持脚掌位置 / 朝向和腿长，0.12s 蹲下 / 起身。
RigPose.local 保存蹲伏 IK 前姿态，IK 只作用于生成 model 的副本，避免状态过渡反复累加下移。
暂停时冻结动画时钟；倒地敌人身体保留以播放死亡姿态，生命 / 选中 / 伤害规则仍按死亡状态处理。

玩家生命 1000，始终操控哥哥；1／2 选择锤子＋鸟铳／链斧＋弓箭，中键显式锁定／解锁，默认不自动补选。左键固定近战，E 近战技、Q 妹妹远程技，Tab 消费本次技能和原目标的联动窗口。
妹妹是无独立生命／碰撞／AI 的逻辑 Companion，技能演出使用独立白灵淼蒙皮；伤害归玩家、发射 actor 为妹妹。E／Q 冷却分开并跨组保留，动作中拒绝新进攻或换组，不排队。

- 锤 E 在 4.5m 内选中目标周围 3m 结算 20 伤害与约 3m 真实击飞。0.85s 内原目标仍浮空可 Tab；接受后自然落地不取消承诺，三枪各自 28 伤害。鸟铳 Q 单枪 40 伤害和 0.45s 僵直。
- 链斧 E 自身范围伤害与减速；弓 Q 射出链斧，35 伤害和 0.90s 来源僵直，未接联动则回收。Tab 只认本次 Q 对原目标的僵直，真实拉近期间三箭各 18 伤害。离手期间不能近战，完成／取消归还链斧。

拉近通过 motor／Avian，接近约 1.4m 停止并按制动预算减速；墙体、失去有效前进或严重高差取消，不能用瞬移或显示根移动伪造接近。
每发在各自固定步标记重新检查目标、视线和阶段免疫，死亡／移除／无敌／模式替换取消未发部分。最后一发锤鸟命中沿用一次 3 步 HitStop；普通 Q 不产生该停顿。
细节与可执行验收见 [.trellis/spec/simulation/combat-v1.md](../.trellis/spec/simulation/combat-v1.md)。

默认使用靶子模式，装配 1 名 1.45 倍愁铃客、100 万生命的被动敌人，关闭追击和攻击，保留碰撞、伤害、击飞与复活。实战模式装配单个标准身高的愁铃客Boss，采用已恢复P1/P2决策池、权重、行动槽与冷却。650×2条生命、0.72s预警、命中体积和14/18伤害为Kiva适配，完整原技能槽到动作/数值尚未恢复。Boss组件独占阶段和行动状态，普通Enemy主动AI/复活跳过它；转阶段无敌、释放弹反破绽、玩家倒地重置和击败后重开均在固定步结算。
进入卡片或 F2 提交模式选择，由 `gameplay::set_enemy_mode` 替换敌人，清理玩家旧目标、联动、拉力和妹妹可见性，保留玩家物理状态、生命、耐力、武器与冷却；暂停时也同步清理。
敌人和玩家倒地 3s 后回到出生点，靶子复活仍为被动，复活不重放死亡期间排队的指令。
`CombatFeedback` 持有最多 128 条单调序号反馈，显示按游标消费，避免零个 / 多个固定步丢失命中特效。
反馈 actor 明确演出者／受害者，source_owner 明确玩家归属，pair／action_id／hit_index 固定事件产生时的武器与动作身份；历史重放不能读取最新套装来猜。
身体动画按 actor 和 sequence 消费，不通过距离或中文 HUD 状态推断是哪一角色发生动作。

## 浏览器边界

`startup.js`、`input.js`、`hud.js`、`enemies.js`、`audio.js` 必须同步安装在 Trunk 生成的模块之前，脚本不依赖 DOM 已创建。
`window.kivaAudio.playEffect(name)` 消费已结算的挥击、命中和防御，`playSniperShot()` 消费妹妹真实鸟铳射击；弓发射临时复用共享 attack cue；伴随的狙击特效事件不重复发声。`setActive(bool)` 同步模拟暂停，可信进入点击 / 画布按下解锁 Web Audio 并缓存随包音频。随包BGM使用官方愁铃客发行录音《惊惧一夜》，原文件/转码来源记录在assets/audio/music-manifest.json，运行时不连接音乐站点；该循环不是已恢复的原bank分轨/阶段切换。音乐至多一条循环，暂停保留播放位置并释放节点；短音效至多 4 声部，未解锁/暂停/失败的声音丢弃，不延迟重放。原生播放为空操作，无新增平台音频依赖。
`window.kivaEnemies.takeMode()` 返回并清空待处理模式（0=无、1=靶子、2=实战），`render(mode)` 显示 Rust 当前模式；模式请求在固定循环前处理，不受暂停门控。
输入桥提供 isActive／takeLook／keys／takeActions／takeDefenseEdges／ready；精确位掩码和有序 Pressed／Released／Cancelled 见 [.trellis/spec/client/input.md](../.trellis/spec/client/input.md)。垂直输入不再调整落点；滚轮不切换人格。

只有收到 `pointerlockchange` 确认且页面有焦点才能操作，Promise 完成不代表捕获成立。
Esc 解锁或失焦会清累计，失焦后必须重新点击，延迟到达的旧请求不能恢复控制。

未锁定时从可信鼠标的 `clientX` 与 `movementX` 学习 CSS 单位增益；DPR 只用于判定校准是否失效，
不直接用于除法。合并事件与父事件二选一，排除窗口边缘截断样本。浏览器保留系统鼠标加速。
Web 桥记录有效捕获期间的可信按键，keys() 返回 W=1 / S=2 / A=4 / D=8 / sprint=16；失锁清零，
忽略新的重复 keydown，防止快速失焦再捕获重放旧按键。原生按 WindowFocused 事件消费中间的失焦，
不能只看最终 Window.focused。原生版使用 Bevy 输入及 `CursorOptions`，Windows 使用 Confined，其他平台使用 Locked。
右键完整短按与取消必须有序保留，失焦不制造释放弹反。C 保留切换奇偶性，Shift 保留物理按住与独立释放事实；捕获点击不同时普攻。

普通战斗特效在失锁暂停时冻结已有实体、同时前移反馈游标以免恢复后补播；HitStop中仍可继续显示效果，寿命到期回收共享资产。

HUD 显示生命、耐力、E／Q 独立冷却、当前组、防御阶段和 Tab 窗口；目标严格来自 Combatant.target，无目标／死亡即隐藏。
renderLoadout 与 renderResources 使用显式武器组／防御枚举，renderTarget 读取实际敌人状态；完整签名见 [.trellis/spec/client/combat-presentation.md](../.trellis/spec/client/combat-presentation.md)。
顶部居中为敌人长血条，底部为玩家生命／耐力／状态，右下为双技能和联动。Web 仅更新安全文字、progress、图标、显隐和无障碍属性；DOM 早期快照缓存重放，无独立冷却时钟。

Web 画面固定 16:9，按窗口最大可容纳面积缩放并保留 12–24px 最小留白；外表面采用 kiva-agent 下拉框的明暗配色、
3px 双向凸起阴影和 1px 描边，随系统切换明暗。

## 扩展原则

新游戏规则放在模拟层，玩法层定义关卡和装配，客户端负责意图和表现。添加会改变位置的机制时，
仍通过模拟驱动物理组件并发布 Body 快照；修改地形时以 Level 共享几何为源。物理接入契约见 [.trellis/spec/simulation/physics.md](../.trellis/spec/simulation/physics.md)。
新增方向、碰撞或时序行为应有无窗口回归，Web 生命周期使用实际桥接脚本的 Node 测试。

资源、转换命令、地图重建边界与本次验收见[愁铃客方案](plans/choulingke-bamboo-arena.md)。

Body::aim_offset()/aim_position() 统一按标准胸口0.9m×scale给出锚点；范围仍比较脚底，视线/命中/狙击与客户端瞄准使用相同实际锚点，玩家普通相机构图不变。
