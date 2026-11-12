# 执行计划

1. 模拟层：定义 `TrackId`、`AttackClock`；玩家由 `ActionSnapshot` 派生，每个固定步写入。
2. Boss：引入招式时间轴（预警 + 出招 + 收招），状态由时间推出；先保持现有的 `ACTIVE` 和命中规则不变，只换计时方式。跑 Boss 回归。
3. 普通敌人：蓄力加出招合成时间轴并发布。
4. 客户端：新增 `attack_clips.rs`，把 `animation.rs` 里 Boss、技能、敌人的选片和时间映射迁到表里；删除这几条路径上的 `clock` 驱动。
5. 新增“不同渲染 dt 下采样秒数相同”的回归测试；更新 `tests/layering.rs` 例外。
6. `make check` + WASM Clippy。

回滚点：第 2 步之后可以单独回滚 Boss 的计时改动。
