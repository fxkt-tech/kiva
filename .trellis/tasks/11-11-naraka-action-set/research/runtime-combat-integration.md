# Research: 永劫动作、三段普攻与音效的运行时接缝

- Query: 如何把真实永劫动作接入三段左键、具体右键阶段和移动状态，并保护已有蒙皮、手指及音频契约？
- Scope: internal；原片段和媒体身份由 animation-assets / audio-assets 研究负责，本文件不推测原资源 ID。
- Date: 2026-11-11

## Findings

### 1. 现有权威规则与展示规则不对等

| 文件 | 当前事实与设计含义 |
|---|---|
| `src/simulation/abilities.rs:5` | `ActionKind::Melee` 只有一种；`ActionSnapshot` 没有普攻阶段，包含 action_id、pair、elapsed、duration 和目标快照。不能用当前角色某个可变“下一段”字段给旧事件补阶段。 |
| `src/simulation/combat.rs:255` | `CombatIntent` 用按位锁存，`request_attack` 置 ATTACK。同一固定步前多次左键自然合并。 |
| `src/simulation/combat.rs:285` | `Combatant` 当前只存单一 `action`、普攻冷却和 E/Q/Tab 联动窗口；这里的 `ComboWindow` 专属两套技能联动，不能拿来存普攻连段。 |
| `src/simulation/combat.rs:627` | `availability_for` 的 free 要求 !busy/!defense/!dodge/!slide；空中普攻当前合法。 |
| `src/simulation/combat.rs:694` | 每固定步取走 commands，攻击忙碌时的新普攻现在一律拒绝而不是排队。 |
| `src/simulation/combat.rs:744` | prepare 推进已有 action 一次，最终标记留待物理之后的 advance；不能在 prepare 过早 retire，漏掉末帧命中。 |
| `src/simulation/combat.rs:835` | 接受优先级 Tab→E→Q→LMB；LMB 不要求锁定，`target=None`。 |
| `src/simulation/combat.rs:890` | 接受时建立不可变身份与数值时序，起手把 movement actions 丢弃。 |
| `src/simulation/combat.rs:1159` | 单个 marker(0, time) 发 Melee 并检查各候选敌人的范围、正面、视线，再逐敌结算；不是动画事件回调扣血。 |
| `src/simulation/combat.rs:1481` | 物理后 advance 完成最后标记后 retire；每段命中去重以 ActionRuntime.marker 位图为准，反馈历史不是账本。 |
| `src/simulation/weapons.rs:30` | 现锤每击 damage24/range2.8/marker0.12/duration0.60；链斧18/2.6/0.08/0.40。均为 Kiva 参数，不是永劫原参数。 |
| `src/simulation/mod.rs:83` | prepare→motor→Avian→publish→advance→boss→availability→resources；整个链受 SimulationActive 与 HitStop 同门控。 |

现有动作表现起手比伤害早：`animation.rs:746` 从 active_action 建立显示提示；真实 Melee 标记在后续 fixed step 发出，`RigPose::frame` 以 action_id 避免再播一次。新增连段必须保留这个防重启机制（`animation.rs:143`）。

### 2. 推荐三段普攻契约（待主会话写入最终设计）

此节是建议的 Kiva 行为，不是已恢复的永劫规则。

