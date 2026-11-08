# 小地图导航增量独立审查

## 审查范围

依据 `prd.md` 的追加验收和 `minimap.md`，阅读 AGENTS、Trellis check、client 索引 Quality Check、browser/pointer/hud-layout 规范及共享跨层指南。
项目仍为单 crate；此轮新增行为仅在 client/Web，不修改 gameplay/simulation。前轮 HUD 全范围审查保留在 [review.md](review.md)。

检查读取链：`Camera + GlobalTransform → ground_footprint → ui::publish_camera → kiva-camera → minimap viewport`，以及写入链：`kivaPointer CSS 坐标 → minimap 逆投影/拖动偏移 → 最新相机意图 → web bridge → CameraRig`。

## 已检查

- Rust 每帧消费并移除 `data-camera-request`，严格解析字段、核对当前玩家身份和有限坐标，暂停/无玩家拒绝；地图边界统一，导航退出跟随且保留 zoom，持有拖动时不执行其他镜头控制。
- 独立相机事件在实际相机变换传播与投影更新后发布，保持旧 `kiva-labels` 数组与完整快照字段；HUD/血条仍保留各自原更新通道。
- 地图固定显示完整世界范围，8 CSS px 内边距；地形、基地、中路和可见单位共用正向转换，点击共用逆转换。DPR 只影响后备像素。
- 矩形取真实四角的轴对齐外接范围并裁切到地图，逐帧更新只操作框，不重绘底图。透视四角在地面为梯形，矩形是外接范围；不得把它描述为与地面视野逐点同形。该取舍已在用户进度说明及设计中明确。
- 左键按下读取 pointer 所有者提供的 CSS 坐标，尊重原生可信事件、Active、锁定及确认、焦点和暂停门控；框内拖动保持最初抓取偏移，不用 JS 预测相机。
- 普通松开保留最后目标供 Rust 消费；取消、失焦、失锁、暂停、resize、玩家/阶段变化清理拖动及旧请求。overlay 继承 data-ui，框自身 pointer-events:none，命中仍由 stage/canvas 实际位置判定。
- HTMLParser 对比 `e00d861:index.html`：74 个旧 ID 全部保留且无重复；技能 `0..3`、装备 `0..5` 顺序不变；新增 stage/viewport 选择器存在。
- `node --check` 对 minimap、其 Node 专项和实际浏览器专项通过；`git diff --check` 通过。

## 审查发现及处理

1. **偏移抓取的边界顺序**：先夹紧鼠标世界点再加抓取偏移，会使部分相机边界不可达。已通知 Web 所有者并独立复核修复：先逆投影并加偏移，再夹最终请求目标，Rust 继续权威夹到既有边界。新增非零偏移双向越界回归。
2. **拖出地图后的同帧按键归属**：仅使用当前 DOM 上 dragging 不能保证多键按下、全部松开都落在一个 Bevy 帧内时的 UI 隔离。root 已将 `uiCaptured` 纳入 pointer 所有者的每次按下快照，桥返回 `[x,y,0|1]`；Rust 严格解析并跳过该次被 UI 捕获的鼠标操作，不依赖消费时的当前 DOM 拖动态。已核对 stop 复位、结束手势不篡改已排队快照及新按下恢复普通归属。
3. **多键释放不等于丢失按键**：左键在另一按钮仍按住时由 `pointermove(button=0,buttons=2)` 表示释放。最初实现把它并入丢失 buttons 的取消分支，丢掉一帧内尚未消费的有效导航目标。已通知 Web 所有者修正：明确左键释放沿用普通 pointerup 保留末目标；未收到明确释放的 buttons 丢失仍取消并丢弃。新增多键开始/移动/释放测试，普通持键移入不会启动手势。

## 已审阅验证证据

