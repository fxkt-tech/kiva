# 角色系统验收记录

基线7eb69c0。实现与验收完成，包含用户补充的重新选择/整场重开。用户现已授权提交与推送；开发服务由用户管理。
仅实际完成的检查记为通过。

## 质量入口

- `make check`通过：格式、原生all-target Clippy（deny warnings）、138项Rust测试、WASM check。
- 138项包括37个客户端unit、6个公开角色场景、56个既有模拟场景、37个既有技能/状态场景、2个重置场景。
  原109项及首版新增21项全部保留，重新选角补充8项。
- `cargo clippy --locked --target wasm32-unknown-unknown -- -D warnings`通过。
- `node --check`检查hud、browser-support、browser-characters、browser-skills、browser-reselection通过；git diff --check通过。
- 最终`trunk build --dist /tmp/kiva-reselection-site`通过，包含捕获确认及整场重新选择实现。
- 独立全范围复核和修复后二次边界复核通过；未发现未处理问题。浏览器由主代理单独运行，避免焦点干扰。
- JSON/JSONL结构、14项上下文路径、Markdown本地链接检查通过。

## 浏览器验收

首版生产构建使用Chromium158与软件WebGPU，下表保留首版验收历史；补充后的最终构建见P6记录。
交互检查使用真实浏览器事件和Pointer Lock，
不写World、属性、生命或控制状态；下表均有实际通过记录。

| 脚本 | 运行模式 | 结果与覆盖 |
|---|---|---|
| browser-characters | headless | 1280×800、800×600、1000×720 DPR2全部通过；无玩家冻结、快速选择/回执、两角色投影、秘术师Q/R、非法/过期/无Launch捕获、重复启动、同实例恢复、拒绝重试、旧Promise、真实失焦 |
| browser-characters截屏 | headed | 三尺寸选角和真实WebGPU入局画面人工检查通过；小窗口可滚动且开始按钮可达 |
| browser-skills | headless | 三尺寸Q/W/E/R、非法点选、组合控制/减速恢复、击飞中真实身体拾取、暂停冻结全部通过 |
| browser-smoke | headed | 完整玩法、普攻后摇、移动/攻击/技能、镜头/边缘/缩放、UI隔离、失焦恢复及高DPI拾取通过 |
| browser-stats | headless | 三尺寸七属性/144px/无单位布局与实际值一致，点击隔离、移动、暂停/恢复通过 |
| browser-cursor | headless | 边界和越界回收、边缘平移、Esc恢复、技能点击、高DPI缩放拾取、捕获拒绝通过 |
| browser-healthbars | headless | 移动单位90/90帧更新、移动镜头20/20帧更新；无连续停滞，完整HUD仍80ms限频 |
| browser-startup | headed | WASM下载失败显示错误、重新加载后ready通过 |

有窗口自动化曾出现原生焦点拒绝及合成鼠标校准时丢锁，未把这些运行记为通过；
交互专项随后在headless Chromium完成，真实渲染外观由headed截图及smoke覆盖。
所有fixture都在finally关闭，没有接管用户开发服务。

## P6：重新选择角色与整场重开

用户明确选择“重开整场试炼”。新增暂停菜单入口和重置说明；同角色重开与切角色均不刷新页面。

- `tests/reset.rs`：实际控制/运动/弹丸/延迟区域/引导、待执行命令/属性/事件清理，静态实体与目录保持，
  新局无残留且属性来源/状态等分配器不复用；2项通过。
- 客户端新增6项：受损/冷却后整局重建、暂停捕获和当前Entity门控、跨局修订、同批尾部丢弃、
  重复/旧玩家请求、无玩家输入反馈、巡游tick缓存和既有镜头复位；37项lib全通过。
- 独立复核无生产缺陷。协议措辞收敛为：返回同批尾部选择意图丢弃、旧版本Launch/旧玩家Return跨批也拒绝；
  当前Select为界面同步意图，没有跨局异步传输上下文。未来引入异步时需扩展局代次，已记录规范。
- `browser-reselection.cjs`在1280×800、800×600、1000×720 DPR2全部通过：实际伤害/提交技能后重开，
  NPC满血原位、旧实体消失、tick0冻结、同角色/换角色、旧Launch真实捕获拒绝、旧玩家返回拒绝、普通恢复同Entity。
  页面timeOrigin保持，实际HUD/标签/技能/反馈/镜头清理符合预期。
- 补充后的`browser-characters`完整三尺寸/捕获失败/旧回调/真实失焦回归通过；`browser-cursor`全部通过。
- headless smoke的WebGPU截图几何断言因已知黑画布不通过，未计为通过；共享WSLg有窗口测试又遇原生捕获拒绝/丢失，
  应用正确保持选择/暂停，未修改生产行为或放宽断言。
- 隔离的临时Xvfb显示下，原`browser-smoke`完整有窗口回归通过，包括实际像素几何、技能/后摇/镜头/缩放/失焦/高DPI。
  三尺寸暂停菜单及返回选角的有窗口检查/截图通过，按钮完整可达、重开说明明确；已检查实际截图。
  Xvfb和HTTP fixture均由测试拥有并在finally退出，临时认证已清理；没有修改系统安装或用户服务。

## 捕获时序问题与预防

Chromium trace实证：实际pointerLockElement更新、原生Promise完成后，pointerlockchange尚未分发；
期间Bevy帧将Selecting的实际捕获当成单独捕获释放，随后事件只能观察到null，不能发出Launch。
同一间隙还会提前消耗Active恢复的just_captured，导致旧DOM技能可能跨过输入边界。

