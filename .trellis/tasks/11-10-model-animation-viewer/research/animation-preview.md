# Research: 动画片段与单角色姿态预览

- Query: 如何复用现有两套动画、蒙皮与手腕挂接，在不推进战斗／物理的情况下确定性播放、暂停、拖动和步进？原片段与最终游戏动作有哪些差别？
- Scope: internal；仅读取现行代码、规范和随包来源清单。
- Date: 2026-11-10
- 已收到主会话范围更新：第一版选“动画片段与单角色姿态预览”，不包含完整技能、第二角色、目标、特效。以下接口为方案建议，不是已实现契约。

## Findings

### 结论与文件

两套解码器均已支持“clip＋绝对时间 → 姿态”，可复用当前 `RigPose → avatar::sync → stickman::sync_attachments`，无需创建 `Body`、`Combatant`、`MotionState` 或运行 `SimulationPlugin`。缺口是稳定目录／源片段时长接口和无历史状态的预览姿态写入接口。不要直接驱动游戏 `animate`，或把它的内部时钟当作 seek API。

| 文件 | 当前事实／用途 |
|---|---|
| `src/client/skeleton.rs:6` | 23 骨骼 CC0 库；纯采样、混合与层级求值 |
| `src/client/source_animation.rs:33` | 986 节点、35 个随包源片段；稀疏 T/R/S 与层级求值 |
| `src/client/animation.rs:44` | `RigPose` 同时承载 CC0 模型空间姿态和可选源姿态；当前游戏状态机有历史缓存 |
| `src/client/animation/poses.rs:83` | 待机／握持／瞄准／蹲伏与手部 IK 的既有纯函数 |
| `src/client/avatar.rs:395` | 结构动作增量、角色自身骨长、独立 palette、源手腕适配 |
| `src/client/stickman.rs:22` | 已适配手腕到武器轴的唯一挂接计算 |
| `assets/animations/README.md` | CC0 作者、版本、导入与许可证；19 clip 的来历 |
| `assets/choulingke/animation_manifest.json` | 35 clip 名称、毫秒时长、VFS、SHA、轨数、省略理由与 Kiva 裁剪 |
| `scripts/import-choulingke-animations.py:118` | 具名／父子匹配、共用动作骨长适配；无法匹配的轨显式省略 |
| `src/client/companion.rs:86`、`src/simulation/abilities.rs:96` | 完整远程演出的快照依赖；第一版不调用这一套 |

### 两个目录的确切清单

**CC0：23 骨骼／19 clip。** 时长为本次只读解析 `stickman.anim` 的 `f32` 元数据，表内四舍五入至毫秒；帧为导入样本数，不是源动画关键帧数。导入器按约 30fps 采样，片尾另保留端点。`assets/animations/README.md` 与 `scripts/import-animations.py:25` 记录选取来源。

| 分组（查看器建议） | clip | 秒 | 样本数 |
|---|---|---:|---:|
| 基础 | `Sword_Idle` | 1.667 | 51 |
| 基础 | `Walk_Loop` | 1.333 | 42 |
| 基础 | `Jog_Fwd_Loop` | 0.933 | 30 |
| 跳跃 | `Jump_Start` | 1.333 | 42 |
| 跳跃 | `Jump_Loop` | 2.500 | 76 |
| 跳跃 | `Jump_Land` | 1.267 | 39 |
| 跳跃 | `NinjaJump_Idle_Loop` | 2.000 | 61 |
| 闪避／反应 | `Roll` | 1.467 | 46 |
| 闪避／反应 | `Hit_Chest` | 0.333 | 12 |
| 闪避／反应 | `Death01` | 2.400 | 74 |
| 剑类 | `Sword_Attack` | 1.533 | 47 |
| 剑类 | `Sword_Regular_A` | 0.433 | 15 |
| 剑类 | `Sword_Regular_A_Rec` | 0.967 | 30 |
| 剑类 | `Sword_Regular_B` | 0.533 | 18 |
| 剑类 | `Sword_Regular_B_Rec` | 1.033 | 32 |
| 剑类 | `Sword_Regular_C` | 2.000 | 61 |
| 剑类 | `Sword_Block` | 1.233 | 39 |
| 剑类 | `Sword_Heavy_Combo` | 4.333 | 132 |
| 剑类 | `Sword_Dash` | 1.567 | 49 |

