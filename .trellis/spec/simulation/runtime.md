# 模拟运行规则

## 接口与状态所有权

- `src/simulation/mod.rs` 显式导出外部接口；内部实现默认私有。
- `spawn_unit` 校验出生位置、数值、技能装配，返回 `Result`。内容构造不绕过该入口。
- `model.rs` 的 `UnitPose`、`UnitState`、`AbilitySlots` 运行态字段使用 `pub(super)`，外部只读 getter。
- `CommandQueue::submit` 表示意图，`SimulationEvent` 表示结果。指令先校验；拒绝时保留原有效指令和动作。
- `movement.rs` 写位置，`actions.rs` 写动作时钟/消耗/冷却，`combat.rs` 写伤害/死亡/重生。
- `effects.rs` 统一快照/投递/排序结算；`statuses.rs` 拥有控制实例与权限归并，不能另写位置或最终属性。
- `equipment.rs`拥有六格实例/装卸/装备CD，使用属性模块同步提交与动作模块统一施法入口。
- `mechanics/compound/battlefield/vision`负责增强、复合执行、临时对象与观察，见[复合机制](champion-primitives.md)。
- 指令前推进增强/充能/对象并刷新视野，结算后再次刷新；构造/reset即时刷新，暂停首帧不能泄露全图。
- `recovery.rs`拥有自然生命与受限资源恢复；`validate_unit_spec`与spawn共享无分配预检。近战、结构与模式组合接口见[战斗基础](combat-foundation.md)。

这些规则可在 `src/simulation/model.rs`、`orders.rs` 和 `tests/simulation.rs` 中对应验证。
属性、伤害和普攻时序的具体接口见 [attributes-and-damage.md](attributes-and-damage.md)。
组合技能、控制、生命身份和强制位移见 [abilities-and-statuses.md](abilities-and-statuses.md)。
装备属性/被动/主动来源及装卸边界见 [equipment.md](equipment.md)。

## 固定步与动作

- `TICK_RATE` / `TICK_SECONDS` 是时间基准，当前 60 Hz。调度通过 `SimulationPlugin` 中的 `.chain()` 声明。
- 公开`SimulationSet::Step`包住完整固定步，模式可在其前后排序或附加门控，通用模拟不依赖具体模式。
- `UnitOrder` 是持续目的，`UnitAction` 是当前动作。成功提交技能清旧MoveTo/AttackMove目的地，普通技能保留AttackTarget/AutoAttack。前摇仅清路线，提交期拒绝仍保留原有效目的。自主Dash/Leap真正开始时清旧目的为Idle；Link以无旧目标的AutoAttack替换，运动结束后从落点附近重新索敌，不能追回旧远处目标。Stop可清后续攻击而不取消已提交运动；外来强制位移保持原规则。超距Enemy使用一次性CastTarget，开始前摇前清成Idle，不自动重复或普攻。
- cast_orders拥有CastTarget来源/实际技能/目标生命复核和区域路线；位置仍由movement写。每tick检查可见性，按0.2秒节流重规划；区域终点保留后追加当前几何扫掠的向内延续，每tick仍以实际射程决定是否开始，避免停在移动目标的旧边界。死亡/丢视野/来源失效/无路取消，新合法操作替换、非法输入保留。
- 追近不扣费/CD，严格preview_cast仍返回OutOfRange；只有实际Enemy命令可转换为追近。到范围后复用begin/commit，控制中断/提交超距不恢复旧目的；装备卸装只清同实例待放。
- 待放进入范围后的begin必须位于advance_actions之前，与直接命令共用首个动作tick；零前摇同tick提交，不能在动作时钟后开始而人为多等待一tick。非零前摇仍按原提交复核、取消后不重试。
- 同一单位在当前tick有待动作阶段提交的零前摇时，consume_commands将其后续输入按FIFO留在CommandQueue、下tick继续；其它单位仍在本tick处理。不能让同帧Q因前面的瞬发E暂时处于Casting而Busy丢失，也不能为连招在输入层按ID延时重试。动作/恢复/CD/伤害时钟不提前或重复推进，Stop仍按FIFO取消后续前摇/持续攻击；暂停/重开沿既有入口清队列。
- 技能前摇结束时重新验证权限、目标生命和资源，整组快照成功后提交扣费、冷却和效果；此前取消不扣资源。
- Point Dash/Standard/Scout在提交时从当前施法位置向原点击点截取最大射程，事件和运动使用同一个解析终点；其它Point保持原射程拒绝。Enemy即时预检仍严格，命令可先走近。
- 动作、持续指令与移动先于效果结算；Direct也只入队，不能在actor遍历中直接控制其它单位。
- 旧状态在移动后、新命中前到期；t末施加N tick限制t+1至t+N，不在新状态应用当步递减。
- Stop清意图/普通路线，不取消已提交运动；受控新操作拒绝，原意图可在解控后继续，被打断施法不重试。
- 已发射弹丸或布置的范围效果不因施法者死亡而失效。新指令和死亡不会退还已提交消耗。
- 普攻冷却独立于移动；相同攻击指令不能不断重置前摇。
- 定点AttackTarget的目标死亡、删除或换生命后转为AutoAttack；与AttackMove共用当前攻距+3的可见敌人最近获取及实体ID平局排序、攻距+6的持有距离。没有候选时原地保留AutoAttack，不返回旧坐标；旧目标仅丢视野仍停止定点追击。Stop/Move替换持续意图，施法沿用既有保留/替换语义，Idle不会自行索敌。
- 自动目标同样保存UnitLifeRef，不能复用旧生命提交攻击；对相同自动目标的重复AttackTarget和AI去重不打断前摇/后摇，不重置攻击间隔。
- 暂停冻结整个固定步链，已入队的一次输入不会因为显示帧而丢失或重复执行。
- 装备CD在actions推进开始、筛选存活单位之前计时，仍位于指令消费之后。
  死亡保留装备/Explicit加成且CD继续推进；重生不重置装备CD，角色技能仍遵循原重生规则。

