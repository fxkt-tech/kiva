# 独立质量复核

审查基线：`e00d861`；范围包含当前任务 PRD、design、implement、`docs/TODO.md` 和用户参考图 `docs/hud_lol_game.png`。依据 `trellis-check` 执行全范围只读审查；未修改产品源码。

## 结论

代码与规范审查通过，未发现需要修复的阻塞问题。真实 Bevy/WebGPU、原生 Pointer Lock 与运行中生命周期回归由主代理在冻结后的最终构建上串行执行；本记录中的 DOM 证据不能替代实际游戏验收。

## 范围与架构

- 参考图的底部紧凑操作栏、左侧属性/头像、中间四技能及下方 HP/MP、右侧三列两排装备、右下小地图、左上选中目标、顶部紧凑对局信息均有对应实现；没有加入左侧训练工具栏或伪造经验、金币等尚无规则的数据。
- `hud.css` 统一拥有对局布局；`style.css` 保留主题、页面与菜单；`match.css` 保留模式事实和结果展示。旧的操作栏/小地图冲突媒体规则已移除。
- 模拟、gameplay 无变更。相机仍保留自由平移、边缘滚屏、滚轮、Y、Space、地图边界和会话重置；默认拉近同时作用于 setup 与 update，共用变换计算。
- `camera::ground_footprint` 用当前 Bevy Camera 的实际逻辑 viewport 四角射线与地面相交；完整状态与既有矩阵读取同一 Camera/GlobalTransform。无有效投影时返回 null，不制造默认视野框。
- `minimap.js` 只读已发布地图、团队可见单位和相机四角；无 World 写口、额外计时器或独立玩法规则。backing 尺寸仅变化时更新，每次 setTransform 重置尺度，无累积缩放或 resize 循环。
- 英雄头顶 MP 来自完整只读状态；创建标签时无完整 mana 元数据则保持隐藏，兵/基地没有伪法力条。完整状态不覆写位置，逐帧位置仍只由 `kiva-labels` 更新。
- 新被动入口复用 `#ability-detail`；Pointer Lock 的 `[data-ui] button` 通用 hover 确实覆盖该按钮；没有技能请求监听或世界点击穿透。其余技能/装备/暂停/重选角协议未改。
- 保留空玩家、Selecting、死亡、MP 为 0、无最大法力、空视野、暂停与重开路径；资源宽度只作有界显示，未在 JS 推导游戏规则。

## 验证与证据

独立执行并通过：

- `git diff --check`。
- `node --check web/hud.js`、`web/minimap.js`、`scripts/browser-stats.cjs`、`scripts/browser-healthbars.cjs`。
- 用 HTMLParser 对比 `e00d861:index.html`：既有 74 个 ID 全部保留且无重复；四技能 `data-slot=0..3`、六装备 `data-equipment-slot=0..5` 顺序保持。
- 实际查看 `/tmp/hud-layout-1280x800-dpr1.png` 与 `/tmp/hud-layout-800x600-dpr1.png`，主要区域均在画面内，无互相遮挡。

审阅主代理与实现代理提供的已执行证据：

- `/tmp/kiva-hud-check.log`：完整 make check 通过，306 通过、2 项既有忽略；包含 native 和 WASM 检查。
- `/tmp/kiva-hud-wasm-clippy.log`：WASM Clippy 通过。
- `/tmp/kiva-hud-node.log`：19 项 startup/pointer Node 测试通过。
- `/tmp/kiva-hud-layout-check.log`：1920×1080、1280×800、800×600、1000×720 DPR2 及额外 640/480 窄窗口 DOM 几何通过。
- `/tmp/kiva-hud-render-check.log`：实际 HUD 模块集成通过，包含首帧无 mana 元数据、完整状态与逐帧位置分离、HP/MP、被动共同 tooltip、DPR backing 与 null footprint。

测试有效性核对：

- 五项新增 native 相机测试使用真实 Bevy 投影，覆盖初始变换一致、两米身体屏幕比例、四角 round-trip、缩放/宽高比/DPR/viewport offset、退化投影和滚轮门控。
- `browser-stats.cjs` 保留既有数值配对、标签、尺寸和输入隔离断言，新增 1920 场景、资源层次、装备网格、默认人物占比、相机四角回投、DPR backing 与真实选中目标位置；指定场景为空时会失败。
- `browser-healthbars.cjs` 仅通过真实 wheel 将视野恢复到旧长路径测试所需范围，同时断言视野面积实际改变。原生 performance/RAF、连续移动、20 帧与 80% 变化阈值未放宽；此脚本不代替默认近镜头比例的 browser-stats 验证。

## 最终收尾

产品源码在本次审查期间未修改；最终实际游戏证据、README/architecture/TODO 勾选由主代理在全部验收通过后完成。当前 HUD 工作不自动提交或推送，任务归档留至后续提交。

## 最终构建与 fixture 增量复核

主代理完成实际游戏验收后，再次独立读取最终脚本与日志。以下证据补足上文当时仍待执行的真实游戏检查：

- `/tmp/kiva-hud-browser-stats.log`：实际 WebGPU 下 1920×1080、1280×800、800×600、1000×720 DPR2 全部通过；包含被动 hover、点击紧凑装备格后的真实主动施放、独立装备冷却、真实重新选角、输入隔离与暂停恢复。
- `/tmp/kiva-hud-browser-smoke.log`：真实 renderer、地面拾取、移动/停止、Y/Space、捕获滚屏、实际滚轮缩放、resize、高 DPI 拾取、失焦冻结及同局恢复通过。
- `/tmp/kiva-hud-browser-healthbars-final.log`：原生 RAF 下人物 20/20 帧位置更新且 20/20 可见；平移相机 19/20 帧更新、20/20 可见、最大连续停滞 1 帧。相机累计真实平移 80.595，整段离地图夹紧边界至少 2.452；HUD 快照节流断言通过。
- `/tmp/kiva-hud-source-before.json` 与 `/tmp/kiva-hud-source-final.json` 内容相同，以上证据对应同一份冻结产品源码。测试独占 Xvfb 与临时认证均已按日志清理。

血条 fixture 的最后一次增量仅适配慢软件 GPU：长达数秒的 20 个原生帧中，单向持键会把静止基地移出窗口。因此测试从地图内部开始，原生 RAF 只读基地 DOM 边界，经暴露给 Node 的测试 binding 串行发送真实 Playwright 左右方向键，保持静止基地在窗口内；不直接改相机、不写 World、不改墙钟。

已核对控制器清理：`finally` 先关闭 active、停止 monitor，再等待串行 pending 输入，最后松开两个方向键；排队操作检查 active，正在执行的操作结束后也会被最终双键释放覆盖。外层 `browser.close`/fixture close 仍为错误路径提供最终清理。binding 和停止函数只存在于该临时测试页面。

20 帧、80% 可见、80% 位置更新和完整 HUD 每 80ms 上限全部保留。相机改用累计行程，避免真实往返最终回到起点造成净位移假失败；同时新增整段最小地图边距检查，并保留最终边距检查，没有降低跟随标准。再次执行 `node --check scripts/browser-healthbars.cjs` 与 `git diff --check` 均通过。

最终增量审查通过，未发现阻塞项，未改产品源码。