**源动画：986 节点／35 clip。** 前 30 个为愁铃客注册中的具名文件，后 5 个为其他人体骨架共用动作；不是 35 个完整技能或各角色专属动作。表中轨数为“已映射／源轨数”，可直接用于说明还原范围。VFS 根为 `Char/`，名称分组仅按来源路径／原名称，不推断技能语义。

| 来源组 | clip | 秒 | 轨数 |
|---|---|---:|---:|
| `c_77_boss_yulingke/duoren/` | `dr_Skill_01_atk_loop` | 0.967 | 154/237 |
| 同上 | `dr_Skill_01_atk_out` | 2.634 | 153/237 |
| 同上 | `dr_Skill_01_in` | 1.734 | 154/237 |
| 同上 | `dr_Skill_01_loop` | 2.001 | 153/236 |
| 同上 | `dr_Skill_02` | 5.301 | 154/237 |
| 同上 | `dr_Skill_03` | 2.467 | 152/233 |
| 同上 | `dr_Skill_04` | 3.134 | 152/233 |
| 同上 | `dr_Skill_05_in` | 5.834 | 154/237 |
| 同上 | `dr_Skill_05_loop_out` | 5.834 | 154/237 |
| `c_77_boss_yulingke/ani/` | `skill_ylk_1` | 6.267 | 82/88 |
| 同上 | `skill_ylk_10` | 4.401 | 977/1000 |
| 同上 | `skill_ylk_12` | 3.967 | 77/81 |
| 同上 | `skill_ylk_4` | 9.401 | 978/1000 |
| 同上 | `skill_ylk_5` | 5.234 | 77/85 |
| 同上 | `skill_ylk_5_1` | 4.834 | 77/85 |
| 同上 | `skill_ylk_6` | 9.734 | 977/999 |
| 同上 | `skill_ylk_7` | 9.567 | 977/999 |
| 同上 | `skill_ylk_P2_10` | 9.501 | 115/115 |
| 同上 | `skill_ylk_P2_11` | 5.501 | 115/115 |
| 同上 | `skill_ylk_P2_13` | 5.934 | 113/118 |
| 同上 | `skill_ylk_P2_14` | 5.834 | 64/66 |
| 同上 | `skill_ylk_P2_2` | 5.667 | 977/999 |
| 同上 | `skill_ylk_P2_4` | 7.301 | 77/81 |
| 同上 | `skill_ylk_P2_6` | 5.501 | 977/999 |
| 同上 | `skill_ylk_P2_7` | 5.501 | 58/70 |
| 同上 | `skill_ylk_P2_9` | 6.134 | 58/70 |
| 同上 | `skill_ylk_P2_9_hit` | 5.334 | 78/80 |
| 同上 | `skill_ylk_P2_executebehit` | 17.467 | 82/84 |
| 同上 | `weakhit_b` | 0.834 | 79/86 |
| 同上 | `weakhit_far_b` | 0.834 | 79/86 |
| 共用卸势 `b_01_char_male/11_qiang/8_skill/skill_huajie_01.anim` | `skill_huajie_01__shuangqiangt` | 4.001 | 246/344 |
| 共用起跳 `b_01_char_male/19_shoujia/1_base/` | `b_fight_env_jump_01_start_Lfoot__zuiquan` | 0.734 | 67/67 |
| 共用空中，同上 | `b_fight_env_jump_02_loop__zuiquan` | 1.267 | 67/67 |
| 共用落地，同上 | `b_fight_env_jump_03_to_idle__zuiquan` | 1.301 | 67/67 |
| NPC 闪避 `c_03_npc/1_male/ani_shoujia/` | `skill_shoujia_dodge_f` | 2.934 | 103/105 |

