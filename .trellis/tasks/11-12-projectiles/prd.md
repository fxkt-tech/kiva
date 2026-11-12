# 远程真实弹体

父任务：`11-12-hit-detection`，设计见父任务 design.md §4。依赖 `11-12-anim-sim-clock`。

## Goal

远程攻击改为有飞行时间的弹体，逐 tick 扫掠命中，可以被遮挡；每个技能自己决定是否锁定目标。

## Requirements

| ID | 需求 |
|---|---|
| R1 | 技能表为 `Shot` / `Volley` 配置 `ProjectileSpec { speed, radius, max_range, guidance }`；`Guidance::Locked { turn_rate }` 追踪原目标，`Guidance::Straight` 直线飞行 |
| R2 | `Locked` 的技能保留现有的锁定准入；`Straight` 的技能不要求锁定，无目标时朝前方发射 |
| R3 | 弹体每 tick 做球体扫掠：先碰到敌人胶囊就命中，碰到场景就销毁；超出射程消失；锁定目标失效后改为直线飞行 |
| R4 | 僵直联动窗口和锤鸟最后一枪的命中停顿，在弹体实际命中时触发，并保持原动作的身份 |
| R5 | 链斧去程是锁定弹体；命中或飞满射程后飞回手中，回到手上时清除 `axe_away` |
| R6 | 客户端按弹体快照插值显示；音效和特效在实际命中时播放 |
| R7 | 当前五个远程技能全部设为 `Locked`，第一版速度见设计 §4.1 |

## Acceptance Criteria

- [x] 无窗口测试：飞行时间符合距离 / 速度；锁定弹体能追上横向移动和下落的目标；直线弹体在目标移开后落空
- [x] 墙、箱子挡住弹道时不命中；挡在路线上的其他敌人先被命中
- [x] 联动窗口在命中时刻开启，三连发每发独立的 `hit_index`；原目标失效后，未发射的部分取消，已在飞的继续飞
- [x] 链斧回到手上之前不能用链斧近战
- [x] 暂停 / HitStop 冻结弹体
- [x] `make check`、WASM Clippy 通过

## 追加：妹妹退场后远程武器留在原地（用户 2026-11-12 报告）

- 现象：释放联动技（以及 Q）之后，妹妹消失，她手里的鸟铳／弓留在原地。
- 原因：`client/companion.rs::sync` 把当前组的手持武器和背部配件设成 `Visibility::Visible`。这个值会无视父节点；妹妹根节点隐藏后 `sync` 提前返回，不再改它们，于是武器停在最后的位置继续显示。
- 修复：显示的部件改用 `Visibility::Inherited`，由根节点决定。
- 回归：`the_sisters_weapon_and_accessory_disappear_with_her_after_a_skill_or_combo`，两组武器各走一遍出场和退场；退回旧写法时失败。

## 验证记录（2026-11-12）

- `make check` 通过：Rust 336 个（1 个忽略，即烘焙入口）、Node 测试通过；WASM Clippy、临时目录 Trunk 构建通过，Cargo.lock 未变；`make bake-hits` 后数据文件无变化。
- 新增 `combat/projectile_tests.rs` 7 个测试，对应上面每条验收；客户端新增弹体显示测试（位置等于模拟的插值位置，落地后回收）。
- 15 个写死“标记时刻即命中”的旧测试改为等弹体落地，并断言发射步与命中步分开。
- 画面未验证：弹体外形、链斧飞行与绳索、妹妹武器消失，都需要启动游戏确认。

## 实现说明

- `Guidance::Straight` 目前只有弹体测试在用，没有技能行使用它；直飞技能“无需锁定即可释放”走的是现有的 `cast_range: None` 准入，但没有一个真实技能端到端验证过。加第一个直飞技能时应补这条测试。
- 弹速远高于人物速度，`turn_rate: 12 rad/s` 下锁定弹基本必中；想让远程可以被走位躲开，调低该行的 `turn_rate` 或 `speed`。
- 链斧回程穿过一切，不再造成伤害。
- `ActionKind::throw_times` 已删除；`shot_times` 仍给出每发的发射时刻。
