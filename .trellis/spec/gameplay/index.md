# 玩法层

## Scope / Trigger

`src/gameplay/mod.rs` 定义确定性的 50m 庭院及玩家装配，修改关卡或出生配置时阅读。
依赖模拟，不导入客户端、窗口、材质和网格。

## Signatures

`setup(&mut World)` 插入 `Level` / `CollisionWorld`，装配玩家和四个练习敌人。
所有战斗实体含 Body / MoveIntent / ActionIntent / MotionState / Facing / Health / SpawnPoint；
玩家另含 Player / Combatant / CombatIntent，敌人另含 Enemy::default()。
`Level { half_size: f32, props: Vec<Prop> }`；
`Prop { center: Vec2, size: Vec3, kind: PropKind }`；kind 为 Crate / Pillar / Stone。

## Contracts

Level.props 是渲染几何和静态碰撞的共同来源。size 是完整三维尺寸，碰撞取 x/z 各一半。
玩家出生在原点，速度 6m/s，半宽 0.35m；玩家 Facing 由客户端输入表达，敌人 Facing 由模拟决策。
玩家生命 100、默认锤子近战；四个敌人速度 2m/s、半宽 0.3m、生命 120。
出生点由玩法指定，复活规则由模拟执行；客户端不能另造目标或生命数据。

## Validation & Error Matrix

内容是程序内可信配置；尺寸须有限且为正，障碍完全位于关卡内。
原点 2m 范围保持空地，内容错误在回归和模拟构造时暴露，不在客户端偷偷修正。

## Good / Base / Bad Cases

Good：添加一个 Prop 会同时改变碰撞和模型。
Base：空出生区能移动到附近障碍。
Bad：只加可见箱子不加碰撞，或分别手写两套坐标。

## Tests Required / Quality Check

`make check`。纯数据测试检查出生区、尺寸、范围和每个 Prop 中心实际不可通过，无需窗口。
装配测试验证五个实体状态齐全、生命有效、处于地面，出生点与位置一致且不在静态碰撞内。

## Wrong vs Correct

错误：客户端决定地图碰撞，玩法创建 Mesh。
正确：玩法提供 Level 和 CollisionWorld，客户端读取 Level 生成表现。