源片段首 30 个不是都已进入当前战斗状态选择器。当前仅选用 7 个：`skill_ylk_1`、`weakhit_b`、共用卸势、跳跃三段、NPC 闪避（`animation.rs:145`）。查看器可展示余下已解码片段，但不能据名称将其挂成完整技能。

### 可直接复用的接口与可见性

| 接口 | 当前签名／限制 | 预览所需最小补充 |
|---|---|---|
| CC0 库 | `AnimationLibrary` 为 `pub(super)`；`bones: Vec<Bone>` 可见，`clips: HashMap<String, Clip>` 私有（`skeleton.rs:15`） | 在客户端内部暴露稳定排序的 clip 描述目录，禁止 UI 直接枚举 HashMap |
| CC0 采样 | `duration(&self, name: &str) -> f32`；`sample(&self, name: &str, seconds: f32, looping: bool) -> Vec<Transform>`（`:101`） | 未知名称会索引 panic；目录选择边界验证，统一清理非有限时间（此 sampler 未处理 NaN） |
| CC0 求值 | `model_pose(&self, local: &[Transform]) -> Vec<Transform>`（`:120`） | 使用现有 +Z→-Z／1.1 基变换，不再转换一次 |
| 源库 | `SourceAnimationLibrary`、`bone`、`clip`、`sample`、`model_pose`、`rest_model` 都在 client 内 `pub(super)`（`source_animation.rs:33,246`） | `clips`、`clip_names` 私有且无时长 getter；新增稳定目录和 duration，不解析运行时 JSON |
| 源采样 | `clip(&str)->Option<usize>`；`sample(clip: usize, seconds:f32, looping:bool, airborne:bool)->Vec<Transform>`（`:250,257`） | 类型化目录 ID／校验整数，元数据建议包括原名、时长、来源组、是否共用适配 |
| 源求值 | `model_pose(&[Transform])->Vec<Mat4>`（`:296`） | 986 节点全层级求值后交给现有 skin 路径 |
| 姿态容器 | `RigPose::new(&AnimationLibrary)`；`model: Vec<Transform>`、`source: Option<SourcePose>`、`companion_hands` 为 `pub(super)`（`animation.rs:39`） | 为预览提供一次性完整设置／替换 pose 的方法，避免 UI 修改零散状态 |
| 姿态助手 | `poses::idle/hold_guard/aim_ranged/crouch_pose` 为 `pub(super)`，但 `mod poses` 是 `animation` 私有子模块（`animation.rs:2`） | 由 `animation` 暴露少量有类型的姿态预设入口，不放开整个实现模块 |

`SourcePose { model: Vec<Mat4>, weight: f32 }` 与已有 `avatar::sync` 已可承载绝对源时间采样结果。`avatar::sync` 查询实体上的 `RigPose`、可选 `ActorView`／`AvatarAppearance`；没有 ActorView 时仍正常求 palette，受击材质默认正常，不要求该实体具有 Body／Health（`avatar.rs:832`）。资源构造本身需要两个动画库，但不意味着要安装模拟插件。

### 建议的确定性采样边界（待实施）

