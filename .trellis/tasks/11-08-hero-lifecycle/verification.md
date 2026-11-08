# 生命周期交付与验证

## 已实现
- `UnitState::death_tick()` 返回本次生命的死亡 tick；`respawn_remaining()` 返回模拟剩余秒数。死亡写入，实际复活清除；出生落点不可用时保留死亡和生命身份，剩余时间停在0，不变成负数。
- 模型已附加 Hero profile 的死亡根保持可见；位置固定为死亡 `UnitPose.position()`，击飞高度归零。小兵仍隐藏并由玩法回收，基地保留平台的旧规则不变；死亡 ground marker 隐藏。
- 程序骨架 fallback 用独立 Death gesture 在0.45模拟秒内倒地并保持。只改骨架局部姿态，不改 `UnitVisual` 世界根或模拟状态。普通暂停完整保留姿态；重生清理旧生命姿态，同显示帧跨死亡/重生沿原有生命身份和事件处理。
- Web `UnitView.respawn_remaining` 与 Native HUD 使用同一 getter。`#respawn-status` 仅格式化为倒计时或等待安全落点，JS 不另计时；活着/选角隐藏。

## 新增行为回归
- `tests/simulation.rs::death_clock_freezes_on_pause_and_waits_at_zero_until_respawn_is_safe`：真实普攻击杀、死亡 tick/剩余时间关系、暂停冻结、阻塞出生点时到0仍死、恢复几何后才推进生命身份并清倒计时。
- `client::character::tests::hero_corpse_stays_at_death_position_is_unpickable_and_freezes_until_respawn`：移动中的真实程序骨架英雄被技能击杀，死亡点和骨骼倒地、尸体可见但不可拾取、暂停冻结、同模型重生和再次死亡后的整局递归清理。
- 既有同显示帧提交→死亡→复活、scene击飞/暂停、模拟死亡/重生回归保留。

## 已执行
- 仅拥有文件的 `rustfmt --edition 2024 --config skip_children=true`：通过。
- `node --check web/hud.js`：通过。
- `git diff --check`：通过。

## 交接时约定的统一验收
本子任务没有并发执行 Cargo/Trunk/浏览器。主代理在资源与音频模块合并后串行运行：

```sh
cargo test --locked --test simulation death_clock_freezes_on_pause_and_waits_at_zero_until_respawn_is_safe
cargo test --locked --lib client::character
cargo test --locked --lib client::scene
make check
```

真实导入模型读取同一 `death_tick` 播放其 Death clip；最终有窗口浏览器还要验证实际尸体/倒计时/重生、暂停和重新选择角色。不能把本文件的语法/格式检查宣称为已完成这些集成验证。

## 主代理最终联合结果

2026-11-08：完整 `CARGO_BUILD_JOBS=2 make check`通过343项Rust测试（2个既有长局soak忽略），包含上述simulation/character/scene回归；WASM Clippy和最终Trunk构建通过。
最终有窗口1280×800 WebGPU以正常指令/真实AI战斗触发源计划寒冰死亡，实际Death模型倒地并保留，暂停时tick/死亡位置/剩余时间精确冻结，恢复后同实体在泉水真实复活并清提示。
800×600、DPR2的实际模型/头像/图标和重新选角同样通过。回退尸体放松姿态、生命跳帧两项独立审查修复已包括在完整Rust门禁。
最终源快照、日志和截图见父任务 [verification.md](../11-08-battle-presentation/verification.md)。