- 每次实体左键只承诺一段，依次 1→2→3；单击不自动把三段全部播完，按住不自动连发。
- 仅在当前是第 1/2 段普攻时允许**最多一条**后继普攻缓冲。整个当前段内可登记一次，连续点击不能堆出三段以后或无限动作；第 3 段不缓冲第 4 段。
- 结束时如有缓冲，在下一固定步按相同准入检查接受下一段，分配新的 action_id；没有缓冲则留 0.45s 的后继输入窗口。0.45s 为建议值，必须在设计标为 Kiva 调校。
- 第 3 段结束回第一段。死亡/复活/F2 清理、有效换组、有效非普攻起手、有效防御/闪避/普通跳跃，以及后继窗到期，重置后继阶段与缓冲。
- 普通受伤目前只触发 Health.hit_remaining 与展示受击，**不会取消玩家已承诺攻击**（`combat.rs:1626`、`boss.rs:492` 只在死亡时 clear_targets）。不要借本需求默默新增硬直取消机制。若最终设计希望受击清后继链，可在普通敌人/Boss 的 `HitOutcome::Damaged` 分支直接清后继链；当前攻击仍按已有承诺规则走完，不能由反馈历史倒推权威打断。
- 新增 `MeleeStage::{One,Two,Three}`。`ActionSnapshot.melee_stage: Option<MeleeStage>`；非普攻为 None。`CombatEvent` 与 `HitContext` 同样携带阶段，普通攻击 marker 和各被击事件复制该次快照；hit_index 仍为该段 action 内的 0，不能兼作 stage。
- 单独的 `MeleeChain` 保存 next_stage、grace_remaining、buffered，不重用 E/Q/Tab `ComboWindow` 或 `SequenceSnapshot`。字段数量与可见性可用私有结构封装。
- 每段初期保留各武器已有伤害、范围与扇区，第三段不擅自增加蓝霸体/破防/额外伤害。时长及命中点写成**纯数值** `MeleeProfile` 表放 simulation；取值可根据源动作解析决定，但不得在 simulation import clip 名或 client 常量。
- 展示以权威 action.elapsed / action.duration 驱动当前段；若源命中点已恢复，可使用起手→接触→收招的分段单调时间映射，使真实打击 marker 与可视挥击接触一致。没有原接触证据时，明确称 Kiva 调校，不把 guessed frame 当原游戏 hit frame。
- 同步保留已跨多个 fixed step 的阶段快照和历史事件身份；不能在音效消费时读取最新 pair/next_stage 给旧事件选声音。
- 暂停清设备意图但冻结已经承诺动作。暂停前缓冲属于尚未承诺输入，应取消，避免恢复后鬼连击；需显式有一个“清未承诺普攻后继”的模拟接口或随 clear/cancel 锁存，下个真实 fixed step 执行。不能让普通 `CombatIntent::clear` 无条件把已承诺 action 取消。

与当前规则的必要差异必须写入 canonical spec：目前 `combat-v1.md` 明文规定动作中不排队新攻击，该条只对普攻第1/2段做有界后继例外，E/Q/Tab 等仍按原规则。

### 3. 右键有阶段快照，但展示层目前丢失它

`src/simulation/defense.rs:12` 已有 Idle/Deflect/Waiting/Guard/Parry 和准确 remaining、独立 cooldown；`advance` 在 `defense.rs:107` 处理拆招到等待/防御，`defense.rs:144` 处理有效 Guard→Released→Parry。短按释放不会创建弹反。

`animation.rs:630` 的 Inputs 只有 `parrying: bool`，真实适配 `animation.rs:777` 读取 `DefenseState::is_active()`，所有阶段都进 `PoseState::Parry`。`advance_source` 再统一选燕云化解片段（`animation.rs:280`）。因此仅换文件名无法完成用户的振刀/振刀成功区分。

建议输入改为 `DefenseDisplay { kind, elapsed/remaining, gesture_id }` 或至少 kind＋该阶段时间；当前 DefenseState 没有手势序号，快速重复同一种防御阶段不能仅比较枚举来重启。可在模拟接受 Pressed 或 Guard→Parry 时增加 gesture_id，携带一个只读 DefenseSnapshot，时钟仍由 simulation 管理。

| Kiva 状态 | 建议表现 | 结算/声音边界 |
|---|---|---|
| Deflect | 横刀振刀起手，按拆招窗映射 | 起手挥动声与真正拆招成功声分开；成功只有 Deflect 反馈。 |
| Guard | 同一振刀动作到已验证持势帧后保持 | 不每帧/每轮重播整段振刀或声音；格挡声只读 Block。 |
| Waiting | 振刀兼容的持势/自然恢复，标注没有有效防御 | 不播放成功声，也不把其当 Guard 免伤。 |
| Parry | 按用户明示，播放横刀振刀成功**动作** | 这是用户指定的动画映射，不表示这次弹反已经命中；成功音效/粒子仍须真实 Parry 反馈。 |
| Idle | 收势回当前待机/移动 | 不制造额外弹反。 |

`combat.rs:437`→`apply_hit` 先判躲避、正面拆招、正面弹反、格挡、伤害；`combat.rs:465` 对应发 Deflect/Parry/Block/Hit，只有 `HitOutcome::Parried` 让敌人/Boss stun0.80。成功音效准确的已有入口就是 `CombatEffect::Parry`。