1. 客户端动画模块增加 `PreviewSelection`（参考姿态／带来源命名空间的 clip／少量姿态预设）、`PreviewPose`（CC0 model＋可选 SourcePose＋可选手部目标）与无状态 `sample_preview(libraries, selection, seconds, parameters) -> Result<PreviewPose, ...>`。名称仅为建议。相同选择、时间和参数必须输出相同结果，无帧历史或 ECS 战斗依赖。
2. **CC0 clip**：直接 `library.sample(name,t,looping)` → `library.model_pose`；清空 `source`／`companion_hands`。不要自动调用游戏的 `hold_guard`，否则“原片段”手臂会被覆盖；武器仍使用同一手腕挂接。
3. **源 clip**：以 CC0 骨骼 rest 的 `model_pose` 作为固定结构回退；`source.sample(id,t,looping,false)` → `source.model_pose` → `SourcePose { weight: 1.0 }`。`false` 保留原竖直腰身变化，根 XZ 仍由现有 sampler 剥除。预览主体显示根固定，禁止把片段根位移加到 Body 或显示根。切换时立即得到所选时刻，不叠加游戏的淡入淡出。缺失源轨在 sampler 内回自身 rest；不匹配的 skin／附件仍走现有结构回退，需如实标注。
4. **参考姿态**：CC0 model 取 `bones.rest`，源层可取 `SourceAnimationLibrary::rest_model`、weight=1；交由同一 avatar 重定向／palette 路径。这是项目结构／源参考姿态，不保证为每个部件的蒙皮绑定姿态。AV03 的 `sourceRestModel` 与 inverse bind 应继续独立保留（`model-assets.md:19`）。最终选择一种明确命名的参考含义，不将两套 rest 混称“绑定姿态”。
5. **少量单角色姿态预设**可选既有“近战持握”“远程持握”“蹲伏持握”，由 `poses::idle`／`hold_guard`／`crouch_pose` 纯组合；俯仰参数需要远程瞄准时复用 `aim_ranged`。源层为空，完整蹲姿权重直接作为参数，IK 只运行一次。预设名称说明是 Kiva 姿态，不假装 dedicated 原作开火／拉弓 clip。是否给每个角色开放同一姿态取决于现有适配，不改变战斗里的角色职责。
6. 预览输出一次性写入 RigPose；新入口应保证无残留 `source`、`companion_hands`、过渡缓存。预览根不挂 `ActorView`；不注册游戏 `animate`／`companion::sync`，只运行预览采样、`avatar::sync`、`stickman::sync_attachments`。已有 `RigPose` 类型可继续服务渲染，游戏状态机仍封装在模块内部。

**不要复制 retarget／IK／蒙皮到 viewer。** `avatar::source_driver_pose` 保留李火旺／白灵淼自己的锚点与骨长（`:584`）；愁铃客按源骨名驱动，其他角色只传递源动作增量（`:613,627`）。`attachment_pose` 使用适配后的右腕位置／旋转增量（`:658`），再调用 `stickman::attachment_transform` 统一近战 +Y／远程 -Z 轴（`stickman.rs:22`）。这些函数足以保证同一采样同时驱动皮肤与武器。

### 播放控制契约建议

- 单独的查看器播放状态：`selection / seconds / playing / speed / looping`；只使用展示时间增量，不依赖 `SimulationActive`、pointer lock、FixedUpdate、HitStop 或技能 cooldown。
- 命令先应用，再采样；暂停仍接受切 clip、seek、步进、换模型／武器，并立即刷新 skin 与附件。暂停仅停止 seconds 自动增长，自由镜头继续更新。
- seek 输入必须有限，夹到 `[0,duration]`；不循环播放到末尾后停住；循环采用现有 `rem_euclid(duration)`。必须明确“seek 到末尾”能看到尾帧，避免开启循环后 duration 被立即映射到 0；例如 seek 采样用 clamp，后续播放增量才 wrap。
- 帧步进用固定“预览步长”（例如 1/30s 或 1/60s，实施时统一），不要称源关键帧步进。源库保留非均匀毫秒稀疏键，没有统一原始 fps。
- 切 clip 重置 seconds=0 并即时求值；换外观／武器保留当前 seconds 与暂停状态。loop 时可能有首尾不连续，不能默认所有 clip 都是循环动作；目录可区分循环名字与其他片段，但不推断其原作播放状态机。
- 所有绝对时间采样无需回放历史，seek 后不要触发战斗事件、音效、标记或伤害。统一校验 speed、duration 和时间边界；目录用稳定名字／来源命名空间，源内部整数索引只保留在库适配层。

### 为什么不能用现有游戏状态机实现拖动

`RigPose::advance`、`advance_source`、`observe`、`Inputs`、`PoseState`、`Gesture` 均为私有，且当前 advance 不是绝对时间函数：

