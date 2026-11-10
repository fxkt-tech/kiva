# 现状调查

调查基准：2026-11-10，main，HEAD e7fed32。
本文件记录本次读到的现状与迁移风险，不把设计提案记作已存在的规范。

## 权威与调度

- docs/architecture.md 及 .trellis/spec/{simulation,gameplay,client}/index.md 确立 client → gameplay → simulation，Avian 独占物理位置和速度。
- src/simulation/mod.rs:42 注册 60Hz FixedUpdate；motor 在物理之前，publish → combat::advance → boss::advance → flush_teleports 在当前空间查询和物理之后。
- src/simulation/physics.rs:325 的 motor 调用 MotionState::advance；当前卸势请求和移动减速同属 MotionState。
- src/simulation/impact.rs 的 HitStop 在固定步开头统一门控；新准备阶段、耐力和控制效果必须使用同一门控。

## 当前战斗与输入

- src/simulation/weapons.rs:4 是 HammerSniper／AxeCannon，:11 是 Melee／Ranged；WeaponMode::persona 直接把模式映射成哥哥／妹妹。
- src/simulation/combat.rs:224 的 CombatIntent 仅有一个 SKILL 位及切模式／套装／目标指令；不能表达分开的 E／Q 或有序右键按下／释放。
- src/simulation/combat.rs:306 的 Combatant 同时持有模式、目标、落点、一个技能冷却、联动、狙击和旋斧状态。
- src/simulation/combat.rs:520 的 select_target 在没有当前目标时自动选择前方敌人。中键显式解锁不能继续走这个自动回补分支。
- src/client/controls.rs:169 的 sample 分别处理原生和 Web；现有 Q 是切换目标，E 是当前模式技能，滚轮切模式，1／2 选套装。
- web/input.js 的 takeActions 是位掩码；右键只在 pointerdown 写入一次 1024，pointerup 只清本地右键按住事实。新流程需要补充有序边沿，不能只看最终 held 值。
- src/simulation/actions.rs:133 的 MotionState 有当前卸势窗口和冷却；:407 的 advance 同时处理跳跃、闪避、蹲伏、滑铲和卸势。
- 当前卸势是 0.25s 窗口／0.65s 冷却，无持续防御、耐力或释放弹反。原生和 Node 测试已有短按、失焦与重复过滤覆盖。

## 命中与控制

- src/simulation/combat.rs:547 的 hurt_enemy 做玩家对敌人的伤害与 Boss 无敌检查；:1091 的 advance_enemies 单独处理普通敌人的来袭。
- src/simulation/boss.rs:229 的 advance 有另一条来袭处理；两处都分别判断闪避／卸势并直接扣玩家血。
- 两处防御逻辑都以面对攻击来源的平面点积 > 0.25 作为正面判定，可作为第一版不额外改动朝向手感的参考。
- Boss::is_invulnerable 在 Transition／Defeated 时为真；Boss::can_be_launched 当前服从该限制。
- Boss 独占其行动和 Deflected 等状态，普通 Enemy 的推进明确跳过 Boss。新僵直不能再写一份 Enemy 恢复倒计时来镜像 Boss。
- 当前没有通用减速状态；链斧 E 必须增加单一权威的减速期限，并作用于实际 motor 速度，而不只改动画。

## 已有联动可复用部分

- src/simulation/combat.rs:274 的 SniperSequence 已保留原目标、固定步 elapsed、真实瞄准点和单次发射标志，现行为一枪。
- 锤子范围 4.5m、冲击半径 3m、击飞相对高度 3m；已有真实物理测量回归，不能改成客户端抬模型。
- 当前第二套是火炮跃进接旋斧，与新链斧弓的规则不同；可以复用物理接入方式，不能只改名称。
- src/client/camera.rs:63、:74、:111 读取 SniperSequence 做低位仰视、碰撞净空与回切；新三枪需延长平台期，第二套需要自己的平视构图。
- 命中停顿现为狙击成功后 3 个固定步；预算和物理冻结行为已有回归。

## 表现与资源缺口

- src/client/combat.rs:82 给同一个 PlayerView 建立四个 WeaponVisual；近战调用 avatar::spawn_blade，远程是程序化狙击枪／火炮。
- src/client/animation.rs:518 根据 ActorView.owner 查询 Body／MotionState／Combatant；简单把妹妹也标成同一个玩家的 ActorView 会让她重放哥哥全部动作。
- src/client/stickman.rs:35 已经是 avatar::spawn 的兼容入口，不再直接生成旧火柴人几何。
- 当前随包角色只有 assets/choulingke 与 assets/lihuowang，另有 CC0 23 骨骼／19 动作的数据；没有专门的妹妹、锤子、鸟铳、链斧、弓箭正式资产。assets/choulingke/animations.bin 是现有原动作，不应声称已含新武器的完整配套动作。
- src/client/appearance.rs:51 会替换哥哥身体和两个近战武器根下的长刀；新武器接入后必须改变武器换装部分，同时保留 F3 身体外观与状态。
- src/client/audio.rs:53 按 CombatEffect 选音，且以 event.actor 是否等于玩家判定枪声；妹妹成为独立逻辑施法者后必须显式携带玩家归属，不能因此漏音或把哥哥当成射手。
- src/client/hud.rs:58／:136、web/hud.js 和 index.html 均围绕单技能冷却和当前近远模式；需要同步改为 E／Q 两槽、当前组、耐力及新的右键状态。
- src/gameplay/mod.rs:224 的 setup 当前把玩家生命装为 100，默认锤狙；:41 的 set_enemy_mode 在暂停时也可清目标并替换敌人。

## 验证入口与历史边界

- Makefile 的 make check 包含 cargo fmt、原生 Clippy、Rust 测试、Node 单元测试与 WASM cargo check。
- web 专属 Rust 代码按现行规范补 WASM Clippy；入口改变需临时目录 Trunk 构建，不能动用户开发服务。
- scripts/input-unit.test.cjs、hud-unit.test.cjs、audio-unit.test.cjs、appearance-unit.test.cjs 已直接执行实际桥脚本；tests/physics_terrain.rs 覆盖真实场地和物理。
- docs/plans/actions-and-linked-weapons.md 是旧方案与历史执行记录，其中含旧操作和当时会话授权，不作为本次产品决策或服务／测试授权。
- 本次讨论完整保留在当前会话，没有调用 trellis mem 重查同一段历史。
