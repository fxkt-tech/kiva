# 执行计划

1. 技能表：加入 `ProjectileSpec`、`Guidance`，五个远程技能填写第一版数值；准入按 `Guidance` 区分。
2. 模拟层：`Projectile` 组件、`advance_projectiles` 系统（制导、shape cast、命中、销毁），加入固定步顺序。
3. `fire_shot` / `fire_volley` 改为只负责发射；联动窗口和 HitStop 移到弹体命中时；链斧的去程和回程。
4. 反馈事件 `ProjectileLaunch` / `ProjectileImpact`；客户端的弹体显示、链斧显示、音效和特效改为读弹体快照和命中事件。
5. 测试：飞行时间、追踪、直线落空、遮挡、挡路敌人、联动、链斧回收、暂停 / HitStop。
6. `make check` + WASM Clippy。

回滚点：第 3 步之前，旧的瞬时结算仍可用。