- `clock/gait_time/air_time/jumps_used/from/transition/crouch_weight/playback/source_from/source_blend` 依赖先前帧；换状态会重置时钟（`animation.rs:278,338`）。
- `advance_source(dt<=0)` 直接返回，因此暂停时 seek 或切源 clip 不会更新（`:135`）；仅公开 clock 也无法解决过渡／源权重缓存。
- 游戏先按 alive／stunned／受击／闪避／滑铲／防御／瞄准／动作／空中等选优先级，再做上下身替换、空中骨盆钉住、IK 和过渡（`:302,413,441`）。蹲姿缓存必须保留 IK 前姿态（`:459`）。
- `animate` 读取真实 Body 位移算步频、MotionState／Health／Facing／Combatant／Boss／DefenseState，加按 actor 和 sequence 去重的 CombatFeedback；`SimulationActive=false` 整个返回，HitStop 则消费反馈但 dt=0（`:535,546,569,585,606,626`）。

第一版纯采样接口与游戏状态机可共享基础姿态助手；无需把这些游戏输入公共化，也无需伪造物理角色。

### 原片段、当前单体动作与完整技能的边界

| 内容 | 当前实现事实 | 查看页含义／第一版范围 |
|---|---|---|
| CC0 19 clip | 已导入的非 root-motion 版本；23 骨结构，已有正面转换 | 片段预览；不是专属锤、斧、狙击、火炮动画 |
| 源 35 clip | 稀疏原毫秒键；已做骨架坐标转换、共用骨长适配／轨省略；runtime 去平面根位移、初始 yaw 对齐 | 可完整播放“随包源片段”，宜说明“原始片段／原地适配”，不能称未加工原件／完整技能 |
| 当前近战／重击 | `skill_ylk_1` 0.9–1.8s 压到 0.42／0.6s；预备 0.3–0.9s 压到 0.4s（`animation.rs:145`） | Kiva crop／速度／混合；不等于 source 整段；第一版不演练完整技能 |
| 防御／受击／闪避 | 卸势 0.1–0.65s；受击取 weakhit_b 前 0.3s；闪避取 0.2–1.0s并读模拟方向（`:163`） | 单体预设如包含它们要复用统一配方；不能把原名字直接等同战斗动作 |
| 移动／双跳／蹲滑／死亡 | CC0 循环／分段、上下身混合、蹲伏 IK、骨盆钉住与死亡时长适配（`:356`） | 可采样 clip 或静态姿态；实际位移／起落／滑动轨迹依赖物理，第一版不演练 |
| 妹妹 Q／联动 | `CompanionState` 的位置／朝向／源／瞄准／elapsed／shot_count 驱动独立 rig；不是一条原开火／拉弓 clip | 不属于第一版单角色片段与预设 |
| 锤击浮空接三枪 | 目标实际浮空、原目标、标记0.30/0.42/0.54、独立妹妹、反馈、HitStop、联动镜头 | 完整演练需要真实模拟或录制后的多角色快照重放；不能靠 source clip sampler 重现 |
| 链斧 Q／拉近三箭 | 离手、绳索、物理拉近／制动／障碍取消、标记0.12/0.28/0.44、独立妹妹与目标 | 同上；不能复制一套时间线到 viewer 假称游戏最终行为 |
| Boss 技能 | 原技能槽→clip／命中帧未恢复，`Boss::clip_name()` 返回 None（`boss.rs:136`） | 原片段可浏览；不以名称补造原 Boss 完整技能 |

补充确切接口：`RigPose::sample_companion(&AnimationLibrary, &CompanionState, pitch:f32, grips:[Vec3;2])` 已按绝对 elapsed＋shot_count 求后坐（`animation.rs:98`），但完整 `companion::sync` 还插值 previous_position、计算局部瞄准方向／源和枪握点，读取逻辑 performer／owner 身份（`companion.rs:86`）。它不应被改造成第一版主角片段 sampler。后续若另做完整动作演练，应复用实际模拟 reset/replay 或保存的展示快照／事件，而不是第二份伤害与标记逻辑。

### 建议验收接口（本轮未执行测试）

