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

## 状态、技能装配与可见坐标契约

### 1. 范围

技能装配、控制投影、击飞表现、拾取或头顶标签变更时，检查以下完整数据流。
模拟规则见[abilities-and-statuses.md](../simulation/abilities-and-statuses.md)。

### 2. 接口

- `UnitView.statuses`: `{kind,remaining,duration,slow_fraction}`数组，时间由Rust计算为秒。
  kind使用`Stun/Root/Slow/Airborne`；非Slow的fraction为null。
- `UnitView.control`: `can_move/can_attack/can_cast/can_dash/is_displaced`布尔值。
- `UnitView.visual_height`: 实际已显示根的y；`screen/body_screen`为实际根+对应偏移的CSS像素投影。
- `AbilityView`: 装配名称、targeting、target_hint、icon、description、available、unavailable_reason，
  以及既有消耗/范围/冷却。`client/abilities.rs`为输入和HUD共用的只读适配。
- `kiva-labels`每帧包含id、screen、body_screen、alive；完整快照继续约80ms。

### 3. 规则与所有权

`UnitVisual`根拥有平面插值+Airborne高度；`UnitGroundBase`反向抵消高度；`UnitModel`只做后摇姿态。
`UnitStatuses::airborne_height`提供归一化连续曲线，峰值配置属于客户端。
拾取AABB中心为实际`GlobalTransform.transform_point(Pickable.offset)`，包围半径同样转换。
标签、完整快照和拾取共用可见根，不再各自用战斗位置/高度公式重算。
暂停保持根和模型实际Transform，死亡隐藏、重生清零，战斗范围/指示仍使用平面位置。

JS展示状态和Rust生成的技能说明，不推导控制权限、减速数值或状态倒计时。
状态区域为`#hero-statuses`、`#target-statuses`，单位标签包含`.status-badges`；
徽章使用`[data-status=kind]`。144px无单位七属性面板保持原契约。
动态装配更新按钮名称、图标、标签和说明；不得按Q/R或具体技能ID推断目标方式。

### 4. 输入与错误矩阵

| 情形 | 处理 |
|---|---|
| Enemy技能点击空地或友方 | 保持瞄准，不提交命令 |
| 点击合法敌方候选 | 提交并退出瞄准；模拟仍复核生命/射程/资源 |
| 合法候选的提交后校验失败 | 提示拒绝，不缓存自动重试 |
| 新控制让已选技能不可用 | 清理瞄准；后续仍由模拟强制校验 |
| Hover/点击飞起的模型 | 按最后实际显示根拾取，不使用当前未插值地面UnitPose |
| 状态/属性/提示区域点击 | 通过data-ui隔离，不产生世界命令 |

### 5. 示例

正常：W同一步显示Root/Slow和最终移速3.85；R飞起后body_screen随可见模型，标签在其上方。
默认：无控制时控制权限允许，状态区域无活动徽章；冷却和能量仍决定技能可用性。
错误：在JS里另开倒计时、给R写固定自身释放，或把飞起的人物点击框留在地面。

### 6. 验证

scene/controls无GPU场景验证可见根拾取、空地/友方瞄准、控制和暂停；
`browser-skills.cjs`验证鼠标Q/W/E/R、状态与移速同快照、击飞可见身体选择和暂停冻结，
覆盖1280×800、800×600和1000×720 DPR2。原cursor/stats/healthbars/smoke保持回归。
浏览器测试仅监听只读诊断事件，不新增写World接口；临时HTTP fixture在finally清理。

### 7. 错误与正确

错误：人物根按击飞曲线飞起，拾取仍用UnitPose，完整HUD还用固定地面高度覆写血条。

正确：只有一个实际显示根；拾取与两个投影通道读取它，逐帧标签坐标仍由独立轻量通道更新。

## 短动画的自动化时序

软件WebGPU与CDP往返会继续推进模拟，不能假设“采样坐标→移动鼠标→等选择→按Esc”能在800ms内完成。
`browser-skills`在弹丸飞行时按演示R的4单位终点提前放置光标，待实际强制运动结束且仍击飞时再点击；
浏览器观察到空中目标被选中便调用原生`exitPointerLock`走生产暂停路径，随后仍强制断言正高度和活动状态。
不要延长生产控制时间或增加写World接口来迁就测试。演示数值变化时同步预定位和位移结果断言。
