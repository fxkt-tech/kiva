# 模拟层

## Scope / Trigger

`src/simulation/` 拥有权威运动、动作、武器、伤害、敌人和复活；变更这些规则或固定步时阅读。
不依赖客户端、浏览器、窗口、相机或渲染资源；物理内部的 Transform 同步不属于客户端表现。
三维位置、角色控制与接触求解见 [physics.md](physics.md)；第一版技能、防御耐力与角色归属见 [combat-v1.md](combat-v1.md)。

## Signatures

- `SimulationPlugin` 注册 60Hz `FixedUpdate`。
- `Body::new(Vec2, speed, radius)` / `Body::at(Vec3, speed, radius)`；`world_position()`、`interpolated_world(alpha)` 为三维脚底快照，`position()` 为 x/z 投影。
- `Body::step_displacement()` 只读实际碰撞后最近固定步位移；MotionState 的 `dodge_elapsed/dodge_direction()` 只读闪避进度 / 方向，用于表现。
- `MoveIntent::set(direction: Vec2)` / `stop()`。
- `spawn_box` / `spawn_terrain` 创建真实三维碰撞实体；`PhysicsPose` 为箱体／地形中心姿态的只读快照，不再提供显示插值。
- `CollisionWorld` / `SolidBox` 只在测试构建中作为无窗口夹具的装配描述，产品构建不包含，不用于产品关卡的运动或战斗查询。
- `ActionIntent::set_shift_held(physical_shift)` / `set_movement_held(bool)` / `request_jump()` / `request_dodge()` / `request_crouch_toggle()` / `clear()`。
- `MotionState::{is_crouched,is_sliding,slide_elapsed,slide_direction}` 只读蹲伏和滑铲状态。
- `MotionState::jumps_used()` 只读当前腾空已用跳跃次数，供二段跳表现使用。
- `Facing::set(Vec2)` / `direction()`；`MotionState` 只读接触和跳跃状态，`launch_up(velocity)` 由模拟击飞调用。旧火炮 Leap 接口已删除，拉近通过 motor 目标速度求解。
- `CombatIntent::{request_attack,request_melee_skill,request_ranged_skill,request_combo,request_toggle_target,request_pair,cancel_melee_chain,clear}`；`Combatant::new(WeaponPair)`，哥哥始终为近战受控角色。
- `Health::new(maximum)` / `current/maximum/is_alive()`；`Enemy::default()` 为普通主动敌人（兼容无窗口夹具），`Enemy::target_dummy()` / `is_passive()` 为被动靶子；`SpawnPoint(Vec2)`。
- `Combatant::clear_targets()` 在敌人集合替换时清理目标和联动，不刷新技能 / 普攻冷却。
- `Combatant::{active_action,sequence,companion,axe_away,availability}` 暴露只读动作和联动；`CompanionState` 独立描述妹妹，窗口／资源见战斗契约。
- `CombatFeedback::events()` 为只读有界历史；`SimulationActive(bool)` 暂停整条固定步链。
- `HitStop::is_active()` 暴露冲击停顿状态；模拟请求 0.04s 狙击命中停顿，固定步向上取整（60Hz 为 3 步 / 50ms）。

- `AttackClock { action_id, track: TrackId, elapsed, duration }` 是三类攻击者共用的攻击时间轴：`Combatant::attack_clock()`（由 `ActionSnapshot::clock()` 派生）、`Enemy::attack_clock()`（蓄力＋0.42s 收势）、`Boss::attack_clock()`（预警＋出招＋收招为一条招式时间轴，晕眩／击飞／转阶段即结束，三连后的停手不属于它）。`TrackId::{PlayerMelee{pair,stage},PlayerSkill(kind),Boss{stage},Enemy}` 只作数据表的键。表现按它采样攻击姿态，不保留渲染端攻击计时。
- `Boss::{phase,state,action_id,skill_slot,elapsed,duration,sequence,clip_name,is_invulnerable}` 是单Boss只读表现接口；`BossPhase::{One,Two}` / `BossState` 的状态由Boss独占。`clip_name()` 在原技能槽到动画映射未恢复前返回None。

