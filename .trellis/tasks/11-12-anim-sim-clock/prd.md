# 攻击动画统一由模拟时钟驱动

父任务：`11-12-hit-detection`，设计见父任务 design.md §1。

## Goal

每个攻击的姿态都按模拟层的动作时间采样，使后续烘焙的判定和画面处在同一条时间轴上。

## Requirements

| ID | 需求 |
|---|---|
| R1 | 模拟层为玩家、Boss、普通敌人发布统一的只读 `AttackClock { action_id, track, elapsed, duration }` |
| R2 | Boss 的预警、出招、收招合成一条招式时间轴；`BossState` 由招式时间推出，对外的只读接口保持可用 |
| R3 | 客户端新增片段映射表 `client/attack_clips.rs`，每个 `TrackId` 一行；技能、普通敌人、Boss 的姿态按 `AttackClock.elapsed` 查表采样，不再用渲染端计时 |
| R4 | 第一版映射沿用现有比例，画面节奏不变 |
| R5 | 暂停和 HitStop 冻结姿态，与现在一致 |

## Acceptance Criteria

- [x] 无窗口测试：同一模拟时间、不同渲染帧率下，技能、敌人、Boss 采样到的片段秒数相同
- [x] Boss 既有测试（预警、单次命中、三连击停手、暂停、HitStop）全部通过
- [x] `tests/layering.rs` 通过（映射表按 `strike_effect()` 选择技能行，不点名技能，无需新增例外）
- [x] `make check`、WASM Clippy 通过

## 验证记录（2026-11-12）

- 新增 `attack_poses_follow_the_simulation_clock_not_the_render_rate`：在同一模拟时刻、不同渲染 dt（1/30 秒×7 帧和 1/144 秒×12 帧）下，锤 E 和普通敌人采样到的片段时间相同。做过变异验证：临时退回渲染计时后测试失败（1.25 ≠ 1.35）。
- 新增 `attack_clip_phase_boundaries_match_the_simulation_timelines`：Boss 进入攻击和收招的时刻、普通敌人出刀的时刻，都与表中边界相差不超过一个 tick。
- `make check` 通过：Rust 319 个、Node 108 个；WASM Clippy 通过；`git diff --check` 通过。
- 画面未验证：第一版映射沿用原有比例，预期节奏不变，需要用户试玩确认。
- 实现与设计 §1.1 的差异：`AttackClock` 由三个组件的方法返回，没有另建组件或系统。Boss 状态切换改为读招式时间轴，但仍保留 `BossState` 与每个状态各自的 elapsed/duration，对外接口不变。CC0 底层手势（权重低于 1 时可见）仍用自己的时钟，只在源片段淡入的短时间内可见，留给 melee-sweep 烘焙按确定路径回放。
