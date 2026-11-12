# 第一版动作与战斗契约

## Scope / Trigger

修改 simulation/combat/、simulation/{abilities,defense,resources,weapons,boss}.rs、physics motor 的战斗约束或相关真实固定步测试时阅读。
client 只提交意图和读取快照；不拥有动作计时、伤害、耐力或强制位移。

## Signatures

- WeaponPair::{HammerMusket,ChainAxeBow}；WeaponMode 仅描述挂点握柄，不代表可控制人格。
- CombatIntent::{request_attack,request_melee_skill,request_ranged_skill,request_combo,request_toggle_target,request_pair,request_next_pair,request_rage,cancel_melee_chain,clear}。
- Energy::{sun,moon,rage,is_enraged,rage_ready} 与 Arsenal::durability(pair) 只读；二者随 `Combatant` 自动添加。
- Combatant::{weapon_pair,target,attack_cooldown,melee_cooldown,ranged_cooldown,combo_window,combo_remaining,active_action,sequence,companion,axe_away,availability,clear_targets}。
- ActionAvailability 包含 attack、melee_skill、ranged_skill、combo、defense、equip、movement_actions。
- `MeleeStage::{One,Two,Three}`；ActionSnapshot、CombatEvent、HitContext 各保留 `melee_stage: Option<MeleeStage>`，非普攻为None。
- ActionSnapshot／SequenceSnapshot 保留 action_id、kind、pair、原目标、elapsed、duration、shot_count、aim_position、source_position、companion_position；SequenceSnapshot 另标 cancelled。
- Companion { owner } 是无 Body／Health 的逻辑施法实体；CompanionState 提供独立演出位置、朝向、射击源、目标点、动作时间与可见性。
- DefenseEdge::{Pressed,Released,Cancelled}；DefenseState::push_edge／clear 提交输入，其余窗口、kind、冷却只读。
- DefenseState::snapshot() 包含kind/elapsed/remaining和单调gesture_id、last_gesture_kind/gesture_elapsed；自然Idle保留最后手势供显示，取消作废，复活不倒退ID。
- MotionFeedback::events() 为独立最多128条运动事实，MotionKind::{Jump,Dodge,Land,Footstep}；MotionEvent携带sequence/actor/position/jump_count/sprinting。
- DefenseKind::{Idle,Deflect,Waiting,Guard,Parry}；Stamina::{current,maximum,quiet_elapsed}。
- HitContext／HitOutcome 供普通敌人与 Boss 共用来袭结算。事件含 sequence、actor、source_owner、pair: Option<WeaponPair>、action_id、hit_index、kind 和真实起止位置。`HitContext.origin` 是攻击者位置（决定防御是否正面），`contact: Option<Vec3>` 是刀刃接触点；有接触点时受击事件的 `position` 取它。
- `HitTrack { duration, samples, windows, blade_radius, reach }` 是一招的刀刃轨迹；`TrackId::{all,duration,hit_track}` 列出全部挥刀招式、给出权威时长与烘焙数据。`ActionKind::{ALL,swings_blade}`、`WeaponPair::melee_skill`、`Boss::{STRIKES,strike_of,strike_duration}` 供烘焙与表现读取；`HitTrack::frontal_reach(scale, hurtbox)` 只在模拟层内用于接敌距离。
- `Projectile` 是飞行中的一发：`owner／pair／action_id／index／position／interpolated(alpha)／direction／age／radius／is_thrown／is_returning／is_locked` 只读；`clear_projectiles(world)` 在敌人集合替换时清掉全部飞行物。`CombatEffect::ShotImpact` 表示一发被敌人以外的东西挡下（位置为落点）。`Combatant::axe_away()` 在近战武器被扔出未回、或拉近的链子占用时为真。
- `HitProbes { enabled }` 是只读的判定几何历史（刀刃切片、受击胶囊、接触点），仅供调试显示；规则不读取它。

## Contracts

### 数据与模块