## Contracts

MoveIntent 的 Vec2 表示世界 `(x,z)` 意图，长度不超过 1；物理位置和速度为完整 Vec3，单位 m / m/s。
Avian 是唯一求解器，Body 只发布脚底快照；模型和相机共用它的 previous / current 插值。
角色、动态箱体、固定场景和真实围墙统一参与三维接触，场景外坠落由复活流程恢复。
动作准备 / motor → 物理 → 当前查询与支撑 / 快照 → 判定探针计时 → 玩家动作结算 → 弹体 → 普通敌人 → Boss → 耐力恢复 → 妹妹快照 → 瞬移清理，在同一固定步明确排序。
持续按住与一次性指令分离；离散位在固定步取走，不能在零固定步的显示帧覆盖未消费请求。
疾跑、防御约束、闪避方向锁定和无敌、跳跃、强制拉近和接触落地均在模拟定义；所有移动接受三维物理约束。
普通跳跃沿用 act 的两次额度，每次将竖直速度设为 6.4m/s；第三次请求消费后忽略，落地 / reset 恢复次数。
清理输入、暂停和击飞不能补回已用次数；闪避 / 进攻动作期间不接受普通跳跃，也不能把请求排队到结束后执行。
Shift 首次按下闪避，按住 0.16s 识别疾跑；长按使闪避由 0.42s 缩短为 0.26s，再衔接 1.6 倍疾跑。
短闪使用 smoothstep 累积 2.4m 的运动目标，实际位移接受物理约束；静止朝 Facing 反方向，无敌仅在经过 0.06–0.28s 窗口的固定步有效；闪避冷却 0.8s。
疾跑锁存跨 Shift 松开保留，仅所有方向键释放才取消；物理方向存在但向量抵消为零不能取消。
清理意图会锁存疾跑取消请求，不能因暂停无固定步或后续 set_held 覆盖而丢失；客户端不能直接修改 MotionState。
方向键 / Shift 释放也必须锁存到固定步，后续按住状态不能覆盖释放事实；新闪避按压重置连续长按计时但不清已有疾跑锁存。
闪避时间始终单调增加，长按导致持续时间缩短时不能把已执行时间压回 0.26s，否则 smoothstep 差值会造成反向位移。
C 按压以 XOR 奇偶性锁存，固定步消费后切换蹲伏；松键保持，蹲行固定 1.6m/s，蹲伏不能疾跑但保留已有疾跑锁存。
非防御的疾跑中 C 启动 0.55s 滑铲，5m/s 线性衰减目标（理想总长 1.375m）、起始世界方向锁定，实际位移由接触限制；结束保持蹲伏，C 可提前站立结束。
接受跳跃 / 闪避、击飞和重置退出蹲伏；无效动作不能误清姿态。清理 Intent 在恢复首步退出蹲伏 / 滑铲，不重置空中或闪避进度。
滑铲中禁止新的攻击 / 起手 / 联动，普通蹲伏仍使用现有攻击和防御规则；不增加躲避伤害或越过障碍的权限。
武器组、两种联动、三发标记、防御与耐力的唯一详细契约见 [combat-v1.md](combat-v1.md)。
锤 E 击飞沿共享重力提供约 3m 相对上升，不能在客户端抬模型；身体范围仍用实际三维位置。
死亡实体不主动移动、不选中、不结算新攻击；3s 后重置权威物理姿态 / 速度与 previous/current，清理排队意图后复活。
被动靶子每步停止主动移动、不进入蓄力 / 攻击，仍接受伤害、击飞和碰撞；死亡复活保留被动身份。
反馈最多 128 条，sequence 严格递增，客户端可跨多固定步消费，不能在每个模拟步清空历史。
每条 CombatEvent 包含明确 actor：攻击为发起者，受击 / 防御 / 倒地 / 复活为接受者；敌人蓄力和实际挥击分别产生事件。
骨骼动画只读取这些接口，不产生位置、高度或伤害，不把动画片段时长当成技能权威计时。

