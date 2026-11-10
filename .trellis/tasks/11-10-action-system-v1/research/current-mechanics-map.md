# Research: 当前游戏机制图谱

- Query: 为当前代码结构 HTML 图核对真实操作、状态所有权、两套战斗链与生命周期；不包含拟议第一版。
- Scope: internal
- Date: 2026-11-10

## Findings

### 当前版本最重要的事实

| 项目 | 当前代码事实 | 证据 |
|---|---|---|
| 玩家生命 | 100，上限不是规划中的 1000 | src/gameplay/mod.rs:256、src/gameplay/mod.rs:265 |
| 玩家与“妹妹” | 仅装配一个 Player、一个 PlayerView；“哥哥／妹妹”是同一 Combatant 的 WeaponMode 对应人格标签，没有独立妹妹实体 | src/gameplay/mod.rs:256、src/client/scene.rs:58、src/simulation/weapons.rs:25 |
| 武器组 | HammerSniper（锤子／狙击枪）与 AxeCannon（斧子／火炮）；不是规划中的鸟铳与弓箭 | src/simulation/weapons.rs:4、src/simulation/weapons.rs:33 |
| 实际近战外观 | 两组近战共享当前身体外观对应的长刀：愁铃长刀或烛泪；内部数据仍分别走锤／斧机制 | src/simulation/combat.rs:346、src/client/hud.rs:234、src/client/appearance.rs:65 |
| 锤狙联动 | Tab 后延迟 0.30s，只发一枪、85 伤害；不是三枪 | src/simulation/combat.rs:274、src/simulation/combat.rs:784 |
| 防御 | 只有右键按下沿的短窗口“卸势／拆招”；无长按格挡、释放弹反、耐力或破防 | src/simulation/actions.rs:184、src/simulation/actions.rs:508、web/input.js:125 |
| 能量等 | 无烈阳、月光、暴怒、强化技、耐久相关运行状态 | src/simulation/combat.rs:306、src/simulation/actions.rs:133；对 src／web／index.html 的关键词检索未发现对应规则 |

术语说明：用户需求已统一称“拆招”，现行 Rust 名称为 parry，HUD／音效等仍有“卸势”字样。图中可以标“拆招（现代码 parry）”，但不要把规划里的“拆招／防御／弹反”三阶段画成现状。

### 现有操作映射

| 输入 | 当前行为 | 源码锚点 |
|---|---|---|
| WASD | 相机 yaw 相对的世界 x/z 移动；基础速度 6m/s | src/client/controls.rs:318、src/gameplay/mod.rs:258 |
| 鼠标水平 | 调整 ViewYaw；不乘帧时间；普通镜头不读垂直作为俯仰 | src/client/controls.rs:59、src/client/controls.rs:308 |
| 鼠标垂直 | 仅斧炮组远程模式且未跃进时选择火炮落点距离，默认 6m、1–8m | src/client/controls.rs:44、src/client/controls.rs:326 |
| 左键 | 当前模式普攻，因此近战与远程都可以左键攻击 | src/client/controls.rs:283、src/simulation/combat.rs:1012 |
| 右键按下 | 提交一次 request_parry；长按与释放不生成新窗口 | src/client/controls.rs:342、web/input.js:125 |
| 滚轮 | 同组近战／远程切换，即哥哥／妹妹人格标签切换；取消旧联动 | src/client/controls.rs:286、src/simulation/combat.rs:747 |
| 1／2 | 选择锤狙／斧炮组，回近战模式，清旧联动 | src/client/controls.rs:275、src/simulation/combat.rs:739 |
| Q | 循环目标；无有效目标时会自动选择面前可见敌人，当前没有中键显式解锁 | src/client/controls.rs:277、src/simulation/combat.rs:520 |
| E | 当前模式起手技；只锤狙近战与斧炮远程有有效起手，其他模式提示先滚轮切换 | src/client/controls.rs:273、src/simulation/combat.rs:873、src/simulation/combat.rs:940、src/simulation/combat.rs:960 |
| Tab | 尝试消费已有窗口，不是通用无条件技能；无效尝试也不排队 | src/simulation/combat.rs:965 |
| Shift | 按下闪避，长按 0.16s 衔接疾跑；疾跑锁存到所有方向键松开 | src/simulation/actions.rs:5、src/simulation/actions.rs:407 |
| C | 蹲伏切换；疾跑中可滑铲，0.55s 后保持蹲伏 | src/simulation/actions.rs:9、src/simulation/actions.rs:460 |
| 空格 | 支持二段跳，竖直起跳速度 6.4m/s；落地恢复额度 | src/simulation/actions.rs:498 |
| F2 | 靶子／实战敌人模式请求；替换敌人、清目标／联动，不重置玩家血与冷却 | src/client/enemies.rs:15、src/gameplay/mod.rs:41 |
| F3 | 换身体与对应近战刀外观；不是武器组切换 | src/client/appearance.rs:22、src/client/appearance.rs:51 |
| Esc／失焦 | 失去捕获，清 Intent，SimulationActive=false | src/client/controls.rs:298 |

普攻数据来自 src/simulation/weapons.rs:50：

