# Web指针坐标与响应

## 1. 范围

适用于Pointer Lock、系统缩放、即时光标、世界/UI拾取及按下位置。
`web/pointer.js`唯一拥有Web的CSS位置、边界、单位校准和按下快照；
`GamePointer`是Rust输入帧的镜像，Native仍读取Window的绝对逻辑坐标。
捕获与开局确认仍遵循[会话契约](character-selection.md)，不由指针模块启动游戏。

## 2. 接口

```text
window.kivaPointer.sync(active: boolean): void
window.kivaPointer.position(): [] | [x, y]
window.kivaPointer.takePress(button: number): [] | [x, y]
window.kivaPointer.clearPresses(): void
web::sync_game_pointer(active: bool)
web::game_pointer_position() -> Option<Vec2>
web::take_pointer_press(MouseButton) -> Option<Vec2>
web::clear_pointer_presses()
GamePointer::observe(point: Vec2) // 输入帧内镜像，非浏览器实时坐标写入入口
```

桥通过wasm-bindgen调用浏览器适配，数组转为有限Vec2；空数组表示无有效样本。
button对应DOM编号：Left0/Middle1/Right2/Back3/Forward4，其它保持编号。
这些接口不提供写World或模拟运行态的权限。

## 3. 坐标、时序与生命周期

- 所有Web位置使用CSS像素。正常未捕获的可信pointermove，以client差值和movement样本估计raw→CSS比例，
  各轴分别忽略首样本、零/反向异常、父/子样本到达屏幕边界的轴、失焦与重入断点；累计加权而非单个短位移取比。
  无有效样本默认浏览器CSS单位1；未捕获时继续学习。DPR变化重置校准，不能在暂停帧重复清掉正常移动参考点。
- 不统一除devicePixelRatio，也不启用unadjustedMovement改变系统加速。
  浏览器movement单位有差异，依据[MDN](https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent/movementX)，
  默认加速约束见[Pointer Lock规范](https://www.w3.org/TR/pointerlock-2/)。
- 选择非空getCoalescedEvents或父事件，禁止父子重复积分；校准用子样本总位移对应父client差。
  锁定中按样本顺序积分并逐样本夹紧，保留同批越界后回移；只在整批末尾更新DOM/hover一次。
- 可信鼠标pointermove在document capture阶段立即更新CSS位置，不等Bevy帧/80ms HUD。
  原生DOM事件和系统鼠标加速保留；这减少输入链额外等待，不代表DOM光标具有系统硬件光标的独立合成能力。
- Rust确认当前玩家、未暂停、实际锁定与事件确认后sync(true)，首次启用居中并清旧按下快照。
  JS再次复核实际锁定、焦点、data-pointer-locked及data-paused；失锁/失焦/暂停立即隐藏并清队列。
  resize夹紧并更新显示；普通恢复重新居中。
- pointerdown及多键chord产生的pointermove保存按下时CSS坐标。Rust仍按WindowEvent顺序处理键盘/鼠标，
  每个按下MouseButtonInput消费同按钮的下一快照，再执行原UI/世界操作；不以后来移动到的位置替代。
  Web的MouseMotion/CursorMoved不再重复积分或逐raw样本做射线拾取，帧末读取最新点供hover和镜头。
- just_captured、无玩家、暂停边界排空旧按下和原窗口/DOM技能请求。Native绝对指针路径保持独立。

## 4. 校验矩阵

| 情形 | 结果 |
|---|---|
| 未激活/无实际锁定/未确认/暂停/失焦 | 不移动捕获指针、不提供按下样本、隐藏并清理 |
| 首个未捕获移动/零位移/裁剪轴 | 不污染校准，保留后续学习机会 |
| 非有限移动样本 | 跳过该样本，不引入NaN位置 |
| 越界后同批反向移动 | 每样本丢弃超出量，反向立即回到画布内 |
| 无按下样本或按钮错配 | 返回空并清队列，禁止旧样本错配到未来点击 |
| 外部绝对坐标出现在捕获期间 | 不代替浏览器相对积分结果 |

## 5. 正常、默认、错误案例

- 正常：150%环境下，正常指针移动30CSSpx时浏览器报30或45movement单位，校准分别为1或2/3，锁定后仍移动30。
- 正常：x998处批量+10、-5，宽1000时结果994；不能先求净+5再夹紧为999。
- 默认：尚无正常移动参考时使用CSS单位；进入前通常已有选择/点击按钮的正常鼠标移动样本。
- 错误：Rust与JS各积分一次、每帧Rust把DOM指针拉回旧坐标、丢失快照后使用最新位置执行点击。

## 6. 验证

`node --test scripts/pointer-unit.test.cjs`加载实际pointer.js，覆盖单位校准、coalesced边界及快照拒绝。
`browser-pointer-response.cjs`对比真实未捕获CSS移动，覆盖DPR1/1.5/2与物理单位fixture；
document capture记录before、window bubble记录after，断言同一事件内已更新。
微任务可在监听器之间运行，不能用前置capture监听器中的queueMicrotask代表所有监听器执行之后。
专项还持有真实游戏RAF并核对kiva-labels计数未变，验证光标继续响应、快速点击保留按下点、暂停与恢复；不写World或改模拟时钟。
原cursor/skills/equipment/characters/reselection及有窗口smoke按影响回归；Rust执行make check和WASM Clippy。

## 7. 错误与正确

错误：测试将移动距离先乘DPR以补偿生产缩放错误，再只检查最终到达指定点。

正确：浏览器辅助函数按正常CSS/CDP位移驱动，专项独立比较未捕获与捕获的移动比例。
位置/单位换算仅在Web指针所有者中实现，Rust输入镜像和相机转换读取它。