技能数值只有一份：`combat/skills.rs` 的 `SkillSpec { duration, cooldown, cast_range, shape }`。`shape` 为 `Blade`（普攻，只有刀刃扫掠，数值取 `MeleeProfile`）、`Area`（刀刃扫掠加一次范围伤害，中心为自身或锁定目标）、`Shot`（妹妹单发；`thrown` 表示发出去的是哥哥的近战武器）或 `Volley`（妹妹三连发，可带拉近）。`Shot`／`Volley` 各带一个 `ProjectileSpec { speed, radius, max_range, guidance }`。准入用的施放距离与结算用的射程读同一个 `cast_range`；下文各节的数字是这张表的说明，以表为准。`WeaponPair::action(Slot)` 把输入槽位映射到动作。
加一个已有形态的技能：加 `ActionKind` 变体、一行 `spec` 和一处槽位映射，不写新的规则代码。联动窗口的保持条件（锤鸟要求原目标浮空、链斧弓要求被本次动作僵直）仍在 `availability::window_valid` 里按武器组判断。
每行另有 `wear`（准入时扣的武器耐久）和 `rage`（强化释放积累的暴怒）；`Area`／`Shot` 带 `empowered`（强化倍率）。
`ActionKind::{shot_times,throw_times,return_window,strike_effect}` 把表里的时间点和起手效果以只读方式提供给表现层；`WeaponPair::ALL` 是全部武器组的声明顺序。
`prepare` 在物理前消费指令、推进已承诺动作并至多准入一个新动作，每一步是具名函数；`resolve_actions` 在物理后按形态结算刀刃扫掠与标记；`advance_enemies` 独立处理普通敌人。`ActionRuntime` 归战斗模块私有。

### 近战判定

所有近战命中（玩家普攻与范围技的刀刃、普通敌人、Boss）只有一种判定：`combat/sweep.rs` 把该招烘焙出的刀刃线段（握点→刀尖，攻击者局部坐标：脚底为原点、-Z 为前）按攻击者的脚底位置、`Facing` 和 `Body::scale` 放到世界里，在每个固定步内从上一 tick 的线段扫到本 tick 的线段，与目标的物理胶囊（脚底、`MotionState` 的实际胶囊高度、`Body::radius`）求相交。没有距离、角度或命中时刻常量。

- 数据在 `combat/hit_tracks.rs`，由 `make bake-hits` 生成，不手改：每招一条 `HitTrack`，逐 tick 采样、有效窗口（闭区间 tick，第 k tick 的扫掠是采样 k-1 到 k）、刀刃半径、最远距离。改片段、片段映射行、近战武器模型或招式时长后必须重新烘焙；`hit_tracks_match_animation` 在数据与动画不一致时失败。
- 一步内刀刃绕握点转过的弧按刀尖位移切成最多 12 片（握点线性、方向球面插值），高速挥击不会漏过目标。攻击者本步的位移计入扫掠起点。
- 只有处于有效窗口的 tick 才扫掠。窗口是刀尖速度不低于 25 m/s 的连续 tick，并且只从“起手过渡结束”之后开始：过渡期间画面里的刀取决于上一个姿态，烘焙无法代表。没有达到阈值的招式取最快时刻前后各一 tick。
- 接触后从攻击者 `aim_position` 向接触点做视线检查（排除双方与传感器），被场景或其他角色挡住不算命中。
- 每个动作对每个目标最多命中一次（`ActionRuntime.struck`、`Enemy.struck`、Boss 的 `strike_resolved`），不论该招有几个窗口；伤害数值因此不随窗口数量变化。
- 命中后仍走 `apply_hit`／`settle_defense`／`hit_enemy`：防御顺序、能量、耐久和事件归属不变。正面判定用攻击者位置，受击事件位置用接触点。
- 玩家普攻在第一个有效 tick 发 `Melee` 事件（半径为 `reach`）。范围技：刀刃碰到的敌人立即结算该技能的伤害与控制；第一个窗口结束的那一 tick（`HitTrack::impact_time`）发范围事件并结算范围内其余敌人。
- 已知局限：烘焙用站立姿态和每种角色的标准外观（玩家为默认外观，敌人与 Boss 为愁铃客）。蹲伏出刀时画面里的刀更低，其他外观的握点略有不同，差异由刀刃半径的 0.08m 余量吸收。

### 调度与权限