| 数据中的武器 | 伤害 | 范围 | 冷却 | 溅射 |
|---|---:|---:|---:|---:|
| 锤机制近战 | 24 | 2.8m | 0.60s | 无 |
| 狙击枪远程 | 40 | 22m | 0.90s | 无 |
| 斧机制近战 | 18 | 2.6m | 0.40s | 无 |
| 火炮远程 | 26 | 18m | 0.80s | 半径 1.6m |

普攻依据当前选中目标、三维距离和可见候选结算；无目标会挥空／打向前方并能对道具施加冲量，不是已经存在一套任意多人扇形近战命中器（src/simulation/combat.rs:1018）。

### 战斗链 A：锤机制击飞 → 单发狙击

1. 锤狙近战模式 E：要求选中目标在 4.5m 内，以目标位置为中心半径 3m，20 伤害；成功命中活敌人时施加约 3m 的物理击飞（src/simulation/combat.rs:874、:886、:902）。
2. 保存 Link::Hammer(original_target) 和 0.85s 窗口，技能冷却 3.0s、普攻冷却 0.45s（src/simulation/combat.rs:910）。
3. 窗口和原目标浮空都有效时 Tab 消费窗口，切为远程模式，创建 SniperSequence，保存原目标及其真实胸口瞄准点（src/simulation/combat.rs:575、:965、:976）。
4. 延迟 0.30s 实际 Shot／85 伤害；射前每步验证目标存活、浮空和自身动作，实际射击还检查视线。只用 fired 布尔保证单发（src/simulation/combat.rs:761、:784）。
5. 射后保持 0.14s、恢复 0.28s，总序列 0.72s；相机前 0.20s 移到低位背后仰视（src/simulation/combat.rs:281；src/client/camera.rs:20、:63、:74）。
6. 发射产生 Shot＋SniperShot，受害者产生 Hit＋SniperImpact；专用事件不追加伤害。成功狙击请求 HitStop，音频只对普通 Shot 播一次枪声（src/simulation/combat.rs:789；src/client/audio.rs:53）。

原目标死亡、落地（射前）、被移除、手动切武器／套装等取消未发射狙击；本枪击倒目标后保留短暂射后恢复和最后瞄准点。Q 改选不会把已经承诺的狙击转向新目标。

### 战斗链 B：火炮跃进 → 空中旋刀

1. 斧炮远程模式，用鼠标垂直选择真实可站立 SurfaceHit；E 校验落点不为空且距离足够，MotionState::start_leap_to 推动一次 0.90s 的物理跃进（src/simulation/combat.rs:709、:940）。
2. 保存 Link::Cannon，窗口等于跃进 0.90s，技能冷却 3.5s；起跳后 aim 固定为承诺落点（src/simulation/combat.rs:709、:944）。
3. 跃进期间 Tab 消费窗口，切回近战模式并启动剩余时长的旋刀（src/simulation/combat.rs:990）。
4. 旋刀沿实际路径检查周围半径 2.8m，对每个敌人只命中一次、42 伤害；spin_hits 同时防止重复推击道具（src/simulation/combat.rs:825、:834）。
5. 落地／跃进终止则旋刀结束；此链不触发锤狙专用镜头或狙击 HitStop。

### 右键与闪避的真实来袭结算

- 拆招窗口 0.25s、冷却 0.65s；仅存活、地面、非闪避／跃进／滑铲等可用状态接受。窗口移动速度 0.35 倍（src/simulation/actions.rs:184、:508、:545）。
- 正面要求面对攻击来源的平面点积 >0.25；成功免伤并停止攻击者预警／移动。普通主动 Enemy 恢复 0.45s（src/simulation/combat.rs:23、:1167）；Boss 进入独占 Deflected 状态 1.1s（src/simulation/boss.rs:37、:452）。
- 闪避无敌判断先于拆招判断；无敌窗口为经过闪避 0.06–0.28s 的固定步，其他时间正常受伤（src/simulation/actions.rs:16、:523；src/simulation/boss.rs:450）。
- 普通 Enemy 来袭和 Boss 来袭目前各自复制了防御判断，并没有第一版拟议的共同 HitOutcome 入口（src/simulation/combat.rs:1153；src/simulation/boss.rs:442）。
- 背面／侧面或窗口外来袭正常扣血；没有格挡减伤，也没有 stamina 消耗。按住／松开右键不能触发额外弹反。

### 敌人、血量与死亡生命周期

| 对象 | 装配与状态 | 生命周期 |
|---|---|---|
| 玩家 | 一个 Player，原点，100 血、速度 6、半径 0.35；有 Combatant／CombatIntent | 归零后停止新动作，3s 后出生点恢复满血，清输入、目标、联动与普攻／技能冷却（combat.rs:85、:668） |
| 靶子 | 一个 Enemy::target_dummy，位置 (0,-4)，1,000,000 血、Body 比例 1.45 | 不追击、不攻击，仍碰撞、受伤、被击飞；死亡 3s 复活后仍被动（gameplay/mod.rs:63；combat.rs:123、:1106） |
| 实战 Boss | 一个标准比例愁铃客，位置 (0,-6)，速度 5.2，每阶段 650 血，共两阶段 | P1 归零后进入 2.2s Transition、补满并转 P2；P2 归零进入 3s Defeated 后重开；玩家死亡导致遭遇重置（boss.rs:34、:109、:265、:320） |

