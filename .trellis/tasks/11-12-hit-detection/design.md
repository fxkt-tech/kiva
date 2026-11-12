# 设计：攻击判定统一

依据 prd.md 的 D1–D5。三个子任务共用这份设计，各子任务的 prd.md 引用对应章节。

## 0. 原则

- **模拟层权威。** 命中只由固定步里的几何相交决定；客户端不回调、不上报命中。
- **一份时间轴。** 每个攻击有一条以模拟动作时间（秒，60Hz tick）为横轴的时间轴。动画按这条时间轴采样，判定数据也在这条时间轴上烘焙。
- **数据和代码分开。** 招式内容（片段映射、时长、伤害、控制、弹体参数）是数据；判定、扫掠、弹体推进是通用代码，新增招式不改代码。

## 1. 攻击时间轴（子任务 anim-sim-clock）

### 1.1 模拟层发布统一快照

新增只读 `AttackClock { action_id, track: TrackId, elapsed, duration }`，三类攻击者都发布这一个组件：

- 玩家：由 `ActionSnapshot` 派生（普攻按 `(pair, stage)`，技能按 `ActionKind`）。
- Boss：把预警、出招、收招合成**一个招式时间轴** `move_elapsed / move_duration`。`BossState::{Telegraph, Attacking, Recovering}` 改为由招式时间和烘焙出的有效窗口推出，不再各自 `enter()` 计时。三连击后 3s 的停手不属于招式时间轴。
- 普通敌人：蓄力加出招合成一个时间轴。

`TrackId` 是模拟层枚举：`PlayerMelee { pair, stage }`、`PlayerSkill(ActionKind)`、`Boss { stage }`、`Enemy`。它只是数据表的键，不携带表现信息。

### 1.2 客户端片段映射表

在 `client/loadout.rs` 旁新增 `client/attack_clips.rs`，每个 `TrackId` 一行：`AttackClip { rig, clip, from_s, to_s }`。动作时间 `[0, duration]` 线性映射到片段的 `[from_s, to_s]`。

- 姿态选择对技能、普通敌人、Boss 一律按 `AttackClock.elapsed` 查表采样，删除对它们的渲染端 `clock` 计时。
- 现有的分段比例（Boss 预警 0–45%、技能 `0.9 + clock/D × 0.9`）作为第一版映射的初始值写进表里，保证手感起点不变。
- `tests/layering.rs` 允许这张表点名具体技能，理由同 `loadout.rs`。

### 1.3 混合的确定性

起手时上一姿态到新片段的 0.08s 混合保留。烘焙从规定的前置状态（站立 idle，连段第 2、3 段从前一段末尾）开始以 1/60s 回放，不依赖渲染帧率。过期校验测试比较的就是这条确定路径。

## 2. 离线烘焙（子任务 melee-sweep）

### 2.1 产物

生成文件 `src/simulation/combat/hit_tracks.rs`，文件头注明“由 `make bake-hits` 生成，勿手改”。每个 `TrackId` 一条：

```rust
pub(super) struct HitTrack {
    pub duration: f32,                 // 动作时长，与技能表一致性由测试检查
    pub samples: &'static [[f32; 6]],  // 每 tick 一帧：握点 xyz、刀尖 xyz（角色局部坐标，脚底为原点，-Z 为前）
    pub windows: &'static [(u16, u16)],// 有效窗口 [起始 tick, 结束 tick]，每个窗口一个 hit_index
    pub blade_radius: f32,             // 刀刃厚度余量
    pub reach: f32,                    // 有效窗口内刀尖的最远水平距离，供 AI 和准入使用
}
```

### 2.2 烘焙过程

- 入口：`make bake-hits`，运行一个默认忽略的客户端测试 `bake_hit_tracks`，把结果写入上述文件。放在测试里，是因为 `bundle` 只在测试构建里直接读 `assets/`，这样不必把文件读取路径带进产品构建。
- 对每个 `TrackId`：用 `PreviewRig` 同一套 `RigPose::frame`，从 §1.3 的前置状态按 1/60s 回放 `AttackClock`。每 tick 用标准外观的 `attachment_pose` 和 `stickman::attachment_transform` 求武器变换，取握点和刀尖（`Y × tip`）。
- 标准外观（D4）：玩家用默认外观，Boss 和普通敌人用愁铃客。刀尖长度从武器网格 +Y 的最大值算出，Boss 长刀从 `weapon1` 网格算出。
- 有效窗口：刀尖线速度 ≥ 阈值（初值 6 m/s）的连续 tick，合并间隔不超过 2 tick 的窗口，丢弃短于 2 tick 的窗口。阈值和合并参数写在烘焙代码顶部的常量里。
- `blade_radius`：武器网格在握点轴垂直方向上的最大半径，再加上外观差异余量（初值 0.08m）。