`animation.rs:375` 目前三个成功反馈都设置 Gesture::Guard 并 restart。新防御源动作不能被同一次反馈错误重播到时间0；反馈用于可叠加视觉响应/声音，防御基础片段则跟 gesture_id、kind 与时钟。

### 4. 现有源动画绑定是燕云专用，不能替换裸矩阵

| 文件锚点 | 耦合 |
|---|---|
| `src/client/source_animation.rs:126` | Default include_bytes 指向 `assets/choulingke/animations.bin`；库本身资源只有一套。 |
| `source_animation.rs:139` | KIVASR01，u16 父索引/键毫秒；`source_animation.rs:172` 强制查 `Bip001` motion root。 |
| `source_animation.rs:296` | 去 root XZ、空中 root Y；heading 为 clip 首帧到 rest 的归一，Naraka 不能默认同 root 或同正面。 |
| `src/client/animation.rs:41` | SourcePose 只有 model Vec 和 weight，没有 rig 身份；source_clip/source_local/from 也默认同数组拓扑。 |
| `animation.rs:242` | 攻击硬编码燕云 skill_ylk_1 裁剪；idle/walk/run/crouch/slide/down 多数仍为 CC0，具体缺失必须列逐状态映射。 |
| `src/client/avatar.rs:180` | 23 driver 到燕云 Bip/Twist 的名字映射与 inverse_rest 初始化时固定。 |
| `avatar.rs:283` | 愁铃客 SourceBone.source 直接按原名保存燕云索引；Naraka palette 目前 None。 |
| `avatar.rs:352` | Naraka fingers 全部按燕云库绑定；缓存位置不是每帧动态名字查找。 |
| `avatar.rs:633` | source_driver_pose 使用预存 source 索引与 rest；源矩阵数组换成别的拓扑会套错或越界。 |
| `avatar.rs:687` / `avatar.rs:730` | Choulingke 源姿态路径直接使用完整燕云矩阵；这个优势只能留给同一原 rig，不能延续到 Naraka donor。 |
| `avatar/hand.rs:84` / `hand.rs:90` | target 默认 gMan，source 默认 Bip001；两侧 MCP 与独立三节指链有具体名字。 |
| `avatar/hand.rs:130` | 燕云需要第四个源 Nub 给末节方向；Naraka 本身可能仅三节，须源证据定义末节方向，不能补造长度。 |

**推荐有限改造：显式两种 SourceRig，复用采样与解剖校准。**

1. `SourceRig::{Yanyun,Naraka}`；`SourcePose { rig, model, weight }`；源片段 ID 同样是 `(rig, clip)`，不能只用 usize。
2. 保留旧燕云文件和 API 兼容入口。将纯 SRT 解码/采样提取为可配置 root/heading 的普通库，资源聚合两套库（或分两 Resource）；Naraka 单独随包二进制和来源清单。不要把 Naraka 片段伪装重命名成燕云关节，或者离线烘焙成986骨假源而丢失原骨架检查。
3. `AvatarAssets` 在加载时为每个来源建立独立 `MotionBinding`：driver 的 source/inverse_rest，原生 palette exact/source、finger binding、palm reference。共用目标 anchors、palette rest/IBM、几何与材质，不重复创建 Mesh/Image。
4. `evaluated_drivers`/`joint_pose`/`attachment_pose` 根据 rig 选缓存 binding。直接完整 palette 仅适用于“目标原生 rig 与 donor rig 真实一致”的路径；Chou+Naraka 必须走本体自有比例与手指重定向，不能因 appearance==Chou 就直接拿 Naraka 索引。
5. 让 hand 绑定接受已证实的 semantic bone names、真实父链和末节方向来源，保留当前解剖框算法。Naraka→Naraka 同名不等于同 rest；男性/女性、body/hair 子palette仍独立。
6. 来源切换清掉 source_local/from 和 clip，避免 `zip` 不同拓扑后悄悄短截混合。可先经当前最终目标姿态过渡；不同 rig 的原局部数组绝不能直接 lerp。
7. 用 actor 身份选择动作集：哥哥（包括 F3 换愁铃客身体）使用 Naraka；Boss/普通敌人保留燕云；妹妹保留实际 Q/Tab 权威时序及双手 IK，只有证据充分的独立远程片段才接入。不能 appearance==Chou 就把哥哥退回 Boss 动作集。