普通 Enemy 仍有独立主动 AI 兼容路径，但产品当前实战为单 Boss，普通 Enemy 推进跳过有 Boss 组件的实体（combat.rs:1101），不能把这两套 AI 画成同一 Boss 同时运行。

Boss 状态图：Idle → Pursuing／Strafing／Telegraph → Attacking → Recovering → Idle；正面拆招使 Attacking → Deflected → Idle；阶段耗尽进入 Transition／Defeated（src/simulation/boss.rs:18、:354）。
伤害为 P1 14／P2 18，预警 0.72s、有效攻击 0.22s、普通恢复 0.65s；命中需距离 <=3.2m、高差 <=0.9m、140° 总扇区并有视线，每次有效攻击窗口只伤害一次（src/simulation/boss.rs:34、:421）。
Boss::is_invulnerable 在 Transition／Defeated 为真；玩家 hurt_enemy 和击飞都尊重此状态（src/simulation/boss.rs:139；src/simulation/combat.rs:547、:893）。

### 暂停、HitStop 与时序

- FixedUpdate 60Hz；HitStop::begin_tick 先锁定本步门控；动作 motor → Avian 求解 → 当前空间查询／publish → combat::advance（含普通敌人）→ boss::advance → flush_teleports（src/simulation/mod.rs:30、:64）。
- SimulationActive=false 时停止整个规则／物理链；显示取当前快照，避免 fixed overstep 引起抖动。设备输入清理不直接改 MotionState。
- 狙击成功请求 0.04s 预算，按 60Hz 向上覆盖 3 步，约 50ms；它从后续固定步开始冻结物理、AI、动作、冷却和狙击序列，不更改发起步已决定的瞬移清理（src/simulation/impact.rs:13、:20、:27）。
- 失锁期间不消耗 HitStop 预算；恢复后不补算漏过的世界时间。专用冲击粒子／镜头后坐允许在 HitStop 中继续表现，但失锁时也冻结（.trellis/spec/client/index.md 对冲击表现的契约）。
- CombatFeedback 是最多 128 条的单调序号历史，Update 端各自游标消费；攻击 actor 为发起者，受击／拆招／死亡／复活 actor 为接受者（src/simulation/combat.rs:22、:177、:199）。

### 文件清单

| 文件 | 图谱中职责 |
|---|---|
| src/simulation/mod.rs | 固定步注册、门控与调度顺序 |
| src/simulation/actions.rs | 跳跃、闪避、疾跑、蹲伏、滑铲、拆招窗口 |
| src/simulation/combat.rs | 意图、目标、技能、两条联动、血量、普通敌人、反馈历史 |
| src/simulation/weapons.rs | 武器组／模式与普攻内容 |
| src/simulation/boss.rs | 独占 Boss 阶段、决策、动作与来袭 |
| src/simulation/physics.rs | Avian motor、碰撞、支撑与快照 |
| src/simulation/impact.rs | HitStop 权威门控 |
| src/gameplay/mod.rs | 玩家／敌人装配、场地和模式替换 |
| src/client/controls.rs、web/input.js | 设备适配、可信捕获和一次消费 |
| src/client/scene.rs | 单一玩家视图及物理结果插值 |
| src/client/camera.rs | 普通相机与锤狙专用转场 |
| src/client/combat.rs、animation.rs、impact.rs | 只读武器、动作、普通特效和专用冲击 |
| src/client/hud.rs、web/hud.js | 当前血量／目标／模式／冷却的只读显示 |
| src/client/appearance.rs | F3 身体与近战刀换装 |
| src/client/audio.rs、web/audio.js | 已结算反馈的音频消费 |

## External References

没有使用外部资料。数值和行为以当前仓库实现为依据；原作 Boss 决策池与本项目调整数值不能混为同一来源。

## Related Specs

可视化交付：[当前游戏结构图谱](../../../../docs/action/current-game-architecture.html)。独立 HTML，展示本研究核对的当前机制以及全模块、帧时序、实体和文件索引。

- .trellis/spec/simulation/index.md
- .trellis/spec/simulation/physics.md
- .trellis/spec/gameplay/index.md
- .trellis/spec/client/index.md
- .trellis/spec/guides/cross-layer-thinking-guide.md

## Caveats / Not Found

- 本研究只核对代码和现行规范，没有启动开发服务或运行浏览器／游戏。
- 没有独立妹妹、耐力、长按格挡、释放弹反、烈阳／月光／暴怒、强化技、耐久、链斧弓或三发联动；这些不能画为当前已实现的节点。
- 源技能槽到完整原动作仍未恢复，Boss::clip_name() 返回 None（boss.rs:135）；图谱不应暗示已完整复刻所有原作技能。
- 普通敌人兼容 AI 与产品 Boss 需区分；两者拆招后硬直分别为 0.45s／1.1s。