60Hz 同一门控：HitStop 步头 → 安装 → 减速计时 → 准备动作与防御 → motor → Avian／当前空间查询 → publish → 玩家动作结算 → 普通敌人 → Boss → 可用性 → 耐力恢复 → 妹妹快照 → 瞬移清理。
动作 elapsed 和冷却每步只推进一次。标记越界时有序执行，各标记只结算一次；不用动画结束回调制造命中。
动作准入在消费输入时验证；快照必须与本步完成后的动作／窗口／死亡状态一致。
默认自动索敌（`Combatant::auto_target()` 为真）：没有目标时，每个准备步锁定 `LOCK_RANGE` 内可见、可攻击的最近敌人，目标倒地、移除或进入无敌后由下一个替上。中键在已锁定时解锁并关闭自动索敌；未锁定时锁定朝向最对准的可见敌人并恢复自动索敌。自动索敌不切换已有的有效目标。`Combatant::without_auto_target()` 构造只靠中键锁定的战斗者（测试夹具用）。需要目标的技能不隐式选择；链斧 E 与普攻可以无锁定执行。
E／Q 冷却独立、跨武器组保留；技能动作中不接受新攻击、右键、切组或跳跃／闪避，不排队补放；当前技能匹配的 Tab 分支及下述普通连击缓冲除外。
普通方向移动可保留；联动主动移动被禁用或由拉近驱动。普通空中近战保留；地面技能与防御不在空中起手。

### 三段普通连击

每次有效左键只承诺当前一段。第1／2段最多缓存一条后继；整段duration结束并退休后，在下一次prepare准入后继，不能在本段marker时提前开下一段。第三段不缓存第四段；未缓冲时保留0.45s续段宽限，过期从第一段开始。
`WeaponPair::melee_profile(stage)` 返回纯数值：三段均保持锤 24 伤害／0.60s duration，链斧 18／0.40s；命中的位置和时刻来自该段的烘焙轨迹（见“近战判定”），每段对每个敌人只结算一次，不因原动作第三段效果而加伤、霸体或破防。

每段安装后发唯一MeleeStart，冻结actor/source_owner/pair/action_id/stage；刀刃第一次进入有效窗口时发唯一的 Melee 事件，它仍是挥击 FX 的入口，MeleeStart不能重复伤害或粒子。后继只在真实准入后才发起手事件。
有效技能／Tab／防御／闪避／跳跃／切组优先于自动后继；预测闪避准入必须与motor同一步先减dt的冷却顺序一致。无效请求不伪造动作取消。
失锁、F2、死亡／复活、实际受伤清未承诺连段；已承诺动作不被暂停或清后继中断。`cancel_melee_chain`和`clear`的取消锁存能跨零固定步保留；普通移动、F3、非滑铲蹲伏不清链。

### 运动事实

Jump／Dodge只在motor真实接受后记录，拒绝的第三次跳跃不发声。Land由物理publish后的真实离地→接触产生，有0.05s离地去抖；初次接地、传送、复活先重建基线，不制造落地。
Footstep累计碰撞后相对支撑面的真实位移，排除空气、堵墙、无输入、滑铲／闪避；步距2m、蹲伏0.8m再乘Body.scale是Kiva适配。各客户端独立消费游标，暂停／HitStop不推进运动事实，不用声音时钟驱动物理。

### 远程弹体

远程攻击在标记时刻只发射，不结算。`combat/projectile.rs` 的每一发是一个没有碰撞体的 `Projectile` 实体，`advance_projectiles` 排在玩家动作结算之后、普通敌人之前，受同一暂停／HitStop 门控。