**必须保护的刚修复合同：**

- 上臂/前臂最小摆动校准到当前源解剖方向，目标自身长度 FK；不能退回 T→A 的全局quat直套（`model-assets.md:20`）。
- 掌框来自实际 wrist+index/middle/little MCP，握点 semantic scaleONE、+Y入掌、柄头朝拇指/食指侧；当前+0.055m适配保留（`model-assets.md:22`）。
- 每指三节保留独立源动态，**局部**旋转混合后 FK；原 IBM 与 sourceRestModel 冻结，不能拿 inverse(IBM) 当rest（`model-assets.md:24`）。
- 女身使用女性自己的23锚点、结构长度与握点IK；妹妹显式武器握点IK优先。未知 twist 分摊不补造控制器。
- 原始 Unity→已导出 glTF→gameplay 的 basis 必须一致。`scripts/import-naraka.py:440` 对已glTF资源用 `diag([-scale,scale,-scale,1])`，注释明确“不再做Unity reflection”；源动画从 rawUnity 出发必须单独核实先前反射，不能只套这个最终矩阵。

### 5. 离线产物、采样与可回滚边界

- 离线提取负责 Unity streamed/dense/constant/compressed 曲线的准确读取；不能在 WASM 引入 UnityPy、游戏安装路径、原配置 JSON parser。
- 推荐小范围新格式或版本升级：显式 magic/version、rig 名、root index、父序、原名与 rest SRT、clip 原名/时长、实际采样键。必要的 dense 烘焙是有误差记录的适配，原曲线和解析证据保留供校验，不声称原始键完全保留。
- 原曲线若是 Hermite/cubic，不能直接拿原关键帧做现库线性插值冒充准确解析。离线正确评估后以60/120Hz烘焙，量化/采样误差对照原曲线记录；是否用u16毫秒须按最大时长和误差决定。
- 同时保留 source sampling 与 root-normalized game sampling 的区别；根XZ只供显示剥除，不写Body。空中rootY不叠加实际jump高度，保留局部弯腿/重心动作。
- 路径/对象/曲线绑定与 source hash 进清单；分清原 clip duration/原事件和 Kiva crop/phase map、循环点/持势帧。
- 导出脚本支持 `--verify` 离线对冻结准备包重建比字节，不强依赖本机安装。原始准备包、runtime 二进制和应用映射分别可审查。旧燕云资源不覆盖；拔掉新动作选择能恢复旧展示，不动模拟/模型。
- 表现文件与规则文件不可互相导入：Simulation `MeleeProfile` 只有数值，Client `NarakaActionMap` 负责 clip/crop/持势/声音。若源clip变化需修改 Kiva 时序，应作为显式数值调校和对应测试更新。

### 6. 动作检查页与游戏共用的接缝

`animation/preview.rs:14` 的 SCENARIOS 当前只有12个游戏单体场景，缺跳跃/落地/闪避/滑铲；`preview.rs:29` 分组为CC0+一个source。`preview.rs:91` seek 在 game 模式逐60Hz重放 RigPose::frame，rawsource 模式绝对采样并隐藏手持；`viewer/session.rs:83` 的 mode 由前缀分类。

建议扩展，不复制一套 Naraka controller：

- 游戏场景新增 attack_1/attack_2/attack_3/完整三段显示串、deflect→guard→parry、jump1/jump2/fall/land/dodge/crouch/slide，以及实际找到的远程语义。
- 场景输入携带真实使用的 stage/defense snapshot 和 elapsed，继续走同一 frame。完整三段的检查串只能叫单体显示检查，不能冒称验证三段伤害或位移。
- 源目录分 `yanyun:<原名>` 与 `naraka:<原名>`，旧 `source:` 可保留别名兼容已有测试/复现文本；真实 duration 来自各库。Trace 加 rig、stage、defense阶段、原 clip time / Kiva时钟，JS不能另算crop。
- `viewer/mod.rs:57` 及游戏 `client/mod.rs:83` 初始化同一新资源；viewer拿同一缓存的 avatar binding。不要只在 viewer 加掌纠偏。
- 当前验收页**不装音频**，`web/audio.js:3` 也直接退出。用户此次只是要求对应游戏音效；若要在验收页听音，应额外设计显式“启用声音”开关/可信手势、时间轴 seek 默认静音、无倒拖补播。不能偷偷启用游戏pointerlock音频桥来破坏独立App。建议本轮先维持查看页静音，原声音校验走离线媒体与实际游戏；主会话可按范围决定。

