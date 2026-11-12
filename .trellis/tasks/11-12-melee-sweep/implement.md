# 执行计划

1. 烘焙器：在客户端加入默认忽略的测试 `bake_hit_tracks`，复用 `PreviewRig` 的固定步回放、标准外观挂点和武器网格的刀尖长度；生成代码写入 `hit_tracks.rs`。Makefile 加 `bake-hits`。
2. 过期校验测试 `hit_tracks_match_animation`。
3. 模拟层通用判定：受击胶囊、线段插值、线段与胶囊最近距离、遮挡射线、`HitLedger`；为几何函数单独写单元测试。
4. 接入玩家普攻（替换 `strike_arc`）和范围技（刀刃扫掠，冲击时刻读烘焙数据）。
5. 接入 Boss 和普通敌人；AI 的出招距离读 `reach`。
6. `HitProbes` 资源与客户端 `hit_debug.rs`（先在 `controls.rs` 确认空闲键位）。
7. 更新受影响的战斗 / Boss 测试：从 `HitTrack` 读窗口 tick 做断言。
8. `make check` + WASM Clippy；`make bake-hits` 两次结果一致。

回滚点：第 4 步（玩家）和第 5 步（敌人 / Boss）可以分别回滚。
