# 模拟层

## Scope / Trigger

`src/simulation/` 拥有权威运动、动作、武器、伤害、敌人和复活；变更这些规则或固定步时阅读。
不依赖客户端、浏览器、窗口、相机、Transform 或具体关卡。

## Signatures

- `SimulationPlugin` 注册 60Hz `FixedUpdate`。
- `Body::new(position: Vec2, speed: f32, half_extent: f32)`；`position()`、`interpolated(alpha)` 只读。
- `Body::step_displacement()` 只读实际碰撞后最近固定步位移；MotionState 的 `dodge_elapsed/dodge_direction()` 只读闪避进度 / 方向，用于表现。
- `MoveIntent::set(direction: Vec2)` / `stop()`。
- `CollisionWorld::new(half_size: f32, solids: Vec<SolidBox>)`；`is_clear(point, half_extent)`。
- `SolidBox::new(center: Vec2, half_size: Vec2)`。
- `ActionIntent::set_held(physical_shift, block)` / `set_movement_held(bool)` / `request_jump()` / `request_dodge()` / `clear()`。
- `Facing::set(Vec2)` / `direction()`；`MotionState` 只读状态与高度插值，`start_leap(origin,target,duration,peak)` / `launch_up(velocity)` 由模拟规则调用。
- `CombatIntent::set_aim(Vec2)`、`request_attack/skill/combo/switch_weapon/cycle_target()`、`request_set(WeaponSet)`、`clear()`。
- `Combatant::new(WeaponSet)`；`weapon_set()`、`weapon_mode()`、`target()`、`aim()`、`cooldown()`、`combo_remaining()` 等只读。
- `Health::new(maximum)` / `current/maximum/is_alive()`；`Enemy::default()`、`SpawnPoint(Vec2)`。
- `CombatFeedback::events()` 为只读有界历史；`SimulationActive(bool)` 暂停整条固定步链。

## Contracts

Vec2 表示世界 `(x,z)`，速度单位 m/s。Body 字段私有；每固定步先存 previous 再进行扫掠与滑动。
MoveIntent 只传世界方向，长度不超过 1。模型和相机共用 previous / current 的显示插值。
碰撞把静态盒扩张人物半宽，最多处理四次接触；世界边界限制整个占地，滑动保留切向位移。
动作 / 高度和位置在同一固定步推进，随后结算战斗与敌人决策。无动作组件的 Body 保持原移动行为。
持续按住与一次性指令分离；离散位在固定步取走，不能在零固定步的显示帧覆盖未消费请求。
地面疾跑 / 格挡、闪避方向锁定和无敌、重力落地、火炮弧线均在模拟定义；水平位移始终走扫掠。
Shift 首次按下闪避，按住 0.16s 识别疾跑；长按使闪避由 0.42s 缩短为 0.26s，再衔接 1.6 倍疾跑。
短闪沿 smoothstep 总移动 2.4m，静止朝 Facing 反方向；无敌仅在经过 0.06–0.28s 窗口的固定步有效。
疾跑锁存跨 Shift 松开保留，仅所有方向键释放才取消；物理方向存在但向量抵消为零不能取消。
清理意图会锁存疾跑取消请求，不能因暂停无固定步或后续 set_held 覆盖而丢失；客户端不能直接修改 MotionState。
方向键 / Shift 释放也必须锁存到固定步，后续按住状态不能覆盖释放事实；新闪避按压重置连续长按计时但不清已有疾跑锁存。
闪避时间始终单调增加，长按导致持续时间缩短时不能把已执行时间压回 0.26s，否则 smoothstep 差值会造成反向位移。
锤子联动保存原选中 Entity，Tab 不被当前 Q 目标替换；火炮旋斧沿弧线每个敌人仅伤害一次。
联动需有效时间和活目标 / 未落地状态；切套装或手动切武器取消；起手技能冷却不被切换刷新。
火炮跃进期间 aim() 固定为起跳时承诺的落点，不能把之后输入的前方点当成当前落点；格挡中 Tab 消耗窗口且不补放。
死亡实体不移动、不选中、不结算新攻击；3s 后 Body 内部 teleport 同步 previous/current，清理排队意图后复活。
反馈最多 128 条，sequence 严格递增，客户端可跨多固定步消费，不能在每个模拟步清空历史。
每条 CombatEvent 包含明确 actor：攻击为发起者，受击 / 防御 / 倒地 / 复活为接受者；敌人蓄力和实际挥击分别产生事件。
骨骼动画只读取这些接口，不产生位置、高度或伤害，不把动画片段时长当成技能权威计时。

## Validation & Error Matrix

| 输入 | 行为 |
|---|---|
| 构造配置非有限 / 非正尺寸与速度 | assert，属于内容错误 |
| 运行时方向非有限 | 归零 |
| 方向长度大于 1 | 限制到 1 |
| 撞墙 / 到地图边缘 | 阻挡法线运动，保留可行切向运动 |
| 非法目标位置 / 空中重复起手 | 清理或拒绝请求，不污染动作状态 |
| 联动超时 / 目标死亡或落地 / 重复 Tab | 不释放旧联动，不重复伤害 |
| 格挡背面来袭 / 闪避无敌结束 | 正常受伤；正面格挡减伤，无敌期拒绝伤害 |
| 失去捕获 | 由客户端设 SimulationActive=false，整条权威链停止推进 |

出生点必须由装配者保证可容纳整个占地；不支持把 Body 直接生成在实体内部。

## Good / Base / Bad Cases

Good：斜向扫掠很长距离仍停在墙前，继续沿墙滑行。
Base：1 秒直行或斜行都为 6m。
Bad：NaN 输入不移动，内容无效时构造失败。

## Tests Required / Quality Check

执行 `make check`。固定步测试直接推进实际 FixedUpdate；验证一秒位移、静止、无效输入、
长步防穿透、贴面离开 / 滑行、角落和地图边界，不能只复制实现公式。
动作测试覆盖真实起跳 / 落地、疾跑速度、格挡、闪避方向 / 冷却 / 无敌、火炮弧线 / 墙体和暂停恢复。
Shift 规则测试覆盖短按不锁疾跑、长按衔接、松 Shift 保留 / 松方向取消、抵消方向键、零固定步释放、
重新按压的独立计时、后段长按不倒退、以及失活取消不重置空中进度。
战斗测试覆盖两条完整联动、失效和一次消费、沿途旋斧、正背面格挡、倒地复活清理和反馈历史上限。

## Wrong vs Correct

错误：在按键系统修改 Body / Transform 作为物理位置。
正确：提交各类 Intent，由 FixedUpdate 唯一推进 Body / MotionState / 战斗状态，显示只读插值。
