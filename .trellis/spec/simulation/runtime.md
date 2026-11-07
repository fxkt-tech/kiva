# 模拟运行规则

## 接口与状态所有权

- `src/simulation/mod.rs` 显式导出外部接口；内部实现默认私有。
- `spawn_unit` 校验出生位置、数值、技能装配，返回 `Result`。内容构造不绕过该入口。
- `model.rs` 的 `UnitPose`、`UnitState`、`AbilitySlots` 运行态字段使用 `pub(super)`，外部只读 getter。
- `CommandQueue::submit` 表示意图，`SimulationEvent` 表示结果。指令先校验；拒绝时保留原有效指令和动作。
- `movement.rs` 写位置，`actions.rs` 写动作时钟/消耗/冷却，`combat.rs` 写伤害/死亡/重生。

这些规则可在 `src/simulation/model.rs`、`orders.rs` 和 `tests/simulation.rs` 中对应验证。
属性、伤害和普攻时序的具体接口见 [attributes-and-damage.md](attributes-and-damage.md)。

## 固定步与动作

- `TICK_RATE` / `TICK_SECONDS` 是时间基准，当前 60 Hz。调度通过 `SimulationPlugin` 中的 `.chain()` 声明。
- `UnitOrder` 是持续目的，`UnitAction` 是当前动作。技能结束后可以继续之前的持续指令。
- 技能前摇结束时重新验证目标和资源，再提交扣费、冷却和效果；此前取消不扣资源。
- 已发射弹丸或布置的范围效果不因施法者死亡而失效。新指令和死亡不会退还已提交消耗。
- 普攻冷却独立于移动；相同攻击指令不能不断重置前摇。
- 暂停冻结整个固定步链，已入队的一次输入不会因为显示帧而丢失或重复执行。

## 空间与数值

- 模拟使用 `Vec2(x, z)`，展示使用 `Vec3(x, y, z)`，时间单位是秒。
- `NavigationMap` 提供寻路与扫掠，普通移动、冲刺和弹丸共用静态障碍几何。
- 在构造和指令校验处拒绝非有限数值；不要依靠显示层过滤错误。
- 不承诺跨平台浮点确定性；相等距离的目标选择已有实体 ID 排序。

## 测试方式

`tests/simulation.rs` 的 `Arena` 只安装 `MinimalPlugins` 和 `SimulationPlugin`，通过公开构造和指令接口运行。
例如 `cancelled_windup_spends_no_mana_or_cooldown` 验证提交时机，
`committed_area_effect_survives_the_casters_death` 验证效果所有权。
新增行为测试沿用这一方式，不公开运行态写权限来方便测试。