### 2.3 过期校验

普通测试 `hit_tracks_match_animation` 会重新烘焙，并与已提交的数据比较：采样点误差 ≤ 1cm，窗口完全一致，`duration` 与技能表一致。不一致时测试失败，并提示运行 `make bake-hits`。改片段映射、换动画、换武器模型、改时长，都会由这个测试拦下。

## 3. 近战扫掠判定（子任务 melee-sweep）

### 3.1 受击体

目标的受击体就是它的物理胶囊：脚底为底，高度取当前 `MotionState` 高度（蹲伏时较矮）乘以 `scale`，半径为 `Body::radius()`。它和 Avian 碰撞体同源，不另建一份。

### 3.2 每 tick 的检测

1. 动作处在有效窗口内时，取上一 tick 和本 tick 的刀刃线段，用攻击者当前的脚底位置和 `Facing` 变换到世界坐标。
2. 在两条线段之间插值成 N 条线段，N 按刀尖位移除以 `blade_radius` 向上取整（至少 1，最多 8），使任何高速挥击都不会漏检。
3. 对每个候选目标，计算线段与胶囊轴线段的最近距离，≤ `blade_radius + 目标半径` 即为接触。接触点取最近点对的中点。
4. 遮挡：从攻击者 `aim_position` 向接触点做射线，排除攻击者、目标和传感器；被挡住则不算接触。
5. 去重：每个动作持有 `HitLedger`，记录已经命中的 `(目标, hit_index)`。同一窗口对同一目标只结算一次，不同窗口可以各结算一次（多段招式）。
6. 结算：`HitContext.origin` 改为接触点，然后照旧调用 `apply_hit` / `settle_defense` / `hit_enemy`。正面判定（`> 0.25`）用接触点相对目标的方向。

### 3.3 三类攻击者

- 玩家普攻：删除 `MeleeProfile.{range, hit_time}` 和 `MELEE_ARC`，保留 `damage` 和 `duration`。
- 玩家范围技（锤 E、链斧 E）：刀刃扫掠照常命中。冲击范围 `Area` 保留原来的形状和半径，但结算时刻 `Area.time` 改为读取烘焙轨迹第一个有效窗口的结束 tick，不再手写。
- Boss：删除 `KivaTuning::{ACTIVE, HIT_RANGE, HIT_HALF_ANGLE, HIT_HEIGHT}`。AI 改为在距离 ≤ `track.reach + 目标半径` 时出招，出招中的前冲规则保留。
- 普通敌人：同上，删除 `REACH` 的命中用途，保留它作为追击停止距离。

### 3.4 调试显示

- 模拟层发布有界的只读 `HitProbes` 资源，保存最近 0.5s 内的扫掠线段、受击胶囊和接触点，以及它们的 `actor` 和 `action_id`。它只是事实快照，不参与规则。
- 客户端新增 `client/hit_debug.rs`，用 Gizmos 绘制，由开发热键切换。具体用哪个键，实现时先查 `controls.rs` 里还空着的键位再定。默认关闭。

## 4. 真实弹体（子任务 projectiles）

### 4.1 数据

`Shot` 和 `Volley` 的命中改为发射弹体。技能表新增：

```rust
pub(super) struct ProjectileSpec {
    pub speed: f32,        // m/s
    pub radius: f32,       // 扫掠球半径
    pub max_range: f32,    // 超出后消失
    pub guidance: Guidance,
}
pub(super) enum Guidance {
    /// 发射时锁定的原目标；每 tick 以不超过 turn_rate 的角速度转向其瞄准点。
    Locked { turn_rate: f32 },
    /// 沿发射方向直线飞行；技能不要求锁定，无目标时朝前方瞄准点发射。
    Straight,
}
```

- 每个技能自己决定是否锁定（用户要求）。准入规则：`Locked` 的技能保留现有的“需要锁定目标且在 `cast_range` 内”；`Straight` 的技能不要求锁定，`cast_range` 只作 `max_range`。
- 第一版数值（待试玩调整）：鸟铳单发 120 m/s，锤鸟三连发 160 m/s，弓箭 60 m/s，链斧 35 m/s；当前五个远程技能全部设为 `Locked { turn_rate: 12 rad/s }`；半径沿用表现半径（子弹 0.22、链斧 0.3）。

### 4.2 推进与命中