## 空间与数值

- 模拟使用 `Vec2(x, z)`，展示使用 `Vec3(x, y, z)`，时间单位是秒。
- `NavigationMap` 提供寻路与扫掠，普通移动、冲刺和弹丸共用静态障碍几何。
- 在构造和指令校验处拒绝非有限数值；不要依靠显示层过滤错误。
- 不承诺跨平台浮点确定性；相等距离的目标选择已有实体 ID 排序。

## 死亡事实与重生倒计时

`UnitState::death_tick() -> Option<u64>`由`combat::die`在实际致死时写入，
`UnitState::respawn_remaining() -> Option<f32>`公开现有模拟倒计时，客户端不能写这两个字段。
存活时两者均为None；死亡时记录tick，只有配置了重生的单位才有remaining。
暂停不推进它们，实际重生才清除death_tick/remaining并推进生命代次。

计时到0但`NavigationMap::resolve_placement`暂时找不到安全点时，仍保留同一死亡生命，
remaining夹到0并在后续tick重试，不能为HUD伪造新倒计时或假复活。
HUD对正值向上取整显示秒数，0显示等待安全位置；倒地动画时长不影响模拟。
`tests/simulation.rs`覆盖致死事实、暂停、真实复活及安全落点阻塞/恢复。

## 显式结束本局

`reset_simulation(&mut World)`是整场试炼重开的模拟清理入口：设置paused=true、tick=0，
移除Unit/Projectile/AreaEffect实体及其linked children，清命令、待处理属性变更、命中包和SimulationEvent消息。
引导、控制、运动、装备和属性运行态随所属单位删除，不直接从客户端改写这些组件。
它保留NavigationMap、技能/角色/装备目录及非模拟静态实体；完整对局的重建由gameplay负责，客户端会话调用其入口。

清队列不能重建身份分配器：AttributeChanges的last_source、PendingImpacts的next_sequence、StatusSequence保持单调，
ModifierSourceId在同一个World内仍不得复用。应用Time墙钟/固定步资源不重建，只有SimulationTick归零。
这与死亡/重生是不同的显式边界；普通死亡后已提交效果仍存活的既有规则不变。
公开行为回归见`tests/reset.rs`，覆盖实际弹丸/延迟区域/引导/控制、队列清理、静态实体保留和新局独立性。

## 测试方式

`tests/simulation.rs` 的 `Arena` 只安装 `MinimalPlugins` 和 `SimulationPlugin`，通过公开构造和指令接口运行。
例如 `cancelled_windup_spends_no_mana_or_cooldown` 验证提交时机，
`committed_area_effect_survives_the_casters_death` 验证效果所有权。
新增行为测试沿用这一方式，不公开运行态写权限来方便测试。