### 7. 音频接入已有保证与缺口

`client/audio.rs:29` 遍历已提交反馈，用独立 Local cursor；`audio.rs:49` 即使 inactive 也前移游标，恢复不补旧声。`audio.rs:58` 所有近战现共享 Attack；`audio.rs:63` Deflect/Block/Parry 混为同一个程序 deflect；Shot 按事件原 pair 分弓/鸟铳，SniperShot不重复。

`web/audio.js:5` MAX_VOICES=4，`audio.js:15` 仅attack/hit/deflect三文件，`audio.js:34` 可信进入后异步fetch+decode一次、失败不排队，`audio.js:62` 统一有界声部；setActive(false)停声并保留BGM offset，blur/hidden/失锁同步停声。BGM与程序鸟铳独立保留。`index.html:25` 已copy整个assets/audio目录，不需新加载外链。

建议：

- 新增 Naraka 专用语义 cue，保留燕云/旧技能 fallback；来源、stage与结算kind决定声音，不能当前套装猜历史。至少分3段挥击（如果原作确实3个媒体/事件）、attack impact、deflect attempt、deflect success、block、parry success；缺失映射必须明确fallback，不应仅把一音复制三份改名。
- 攻击挥击优先消费每段已有的权威 Melee marker，若需要起手预声则加明确一次性 action-start cue，不能在每帧看busy就play。实质命中只消费 Hit；无目标挥空不得播放impact。成功振刀只读Parry/Deflect结算反馈，不读右键kind或动画正在播success的事实。
- 历史 Hit/Deflect 的 `actor` 是受害者，但 `source_owner/pair/stage` 是来袭源的信息；不能拿 attacker pair 给玩家格挡误选玩家自己的武器声，必须分别建“attack source”与“defender response”的选择规则。原pair合同需保持。
- 目前没有跳跃、落地、脚步的音效事件；`MotionState::advance` 在 `actions.rs:377` 接受真跳跃、`MotionState::publish` 在 `actions.rs:294` 判断实际落地；输入键边沿不是成功事件。若确认对应媒体可用，可增加小型 `MotionFeedback` 固定步有界事件流/单调序号，或明确的只读运动事件计数；不要拿 `CombatEffect::Dodge` 当闪避起手，因为当前它仅表示来袭被无敌躲过。
- 脚步若加入，以 grounded且真实物理水平累计距离/源动作步点驱动，排除blocked、空中、teleport、F2/respawn，不能按WASD held或显示帧数播放。记录来源循环/步点与 Kiva步距适配。物理规则本身不因音效改变。
- 真实step反馈可在物理publish后追加音频语义；有界consumer独立cursor、暂停前移、无恢复补播。脚步低优先级，不能挤掉四声部中的弹反/命中所有关键信息；是否保留4声部或增加受预算限制的优先级需依据新音源峰值验证。
- 当前**没有音量/静音UI或setVolume API**；只有固定gain0.18与active门控。PRD不要写成“沿用现有音量/静音控件”；可要求保留现有失锁静音，若新增用户音量需要明确小范围扩展与Node覆盖。

## 实施面与依赖建议

