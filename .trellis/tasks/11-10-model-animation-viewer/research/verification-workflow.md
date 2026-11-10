# Research: 用同一游戏动画计算检查单体动作

- Query: 第一版保持单角色、无完整战斗的范围，如何检查实际游戏模型、武器位置和裁剪／混合／IK／过渡，而不退化为原片段图库？比较共享纯采样配方与共享状态机重放。
- Scope: internal；只读现行客户端代码、模拟快照与规范。本文件更新主检查流程的建议；`animation-preview.md` 的片段清单和源片段比较结论仍有效。
- Date: 2026-11-10

## Findings

### 建议：主流程复用状态机，原片段作为比较

采用 **B：从游戏 `animate` 提取同一个单帧计算入口，查看器从初始 RigPose 以固定 1/60s 输入帧重放到指定帧**。查看器只生成有限的单体检查场景（地面状态、速度输入、一次动作／反应通知），不创建物理角色，不运行伤害、目标、技能冷却或第二 actor。源片段页继续直接绝对时间采样，承担来源对照。

这比重新实现“绝对时间→最终游戏姿态”更小、更忠实：现有算法的 `from/source_from`、淡入淡出、蹲起 IK 权重、受击中断、收招回待机都保留原样。查看器不得再写一份 clip 名称、裁剪区间、时长缩放或 IK 公式。此前 `animation-preview.md` 建议的纯 `sample_preview` 适合原片段／结构参考；**不应作为“游戏单体动作检查”的唯一实现**。

| 选择 | 能覆盖什么 | 代价／风险 | 本次判断 |
|---|---|---|---|
| A1：抽共享 action recipe／clip-time／postprocess，游戏仍保留状态机 | 裁剪与部分 IK 公式统一 | 独立绝对 sampler 仍缺过渡来源、源层权重、事件中断与历史；同一 recipe 不等于同一最终姿态 | 不能单独完成本次检查目的 |
| A2：把状态机改成可由绝对时间描述的纯函数，游戏也迁移 | 可能直接 O(1) seek | 必须显式建模之前的姿态／事件历史、动作优先级和多层过渡，属于较大动画重构；易改变当前游戏 | 第一版不需要 |
| B：同一 `RigPose` 状态计算＋固定步重放 | 当前 CC0／源裁剪、状态优先级、过渡、IK、源层淡入淡出、回待机 | seek 为有界重放；固定 60Hz 不代表任意真实游戏渲染帧的逐帧复刻 | 推荐最小共享边界 |

### 已确认文件与代码证据

| 文件 | 事实 |
|---|---|
| `src/client/animation.rs:44` | RigPose 同时拥有最终姿态和全部播放／过渡历史 |
| `src/client/animation.rs:135` | `advance_source` 使用已选状态／时钟，执行真实源片段裁剪、混合和淡入淡出；dt<=0直接返回 |
| `src/client/animation.rs:241` | `observe` 唯一将 CombatEffect／武器组／敌人刀身份映射成 Gesture，Respawn 重置整个 rig |
| `src/client/animation.rs:278` | `advance` 统一动作寿命、状态优先级、步频、上下身替换、过渡、空中骨盆约束及蹲姿 IK |
| `src/client/animation.rs:546` | `animate` 是游戏 ECS 适配层：读组件／反馈、去重、构造 Inputs，再调用两个 advance |
| `src/client/animation.rs:714` | 现有 `idle_input` 测试夹具已证明无物理组件也可调用相同私有计算 |
| `src/client/animation.rs:735` | 现有测试直接用 `observe → advance → advance_source`，验证源片段选择、状态优先级、暂停及重置 |
| `src/client/companion.rs:117` | 真正远程路径使用源位置、局部瞄准、实际武器握点，而不是主角通用 mode 开关 |
| `src/client/avatar.rs:832` | 只有 RigPose／外观的显示实体也能驱动实际 palette；ActorView／Health 均非必需 |
| `src/client/avatar.rs:658`、`src/client/stickman.rs:66` | 已有适配手腕与武器轴必须同帧继续复用 |

### 最小共享单帧入口

以下命名为待实施建议，不是当前已有 API。

