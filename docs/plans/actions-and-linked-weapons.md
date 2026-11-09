# 动作与双人格武器实现方案

## 目标与范围

落实 `docs/TODO.md` 的动作、两套武器和 Tab 联动，形成可直接操作的庭院战斗 Demo。
保留现有 16:9 外框、镜头比例、鼠标水平转向、WASD 和单 crate 分层。
本次不引入外部资源、联网、背包、装备词条或通用技能脚本框架。
沿用本会话不创建 Trellis 任务的选择；用户已授权先规划再自主实现。

## 实现顺序

1. **权威动作与高度**：疾跑、跳跃、锁定方向的闪避、格挡、固定步重力和显示插值。
2. **战斗基础**：近远程模式、哥哥/妹妹人格、套装、目标锁定、生命、伤害与可复活的练习敌人。
3. **联动规则**：锤子击飞 → 限时 Tab 狙击；火炮跳向瞄准位置 → 空中 Tab 斧子旋转。
4. **客户端呈现**：武器外形、空中动作、目标标识、命中反馈、技能特效和状态 HUD。
5. **验证与文档**：真实 FixedUpdate 无窗口回归、输入桥 Node 测试、Clippy、WASM 和临时目录 Trunk 构建。

禁止浏览器测试和启动/关闭开发服务。此前纯镜头调整不测试的约定不替代本次新战斗规则的无窗口验证。

## 架构与所有权

- `simulation`：唯一推进位置、高度、动作、伤害、武器冷却、目标和联动窗口。只接收世界方向/位置和离散意图，不读取设备、DOM、Transform 或 gameplay。
- `gameplay`：装配玩家、练习敌人与出生点，保留 Level 几何的单一数据源。
- `client`：采样输入、读取模拟快照和显示特效。显示高度与平面位置一起插值，相机仍以相同人物显示位置为依据。
- `web`：可信输入/捕获边界；持续状态与一次性指令分开，失锁/失焦全部清空，单次输入跨零个或多个固定步不丢失、不重放。

不把规则放进 JS，也不让模型位置反写模拟。模拟按 movement → combat 固定顺序执行；客户端按角色同步 → 战斗显示/HUD → 相机执行。

## 操作与规则

| 操作 | 行为 |
|---|---|
| Shift 点按 / 长按 | 按下即闪避；按住 0.16s 后锁定疾跑，短闪 0.26s 结束后衔接；松 Shift 继续跑，松所有方向键停止 |
| Space | 地面跳跃，不允许空中重复起跳 |
| 鼠标右键按住 | 地面格挡，降低前方来袭伤害；松开/失焦解除 |
| 滚轮 | 一次有效滚动切换近战哥哥/远程妹妹，不改变所选套装 |
| 鼠标上下（火炮模式） | 选择落点距离，默认 6m，范围 1–8m；左右仍调整方向，镜头俯角固定 |
| 1 / 2 | 锤子+狙击枪 / 斧子+火炮；切换套装清除旧联动 |
| 左键 | 当前武器普攻，有范围、冷却和命中反馈 |
| Q | 在可用敌人之间切换锁定；无锁定时按面朝方向自动选目标 |
| E | 当前武器起手技能 |
| Tab | 仅窗口有效时释放联动，一次窗口只能消费一次 |

锤子技能要求选中近处敌人，以其为中心击飞附近敌人；击飞期间 Tab 切换妹妹并狙击原锁定目标。
火炮技能跳向面朝方向的地面瞄准点；空中 Tab 切换哥哥，以斧子旋转攻击途中附近敌人。
地面瞄准标记实时显示目标落点；起跳后冻结已经选定的落点，不能随移动或新瞄准漂移。
所有强制位移仍经过现有静态碰撞和地图边界。
敌人提供接近、攻击与伤害反馈，死亡后在出生点复活，玩家倒地后也可继续练习。

## 并行接口契约

动作模块导出 `ActionIntent`、`MotionState`、`Facing`：

- `ActionIntent::set_held(physical_shift, block)`、`set_movement_held(bool)`、`request_jump()`、`request_dodge()`、`clear()`。
- `Facing::set(Vec2)` / `direction()`，默认世界 -Z。
- `MotionState::height()`、`interpolated_height(alpha)`、`is_grounded()`、`is_dodging()`、`is_blocking()`、`is_sprinting()`、`is_leaping()`。
- `MotionState::start_leap(origin: Vec2, target: Vec2, duration: f32, peak: f32)`；`launch_up(velocity: f32)`；`reset()`。
- `Body::teleport(Vec2)` 为 `pub(super)`，仅供模拟内部复活使用，同步 previous/current。
- `movement::advance` 对没有动作组件的既有 Body 保持原行为。

