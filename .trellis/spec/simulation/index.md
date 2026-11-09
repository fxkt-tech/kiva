# 模拟层

## Scope / Trigger

`src/simulation/` 拥有权威运动、动作、武器、伤害、敌人和复活；变更这些规则或固定步时阅读。
不依赖客户端、浏览器、窗口、相机或渲染资源；物理内部的 Transform 同步不属于客户端表现。
三维位置、角色控制与接触求解的详细契约见 [physics.md](physics.md)。

## Signatures

- `SimulationPlugin` 注册 60Hz `FixedUpdate`。
- `Body::new(Vec2, speed, radius)` / `Body::at(Vec3, speed, radius)`；`world_position()`、`interpolated_world(alpha)` 为三维脚底快照，`position()` 为 x/z 投影。
- `Body::step_displacement()` 只读实际碰撞后最近固定步位移；MotionState 的 `dodge_elapsed/dodge_direction()` 只读闪避进度 / 方向，用于表现。
- `MoveIntent::set(direction: Vec2)` / `stop()`。
- `spawn_box` / `spawn_terrain` 创建真实三维碰撞实体；`PhysicsPose` 为道具中心姿态只读快照。
- `CollisionWorld` / `SolidBox` 只作为旧无窗口夹具的装配适配，不用于产品关卡的运动或战斗查询。
- `ActionIntent::set_shift_held(physical_shift)` / `set_movement_held(bool)` / `request_jump()` / `request_dodge()` / `request_crouch_toggle()` / `request_parry()` / `clear()`。
- `MotionState::{is_crouched,is_sliding,slide_elapsed,slide_direction}` 只读蹲伏和滑铲状态。
- `MotionState::jumps_used()` 只读当前腾空已用跳跃次数，供二段跳表现使用。
- `Facing::set(Vec2)` / `direction()`；`MotionState` 只读接触状态与高度适配器，`start_leap_to(Vec3,Vec3,duration)` / `launch_up(velocity)` 由模拟规则调用；三维跃进按目的地、飞行时间和共享重力计算，旧二维 `start_leap(...,peak)` 仅保留参数校验适配。
- `CombatIntent::set_aim(Vec2)`、`request_attack/skill/combo/switch_weapon/cycle_target()`、`request_set(WeaponSet)`、`clear()`。
- `Combatant::new(WeaponSet)`；`aim() -> Vec3` / `aim_surface() -> Option<SurfaceHit>` 保存同源预览与承诺落点，其余武器 / 目标 / 冷却访问器只读。
- `Health::new(maximum)` / `current/maximum/is_alive()`；`Enemy::default()` 为普通主动敌人（兼容无窗口夹具），`Enemy::target_dummy()` / `is_passive()` 为被动靶子；`SpawnPoint(Vec2)`。
- `Combatant::clear_targets()` 在敌人集合替换时清理目标和联动，不刷新技能 / 普攻冷却。
- `Combatant::sniper_sequence() -> Option<SniperSequence>` / `is_sniping()` 暴露锤狙联动的只读固定步时间线；`SniperSequence::{target,elapsed,aim_position,has_fired}` 保存原目标和真实瞄准点。
- `CombatFeedback::events()` 为只读有界历史；`SimulationActive(bool)` 暂停整条固定步链。
- `HitStop::is_active()` 暴露冲击停顿状态；模拟请求 0.04s 狙击命中停顿，固定步向上取整（60Hz 为 3 步 / 50ms）。

- `Boss::{phase,state,action_id,skill_slot,elapsed,duration,sequence,clip_name,is_invulnerable}` 是单Boss只读表现接口；`BossPhase::{One,Two}` / `BossState` 的状态由Boss独占。`clip_name()` 在原技能槽到动画映射未恢复前返回None。

## Contracts