1. `animation.rs` 内保留私有 `Inputs`、`PoseState`、`Gesture` 和现有算法；新增私有 `AnimationFrame`／`drive_frame`。`animation/preview.rs` 作为其子模块负责检查场景和重放，对 viewer 仅导出高层 preset／seek 接口。这样无需把每个内部字段暴露给整个 client。
2. 游戏 `animate` 继续负责 `ActorView.owner` 查询、`SimulationActive` 门控、Body实际位移→speed、Facing→局部方向、MotionState／DefenseState／Boss→Inputs，以及筛出归当前 actor 的反馈。共享 frame 包含：Inputs、当前 WeaponPair、是否具有 Combatant（决定原刀 Slash 身份）、可选 active action 的 id/kind、actor已筛选反馈的 sequence/action_id/effect、latest sequence、dt。
3. 将 `animation.rs:569–599` 的动作通知／事件处理与 `:631–634` 的 advance 调用集中到共享入口；顺序严格保持：**新 action_id→observe；按原顺序消费反馈并去重；记录游标；advance；advance_source**。ActionKind→CombatEffect 的映射也只保留一处。查看器生成“近战动作开始”等语义 cue，由同一入口做映射，不自行调用另一套 gesture 配方。
4. 第一版不需要真实 ECS `CombatEvent.actor`／`Combatant`：预览检查场景产生同形的只读单体 frame／cue。游戏适配层仍验证 actor 与 sequence；预览的有限通知用稳定 id／序号在规定帧出现一次。不伪造 World 里的战斗反馈，不产生声音／粒子／伤害。
5. 游戏现有行为必须原样共享，包括当前 `animate` 的主角 `mode=Melee`、`sniper=None`（`:570,623`），以及按当前 set 处理 observe 的方式（`:569,596`）。不能在 viewer 单独改善一个姿态或历史事件武器选择，然后宣称是在检查游戏结果；发现差异另开修复，先改共享路径并回归。

共享入口之后，两个入口都仍产出同一个 `RigPose`，并使用同一个 `avatar::sync → stickman::sync_attachments`；皮肤、角色自身骨长、源 wrist delta 与实际武器握点路径不分叉。

### 从共享计算读出的 PoseTrace

主会话已确认需要显示当前实际clip／时间／适配状态。建议 `drive_frame` 同时提交一个小型只读 `PoseTrace`，与RigPose是同一帧的输出；JS仅展示，不再按preset名字换算源时间。

- 在**已有采样调用处**记录来源、clip稳定名、请求时间／实际夹取或循环后的时间、looping，以及角色状态／clock／gait_time。`advance_source` 的选择值在`:201–204`已经算出，`poses::gesture` 的source_time在`poses.rs:224–243`已经算出，直接将这些实际值带出。
- 时间归一化应由采样器的同一个helper或`sample_with_info`返回，避免Rust trace和sampler各写一次clamp/wrap。追踪不能再做一遍采样或改变原时间计算顺序。
- 记录实际执行的适配标志：source crop／source fade或crossfade、source weight、transition weight、持握／瞄准IK、蹲姿IK及weight、上下身替换、空中骨盆约束、根XZ剥除／空中根Y约束。标志在执行该分支时写入，不能在JS依据状态名推断。
- 混合中可能同时有CC0底层、源当前片段和淡出的旧源；trace用有角色标记的有限列表（base／gesture／lower-body／source）和当前权重表达，不能显示一个“当前clip”冒充最终完整来源。`source_from`为过去混合快照时标注“来自上一姿态的过渡”，不伪造它是单一clip。
- `advance_source(dt=0)`保留原source信息，源淡出期保留旧源标识至weight=0；原片段页可以另显示直接采样信息。Trace与pose在seek完成后一起提交，暂停后不得只更新时间文字。
- 轻量实现可让原纯姿态助手返回或填写固定容量的采样说明，再由共享driver收集；无需引入完整动画图、逐骨日志或无界历史。Trace不参与动作决策，只解释已经算出的结果。

### 所需历史字段与重置

`RigPose::new`（`:71`）是确定的检查场景起点，不能只重置 clock：

- 23骨层：`local/from/state/clock/gait_time/transition/air_time/was_airborne/jumps_used/crouch_weight/playback/restart`。
- 源层：`source/source_local/source_from/source_clip/source_blend`；源淡出可能跨动作结束继续存在。
- 事件层：`event_sequence/action_id`，否则 seek 后动作可能被错误去重或再触发。
- 展示层：`model/companion_hands`；切去远程检查或源片段再返回时不能残留手部约束。

缓存／检查点应复制整个可克隆 driver 状态，或者显式封装成 driver-owned snapshot；不能仅缓存最终骨骼。`Playback`、`SourcePose`、RigPose 目前没有 Clone 派生，新增 Clone 或内部保存结构均是待实施工作。皮肤资产／网格／材质句柄不进逐帧检查点。

### 无物理组件即可提供的检查场景

各场景声明“地面脚底固定”等输入条件。这些条件是受控检查参数，不是第二套运动规则。时间线可包含短待机准备和收招段，以观察切换；这两段长度是查看器场景配置，实际 gesture 时长／裁剪／恢复仍由共享代码决定。

