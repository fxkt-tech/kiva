# 浏览器输入与 HUD

## 单向展示

`src/client/ui.rs` 将模拟只读状态序列化；`src/client/web.rs` 分发浏览器事件；
`web/hud.js` 更新 DOM。伤害、冷却、移速等规则不在 JavaScript 中计算。
`window.__kivaState` 是诊断投影，不是修改游戏状态的接口。

完整 HUD 快照约每 80 ms 发布一次。头顶血条位置通过 `kiva-labels` 每个显示帧发布；
慢速完整快照不能覆盖血条位置。见 `publish`、`publish_labels` 和对应浏览器监听器。

## 鼠标与坐标

- `src/client/pointer.rs` 的 `GamePointer` 拥有被限制在画布内的 CSS 像素坐标。
- 进入/恢复游戏必须成功捕获鼠标；Esc、失焦或捕获丢失释放并暂停，失败则保持暂停。
- 捕获中使用相对鼠标位移，忽略锁定系统光标的绝对坐标。
- viewport 逻辑坐标与 CSS 像素通过 `web.rs` 双向转换，高 DPI 下不能直接混用。
- HUD 使用 `[data-ui]` 隔离输入。游戏光标通过真实 DOM 按钮激活技能选择，不能穿透为地面指令。
- `controls.rs` 顺序消费 `WindowEvent`，保留同帧按键与鼠标事件先后。

## 内容与回归

演示技能和单位在 `src/client/content.rs` 装配，具体 ID 不进入相机、输入或伤害结算分支。
模拟接口变动必须同步更新演示内容、Rust 测试内容及依赖伤害/位置数值的浏览器断言。
新增 HUD 区域提供可读标签和稳定选择器，实际值来自 Rust 快照。

普攻后摇由 `UnitAction::AttackBackswing` 只读驱动：HUD 显示“攻击后摇”，
`sync_unit_models` 仅对 `UnitModel` 子层的身体/头部/武器做收招插值。
姿态不得改动 `UnitVisual` 根变换、底座、拾取范围或血条锚点，也不得由动画触发伤害或结束冷却。
暂停时保留已经显示的模型 Transform；不要把插值比例改成0后重算姿态，这会使短后摇倒退。
`client::scene::tests::pause_retains_rendered_backswing_pose_until_presentation_resumes` 用真实普攻与
0.9 tick 插值验证暂停冻结、恢复更新和停止后回到中立姿态。

`scripts/browser-support.cjs` 已提供 Pointer Lock 下的 `movePointer`、`clickPointer`、`clickUi`、
`clickWorld` 和临时 HTTP fixture。新增浏览器测试复用这些辅助函数。


## 浏览器测试驱动的捕获校准

有窗口 Chromium 捕获后会重置 CDP 合成鼠标的移动锚点，无头模式会保留点击锚点。
`resumeGame` 先验证游戏自己已将光标居中，再将合成鼠标移动到已知位置、等待两个显示帧，
最后通过反馈式 `movePointer` 恢复中心，避免测试误把锚点差值当成游戏相对移动。
校准前的真实居中断言不能删除，否则测试会自行修正并掩盖生产代码的居中缺陷。
这仅是测试驱动适配，不改变游戏输入规则。