- 每步先按制导转向，再从上一位置到新位置做球体扫掠（半径取该行，排除发射者与传感器，已倒地或处于无敌阶段的敌人不挡路）：先碰到的是敌人就由它吃下这一发，是别的东西就停在那里并发 `ShotImpact`。飞满 `max_range` 即消失。挡在路线上的其他敌人会替原目标挨打。
- **是否锁定是每个技能行的数据**：`Guidance::Locked { turn_rate }` 每步以不超过该角速度转向发射时记下的原目标的瞄准点，最后一步之内直接飞向它；原目标失效后保持当时方向直飞。`Guidance::Straight` 沿发射方向直飞，不追任何目标。锁定的技能保留“需要锁定目标且在 `cast_range` 内”的准入；直飞的技能把 `cast_range` 留空，不需要锁定，没有目标时朝正前方发射（`DEFAULT_AIM_DISTANCE`）。当前四个远程技能全部是 `Locked { turn_rate: 12 }`；`Straight` 由弹体测试覆盖，暂无技能行使用。
- 命中走共同的 `hit_enemy`：伤害、控制、烈阳、事件归属不变，受击事件位置是弹体接触点，`origin` 是枪口。弹体带着发射时冻结的动作快照，命中事件的 `action_id／pair／hit_index` 不随之后的换组或新动作改变。
- 命中才触发的效果都移到弹体落点：Q 的僵直与联动窗口（仍要求打中的是该动作的原目标）、锤鸟每发的 `SniperImpact`、最后一发的 HitStop。`Shot`／`SniperShot`／`AxeThrow` 事件在发射时发出。
- 三连发每发在自己的标记时刻朝原目标当时的位置发射；锁定的连发在原目标失效后取消尚未发射的部分，已在飞的继续飞。
- 被扔出的近战武器（链斧）命中或飞满射程后原速飞回主人胸口，回程不再碰撞；回到手上或拉近联动开始接管时才算归还。归还前 `axe_away()` 为真，该组不能普攻或用近战技能，切到另一组不受影响。
- 主人倒地时他的飞行物立即消失；F2 换敌人经 `clear_projectiles` 清空。
- 初值（待试玩）：鸟铳 120 m/s、锤鸟连发 160 m/s、弓箭 60 m/s、链斧 35 m/s；射程 26m（链斧 16m）。

### 两套技能与联动

锤 E 在原目标 4.5m 内、其周围 3m 内结算 20 伤害和约 3m 真实物理击飞；必须原目标实际被击飞才开放对应窗口。范围在烘焙轨迹的落点时刻结算，刀刃先碰到的敌人先结算，每个敌人一次。
鸟铳 Q 是妹妹单枪（0.18s 发射）、40 伤害及 0.45s 僵直。锤鸟 Tab 消费本次 E 的原目标窗口，0.30／0.42／0.54s 三发分别发射、各 28 伤害，总序列 0.96s。伤害在弹体到达时结算。
链斧 E 围绕自身造成范围伤害并施加单一 SlowEffect，减速进入真实 motor；同类效果不叠乘。
弓 Q 在 0.15s 发射链斧、35 伤害和 0.90s 僵直，离手期间不能再次用该链斧近战；未接联动则链斧飞回手中才归还，与技能时长无关。
链斧弓 Tab 必须原目标仍被本次 source_action_id 僵直；0.12／0.28／0.44s 三箭分别发射、各 18 伤害，总序列 0.93s。
拉近接受胶囊碰撞，最大平面速度 18m/s、最多 0.65s、接近约 1.4m 停止；高差、墙体或持续无有效前进时取消。不能改显示根／Position 伪造接近。
Tab 接受时要求浮空／本次僵直；承诺后自然落地或僵直到期不取消已经接受的三发。每发发射前仍检查原目标是否有效，不跳转新目标；视线、范围和阶段无敌由弹体的飞行决定。
最后一发锤鸟成功命中时沿用一次 3 固定步 HitStop；普通 Q 不触发这个专用停顿。

### 防御与耐力

拆招窗 0.20s，持续按住达到 0.20s 进入防御；从有效防御物理松开开启 0.18s 弹反窗。拆招／弹反各自再触发冷却 0.65s。
短按松开保留已开启的拆招直到结束，不额外弹反；取消、失焦与死亡都不能当 Released。
闪避无敌优先，其次正面拆招、弹反、有效格挡、普通受伤。正面点积阈值沿用 >0.25。
拆招免伤且不额外僵直；弹反免伤，对攻击者反击 `PARRY_DAMAGE`（10）并使其进入 0.80s 僵直。反击走共同命中（`apply_hit`），事件顺序为玩家的 Parry 之后紧跟攻击者的 Hit；普通敌人与 Boss 都经 `settle_defense` 结算。
耐力上限 100，成功格挡扣 20 并承受原伤害的 50%；刚好扣到 0 的这一击仍成立，之后不足时正常受伤，无破防和命中次数上限。
拆招／弹反／闪避不扣耐力。实际到达命中判定的来袭重置安静期；进攻活动期间不恢复，安静 1.0s 后每秒恢复 15。
恢复排在所有来袭之后；SimulationActive／HitStop 冻结恢复与所有规则时间。复活补满，F2 不补满。
防御移动为常速 0.35 倍；跳跃／闪避可退出防御姿态，设备层不能直接改 MotionState。