- `/tmp/kiva-minimap-camera-native-final.log`：11 项相机测试通过，含新增 4 项定位/边界/非法与过期/暂停/保持手势测试。
- `/tmp/kiva-minimap-unit-red.log`：旧模块专项 7 项中 6 项准确失败，覆盖全图轮廓、矩形、点击拖动及生命周期缺口。
- `/tmp/kiva-minimap-web-node-final.log`：最终 9 项小地图专项与扩展后的 pointer/startup 合计 30 项通过，包含上面的偏移边界、UI 按下归属及多键释放回归。
- `/tmp/kiva-minimap-check-final.log`：完整 make check 通过，Rust 各组相加 311 通过、2 项既有长跑默认忽略；native 全 target Clippy、fmt 与 WASM check 均通过。
- `/tmp/kiva-minimap-wasm-clippy-final.log`：标准 WASM Clippy 通过；`/tmp/kiva-minimap-node-final.log`：最终 30 项 Node 通过。
- `/tmp/kiva-minimap-source-before.json` 与 final 清单字节相同，独立核对 90 项运行时源实际 SHA256 全部匹配；清单 SHA256 `5241fcc9f8ae04d7da46425809ea510f1b6c9e32db1cc2d6e15678c0d18ea322`。临时构建成功，`kiva-fd397e469e2c3600_bg.wasm` SHA256 为 `e79bfa591a83f28ba642c72a81d6d120d0c27725ccfde3b20e0562e01c8016d8`。
- 已独立目检 1280×800、800×600、1000×720 DPR2 的 `minimap-*.png`：全图边界、中路、基地及矩形框清晰，HUD 与小地图保持可用布局。
- 第一轮浏览器 1280 场景通过，800 的拖动预期点等待超时；原日志保留。原因是辅助函数仅保证小于 2 CSS px 的合成光标端点误差，而小地图的 CSS→world 换算会放大该误差。最终 fixture 读取真实指针按下和移动端点，使用独立地图比例核对相机，容差反而从 0.9 收紧到 0.04 world；实际主镜头投影收紧至 2 CSS px。未修改产品源或放宽阈值。
- `/tmp/kiva-minimap-browser-800.log`、`/tmp/kiva-minimap-browser-1280-final.log`、`/tmp/kiva-minimap-browser-dpr2.log` 三个场景分别通过；包含真实点击/偏移拖动、UI 隔离、边界、滚轮、逐帧框、同一输入帧 chord 全释放、暂停恢复；1280 还覆盖 resize 与重选。DPR2 最终版逐个采样相机事件，独立核对框的 left/top/width/height 与该事件 footprint。
- `/tmp/kiva-minimap-smoke.log`：真实 renderer、拾取/走位/停止、HUD、Y/Space、边缘平移、缩放、窗口变化、失焦冻结与同实例恢复、DPR2 拾取通过。
- `/tmp/kiva-minimap-pointer-response.log`：DPR1/1.5/2 原生 movement 单位及 DPR1.5 物理单位场景全部通过，实际 gain 分别为 1 和 2/3；同事件光标显示、持有真实游戏 RAF 期间的指针响应、按下位置、暂停恢复均通过。
- `/tmp/kiva-minimap-healthbars.log`：原生时钟/RAF，人物 20/20 帧更新且 20/20 可见；相机 19/20 帧更新且 20/20 可见，最大连续停滞 1 帧，累计真实平移 80.595，整个采样距地图夹紧边界至少 2.452。既有 20 帧/80% 跟随与 HUD 快照节流阈值未放宽。
- `/tmp/kiva-minimap-regressions.log`：1280-final、DPR2、smoke、pointer-response、healthbars 五个串行 stage 全部退出 0，自有 Xvfb 与临时认证已清理；800 的独立最终验收也退出 0。
- 最终再次逐项核对运行时 90 文件仍与已验收构建相同；JS 语法及 `git diff --check` 通过。

## 最终结论

代码审查与验收全部通过，无剩余阻塞项。已发现的偏移夹紧、事件时 UI 归属、多键释放三类问题均已修正并保留回归；最终文档 API/行为与实现一致，质量门和真实 WebGPU 证据对应同一份冻结产品源。

本代理只修改审查记录，未修改产品源码、未提交推送或操作开发服务。HUD 工作区保持未提交，任务勾选与收尾记录由 root 完成。