## Validation & Error Matrix

| 输入 | 行为 |
|---|---|
| 构造配置非有限 / 非正尺寸与速度 | assert，属于内容错误 |
| 运行时方向非有限 | 归零 |
| 方向长度大于 1 | 限制到 1 |
| 撞墙 / 跨越有限围墙 | 按实际三维碰撞阻挡 / 滑动；跌出场景后复活，不设置无限高边界 |
| 非法目标位置 / 空中重复起手 | 清理或拒绝请求，不污染动作状态 |
| 联动超时 / 目标死亡或落地 / 重复 Tab | 不释放旧联动，不重复伤害 |
| 防御背面来袭 / 闪避无敌结束 | 正常受伤；有效窗口内正面拆招／弹反免伤，无敌期拒绝伤害 |
| 失去捕获 | 由客户端设 SimulationActive=false，整条权威链停止推进 |

出生点必须由装配者保证可容纳整个占地；不支持把 Body 直接生成在实体内部。

## Good / Base / Bad Cases

Good：高速斜向移动受真实墙面阻挡，继续沿墙滑行；足够高时能越过低障碍。
Base：平地直行和斜行追踪相同的 6m/s 目标，接触与加速按物理容差检查。
Bad：NaN 输入不移动，内容无效时构造失败。

## Tests Required / Quality Check

执行 `make check`。固定步测试直接推进实际 FixedUpdate；验证一秒位移、静止、无效输入、
高速防穿透、贴面离开 / 滑行、角落和有限围墙，不能只复制实现公式。
动作测试覆盖真实起跳 / 落地、疾跑速度、防御、闪避方向 / 冷却 / 无敌、拉近 / 墙体和暂停恢复。
二段跳覆盖上升 / 下落时二次起跳、拒绝第三次、落地恢复、暂停 / 输入清理不补次数、重置恢复，以及闪避 / 进攻动作隔离。
Shift 规则测试覆盖短按不锁疾跑、长按衔接、松 Shift 保留 / 松方向取消、抵消方向键、零固定步释放、
重新按压的独立计时、后段长按不倒退、以及失活取消不重置空中进度。
战斗回归按 combat-v1.md 覆盖两套技能／三发联动、防御耐力、原目标／来源、独立冷却、取消和实际拉近碰撞。
命中停顿用真实 FixedUpdate 验证角色与下落箱体的位置 / 速度 / 快照 / 冷却一起冻结 3 步，第四步恢复；失锁保留预算，普通射击不触发。
击飞回归通过真实物理测量平地与非零高台的相对最高点为 3m（允许固定步误差），靶子回归覆盖复活后仍不攻击。
蹲伏 / 滑铲测试覆盖按压奇偶性、1.6m/s 实际位移、方向锁定与减速积分、墙体 / 边界、起身 / 动作中断、暂停 / 死亡复活清理，以及现有攻击 / 防御 / 拉近链路。

## Wrong vs Correct

错误：在按键或显示系统修改物理 Position / Transform，或者独立积分 Body 高度。
正确：提交 Intent，由 FixedUpdate 驱动物理求解与战斗，显示只写 owner 关联的视图根。

DefenseState 独占拆招／防御／弹反，MotionState 不保存第二份防御窗口。MotionState::reset 保留实际胶囊高度，不能以默认身高重新解释放大角色的中心。

Body::aim_offset()/aim_position() 统一按标准胸口0.9m×scale给出锚点；范围仍比较脚底，视线/命中/狙击与客户端瞄准使用相同实际锚点，玩家普通相机构图不变。

产品实战挂载Boss组件，普通Enemy主动AI和复活循环必须跳过它，不能镜像维护一份Enemy.windup。Boss决策排在玩家战斗之后、瞬移清理之前，仍接受SimulationActive/HitStop同一门控。