MoveIntent 的 Vec2 表示世界 `(x,z)` 意图，长度不超过 1；物理位置和速度为完整 Vec3，单位 m / m/s。
Avian 是唯一求解器，Body 只发布脚底快照；模型和相机共用它的 previous / current 插值。
角色、动态箱体、固定场景和真实围墙统一参与三维接触，场景外坠落由复活流程恢复。
动作 / motor → 物理 → 当前查询与支撑 / 快照 → 战斗 / AI → 瞬移清理，在同一固定步明确排序。
持续按住与一次性指令分离；离散位在固定步取走，不能在零固定步的显示帧覆盖未消费请求。
疾跑 / 卸势、闪避方向锁定和无敌、跳跃、火炮发射和接触落地均在模拟定义；所有移动接受三维物理约束。
普通跳跃沿用 act 的两次额度，每次将竖直速度设为 6.4m/s；第三次请求消费后忽略，落地 / reset 恢复次数。
清理输入、暂停和击飞不能补回已用次数；闪避 / 火炮跃进期间不接受普通跳跃，也不能把请求排队到结束后执行。
Shift 首次按下闪避，按住 0.16s 识别疾跑；长按使闪避由 0.42s 缩短为 0.26s，再衔接 1.6 倍疾跑。
短闪使用 smoothstep 累积 2.4m 的运动目标，实际位移接受物理约束；静止朝 Facing 反方向，无敌仅在经过 0.06–0.28s 窗口的固定步有效。
疾跑锁存跨 Shift 松开保留，仅所有方向键释放才取消；物理方向存在但向量抵消为零不能取消。
清理意图会锁存疾跑取消请求，不能因暂停无固定步或后续 set_held 覆盖而丢失；客户端不能直接修改 MotionState。
方向键 / Shift 释放也必须锁存到固定步，后续按住状态不能覆盖释放事实；新闪避按压重置连续长按计时但不清已有疾跑锁存。
闪避时间始终单调增加，长按导致持续时间缩短时不能把已执行时间压回 0.26s，否则 smoothstep 差值会造成反向位移。
C 按压以 XOR 奇偶性锁存，固定步消费后切换蹲伏；松键保持，蹲行固定 1.6m/s，蹲伏不能疾跑但保留已有疾跑锁存。
非卸势的疾跑中 C 启动 0.55s 滑铲，5m/s 线性衰减目标、起始世界方向锁定，实际位移由接触限制；结束保持蹲伏，C 可提前站立结束。
接受跳跃 / 闪避 / 跃进、击飞和重置退出蹲伏；无效动作不能误清姿态。清理 Intent 在恢复首步退出蹲伏 / 滑铲，不重置空中或闪避进度。
滑铲中禁止新的攻击 / 起手 / 联动，普通蹲伏仍使用现有攻击和卸势规则；不增加躲避伤害或越过障碍的权限。
锤子联动保存原选中 Entity，Tab 不被当前 Q 目标替换；火炮旋斧沿实际运动轨迹每个敌人仅伤害一次。
有效锤子 Tab 先消费窗口、切换狙击枪并进入 SniperSequence，0.30s 瞄准后才产生一次 Shot 和 85 伤害；随后保持 0.14s、恢复 0.28s，总计 0.72s。伤害时机由 FixedUpdate 决定，不等待渲染回调。
开枪前重新检查原目标存活 / 浮空、动作状态与视线；手动切套装 / 武器、倒地、目标移除及模式替换清理未发射的动作，不补射。已由本次狙击击倒的目标保留射后恢复时间。
序列期间新普攻 / 起手 / Tab 不排队，起手冷却不刷新；暂停冻结序列。狙击 aim_position 随原目标物理位置更新，不因 Q 改选另一个敌人。
成功狙击保留普通 Shot / Hit，再追加 `SniperShot`（actor=射手、position=射手胸口、end=命中点）和 `SniperImpact`（actor=受击者、position=命中点、end=射手胸口），不重复伤害；取消和普通攻击不生成专用事件。
HitStop 在固定步开头锁定本步门控，统一冻结 motor、物理求解、快照、战斗 / AI / 冷却及 SniperSequence；命中后请求预算不能改变本步决定，以保证同一步瞬移清理仍执行。失锁不消耗预算，结束只恢复正常固定步，不补算重力或改写速度。
狙击击倒当前选中目标时，伤害同一步立即清理 Combatant.target，不能等 HitStop 后才移除死亡选择；射后序列独立保留原目标和最后瞄准点。
锤子击飞按共享重力 18m/s² 提供约 10.3923m/s 的向上初速度，目标从实际起飞高度上升约 3m；地形和顶面碰撞仍由物理决定。
联动需有效时间和活目标 / 未落地状态；切套装或手动切武器取消；起手技能冷却不被切换刷新。
火炮跃进期间 aim() 固定为起跳时承诺的落点，不能把之后输入的前方点当成当前落点；卸势中 Tab 消耗窗口且不补放。
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
| 卸势背面来袭 / 闪避无敌结束 | 正常受伤；有效窗口内正面卸势免伤，无敌期拒绝伤害 |
| 失去捕获 | 由客户端设 SimulationActive=false，整条权威链停止推进 |

