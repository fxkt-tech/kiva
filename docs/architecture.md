# Kiva 竹林演武架构

本项目是单 crate 的 Bevy / Avian 三维物理游戏，通过 Trunk / WASM / WebGPU 在浏览器运行，也支持原生桌面。
本文只回答三件事：状态归谁、一帧里谁先谁后、新东西该放哪。具体数值、接口签名和资源来源不在这里，见文末的[去哪里找](#去哪里找)。
旧 MOBA 实现、资源和活跃规范已移出当前版本，完整历史在 `lol` 分支；Trellis 归档只作历史参考。

## 分层

依赖方向固定为 `client → gameplay → simulation`，只能单向。

| 层 | 路径 | 负责 | 不能做 |
|---|---|---|---|
| 模拟 | `src/simulation/` | 权威运动、动作、战斗、Boss、生命与复活，全部在 60Hz 固定步里结算 | 引用客户端、输入设备、相机、网格或浏览器；携带界面文字 |
| 玩法 | `src/gameplay/` | 关卡几何、实体装配、敌人模式 | 创建 Mesh 或决定表现 |
| 客户端 | `src/client/`、`web/`、`index.html` | 设备输入转意图、显示根、骨骼与蒙皮、镜头、特效、音频、HUD、加载 | 写物理实体的位置或 `MotionState`；拥有伤害、冷却等规则时钟 |

单 crate 没有编译期隔离，分层由 `tests/layering.rs` 检查：下层的产品代码不得引用上层，客户端各模块之间不得形成环形依赖（单元测试可以跨层搭夹具）。多个模块共用的显示标记（`ActorView`、`PlayerView`、`EnemyBody`）和清单汇总放在 `client/mod.rs`，不要让底层模块反过来引用使用它的模块。

## 状态所有权

| 模块 | 唯一负责的状态 / 行为 |
|---|---|
| `simulation/physics.rs` | Avian 权威三维姿态、角色控制、接触支撑、箱体／地形姿态快照与瞬移清理 |
| `simulation/movement.rs` | `Body` 只读三维脚底快照与平面投影；`MoveIntent` 世界 x/z 移动意图 |
| `simulation/actions.rs` | `ActionIntent`、`Facing`、`MotionState`；动作计时、跳跃额度及接触结果 |
| `simulation/combat/` | 战斗规则，按职责分文件：`skills`（全部技能数值的数据表）、`prepare`（指令消费与准入）、`resolve`（按技能形态结算刀刃扫掠与标记）、`sweep`（唯一的近战判定：刀刃扫掠对物理胶囊）、`hit_tracks`（生成的刀刃轨迹数据）、`projectile`（飞行中的远程弹体：制导、球体扫掠与落点结算）、`hit`（共同命中与防御顺序）、`enemy`（普通敌人）、`energy`（烈阳／月光／暴怒）、`arsenal`（武器耐久）、`combatant`／`health`／`feedback`／`companion`／`availability`／`runtime` |
| `simulation/abilities.rs` | 动作／联动的只读快照类型、联动窗口与逻辑妹妹状态 |
| `simulation/defense.rs`、`resources.rs` | 有序右键、拆招／防御／弹反、耐力与减速的唯一状态 |
| `simulation/boss.rs` | 单愁铃客 Boss 的行动循环、阶段、预警、命中和复活 |
| `simulation/impact.rs` | 狙击命中停顿预算、固定步门控与恢复 |
| `simulation/weapons.rs` | 锤鸟／链斧弓绑定组与哥哥近战普攻数据；近远枚举只作握柄类型 |
| `simulation/geometry.rs` | 仅测试构建：无窗口夹具的平地／立柱描述及其装配，产品构建不包含 |
| `gameplay/mod.rs`、`gameplay/terrain.rs` | `Level`、`Boundary`、`Player`、`EnemyMode`、共享地形几何及实体装配 |
| `client/controls.rs` | `ViewYaw`，设备输入转世界方向；锁定目标时把 yaw 转向目标，使镜头绕目标转；捕获失效时停止意图 |
| `client/bundle.rs` | 运行时资源文件的请求、到齐状态与一次性交付，不认识任何使用方；游戏清单、`Assemble` 调度与 `Assembled` 门槛在 `client/mod.rs` |
| `client/loading.rs`、`web/loader.js` | WASM真实下载流进度、首次可见场景渲染准备／提交门槛，以及主线程最终就绪通知 |
| `client/enemies.rs`、`web/enemies.js` | 进入卡片 / F2 的敌人模式请求及模式显示，调用玩法装配接口 |
| `client/appearance.rs`、`web/appearance.js` | 进入 / 暂停卡片和 F3 的玩家外观选择，只替换显示资产，不改变玩法武器套装 |
| `client/scene.rs` | 玩家显示根和插值后的显示 `Transform` |
| `client/stickman.rs` | 跟随蒙皮角色手腕的武器挂点 |
| `client/avatar.rs` | 两套随包角色 / 近战外观、各自蒙皮与材质缓存、结构动作适配；只写显示骨骼 |
| `client/environment.rs`、`environment/ground_cover.rs` | 玩法同源竹干/地形、共享草叶装饰、雾和灯光 |
| `client/rendering.rs`、`client/texture_mips.rs` | 相机 deferred/SSR/FXAA pass；重复纹理的线性颜色与单位法线 mip |
| `client/source_animation.rs` | 燕云与永劫两套源动画的独立来源、有界解码、稀疏采样和层级求值 |
| `client/forest_material.rs`、`assets/shaders/forest.wesl` | 标准PBR地表分层与干湿响应、共享环境材质和近叶可见/深度裁切，不改变模拟碰撞 |
| `client/skeleton.rs` | 随包 CC0 动画的解码、局部采样、父子骨骼求值 |
| `client/animation.rs`、`animation/poses.rs` | 根据只读模拟快照与明确 actor 的事件选择 / 混合身体姿态 |
| `client/animation/attack_clips.rs` | 每条攻击时间轴（`TrackId`）播放的源片段与时间映射，一招一行；攻击姿态只按模拟的 `AttackClock` 采样 |
| `client/loadout.rs` | 每组武器的表现数据，一组一行：名称、模型文件、挥击姿态、弹道外观、音效、妹妹片段、联动镜头、HUD 编号 |
| `client/camera.rs` | 从相同显示位置与朝向派生跟随镜头 |
| `client/combat.rs` | 只读敌人 / 武器 / 标记 / 特效，按反馈序号消费并回收临时实体 |
| `client/hit_debug.rs` | F4 调试显示：绘制模拟记录的刀刃扫掠、受击胶囊和接触点，不参与判定 |
| `client/companion.rs`、`weapons.rs`、`weapon_models.rs` | 独立白灵淼、伏浪锚／谪星摧山／引冰落／羿之影共享源模型、链斧离手／回收及绳索表现 |
| `client/impact.rs` | 狙击枪口、弹道、空中命中火花和镜头后坐的共享资产及显示时钟 |
| `client/audio.rs`、`web/audio.js` | 实际战斗反馈的一次消费、短音效与暂停同步的音乐循环 |
| `client/hud.rs`、`web/hud.js` | 把玩家与当前选中敌人的真实快照显示为血条、人格 / 武器、技能可用性、冷却和联动提示 |
| `web/input.js` | 浏览器锁定确认、焦点、鼠标位移单位与一次性消费 |
| `web/startup.js` | 在 WASM 加载前记录真实阶段／进度／成功／失败，给晚订阅者重放；最终ready保持 |

## 权威与数据流

- **运动权威只有一个**：Avian 的 Position / Rotation / 速度。`Body` 和 `PhysicsPose` 只保存物理结果的只读快照。
- **客户端只提交意图**：`MoveIntent`、`ActionIntent`、`CombatIntent`、`Facing` 和防御边沿。持续按住与一次性指令分开；一次性指令在实际固定步里消费，显示帧没有固定步时保留，有多个固定步时不重复。
- **显示根独立于物理实体**：客户端使用 `ActorView { owner }` 和 `PlayerView`，读取快照并按 fixed overstep 插值，不能用模型 `Transform` 反写物理实体。角色显示和相机用脚底锚点。
- **几何同源**：`Level.ground/forest_trunks/forest_rocks` 的同一份顶点和三角形同时生成 Collider 和 Mesh；围墙使用共享的 `Boundary` 尺寸。
- **事件单向流出**：`CombatFeedback` 和 `MotionFeedback` 是有界、序号单调的事实历史。特效、冲击和音频各持一个 `client::FeedbackCursor`，共用一条规则：每个事件至多呈现一次，暂停期间到达的事件只消费不呈现。反馈带明确的 actor、归属玩家和产生时的武器／动作身份，表现不能靠距离或最新状态去猜。
- **模拟不带界面文字**：指令结果以 `CombatStatus` 枚举发布，武器名、状态文案和敌人模式名称都在客户端。
- **暂停是整条链一起停**：`SimulationActive` 同时门控物理求解和全部规则；`HitStop` 在步头锁定同一步的执行决定。暂停或命中停顿时显示统一取当前快照，不做插值。
- **动画只写表现**：动画写 `RigPose`，附件同步写肢体与武器 `Transform`，都不写人物显示根、`Body` 或相机；动画片段时长不是技能计时。

## 一帧中的顺序

1. **装配**（只发生一次）：Startup 只通知初始化阶段并请求运行时资源清单。清单全部到齐的那一帧，`First` 中的 `assemble` 运行一次 `Assemble` 调度：源动画库 → 玩法 / 物理场景装配 → 显示根 → 战斗表现 → 相机 → 通知进入画面准备阶段。到齐前输入采样、表现同步和模拟规则都不运行；任一文件失败即报告失败。
2. **采样输入**：Bevy 输入更新后，在 `RunFixedMainLoopSystems::BeforeFixedMainLoop` 处理敌人模式请求、采样控制并提交意图。
3. **固定步**（`FixedUpdate`，60Hz）：命中停顿步头 → 减速计时 → 暴怒消耗与释放 → 动作／防御准备 → motor → Avian 求解 → 当前空间查询、接触与快照 → 判定探针计时 → 玩家动作结算 → 弹体推进与命中 → 普通敌人 → Boss → 可用性 → 耐力恢复 → 妹妹快照 → 瞬移清理。
4. **表现**（`Update`，单链顺序执行）：外观替换 → 人物显示根 → 敌人显示树回收与补齐 → 武器显隐 → 身体姿态 → 妹妹 → 蒙皮骨骼 → 骨骼附件 → 绳索 → 标记与特效 → 冲击 → 音频 → HUD → 相机。相机读取同一个显示根位置，不追加平滑延迟。
5. **渲染与就绪**：Bevy 完成全局变换传播和渲染。首次进入由渲染子应用在真实提交后确认主视图有输出、可见网格和管线已就绪，再由主线程放行 Web 入口；资源到齐或实体装配完成都不能替代。

## 运行时资源

角色、源动画、武器和环境贴图是随包文件，不编入二进制。`client/bundle.rs` 经 Bevy AssetServer 加载：Web 从页面同源路径拉取，原生读取 `assets/`；不读取原游戏目录，也不访问其他来源。每个文件只有一个解码方，字节交付后即释放。游戏在清单全部到齐后装配；验收页（`?view=model`）只等动画库和默认配置，其余在选中时加载。CC0 结构骨架和 shader 仍嵌入。

## 浏览器边界

`web/*.js` 只管理浏览器边界：加载进度、指针锁定与焦点、按键和鼠标的一次性消费、HUD 的 DOM 更新、Web Audio。它们必须同步安装在 Trunk 生成的模块之前，不依赖 DOM 已创建。JS 不持有任何规则时钟：冷却、长按识别、命中都来自 Rust 的快照。wasm-bindgen 绑定集中在 `client/web.rs` 及各模块的 `extern` 块，原生构建不需要 JS。

`?view=model` 在 App 创建前选择独立的验收组合，不安装模拟、地图、AI、HUD、音频和鼠标捕获，但与游戏共用资产、姿态、蒙皮和武器挂接。

## 扩展原则

- 新游戏规则放模拟层，关卡和装配放玩法层，意图和表现放客户端。
- 近战判定是基础设施，不是内容：命中只由烘焙的刀刃轨迹对物理胶囊的扫掠决定。新增或修改一个挥刀招式是改数据（技能表、`client/animation/attack_clips.rs` 的片段映射行）后运行 `make bake-hits`，不写判定代码，也不手填距离、角度或命中时刻。
- 内容是数据：技能的时长、冷却、距离、标记时间、伤害和控制只写在 `combat/skills.rs` 的表里。加一个已有形态（范围、单发、三连发）的技能是加一行数据和一处槽位映射；只有新形态才写新的结算函数。
- 武器组的表现只写在 `client/loadout.rs` 的表里，表现模块读行而不按具体武器组分支。表现需要的时间点（妹妹每发的时刻、联动回切）通过 `ActionKind::{shot_times,return_window,strike_effect}` 从模拟的同一张技能表读取；飞行中的子弹、箭和链斧直接读模拟的 `Projectile`，客户端不保留副本。
- `tests/layering.rs` 会拒绝在这些表之外点名具体的武器组或技能；例外清单和理由写在该测试里（生成的 `hit_tracks.rs` 是其中一张表）。
- 会改变位置的机制仍通过模拟驱动物理组件并发布 `Body` 快照；修改地形以 `Level` 共享几何为源。
- 新的表现消费者使用 `FeedbackCursor`，不要另写一套“暂停不补播”的判断。
- 新的随包资源走 `bundle`，在所属模块里登记路径，不要 `include_bytes!`。
- 新增方向、碰撞或时序行为应有无窗口回归；Web 生命周期使用实际桥接脚本的 Node 测试。构建和纯代码测试不能代替画面与手感验收。
- 调参数值和接口签名只写在对应规范里，本文不重复，避免两处漂移。

## 去哪里找

| 想知道 | 看这里 |
|---|---|
| 移动、跳跃、闪避、蹲伏、滑铲的规则与数值 | [模拟层规范](../.trellis/spec/simulation/index.md) |
| 胶囊、接地、坡面、重力、固定步顺序 | [物理契约](../.trellis/spec/simulation/physics.md) |
| 武器组、普攻连段、技能、联动、防御、耐力、Boss | [战斗契约](../.trellis/spec/simulation/combat-v1.md)、[模拟层规范](../.trellis/spec/simulation/index.md) |
| 地图、小山、围墙、敌人模式与装配 | [玩法层规范](../.trellis/spec/gameplay/index.md) |
| 相机构图、显示链、环境渲染、HUD 布局 | [客户端规范](../.trellis/spec/client/index.md) |
| 按键位掩码、右键边沿、捕获与鼠标校准 | [输入契约](../.trellis/spec/client/input.md) |
| 加载阶段、资源清单、就绪门槛 | [启动规范](../.trellis/spec/client/startup.md) |
| 妹妹、联动镜头、武器表现、事件归属 | [战斗表现契约](../.trellis/spec/client/combat-presentation.md) |
| 角色／武器二进制格式、挂点、手部重定向 | [模型规范](../.trellis/spec/client/model-assets.md) |
| 永劫／燕云源动画、片段映射、音效 cue 与混音 | [动作与音效契约](../.trellis/spec/client/naraka-motion.md) |
| 验收页的操作与契约 | [验收页说明](model-viewer.md)、[验收页契约](../.trellis/spec/client/model-viewer.md) |
| 各资源的来源、转换命令和已知局限 | `assets/*/README.md`，例如 [愁铃客](../assets/choulingke/README.md)、[李火旺](../assets/lihuowang/README.md)、[白灵淼](../assets/bailingmiao/README.md)、[音频](../assets/audio/README.md) |
| 历史方案与调查记录 | [`docs/plans/`](plans/)，例如[愁铃客竹林方案](plans/choulingke-bamboo-arena.md) |
