# 第一版动作输入契约

## Scope / Trigger

修改 controls.rs、client/web.rs、web/input.js 或设备输入回归时阅读。设备层只提交意图，右键长按识别、冷却、命中和耐力均由固定步模拟负责。

## Signatures

- window.kivaInput：isActive(): boolean、takeLook(): number、keys(): number、takeActions(): number、takeDefenseEdges(): number[]、ready(): void。
- takeDefenseEdges：1=Pressed，2=Released，3=Cancelled；最多 64 条，按发生顺序一次消费。
- keys：W=1、S=2、A=4、D=8、左右 Shift 聚合为 sprint=16。
- takeActions：jump=1、dodge=2、attack=4、melee_skill=8、combo=16、toggle_target=32、pair1=64、pair2=128、ranged_skill=256、crouch=512、movement_released=1024、shift_released=2048。
- Rust 把边沿提交给 DefenseState::push_edge；清理调用 DefenseState::clear，不能伪造 Released。
- CombatIntent 分别接收 request_melee_skill、request_ranged_skill、request_toggle_target、request_pair(WeaponPair)；无 takeAim、滚轮切人格或 Q 切目标。

## Contracts

输入在 BeforeFixedMainLoop 采样。鼠标水平以 0.0025 rad/CSSpx 更新 ViewYaw，不乘 dt、不盲目除 DPR；垂直不改变镜头俯仰。
左键始终为哥哥近战，E/Q 为普通近/远技能，中键按压奇偶性切锁定，1/2 选武器组；长按按键不重复施放。C 按压使用 XOR 奇偶性，松键不取消蹲伏。
原生按当前窗口的 KeyboardInput／MouseButtonInput 顺序消费，不能仅比较两帧最终 pressed 快照。Shift 两键聚合首次按下才闪避；物理方向键事实独立于方向向量是否抵消。
movement_released／shift_released 锁存所有方向键／所有 Shift 释放事实；即使同次采样前已重新按下，也必须先 set_*_held(false)，再提交最终 held 状态。否则旧疾跑锁存与长按时长会跨一次完整松键错误延续。原生与 Web 同义，不能只处理最终位掩码。
右键有序队列保留同帧短按。显示帧零固定步时 Rust 中意图保留，多固定步只消费一次；长按计时不使用 JS 墙钟。
捕获必须由 pointerlockchange 加焦点共同确认，Promise 完成不代表捕获成功。捕获点击不同时攻击。
首次入口的ready由必要场景渲染准备与提交决定，不能在Startup末尾提前放行；加载进度和捕获进行中反馈见 [startup.md](startup.md)。ready前按钮与画布都不能申请捕获；恢复不重新加载。
Esc、blur、pointercancel、丢失捕获均取消，不能开启释放弹反。取消锁存跨过同帧失焦和重新捕获；队列溢出也先取消，保持按住不能自动重建 Pressed，要求一次新按压。
失活清 MoveIntent、ActionIntent、CombatIntent 和 DefenseState 意图；不改 MotionState、不补跳跃额度、不清已承诺技能／冷却。
联动期间丢弃转向样本而非累计，Facing 对齐序列原目标；完成后恢复正常视角。

## Validation & Error Matrix

| 情况 | 结果 |
|---|---|
| 右键按下和松开发生于一次采样前 | 依次消费 Pressed／Released，仍有一次拆招，不额外弹反 |
| 失焦后同帧重新捕获 | Cancelled 不丢失，不重放先前按住或攻击 |
| 重复 keydown／重复 pointerdown | 不增加额外请求 |
| 右键队列超过 64 | 取消手势，要求重新按压 |
| 其他窗口／非可信浏览器事件 | 忽略 |
| 无固定步／多个固定步 | 请求保留／只消费一次 |
| 输入方向非有限／长度超限 | 模拟归零／限长，客户端不修改位置补救 |

## Good / Base / Bad Cases

Good：快速右键点按后立即失焦再进入，旧按压被取消，没有释放弹反。
Base：E、Q 分别请求技能，中键锁定／解锁，空格按下即请求跳跃。
Bad：JS 计时后直接宣布进入防御，或失焦调用 Released 产生弹反。

## Tests Required

实际 scripts/input-unit.test.cjs 执行桥脚本，覆盖 E/Q、中键、完整短按、释放、取消、溢出、repeat、捕获、同帧失焦再捕获、Shift／C 和 CSS 像素校准。
原生事件回归检查窗口、按压/释放顺序、左右 Shift 和自动 repeat；真实 sample → FixedUpdate 回归覆盖零/多步、防御取消、已承诺技能在输入清理后保持。
执行 make check 与 WASM Clippy，确保 Vec<u8> 与浏览器 number[] 的实际 wasm-bindgen 桥可编译。

## Wrong vs Correct

错误：只同步右键最终 held=false，丢失短按；失焦被当作松开。
正确：有序边沿与独立 Cancelled，模拟自行识别手势，设备清理不产生动作。