出生点必须由装配者保证可容纳整个占地；不支持把 Body 直接生成在实体内部。

## Good / Base / Bad Cases

Good：高速斜向移动受真实墙面阻挡，继续沿墙滑行；足够高时能越过低障碍。
Base：平地直行和斜行追踪相同的 6m/s 目标，接触与加速按物理容差检查。
Bad：NaN 输入不移动，内容无效时构造失败。

## Tests Required / Quality Check

执行 `make check`。固定步测试直接推进实际 FixedUpdate；验证一秒位移、静止、无效输入、
高速防穿透、贴面离开 / 滑行、角落和有限围墙，不能只复制实现公式。
动作测试覆盖真实起跳 / 落地、疾跑速度、卸势、闪避方向 / 冷却 / 无敌、火炮弧线 / 墙体和暂停恢复。
二段跳覆盖上升 / 下落时二次起跳、拒绝第三次、落地恢复、暂停 / 输入清理不补次数、重置恢复，以及闪避 / 跃进隔离。
Shift 规则测试覆盖短按不锁疾跑、长按衔接、松 Shift 保留 / 松方向取消、抵消方向键、零固定步释放、
重新按压的独立计时、后段长按不倒退、以及失活取消不重置空中进度。
战斗测试覆盖两条完整联动、失效和一次消费、沿途旋刀、正背面卸势、倒地复活清理和反馈历史上限。
狙击回归覆盖 Tab 不立即伤害 / 发 Shot、0.30s 后仅命中原目标一次、重复指令、暂停 / 恢复、切套装 / 武器 / 目标移除 / 死亡 / 遮挡取消，以及非零海拔和射后恢复。
命中停顿用真实 FixedUpdate 验证角色与下落箱体的位置 / 速度 / 快照 / 冷却一起冻结 3 步，第四步恢复；失锁保留预算，普通射击不触发。
击飞回归通过真实物理测量平地与非零高台的相对最高点为 3m（允许固定步误差），靶子回归覆盖复活后仍不攻击。
蹲伏 / 滑铲测试覆盖按压奇偶性、1.6m/s 实际位移、方向锁定与减速积分、墙体 / 边界、起身 / 动作中断、暂停 / 死亡复活清理，以及现有攻击 / 卸势 / 跃进链路。

## Wrong vs Correct

错误：在按键或显示系统修改物理 Position / Transform，或者独立积分 Body 高度。
正确：提交 Intent，由 FixedUpdate 驱动物理求解与战斗，显示只写 owner 关联的视图根。

卸势以 request_parry 单次锁存，地面/存活/非闪避跃进滑铲且非狙击/旋转时接受，窗口0.25s、冷却0.65s。窗口外和侧后方近战正常12伤害，窗口内正面免伤并发Deflect（actor=防守者），攻击者停止预警/移动，恢复0.45s。按住或冷却内重复请求不延长/排队重放；旧持续格挡减伤已移除。窗口内移动0.35倍，所有计时受同一固定步与HitStop门控；clear在恢复首步取消窗口且保留冷却，死亡即时取消，复活清空计时/输入。MotionState::reset保留实际胶囊高度，不能以默认身高重新解释放大角色的中心。

Body::aim_offset()/aim_position() 统一按标准胸口0.9m×scale给出锚点；范围仍比较脚底，视线/命中/狙击与客户端瞄准使用相同实际锚点，玩家普通相机构图不变。

产品实战挂载Boss组件，普通Enemy主动AI和复活循环必须跳过它，不能镜像维护一份Enemy.windup。Boss决策排在玩家战斗之后、瞬移清理之前，仍接受SimulationActive/HitStop同一门控。已恢复P1/P2距离池、权重、行动槽和零权重冷却；未知伤害/体积/预警/持续时间集中在KivaTuning。两阶段生命属于Kiva适配，不能把原树的65/75/30%增益条件当阶段阈值。转阶段与击败时伤害/击飞统一拒绝；卸势将Boss置Deflected，攻击者停手，不能由客户端动画决定命中。Boss无存活玩家时重置遭遇；死亡、复活和模式切换清理意图与目标，不遗留旧攻击。回归使用真实FixedUpdate覆盖决策/预警/单次命中、正反卸势、无敌/遮挡、高差、阶段/死亡/复活、暂停/HitStop及模式切换。
