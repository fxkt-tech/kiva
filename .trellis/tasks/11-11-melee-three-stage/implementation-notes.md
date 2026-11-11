# Implementation boundary

用户已在父方案总结后批准“开始实现”。本子任务只实现模拟的三段普攻/防御手势快照与真实运动反馈，不解析或选择动作/媒体资源。

最小行为差：现有动作中点击一律丢弃、无阶段身份；改为第1/2段各最多一条后继，0.45秒窗口，第三段后重置，每段仍一次权威marker。旧技能窗口、物理伤害和实际防御判定不变。

状态所有者：abilities的只读MeleeStage快照、combat的私有MeleeChain和取消锁存、defense阶段/历史手势、physics的真实接受/接触/位移采样。motion_feedback独立有界历史只发布事实，不改变运动求解。

文件边界：src/simulation/{abilities,combat,weapons,defense,actions,physics,boss,mod}.rs及其测试；客户端只做stage构造器/新事件穷举的必要机械适配并通知主会话。controls由表现任务负责，使用本任务公开取消锁存API。

不改模型/动作资产、JS音频、现有BGM、Boss决策/数值、不提交或推送。纯数值MeleeProfile替代旧tuple时，各武器现有伤害/范围/marker/总时长保持，由真实FixedUpdate回归证明。

## Public consumption contract

- MeleeStage::{One,Two,Three} is copied into ActionSnapshot/HitContext/CombatEvent.melee_stage. Non-melee/Enemy/Boss events use None; hit_index stays zero for each ordinary action. WeaponPair::melee_profile(stage) exposes damage/range/hit_time/duration as pure rule data.
- CombatEffect::MeleeStart emits once at commitment with actor/source_owner/pair/action_id/stage; CombatEffect::Melee remains the real hit marker. Consumers must not play the full whoosh from both. Source pose can observe the current ActionSnapshot before the first hit.
- CombatIntent::cancel_melee_chain() latches only uncommitted-chain cancellation. CombatIntent::clear() also latches it. Both survive a zero-tick pause and do not clear the current ActionRuntime or cooldown.
- DefenseState::snapshot() returns kind/elapsed/remaining plus gesture_id/last_gesture_kind/gesture_elapsed. Natural expiry keeps the last gesture; cancellation/motion exits invalidate its kind to Idle. Respawn clears state/cooldowns but preserves the monotonic gesture counter.
- MotionFeedback::events() yields MotionEvent { sequence, actor, kind, position, jump_count, sprinting }. MotionKind::{Jump,Dodge,Land,Footstep}. This is a separate 128-event monotonic history, with an independent consumer cursor. Initial spawn/teleport do not create a landing or a stride; accepted jumps/dodges are emitted by motor, contact/stride by post-solver publish. Only controlled Combatants participate.
- Stride adaptation uses actual horizontal displacement relative to the support velocity and nonzero movement intent. Normal stride2m/crouched0.8m multiplied by Body.scale; sprint reaches it faster. No stride while dodging/sliding/airborne/blocked; speed<0.15m/s and abnormal displacements are discarded. Tiny nonjump support gaps under0.05s are filtered; a real short jump still lands once.
- New ordinary buffered follow-up has one admission attempt after retirement. Valid skill/pair/defense/movement wins. C reaches the motor first: an actual slide clears the chain, ordinary crouch preserves one queued follow-up for the next rule tick. This avoids silently dropping either C or the user's buffered stage.

## Mechanical client adaptations

src/client/animation.rs ignores MeleeStart in the existing gesture observer; src/client/combat.rs skips the new event rather than spawning its default hit sphere; src/client/audio.rs test event constructor supplies melee_stage=None. The later presentation worker owns real event/pose/audio integration. No controls.js/Rust control behavior was edited.

## Check notes

The first new mode-cleanup fixture attempted another attack while the preserved attack cooldown was still active; corrected the fixture to wait that existing cooldown. A Boss fixture initially buffered after the first active hit had already happened; corrected to buffer during the real telegraph before its one strike. Neither correction changed production balance/physics.

Clippy's large publish query/8 parameters were addressed by PublishedActors alias and MotionRecorder SystemParam, which groups the shared fixed clock and bounded motion fact stream for motor/publish. A test borrow is scoped instead of calling drop on a non-Drop Mut.

## Final scoped validation

- cargo test --locked simulation:: -- --quiet:109 passed,0 failed/ignored (16 new cases).
- Strengthened actual_stride_feedback case passed after adding airborne+moving teleport.
- cargo test --locked --test physics_terrain -- --quiet:9 passed.
- cargo clippy --locked --all-targets -- -D warnings:passed.
- cargo check --locked --target wasm32-unknown-unknown:passed.
- cargo clippy --locked --target wasm32-unknown-unknown -- -D warnings:passed.
- cargo fmt --all -- --check and git diff --check:passed.

动作客户端已交接naraka_motion_client，不再编辑。主会话独立check、完整make check及真实浏览器集成验收仍需在其余切片汇合后执行。本子任务不冒称这些已经通过。