### 烈阳、月光与暴怒

`Energy` 组件独立于 `Combatant`，由战斗规则系统独占写入。数值：烈阳／月光上限 10，暴怒上限 20、暴怒状态持续 20s（`Energy::{SUN_MAX,MOON_MAX,RAGE_MAX,RAGE_DURATION}`），暴怒中普攻伤害 ×2。
烈阳：玩家每一次造成伤害的命中 +1（范围技命中两个敌人为 2，三连发最多 3；被闪避／无敌／目标失效不计），弹反成立 +1。月光：来袭被闪避无敌或拆招化解 +1；格挡、弹反和受伤不计。
强化：烈阳满时下一次 E、月光满时下一次 Q 自动强化——伤害乘以该技能行的 `empowered`（当前均为 1.5），对应能量清零，并积累该行的 `rage`（E 为 1、Q 为 2）。强化在准入时判定并固定在动作上（`ActionRuntime.damage_scale`），被拒绝的请求不消耗能量。联动不强化。
暴怒：暴怒值满后 X（`CombatIntent::request_rage`）进入暴怒状态，无额外攻击动作；请求由独立系统 `release_rage` 消费，未满或倒地时请求被丢弃。暴怒中：普攻伤害 ×2；E／Q 必定强化，不消耗烈阳／月光，也不再积累暴怒；联动窗口成立后的下一步自动准入联动，等同按下 Tab。暴怒值在规则时间内匀速耗尽（`drain_rage`），暂停与 HitStop 不消耗；耗尽即结束。
复活把三种能量清零并结束暴怒；F2 换敌人保留。

### 武器耐久

`Arsenal` 组件记录每组武器的耐久，上限 50。准入一个动作时扣该技能行的 `wear`（普攻 1、E／Q 各 10、联动 0）；扣到 0 为止，不因耐久不足拒绝动作，被拒绝的请求不扣。
耐久为 0 的武器组在没有进行中的动作且没有打开的联动窗口时自动换到下一组（`WeaponPair::next`，按 `ALL` 循环），被换下的那组立即回满，状态为 `WeaponExhausted`。自动换组不要求站在地面，也不被防御阻止。
Z（`request_next_pair`）手动循环换组，沿用原有条件（空闲且在地面），不恢复耐久。复活回满全部武器组；F2 保留。

### 归属与生命周期

玩家始终是哥哥，生命 1000。妹妹事件 actor=逻辑 Companion，source_owner=玩家；受击事件 actor=受害者。不能靠距离、中文状态或 source_owner 把妹妹射击动画播给哥哥。
ComboWindow 保存 source_action_id、pair、original_target 和 remaining；僵直来源由普通 Enemy 或 Boss 自己的唯一状态持有，不在 Enemy 里镜像 Boss 倒计时。
玩家死亡、目标死亡／移除、Boss 无敌阶段、模式替换取消未发射部分；镜头快照可有界恢复并保留最后瞄准点。F2 在暂停时也必须同步隐藏妹妹、清掉拉力／旧动作，不能等待恢复后的固定步。
暂停只清设备意图并冻结已承诺技能；恢复不补旧输入，不恢复跳跃次数。F3 不改动作、物理、武器组或妹妹。
玩家／靶子死亡 3s 后复活，清旧输入、目标、联动、临时控制效果和冷却；靶子保持被动。Boss 新遭遇清理旧减速等控制效果。
反馈最多 128 条、序号单调增加，各显示消费者各自游标；玩家最近一次指令结果以 `CombatStatus`（Ready／PairSwitched／WeaponExhausted／Enraged／Started(kind)／TargetOutOfReach／Busy／Guarding／Unavailable）发布，不携带界面文字；不能把显示历史当作命中去重账本。

## Validation & Error Matrix

