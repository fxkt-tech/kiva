# 浏览器输入与 HUD

## 单向展示

`src/client/ui.rs` 将模拟只读状态序列化；`src/client/web.rs` 分发浏览器事件；
`web/hud.js` 更新 DOM。伤害、冷却、移速等规则不在 JavaScript 中计算。
`window.__kivaState` 是诊断投影，不是修改游戏状态的接口。
六格装备的暂停管理、独立主动意图、静态目录与回执见[装备交互](equipment.md)。

完整 HUD 快照约每 80 ms 发布一次。头顶血条位置通过 `kiva-labels` 每个显示帧发布；
慢速完整快照不能覆盖血条位置。见 `publish`、`publish_labels` 和对应浏览器监听器。

## 鼠标与坐标

- `web/pointer.js`拥有画布内的实时CSS坐标和按下快照，`GamePointer`只镜像输入帧坐标；详见[指针契约](pointer.md)。
- 进入/恢复游戏必须成功捕获鼠标；Esc、失焦或捕获丢失释放并暂停，失败则保持暂停。
- 实际锁定状态可领先捕获事件；等待本次pointerlockchange确认后才启用游戏光标与输入，详见[会话契约](character-selection.md)。
- 捕获中使用相对鼠标位移，忽略锁定系统光标的绝对坐标。
- viewport 逻辑坐标与 CSS 像素通过 `web.rs` 双向转换，高 DPI 下不能直接混用。
- HUD 使用 `[data-ui]` 隔离输入。游戏光标通过真实 DOM 按钮激活技能选择，不能穿透为地面指令。
- `controls.rs` 顺序消费 `WindowEvent`，保留同帧按键与鼠标事件先后。
- Web移动直接更新DOM，Rust不再用MouseMotion除DPR后二次积分；鼠标按下消费事件位置，帧末才读取最新位置。

## 内容与回归

技能/角色/装备目录由 `src/gameplay/content/` 装配，对局由 `gameplay::start_match` 创建；具体 ID 不进入相机、输入或伤害结算分支。
模拟接口变动必须同步更新演示内容、Rust 测试内容及依赖伤害/位置数值的浏览器断言。
新增 HUD 区域提供可读标签和稳定选择器，实际值来自 Rust 快照。

英雄和小兵使用本地源模型，基地保留轻量模型；`client/character`的刚性骨骼承担加载回退，见[中路对局](lane-match.md)。待机、真实移动、普攻前后摇、技能及控制姿态均只读模拟。
骨骼采样、时间、暂停和生命周期契约见[导入模型](imported-models.md)和[回退骨骼](character-animation.md)。
姿态不能改动`UnitVisual`世界根、底座、模拟碰撞或血条锚点，也不能触发伤害或结束冷却。模型命中跟随实际部件，见[模型拾取](unit-picking.md)。

`scripts/browser-lane-support.cjs`按真实地图派生近家/中线位置，等待固定步分批兵线，并在目标离开画布时先用真实箭头平移；不可伪造offscreen点击。

`scripts/browser-support.cjs` 已提供 Pointer Lock 下的 `movePointer`、`clickPointer`、`clickUi`、
`clickWorld` 和临时 HTTP fixture。新增浏览器测试复用这些辅助函数。


## 浏览器测试驱动的捕获校准

有窗口 Chromium 捕获后会重置 CDP 合成鼠标的移动锚点，无头模式会保留点击锚点。
`resumeGame` 先验证游戏自己已将光标居中，再将合成鼠标移动到已知位置、等待两个显示帧，
最后通过反馈式 `movePointer` 恢复中心，避免测试误把锚点差值当成游戏相对移动。
校准前的真实居中断言不能删除，否则测试会自行修正并掩盖生产代码的居中缺陷。
这仅是测试驱动适配，不改变游戏输入规则。

每次`resumeGame`申请前通过`page.bringToFront()`使浏览器原生视图处于前台。
Playwright模拟的DOM焦点不能代替Chromium的原生焦点检查；生产捕获拒绝仍显示重试并保持暂停，
测试不得通过自动重试或忽略拒绝来隐藏失败。
共享桌面/WSLg若仍干扰原生焦点，应使用测试独占的临时Xvfb显示进行有窗口检查；
测试必须在finally清理自己创建的显示进程/认证和HTTP fixture，不停止用户服务或修改生产暂停规则。

## 状态、技能装配与可见坐标契约

### 1. 范围

技能装配、控制投影、击飞表现、拾取或头顶标签变更时，检查以下完整数据流。
模拟规则见[abilities-and-statuses.md](../simulation/abilities-and-statuses.md)。
选角/无玩家阶段/开局与捕获的组合契约见[character-selection.md](character-selection.md)。

### 2. 接口

- `UnitView.statuses`: `{kind,remaining,duration,slow_fraction}`数组，时间由Rust计算为秒。
  kind使用`Stun/Root/Slow/Airborne`；非Slow的fraction为null。
- `UnitView.control`: `can_move/can_attack/can_cast/can_dash/is_displaced`布尔值。
- `UnitView.visual_height`: 实际已显示根的y；`screen/body_screen`为实际根+对应偏移的CSS像素投影。
- `AbilityView`: 装配名称、targeting、target_hint、icon、description、available、unavailable_reason，
  以及既有消耗/范围/冷却。`client/abilities.rs`为输入和HUD共用的只读适配。
- `kiva-labels`每帧包含id、screen、body_screen、alive；完整快照继续约80ms。

### 3. 规则与所有权