| 单体检查项目 | 给共享驱动的条件／通知 | 无物理能否忠实计算当前骨骼 | 必须说明的限制 |
|---|---|---|---|
| 站立／待机 | alive=true；grounded；speed=0；无通知 | 是，原 `poses::idle`＋持握 IK | 不是原模型绑定姿态 |
| 行走／疾跑 | grounded；显式非零实际速度输入；sprinting=false/true | 是，完全复用 `gait_time` 公式与 clip | 原地检查步态；不验证位移、撞墙后速度、脚底与地形。速度作为场景输入展示，不能复制一份游戏速度规则 |
| 站→蹲→站／蹲行 | grounded；切 crouched；蹲行再给 speed | 是，复用0.12s权重、IK前缓存和脚部IK | 不验证胶囊、低顶或滑动摩擦；不能直接把 crouch_weight 拉到1绕过过渡 |
| 当前普攻 | 玩家路径的 ActionKind::Melee 新id，pair按当前近战；grounded | 是，原 observe 决定 Hammer/Axe，原 source 与CC0混合保留 | 是单体视觉动作，非范围／命中测试；敌人刀 Slash 身份应明确区分 |
| 锤击／斧扫动作 | HammerSlam／ChainSweep 新id，grounded | 是，原手势／源裁剪／恢复 | 不承诺击飞、伤害、减速、离手或命中时序 |
| 防御保持 | `parrying=true`（当前实际含义为 DefenseState::is_active）再释放 | 是，完全复用 PoseState::Parry 与源卸势裁剪 | 当前 Waiting/Deflect/Guard/Parry 都经同一bool；不能画四套不存在的不同骨骼动作 |
| 成功防御／卸势反馈 | 一次 Deflect／Block／Parry effect；明确当时防御bool是否保持 | 是，三者在当前 observe 都映射 Gesture::Guard | 应称“当前防御反馈动作”，不假称已验证弹反窗口、正反方向或免伤；防御保持时更高优先级仍由原算法决定 |
| 被卸势（Boss反应） | `boss_deflected=Some(elapsed,duration)`；地面；无真实Boss组件 | 是，PoseState::Deflected 会中断旧挥击 | duration是明确检查输入；不代表恢复未知Boss技能槽/原作完整卸势配置 |
| 普通／重受击 | 一次 Hit／SniperImpact；grounded；重受击场景可包含既定dt=0停顿帧 | 是，复用真实短寿命、腿部保持、源反应与过渡 | 不验证真实HitStop触发、伤害、浮空落点。若检查纯反应可不模拟停顿，但名称需明确 |
| 倒地 | alive从true变false，随后保持 | 是，真实Down优先级、Death01缩放和保持 | Defeat effect本身被observe忽略；必须驱动alive=false。不运行复活计时/物理复位 |
| 复活后的动作重置 | 一次 Respawn cue＋alive=true | 是，同一 `RigPose::new` 重置 | 不承诺出生点或无敌期 |
| 远程持握 | 使用下一节的真实远程 evaluator，elapsed可推进但shot_count=0 | 是，限定真实支持的远程角色／挂接路径 | 不使用主角mode=Ranged作为当前游戏远程演出的替身 |

跳跃／二段跳／落地、定向闪避／滑铲在技术上也可用明确 `airborne/jumps_used/dodge_elapsed/dodge_yaw/sliding/slide_yaw` 时间线检查骨骼（`:483`），无需Body。但必须另外定义起落／朝向输入场景，且不能验证实际高度、落地、位移或碰撞。第一版不必为本次目标自动扩展这些场景；源码片段仍可对照。真实目标、第二 actor、完整技能、拉近和特效均保持范围外。

### 远程持握的真实路径

当前游戏哥哥 `animate` 永远 `mode=Melee`，`sniper=None`；测试中可以调用的通用 `Inputs.mode=Ranged`／SniperPose 并不证明生产路径实际这样使用（`animation.rs:570,623,885`）。真正妹妹由 `companion::sync` 调用 `RigPose::sample_companion`（`:98`）：

- 根据 `CompanionState` 取得脚底位置、facing、source_position、aim_position、elapsed、shot_count；从世界方向转换到角色局部方向（`companion.rs:117–133`）。
- 由实际 `WeaponModel::grips` 与局部武器frame求双手目标（`:134`）；`sample_companion` 保留既有下身和后坐，`avatar` 在自身女性骨长上求IK。
- 武器根直接使用局部 source／rotation（`:176`），背部附件走 `AvatarAssets::body_mount`，**不等同于近战统一绑右手腕**。