修复由web适配层区分actual与event_confirmed；每次申请清除旧确认。Selecting仅在事件确认后、
消费请求仍无玩家时释放捕获。controls先计算运行门控，再以confirmed&&!paused同步GamePointer。
Launch仍复核真实捕获/焦点，不能以DOM确认代替；单独捕获不能启动模拟。
新增3项确定性Rust回归，浏览器另延迟真实捕获通知四个显示帧，覆盖初次开局和同实例恢复。
契约已写回设计文档、架构及客户端规范，未使用延时重试、放宽断言或修改生产角色/技能数值。

随后有窗口自动化的恢复请求曾被原生浏览器以WrongDocumentError拒绝，应用保持Active同实例、暂停并提示重试。
匹配版本Chromium158源码中[RequestMouseLock](https://github.com/chromium/chromium/blob/158.0.8127.18/content/browser/renderer_host/render_widget_host_impl.cc)
与[CanBePointerLocked](https://github.com/chromium/chromium/blob/158.0.8127.18/content/browser/renderer_host/render_widget_host_view_base.cc)
将原生视图无焦点映射为该错误；[Playwright](https://github.com/microsoft/playwright/blob/v1.65.0/packages/playwright-core/src/server/chromium/crPage.ts)
默认的焦点模拟使DOM hasFocus=true不足以证明原生焦点。因此恢复辅助函数先bringToFront建立前台条件后复验。
这些来源支持原生焦点门控，不能证明具体谁移走了焦点；没有为此修改生产规则或增加自动重试。

既有短击飞测试还暴露了自动化时序问题：状态存在与控制权限原来分两次CDP读取，
以及等待80ms HUD/多次光标反馈时短动画可能已结束。帧记录确认软件渲染可超过半秒一帧。
修正为返回命中状态的同一快照、施法前准备坐标、预规划真实点击后移动，
并在同一个等待条件核对光标位置与活动击飞。可信左键后的下一显示帧发送既有暂停意图并释放锁，
冻结后仍强制检查实际选中、205生命、禁止施法、活动Airborne、正高度与冻结不变。
原2.6瞄点/4单位位移/0.8秒控制和全部生产数值保持；最终三尺寸一次完整运行通过。

## 最终验收映射

| 验收 | 结果 | 实际证据 |
|---|---|---|
| AC1 | 通过 | characters公开注册拒绝、目录保持和ID顺序场景 |
| AC2 | 通过 | 默认旅者及秘术师开局；唯一角色实体、名称/属性/实际技能匹配 |
| AC3 | 通过 | Explicit完整替换、空槽、重复ID独立冷却和按实际槽施法场景 |
| AC4 | 通过 | 同角色多实例生命/属性/状态/冷却独立，目录保持 |
| AC5 | 通过 | 纯预览与构造共享初始计算，攻速边界；浏览器七项初始HUD一致 |
| AC6 | 通过 | 正常死亡重生保持身份/显式装配，既有生命代次和清理规则 |
| AC7 | 通过 | Selecting无玩家/空技能/tick0/NPC冻结；DOM未暂停及单独捕获不能开始 |
| AC8 | 通过 | 最新选择回执、快速切换、未知选择及旧版本Launch拒绝 |
| AC9 | 通过 | 注册/创建失败原子性、失焦/捕获拒绝保持选择并可重试 |
| AC10 | 通过 | 重复Launch/Active选角拒绝，同一Entity/技能冷却不重置 |
| AC11 | 通过 | Esc和真实文档失焦，恢复仍同角色/装配/Entity；核心重生保持 |
| AC12 | 通过 | 原窗口事件顺序、无玩家排空、4帧事件延迟、居中/旧DOM技能丢弃及UI隔离 |
| AC13 | 通过 | 三尺寸截图/DOM/键盘选择与开始，144px短标签无单位HUD |
| AC14 | 通过 | NPC/raw spawn保留；Native自动默认角色同路径测试和全target编译；无玩家consumer检查 |
| AC15 | 通过 | 130项Rust、WASM Clippy、JS语法、Trunk、全部相关浏览器入口 |
| AC16 | 通过 | 仅内容注册区使用演示角色ID；输入/战斗共用规则；公开Explicit装配场景验证后续模式入口 |
| AC17 | 通过 | 真实暂停菜单鼠标/键盘返回；无刷新、无玩家、tick0冻结；三尺寸按钮可达 |
| AC18 | 通过 | 两项公开reset测试覆盖载体/引导/队列/事件/子实体清理；新NPC满血原位、旧实体消失 |
| AC19 | 通过 | 浏览器换角色及同角色多次重开，新玩家/初始技能与HP正确，地图保持、无额外玩家 |
| AC20 | 通过 | 会话跨局版本、当前玩家ID、重复返回和批次边界测试；真实捕获下旧Launch拒绝 |
| AC21 | 通过 | 无玩家清理输入反馈，既有camera复位、patrol缓存清理；浏览器普通恢复仍同Entity |
| AC22 | 通过 | 原130项保留，新增8项共138通过；最终WASM/Clippy/Trunk与三尺寸重新选择及相关回归通过 |

开发服务由用户管理；浏览器使用自建临时HTTP fixture并在finally关闭。
最终构建目录为/tmp/kiva-reselection-site，测试环境复用本机已验证的Chromium软件WebGPU配置。
