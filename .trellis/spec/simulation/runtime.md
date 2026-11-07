# 模拟运行规则

## 接口与状态所有权

- `src/simulation/mod.rs` 显式导出外部接口；内部实现默认私有。
- `spawn_unit` 校验出生位置、数值、技能装配，返回 `Result`。内容构造不绕过该入口。
- `model.rs` 的 `UnitPose`、`UnitState`、`AbilitySlots` 运行态字段使用 `pub(super)`，外部只读 getter。
- `CommandQueue::submit` 表示意图，`SimulationEvent` 表示结果。指令先校验；拒绝时保留原有效指令和动作。
- `movement.rs` 写位置，`actions.rs` 写动作时钟/消耗/冷却，`combat.rs` 写伤害/死亡/重生。
- `effects.rs` 统一快照/投递/排序结算；`statuses.rs` 拥有控制实例与权限归并，不能另写位置或最终属性。

这些规则可在 `src/simulation/model.rs`、`orders.rs` 和 `tests/simulation.rs` 中对应验证。
属性、伤害和普攻时序的具体接口见 [attributes-and-damage.md](attributes-and-damage.md)。
组合技能、控制、生命身份和强制位移见 [abilities-and-statuses.md](abilities-and-statuses.md)。

## 固定步与动作

- `TICK_RATE` / `TICK_SECONDS` 是时间基准，当前 60 Hz。调度通过 `SimulationPlugin` 中的 `.chain()` 声明。
- `UnitOrder` 是持续目的，`UnitAction` 是当前动作。技能结束后可以继续之前的持续指令。
- 技能前摇结束时重新验证权限、目标生命和资源，整组快照成功后提交扣费、冷却和效果；此前取消不扣资源。
- 动作、持续指令与移动先于效果结算；Direct也只入队，不能在actor遍历中直接控制其它单位。
- 旧状态在移动后、新命中前到期；t末施加N tick限制t+1至t+N，不在新状态应用当步递减。
- Stop清意图/普通路线，不取消已提交运动；受控新操作拒绝，原意图可在解控后继续，被打断施法不重试。
- 已发射弹丸或布置的范围效果不因施法者死亡而失效。新指令和死亡不会退还已提交消耗。
- 普攻冷却独立于移动；相同攻击指令不能不断重置前摇。
- 暂停冻结整个固定步链，已入队的一次输入不会因为显示帧而丢失或重复执行。

## 空间与数值

- 模拟使用 `Vec2(x, z)`，展示使用 `Vec3(x, y, z)`，时间单位是秒。
- `NavigationMap` 提供寻路与扫掠，普通移动、冲刺和弹丸共用静态障碍几何。
- 在构造和指令校验处拒绝非有限数值；不要依靠显示层过滤错误。
- 不承诺跨平台浮点确定性；相等距离的目标选择已有实体 ID 排序。

## 显式结束本局

`reset_simulation(&mut World)`是整场试炼重开的模拟清理入口：设置paused=true、tick=0，
移除Unit/Projectile/AreaEffect实体及其linked children，清命令、待处理属性变更、命中包和SimulationEvent消息。
引导、控制、运动和属性运行态随所属单位删除，不直接从客户端改写这些组件。
它保留NavigationMap、技能/角色目录及非模拟静态实体，NPC和玩家的重新创建由客户端内容/会话协调。

清队列不能重建身份分配器：AttributeChanges的last_source、PendingImpacts的next_sequence、StatusSequence保持单调，
ModifierSourceId在同一个World内仍不得复用。应用Time墙钟/固定步资源不重建，只有SimulationTick归零。
这与死亡/重生是不同的显式边界；普通死亡后已提交效果仍存活的既有规则不变。
公开行为回归见`tests/reset.rs`，覆盖实际弹丸/延迟区域/引导/控制、队列清理、静态实体保留和新局独立性。

## 测试方式

`tests/simulation.rs` 的 `Arena` 只安装 `MinimalPlugins` 和 `SimulationPlugin`，通过公开构造和指令接口运行。
例如 `cancelled_windup_spends_no_mana_or_cooldown` 验证提交时机，
`committed_area_effect_survives_the_casters_death` 验证效果所有权。
新增行为测试沿用这一方式，不公开运行态写权限来方便测试。