战斗模块导出 `WeaponSet::{HammerSniper,AxeCannon}`、`WeaponMode::{Melee,Ranged}`、`Combatant`、`CombatIntent`、`Health`、`Enemy`、`SpawnPoint`、`CombatFeedback`：

- `CombatIntent::set_aim(Vec2)`、`request_attack()`、`request_skill()`、`request_combo()`、`request_switch_weapon()`、`request_set(WeaponSet)`、`request_cycle_target()`、`clear()`。
- `Combatant::new(WeaponSet)`、`weapon_set()`、`weapon_mode()`、`target()`、`combo_remaining()`、`cooldown()`、`aim()`；HUD 只读这些状态。
- `Health::new(max)`、`current()`、`maximum()`、`is_alive()`。
- `Enemy` 为标记，`SpawnPoint(pub Vec2)`；战斗需要的组件由玩法统一装配。
- `CombatFeedback` 公开只读帧间反馈快照，具体类型由战斗实现者补充并告知主会话；不能在显示帧间丢失事件。

## 验收

- 疾跑速度增加、跳跃落地、闪避方向和无敌、正面格挡减伤可在实际固定步验证。
- 捕获丢失后没有残留按键、攻击、切换或联动；Tab 不触发浏览器焦点跳转。
- 滚轮在同一套装内切换两人格，1/2 切换套装并取消旧联动。
- 两条联动能够完整执行；超时、目标死亡、重复 Tab、空中重复技能均不能错误触发。
- 火炮落点可用鼠标选择方向及距离，标记与实际落点一致，起跳后不漂移；格挡时按 Tab 是立即尝试并消耗窗口，不延迟补放。
- 角色/敌人/目标/技能反馈可见，HUD 显示人格、武器、生命、冷却和 Tab 窗口。
- 原有墙体扫掠、地图边界、显示插值及镜头构图不回退；所有新规则可无窗口运行。

## 执行记录

- [x] 读取需求、现有架构和各层规范，确定顺序与接口。
- [x] 动作与高度。
- [x] 战斗、武器、联动与练习敌人。
- [x] 输入、表现、HUD。
- [x] 无浏览器验证与规范同步。

## 初版验证结果（2026-11-09）

- `make check` 通过：格式检查、原生 Clippy、35 项 Rust 测试、21 项 Node 测试、WASM 检查。
- `cargo clippy --locked --target wasm32-unknown-unknown -- -D warnings` 通过。
- Trunk 临时目录构建通过，检查产物含入口、WASM、input / startup / hud / style；临时产物已自动清理。
  本机 Trunk 0.17.5 不接受 `--locked`，因此以 `CARGO_NET_OFFLINE=true` 构建，并验证 Cargo.lock 哈希不变。
- 无窗口场景测试运行真实 Startup / Update，验证 ECS 装配、单一可见武器、战斗反馈消费、特效回收和资产复用。
- 交叉审查修正了暂停插值循环、复活当帧旧命令重放、火炮落点漂移，并补充可选择落点距离。
- 未使用浏览器、未启动或关闭开发服务、未提交代码。画面与手感没有进行浏览器验收；模型 / 武器 / 特效为程序化几何。

## 追加：火柴人与 Shift 操作

参考 `origin/act` 的 `bd3e344`，移植 Shift 手势与闪避曲线，不引入该分支的资源或整个战斗框架。
玩家和敌人统一使用共用火柴人构造器，保留现有武器、碰撞、镜头锚点和受击反馈。
完整短闪为 0.42s / 2.4m，静止后闪；长按识别后在 0.26s 结束短闪，再衔接疾跑。
识别和锁存均属于固定步模拟；输入只提交 Shift 物理状态、独立的方向键按住事实和一次闪避请求。

追加验证通过：`make check`（51 项 Rust、24 项 Node、格式 / 原生 Clippy / WASM 检查）、WASM Clippy、临时目录 Trunk 构建。
构建入口已确认使用新的 Shift 提示，Cargo.lock 未改变；未使用浏览器或启动开发服务。
审查补齐了左右 Shift 聚合、原生帧间重复按压、零固定步释放锁存、独立长按计时及闪避时间不回退的回归。

随后追加的真实骨骼动画资产与姿态移植见 [act 动画移植方案](act-animation-port.md)，不再仅使用固定火柴人姿态。