Boss 的决策是原作行为树的逐节点转写，集中在 `simulation/boss/source.rs`（`Mind` 为黑板）：P1/P2 主树 `51000025_BOSS01_fight_P1`／`51000031_BOSS01_fight_P2` 的 `ConditionPriorityNode` 顺序为连招 > 格挡反制（`GedangCondition`，目标格挡了上一击时交替出 6/7）> `attack_count` 未满（P1 3、P2 5）时按距离池选招 > 否则游走 1–2s 并清零计数；动作分派树给出 57 个技能槽、计入 `attack_count` 的动作和冷却；连招树按技能槽末三位标签与距离设置 `BOSS01_Combo`，招式最后一个有效窗口过后立即检查，成立则跳过收招直接接下一招。开场（`PassNode`）P1 游走 2s 后出 45，P2 直接出 21；P2 生命 ≤75% 每 70s 一次终结技 60（6m 外记账，由下一招 22 的标签 605 补出）；`CheckTimerDecorator` 首次立即通过，之后按周期。动作 1 游走 3s，2／3 逼近到 6／5m（5s 超时）。动作 1/2/3 与打满一轮后的游走开始前，若目标在 Boss 朝向的 135°–225° 范围内（`JudgeTargetInRangeAction`），先进入 `BossState::Turning` 原地转身（`TurningAction`，3s 超时；转速原作取转身动画速度，未恢复，Kiva 用 180°/s），转到对准后才移动；开场游走与出招不做这一判断，出招中方向在预警 65% 后锁定。动作号、权重、距离、计时都取原值；唯一改动的原数是冷却乘 `COOLDOWN_SCALE`（0.25，Kiva 适配：全长时近距离池约四招后耗尽，Boss 三分之一时间在游走）。依赖 Kiva 没有的系统的分支（成就增益 450100/吸魂 27、自身 450079、金身受击 34、锁定/闪避/80/81 事件标签、障碍探测）不实现，代码里注明。

每个动作号经 `STRIKE_OF` 映射到 `Boss::STRIKES` 的一行（17 招，Kiva 适配：槽位→片段映射未恢复，P1 动作用 P1 片段、P2 动作用 P2 片段、反制类用最短的几招），`TrackId::Boss { strike }` 以招式行索引片段和烘焙数据。命中体积与出刀时刻不在这里调，取自每一招的烘焙刀刃轨迹：预警持续到刀刃进入第一个有效窗口（0.38–1.0s），出招持续到最后一个窗口结束，无连招时收招补足该招时长。预警前 65% 转向目标，之后方向锁定但继续逼近，直到目标进入 `HitTrack::frontal_reach` 的 75%（出招时为 87.5%）：在预警首个 tick 用目标真实受击胶囊求出刀刃仍能扫到正前方目标的最远脚距；多段原作招式只在侧面摸到最大 `reach`，不能用它决定停步距离。每招对目标只结算一次。未知伤害集中在 KivaTuning（P1/P2 14/18、每阶段生命 650）；两阶段生命属于 Kiva 适配，不能把原树的 65/75/30% 条件当阶段阈值，转阶段换一份新黑板。转阶段与击败时伤害/击飞统一拒绝；弹反将 Boss 置 Stunned，攻击者停手，不能由客户端动画决定命中。普通敌人的攻击同样是一条 0.84s 的时间轴：刀刃进入有效窗口前为蓄力，之后扫到谁就结算谁，整段站定；接敌距离为 `reach`＋目标半径的 75%。Boss无存活玩家时重置遭遇；死亡、复活和模式切换清理意图与目标，不遗留旧攻击。回归使用真实FixedUpdate覆盖开场/距离池/连招/计数游走/格挡反制、单次命中、正反防御、无敌/遮挡、阶段/死亡/复活、暂停/HitStop及模式切换。