- 模拟层新增 `Projectile` 组件实体（无 Body、无碰撞体），由新系统 `advance_projectiles` 推进。它排在“玩家动作结算”之后、“普通敌人”之前，受 `SimulationActive` / `HitStop` 同一门控。
- 每 tick：按制导更新方向，然后从上一位置到新位置做球体 shape cast，排除发射者、妹妹和传感器。最先碰到的是敌人胶囊就结算命中，碰到场景就销毁。超出 `max_range`，或锁定目标在飞行中失效时，`Locked` 退化为 `Straight` 继续飞。
- 命中复用 `hit_enemy`；`Blow.origin` 为命中点，`source` 为妹妹（链斧为玩家）。
- 动作和弹体之间：动作的标记时刻只负责发射。僵直的联动窗口（Q 的 `window`）和锤鸟最后一枪的命中停顿，改为在弹体实际命中时触发，并带上发射时的 `source_action_id`。联动三连发里，原目标失效会取消尚未发射的部分，已在飞的弹体继续飞。
- 链斧：去程是一个 `Locked` 弹体。命中或飞满 `max_range` 后进入回程，以同一速度飞回玩家手部，`axe_away` 在回到手上时清除，不再按固定时间线回切。

### 4.3 表现

- `CombatEvent` 新增 `ProjectileLaunch` 和 `ProjectileImpact`。前者在发射时发出，带上弹体 id；后者带上命中点，命中场景时也发出。
- 客户端按弹体快照的 previous/current 位置插值显示子弹、箭和链斧，替换现有 0.16s 曳光方块和链斧的线性插值。音效和特效改为在 `ProjectileImpact` 时播放。

## 5. 兼容与影响

- **手感会变**：近战命中时刻从固定 `hit_time` 改为烘焙出的接触帧；Boss 的有效窗口由动画决定；远程命中晚一个飞行时间。这是本任务的目的，但需要用户试玩验收。
- **测试**：依赖精确 marker tick 和距离常量的战斗 / Boss 测试，改为从 `HitTrack` 读取窗口 tick 再做断言；不能复制实现公式。
- **回滚**：每个子任务独立提交。回滚 melee-sweep 时，保留 anim-sim-clock 也能正常运行。

## 待用户决定

无。具体数值（速度、阈值、余量）写为初值，实现后由试玩调整。

## 实现中的修订（melee-sweep，2026-11-12）

以下几处与上文不同，以这里为准；规范已按实现同步。

- **每个动作对每个目标只命中一次**（§2.1、§3.2 原写每个窗口各一次）。自动识别出的窗口数量取决于动画（一段普攻可能有两三次挥刀），按窗口计次会让伤害随动画变化；按动作计次保持每段伤害不变，窗口只决定刀什么时候有威胁。多段伤害的招式以后作为数据列加入。
- **有效窗口用绝对刀尖速度（25 m/s）判定，并且只在起手过渡结束后开始**（§2.2 原为 6 m/s、未处理过渡）。实测蓄力和收招的刀尖速度在 20 m/s 以下，挥刀在 30–115 m/s。起手时有 0.08s 的姿态混合（技能切换骨架时约 0.15s），这段时间画面里的刀取决于上一个姿态，任何烘焙都无法代表，所以不让它命中。过渡结束的 tick 由烘焙器自动量出：把同一招再回放一遍并让过渡先走完，两次回放开始重合的地方就是。
- **刀刃半径不计护手**（§2.2）：只量刀身外侧 75% 的最大粗细再加 0.08m。愁铃客长刀靠近握点有 0.26m 宽的饰物，刀身只有 0.02–0.04m。
- **刀尖取网格真实的尖端**，不假设刀沿 +Y 笔直。
- **正面防御判定仍用攻击者位置**（§3.2 原写接触点）：刀尖可能绕到身侧，按接触点判会把正面来的攻击判成背后。受击事件的位置改为接触点。
- **Boss 的片段映射按招重新对齐**（§1.2 原写沿用比例）：原映射把预警铺在片段的 0–45%，而三段片段真正的挥刀分别在 13–22%、15–53%、41–65%，原来是刀挥完了才结算。现在每招一行：0.72s 预警放蓄力，随后按片段原速放完挥刀，再放收势。预警时长因此保持约 0.72s。普通敌人同理。
- **Boss 在预警的后 35% 继续逼近**（方向已锁定）：真实刀长够到的距离约 1.9–2.4m，比原来抽象的 3.2m 短，不继续逼近的话从 6m 起手基本都会挥空。实测改后能命中的最远起手距离约 5.2m，与原来的约 5.3m 相当。
- **`HitProbes` 默认不记录**，由调试显示打开。
- **蹲伏出刀**按站立轨迹判定，是已知局限。