| 情况 | 行为 |
|---|---|
| 无锁定／超范围／隔墙／冷却中起手 | 拒绝并消费请求，不开启新冷却 |
| 起手后目标走开或遮挡 | 锁定的弹体追着目标飞，被墙或别的敌人挡下则这一发落空；保留已经开始的冷却 |
| 弹体飞行中原目标死亡或进入无敌 | 弹体沿当时方向直飞，可能打中路线上的其他敌人 |
| 链斧未回手时普攻／近战技能 | 不可用；切到另一组可正常使用 |
| 目标在正前方但刀刃够不着，或隔着墙 | 不命中；刀刃实际扫过的侧面目标命中 |
| 烘焙数据与动画、武器或时长不一致 | `hit_tracks_match_animation` 失败，提示运行 `make bake-hits` |
| 重复 Tab／换组／技能动作中的攻击 | 活动动作不被替换，不排队，不重复伤害 |
| 普攻期间反复左键 | 第1／2段只缓存一条后继，第三段无第四段 |
| 耐力不足／背面攻击 | 正常受伤，无破防惩罚 |
| 右键队列取消／溢出 | 取消当前手势，重新按下才可重建 |
| 目标落地或僵直到期 | 尚未接受的窗口失效，已接受的三发按原目标继续复验 |
| Boss 转阶段／死亡／F2 替换 | 停止旧攻击与拉力，清妹妹／目标，镜头恢复 |
| 暂停／HitStop | 不补算规则时间、不推进耐力或冷却，也不消耗暴怒 |
| 暴怒值未满或倒地时按 X | 请求被消费，不进入暴怒，无事件 |
| 耐久为 0 但动作或联动窗口仍在 | 不换组；结束后的首个准备步换组并回满旧组 |

## Good / Base / Bad Cases

Good：E 击飞原目标后改锁另一个敌人，Tab 三枪仍只向原目标分别结算。
Base：右键长按减伤，松开在有效窗内弹反；耐力见底只使后续格挡失效。
Bad：只播放三次闪光却扣一笔伤害；链斧拉近穿墙；F2 后在暂停画面留下旧妹妹。

## Tests Required

使用真实 FixedUpdate 测试逐段唯一start／marker、单缓冲、0.45s续段／过期、第三段封顶、同帧优先级和零固定步取消；覆盖最后半步闪避冷却竞争、实际运动声事实与重置。
使用真实 FixedUpdate 测试三发时间点前后、独立 hit_index、原目标／stun_source、视线、阶段免疫、死亡和取消，不只断言最终总血量。
测量实际高台／平地击飞最高点、拉近碰撞／停距／高差及减速位移；保留原物理、二段跳、Shift／蹲伏、HitStop／暂停回归。
能量覆盖每次命中的烈阳、拆招的月光、强化 E／Q 的伤害与清零、暴怒积累／释放／普攻翻倍／自动联动／20s 耗尽、弹反反击与事件顺序、耐久消耗／耗尽换组／联动窗口保留／Z 循环、复活重置（`combat/energy_tests.rs`）。
防御覆盖短按／长按／释放／取消、独立冷却边界、正背面、第五次有效格挡、低耐力恢复和同一步快照一致性。
弹体覆盖（`combat/projectile_tests.rs`）：飞行时间等于距离除以速度、锁定弹追上横移和下落的目标而直飞弹落空、无目标直飞到射程尽头消失、挡路的敌人先中、墙挡下并报告落点、倒地者不挡路、暂停冻结、主人倒地和换敌人清空、链斧回手前不能挥、每个远程技能行的制导与准入一致。依赖命中时刻的旧测试改为等弹体落地。
近战判定覆盖：几何单元测试（弧而非弦、够不着、身后、蹲伏、朝向／缩放）、真实固定步里的侧面命中／够不着／隔墙、Boss 阶段随烘焙窗口切换；依赖命中时刻的测试从 `HitTrack` 读取接触 tick，不写死。客户端 `hit_bake_tests::live` 在真实战斗里核对画面中的刀与烘焙数据在有效窗口内相差不超过 3cm。
执行 make check；Web Rust 另跑 WASM Clippy，打包另用临时目录 Trunk 构建。

## Wrong vs Correct

错误：客户端发射动画回调直接扣血，妹妹共享玩家 actor，Boss 再存一份 Enemy 僵直计时。
正确：固定步动作标记结算一次伤害，Companion 作为实际施法者，Boss 自己持有控制来源和期限。