| 可验证切片 | 文件/产物 | 前置依赖 |
|---|---|---|
| 源动作解析和冻结准备包 | 新scripts importer/decoder；assets/prepared/naraka 动作清单；runtime animations.bin | animation-assets研究必须证明具体曲线可采样、映射身份、骨架。 |
| 源声音解析和冻结准备包 | 新scripts音频importer；assets/audio/naraka-*.ogg与provenance | audio-assets证明事件→媒体、可解码及峰值。 |
| 权威三段普攻 | simulation/{abilities,combat,weapons}.rs、combat_tests、公开exports | 可先数值方案实现；最终数值在源片段节奏审阅后冻结，不能依赖client。 |
| 双来源采样/重定向 | client/{source_animation,animation,avatar}.rs，avatar/hand.rs；缓存与测试 | 依赖真实source骨架和帧，三段snapshot/defense snapshot契约先定。 |
| 共享viewer与动作映射 | animation/preview.rs、viewer/{mod,session,snapshot,tests}.rs，必要web目录 | 依赖双来源API和权威阶段类型；不能先做独立假动画。 |
| 结算音效与运动cue | client/audio.rs、web/audio.js、scripts/audio-unit.test.cjs，必要simulation motion反馈 | 源媒体及权威stage/cue完成；保留BGM/Shot/去重。 |
| 集成验收及规范 | Trellis specs、browser QA报告 | 所有切片汇合、make check、WASMClippy、临时Trunk。 |

## 验证清单

1. 实际FixedUpdate逐段边界：每段新action_id与stage1/2/3；marker前后一固定步独立命中；总伤害准确；挥空、视线阻挡、超范围无Hit；第3段不生成第4段。
2. 一条缓冲上限、早按/晚按/多击合并、窗口到期重置、后继未满足准入丢弃；所有原E/Q/Tab不被排队普攻劫持；F2/死亡/暂停/换组/目标无敌的取消和快照同一步一致。
3. 活动动作与新stage同时被display取到，模拟marker事件不重新restart动画；低显示帧率下历史音效仍按各自stage/pair/action身份消费一次。
4. Deflect/Waiting/Guard/Parry具体映射、同阶段重复gesture重启、长按不循环发声、空弹反仅动作无成功声、真实Parry一次成功声、短按不能变弹反。
5. 各原片段取0/中/末及所有帧有限性；原曲线对照烘焙误差；seek正放/倒拖/换来源不受旧source_local数组污染。
6. 两来源×三角色×左右手×五指、source weight0/.25/.5/.75/1，实际加权顶点、臂段/指节长度与源解剖方向、signed柄头朝向、真实palette腕MCP握点。不能只比较游戏和viewer彼此一致。
7. Boss仍使用燕云完整palette；哥哥F3换成Chou身体仍使用Naraka玩家动作；白灵淼Q/Tab真握点与枪口0/-Z、女性比例、实体/材质/网格数量稳定。
8. 音频Node：新增cue仅白名单同源路径，trusted unlock一次、未decode丢弃不补播、inactive游标推进、重复history无重复、4声部/峰值预算、BGM单循环继续有效。声音来源说明与导出文件哈希一致。
9. 若新增运动音效，用实际接受跳/双跳、拒绝第三跳、实际落地、blocked静止无脚步、teleport/respawn不踏步、死亡/失锁/HitStop不补播回归。
10. `make check`、`cargo clippy --locked --target wasm32-unknown-unknown -- -D warnings`、临时目录Trunk；既有有界面Chromium+隔离Xvfb浏览器脚本扩展3段/右键阶段/跳跃/来源切换。真实硬件GPU验收与SwiftShader分列，不为无头截图合成问题改渲染。

## Related specs / references

- `.trellis/workflow.md`：planning研究输出、分层、Phase关口和research写入范围。
- `.trellis/spec/simulation/combat-v1.md`：当前不排队承诺、独立E/Q、真实防御结算、事件归属。
- `.trellis/spec/client/model-assets.md`：自有rest、arm/palm/finger、源/目标native轴与IK优先级。
- `.trellis/spec/client/combat-presentation.md`：actor/source_owner/pair、妹妹独立、声音去重与来源。
- `.trellis/spec/client/model-viewer.md`：同一frame/挂点、无SimulationPlugin独立App、seek重放、时间轴及rawsource。
- 未使用外网；依赖版本及实际源曲线/音频解码由其它研究文件提供。

## Caveats / Not Found

- 本文件没有确认任何横刀clip名、source hit marker或原音效event/media ID，不可据此宣称已提取。
- 泛化数据格式、stage缓冲和0.45s窗口为研究建议，尚未写产品代码或修改现有规范。
- 当前死亡等CC0回退和妹妹手写IK只能在找到可靠真实片段后逐项替换，不能为了“全部永劫”给未知动作贴名。
- 当前 viewer 不支持音效，现有 Web 也无用户音量控件；两者是否扩展由最终计划明确。