若第一版包含“白灵淼远程持握”，最小抽取是从 `companion::sync` 中共享 `evaluate_companion_frame(state, root_transform, weapon_grips)`，返回局部武器frame／pitch／grips，游戏和 viewer 同用；继续调用相同 `sample_companion`、palette 与 body_mount。输入可为一个受控的 source/aim方向快照，不需要逻辑哥哥、目标实体、伤害或第二角色。`shot_count=0` 即可检查持握，不增加Q／联动演练。

源位置相对脚底的正常平地配置可由既有游戏快照作为 fixture 留存（模拟目前 `combat.rs:525–547` 求伴随落脚后以Y1.2和前方0.55求射击源）；不要在 viewer 重新写一份地形选点／站位算法。只共享显示frame求值，不抽取／改变模拟规则。其他角色拿远程武器的跨角色适配可展示为额外姿态检查，不能标成其当前游戏实际动作。

### 确定性播放、暂停和 seek

- **状态基准**：每个检查场景有固定初始状态／输入时间线／版本和有限时长。frame0是 `RigPose::new` 的已显示初始结果。第n帧先提交该帧通知再以 `h=1/60` 调共享入口。动作开始、结束和中断必须落在明确帧，不依赖UI渲染速度。
- **时间网格**：游戏单体检查 seek 采用整数帧 `n`（界面可显示n/60秒）；拖动统一量化到该网格，避免“seek时补一个零头dt”使恢复播放偏离canonical状态。原片段对照仍可连续时间采样。步进一帧就是±1；向后从起点或最近完整检查点重放。
- **倍速**：只改变墙钟时间积累出多少个canonical帧；每次算法dt仍为1/60，不把倍速乘进advance。高倍速跳过展示帧也必须执行所有中间计算／通知；不可丢失事件。
- **暂停**：保留已求值RigPose，不重复调用advance(dt=0)来“刷新”。现有advance即使dt=0仍可响应通知／状态改变，而advance_source会返回，两者行为不同。暂停时的seek是独立重建到所选帧，再统一刷新palette／附件；相机继续更新。
- **切换／循环**：切场景或循环回零必须完整重置driver与通知游标；循环重播完整检查场景，不能把上轮源淡出／蹲姿缓存带到下轮。换外观／武器保留目标frame，但更换会影响语义输入的武器组时从头重放到该frame；不重新解释已有动作中途的pair。
- **显示提交**：seek重放可以在临时driver里执行，全部完成才替换当前显示RigPose和时间确认；不能让skin是t、武器是t-1。多次拖动只提交最新请求版本，旧重放可取消。

**一致性表述必须准确**：生产 `animate` 使用 `Time.delta_secs()`（`:626`），不是固定60Hz动画调度。因此B保证“同一游戏动画算法，在声明的60Hz单体输入场景下的可重放结果”；不保证任意真实游玩过程、不同帧率和事件到达顺序的逐帧相同。这个边界不会使clip/IK另写一套，但验收要包括常见delta序列与源淡出／状态转换差异，不能口头宣称时间步完全无关。

### 有界重放与性能验证

当前没有对这套重放的WASM性能测量，本轮也不运行测试／浏览器。不能承诺“seek必定不卡”。建议先用以下可测试限制设计：

1. 检查场景设置有限 `N_max`，实现初稿可评估6s＝360动画step这一候选上限；最后采用值由真实WASM测量决定，不是本轮已承诺性能。最长17.467s源clip走stateless sampler，无需1048帧游戏状态机回放。
2. 每次seek只从reset或最近有效检查点前向重放，永不倒着积分状态。记录实际step数，断言不超过目标与上限；缓存键含场景参数、pair／角色适配模式、动画资产版本和步长。
3. 首版可先无检查点，测量真实WASM下最坏场景（源层开关、受击、蹲起、不同模型）的seek耗时和分配量；验收设明确交互预算，例如拖动确认P95≤50ms，数值需主方案确认。这是待测指标，不是已验证结论。
4. 若同步重放超过预算，先改成有步数／毫秒预算的分帧重放、拖动合并、旧请求取消；保持当前完整pose直到新结果就绪。再按需要增加有限检查点（例如每30/60帧），复制全部driver状态；检查点数量／内存上限要实测。每step仍可能超过预算，不能只凭“限制步数”宣称毫秒上界。
5. 长时间持续播放使用canonical帧单调计数，在场景尾重置；不建立无限历史。检查点与输入trace不缓存mesh／material／纹理，也不每步运行avatar skin更新，后者只在显示提交时调用。