`UnitVisual`根拥有平面插值+Airborne高度；`UnitGroundBase`反向抵消高度；`CharacterModel`子树只负责骨骼表现。
`UnitStatuses::airborne_height`提供归一化连续曲线，峰值配置属于客户端。
拾取由picking逐个实际模型部件的Aabb/GlobalTransform投影判断，使用6 CSS px有限容错；根Pickable仅在无Mesh边界时fallback。
标签、完整快照和拾取仍读取同一可见根/其实际子树，不另用战斗位置重算；血条和名字不参与命中。公共候选与操作目标资格分离，详见[模型拾取](unit-picking.md)。
暂停保持根和模型实际Transform；死英雄根位于死亡点并保留尸体，真实重生后复位，战斗范围/指示仍使用平面位置。

JS展示状态和Rust生成的技能说明，不推导控制权限、减速数值或状态倒计时。
状态区域为`#hero-statuses`、`#target-statuses`，单位标签包含`.status-badges`；
徽章使用`[data-status=kind]`。七属性无单位面板保持数值契约，布局和相机取景见[hud-layout.md](hud-layout.md)。
动态装配更新按钮名称、图标、标签和说明；不得按Q/R或具体技能ID推断目标方式。
Point Dash超距仍显示可释放颜色，预览线终点截到射程圈；角色与装备共用execution.is_dash()和技能描述。
鼠标输入仍提交原点击点，权威终点由模拟提交时解析，不能只在Web输入侧截短来绕过模拟校验。

### 4. 输入与错误矩阵

| 情形 | 处理 |
|---|---|
| Enemy技能点击空地或友方 | 保持瞄准，不提交命令 |
| 点击Enemy的可见合法超距候选 | 公共preview仅OutOfRange时仍提交一次原Unit命令并退出瞄准，模拟负责走近后施法 |
| 点击敌方结构或其它预检拒绝候选 | 反馈拒绝并保留瞄准，不提交命令 |
| 点击通过只读预检的敌方候选 | 提交并退出瞄准；模拟仍复核生命/射程/资源 |
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
施法点/落点投影在提交R前计算，提交R使用即时按下/释放，不把默认90ms长按和坐标计算占在击飞窗口内。
真实左键之后，下一次逐帧标签通知先发送既有`data-paused=true`暂停意图，再调用原生`exitPointerLock`，顺序与Esc适配一致；
原生解锁通知可能晚一帧，不能让下一固定步继续推进。
不等待约80ms的完整HUD才暂停，随后仍强制断言实际选中目标、正高度和活动状态。
不要延长生产控制时间或增加写World接口来迁就测试。演示数值变化时同步预定位和位移结果断言。

即使预定位，慢软件GPU仍可能在可信pointerdown后的单帧内追赶15个固定步，耗尽剩余250ms控制。
交互脚本通过`browser-lane-support.installFrameClock`在导航前安装测试专用的performance.now/原生RAF包装，
每个不同原生RAF时间戳推进1000/60ms，同批回调不重复推进，回调时间与performance.now一致。
原生定时器、捕获和可信鼠标事件保留；当前context中不切回墙钟，避免大幅跳时。
长距离走位、状态尾段等待、暂停验证和完整对局的长时推进可临时改为每帧250ms，
模拟仍执行全部固定步，不能跳过战斗或兵波；短控制和真实拾取窗口恢复1000/60ms。
暂停断言等待测试时钟至少推进400ms且收到其后的新kiva-state，再比较tick/姿态/状态/坐标，
不能只等待宿主400ms或在停止测试时钟后断言冻结。
这是按显示帧归一的交互/姿态检查，不证明真实墙钟时长或长帧性能；规则时序由核心tick场景覆盖，
原生显示频率与HUD节流证据由不安装帧时钟的`browser-healthbars`提供。
帧时钟启用后，browser-skills使用120秒宿主等待预算；慢软件GPU可能在20秒内只推进少量虚拟tick，
增加执行预算不能修改生产控制时长或跳过活动状态/位置断言。
可用`KIVA_SKILLS_CASE=1000x720@2`单独执行指定尺寸，默认仍执行三种尺寸；未知case必须报错，不能零场景通过。

`status`等待短控制效果时直接返回满足条件的快照对象，并读取/释放该JSHandle。
不要先waitForFunction、再另一次evaluate读取当前快照；第二次CDP往返可能已跨过状态结束，
把不同时间的状态存在性和控制权限拼到同一个断言里。


## 启动通知与DOM顺序

`startup.js`必须以同步经典script安装在Trunk的Rust标记之前；该标记会原地生成含顶层await的module，
加载失败可能早于defer HUD执行。早期适配只记录loading/failed/ready，不访问DOM；
`kivaStartup.subscribe`即时回放当前状态，HUD就绪后据此显示loading失败、重试按钮或开局菜单。
`__kivaStartup`仅为诊断投影，HUD不得自行重置为loading或另注册一套启动事件所有者。

实际kiva-ready可以覆盖此前非致命的启动错误；一旦ready，不再接受迟到的error/unhandledrejection/
TrunkApplicationError覆盖。应用加载失败后的重试仍使用页面reload，不能用定时延迟或自动重试隐藏错误。

`node --test scripts/startup-unit.test.cjs`在没有DOM/GPU的环境验证三类早期错误、晚订阅、早ready与ready胜出。
`browser-startup.cjs`以请求屏障阻塞hud.js，先实际abort WASM并确认failed已保留，再放行HUD检查alert/retry；
成功reload后再派发旧Trunk错误确认不会覆盖ready。
另一个正常WASM场景阻塞HUD直到实际kiva-ready和首个9项equipment_catalog均已发出，再放行并真实选角/捕获/按P，
断言九件目录仍完整；只读初始目录由同步equipment模块缓存，startup仍只拥有三态，不保留整局快照。
Node状态测试同时在没有DOM的VM加载两个同步head模块并约束其安装顺序。
该浏览器检查与重建/最终smoke串行运行。