| 边界 | 有意义的验证 |
|---|---|
| 目录 | 19＋35 个 stable ID 无重复，时长与二进制一致；非法 ID／NaN 时间／非法 speed 有明确处理，不能索引 panic |
| 绝对采样 | 对各库每个 clip 比较直接 seek(t)、先 seek(u)再seek(t)、倒拖／重复采样，输出相同；loop／clamp／末帧可见 |
| 播放控制 | pause 任意展示帧不改秒数／palette／挂点；暂停 seek、步进、切 clip 可立即更新；倍速与分帧差异只改变 seconds；loop 连续跨多圈无累加漂移 |
| 双来源切换 | 源→CC0→姿态预设→源不残留 source weight、手部目标或 crouch 偏移；同 t 仍与首次采样一致 |
| 根／骨长 | 源 XZ 根固定、竖直腰身保留；展示根固定，无 Body 或物理插件也能显示。不同角色保留自身骨长、源 motion 与 CC0均产生真实 skin 顶点变化 |
| 蒙皮／手腕 | 对愁铃客／李火旺／白灵淼、两来源与暂停 seek，核实际 palette、`attachment_pose` 和武器握柄同帧一致；近战 +Y／远程 -Z 正确，不直接读未适配 source 手腕 |
| 参考姿态 | 维持 sourceRestModel 与 IBM 原值，参考姿态不强制 identity skin；AV03 current rest≠bind 的回归保留 |
| 架构 | viewer 只装展示组件即可采样，无 CombatFeedback、SimulationPlugin、Body；无反馈／伤害／音效副作用，正常游戏既有采样路径回归保留 |

已有可延伸回归：`skeleton.rs:181,221`（有限性、根与 wrap/clamp）、`source_animation.rs:315,337`（35片段／根／稀疏rest）、`avatar.rs:1163`（源骨与手腕）、`:1371`（Naraka 自身骨长与实际顶点）、`:1641`（女性手握与真实联动）、`stickman.rs:128`（武器轴）。第一版不必新写战斗模拟测试；使用已有完整游戏回归防止共享渲染接口漂移。

### Related specs / External references

- `.trellis/spec/client/index.md`：动画只写 RigPose、根运动归模拟、暂停／IK 前缓存、更新顺序与材质复用。
- `.trellis/spec/client/model-assets.md:18`：自身结构骨长、独立 rest／IBM、女性握持与武器轴。
- `.trellis/spec/client/combat-presentation.md:21`、`.trellis/spec/simulation/combat-v1.md:31`：妹妹归属、真实事件标记／目标／镜头；用于划定第一版以外的完整技能边界。
- `docs/architecture.md`：client → gameplay → simulation 单向依赖。
- 外部来源仅记录本地 provenance，未联网核对：Quaternius Universal Animation Library Standard v3.0／Library 2 Standard v2.2、CC0 1.0，链接见 `assets/animations/README.md`。原作 35 clip 的来源／哈希以 `animation_manifest.json` 为准，不套用 CC0 许可。

## Caveats / Not Found

- 未写产品代码、未运行构建／测试／服务／浏览器。所有新类型／方法和预设均为待评审的实现建议。
- 35 个“源动画”是随包已转换／部分映射结果，不是完整原作控制器、技能、布料／发丝物理。共用的 5 个也不是愁铃客专属原版。
- 现有 source.sample 不提供“保留全 root motion／原初始yaw”的模式；第一版复用它意味着原地归一化。CC0 数据导入时已去根位移，运行时无法恢复原 root-motion 轨迹。
- 原注册129项中的90项共享骨名字典未恢复、9路径未定位；不能在列表中加占位“可播放”片段。独立附件父子不匹配轨已省略，完整原 cloth／source weapon 演出未恢复。
- `assets/choulingke/README.md`、旧计划和 manifest 的文字可能是历史摘要；具体片段裁剪以当前 `animation.rs` 为准，例如当前 Hit/HeavyHit 的源时间推进按0.3s归一，Gesture 的寿命又分别为0.24/0.30s，不能只把 manifest 的简述直接复制成执行配方。
- 参考姿态不是“逆绑定抵消全网格”的承诺；特别 AV03 不能将 imported current rest 叫作蒙皮绑定姿态。
- 查看器若同时对某角色装远程武器与源近战 clip，只能表示挂接检查；不是证明该组合有原作专属动作／双手准确握持。预设适配范围需要在方案写清。