如果固定60Hz检查仍无法满足对具体游戏异常的重现，需要的是**记录真实单体适配器输入＋每帧dt＋通知顺序，再用同一drive_frame重放**。它保留单角色和无物理运行边界，但需要单独的trace版本／来源契约；不是自动加入第一版。只有在共享路径一致性／有界重放验收失败且无法修复时，方案才应显式缩减或另评审；不能悄悄切回原片段并继续称“游戏动作检查”。

### 可验证的接口验收（仅建议，本轮未执行）

| 断言 | 验证方法 |
|---|---|
| 共享算法而非相似效果 | 小型真实game animate夹具产生60Hz输入轨迹；用无Body的viewer重放相同frame／dt／通知，每帧比较CC0 model、source model/weight和driver状态，覆盖攻击开始→恢复、蹲起、防御、受击中断、倒地。保留既有真实模拟回归，不能只测试两个调用同一函数的薄wrapper |
| 没有复制动作公式 | 查看器仅配置状态／通知／时间段，clip名／裁剪／IK只在原共享算法；改一个游戏裁剪后同一检查场景输出随之改变 |
| 触发与去重不变 | active_action与同action反馈同帧时只启动一次；重复sequence、非当前actor、Respawn与新动作排序按现有语义；预览seek不重复外部反馈 |
| 任意seek确定 | 顺序播放到n、从0 seek(n)、n→m→n、检查点seek(n)输出一致；暂停再seek、切场景再返回、循环第k轮同n一致 |
| 倍速与UI帧率独立 | 不同墙钟分帧／0.25×、1×、2×达到同canonical帧时输出相同；若跟不上则延迟提交而非跳过中间计算 |
| Trace与最终计算一致 | 在实际sample调用点验证记录的clip／归一化时间、执行分支与输出；source淡入／淡出、同clip重启、暂停seek及源→CC0时元数据随同帧提交，不由JS复制配方计算 |
| 实际skin与武器一致 | 使用三套实际角色和受支持武器组合，比较同帧palette变换／真实蒙皮顶点／适配手腕／武器根，特别攻击起止源层淡出、蹲起、暂停换装；不能只比较抽象RigPose手坐标 |
| 远程检查与游戏路径相同 | 相同CompanionState和真实WeaponModel握点输入，在游戏sync与抽出evaluator下比较双手／脚／枪口0与-Z／背部附件；单figure不需第二actor |
| 有界与响应 | 统计最坏step数、检查点数／字节、真实WASM seek P95；快速拖动取消旧请求，过期结果不覆盖新结果；模型／武器多次更换资源数量稳定 |

现有可延伸证据：`animation.rs:735`（直接共享计算）、`:985`（真实运动）、`:1086`（动作回收）、`:1248`（暂停）、`:1283`（倒地／重置）、`:1368`（蹲起）、`avatar.rs:1825`（实际蒙皮全过程）。这些测试已存在；本轮未执行或修改。

### Related specs / External references

- `.trellis/spec/client/index.md:58–64,120`：动画权威边界、IK前缓存、源片段与角色手腕共用适配。
- `.trellis/spec/client/model-assets.md:18–21,50–58`：自身骨长、当前源rest与IBM独立、实际female palette与枪口验证。
- `.trellis/spec/client/combat-presentation.md:21–26,42`：真实远程角色／事件归属、武器来源和射击源frame。
- `.trellis/spec/simulation/combat-v1.md`、`docs/architecture.md`：动作／物理权威不能移到viewer。
- 无外部库／新文档依赖；不需要AnimationGraph或新重定向框架，未联网查询。

## Caveats / Not Found

- 仅写本研究文件；没有产品实现、测试／构建／浏览器执行，也没有新增检查场景或性能证据。
- 本文推荐共享计算重放，替代先前研究中“仅纯绝对片段采样足以担当主验证”的不完整建议；原片段清单仍保留为比较资料。
- B验证受控输入下的当前游戏单体视觉，不验证战斗规则、真实输入边界、目标／伤害／完整技能／物理接触；这些继续由现有游戏回归与实际游玩验收负责。
- “成功卸势反馈”“Boss被卸势反应”是两条不同输入，不能混名；现有防御四阶段在动画输入层合并，查看器应展示当前事实，不补造美术区分。
- 当前白灵淼真实远程使用不同的握点／武器frame流程；统一改为腕挂或主角Ranged姿态会制造查看器与游戏不一致。
- 参考姿态仍应如此命名，AV03 current source rest不能改写为绑定姿态；任何viewer-only的视觉纠正都违反本工具核对游戏结果的目的。
