# 指针速度与响应验收

基线18e3589，用户实际环境Windows11/Edge/系统150%/网页100%。开发服务保持由用户管理。
本轮修复已验收，用户现已明确授权提交/推送；只记录已经执行的结果。

## 复现与修复

- 旧已验收构建/tmp/kiva-equipment-site：DPR1.5的movementX12仅移动8CSSpx；DPR2仅6。
  这是Linux Chromium DPI模拟，并非Windows实体硬件速度测量。
- /tmp/kiva-pointer-probe.cjs的可信移动后光标在事件回调中不变，下一引擎帧5–10ms后变化；严格即时响应断言失败。
- 正式browser-pointer-response专项修正为document capture的before与window bubble的after，用WeakMap绑定同一event；
  旧构建仍因movement24而before/after同为500严格失败，未以最终到达目标代替灵敏度检验。
- Web唯一指针所有者校准正常CSS位移，默认保留系统加速；DOM事件即时绘制。Rust只消费按下快照与帧末镜像，
  不重复除DPR/积分，不逐raw MouseMotion做射线查询。Native保留原绝对输入。
- 独立复核协同补齐coalesced逐样本夹紧、按下错配清队列、部分/内部裁剪样本跳过、同事件和真实停帧证明。

## 质量门禁

- make check通过：格式、native all-target Clippy、原158项Rust测试全部通过、WASM check。
- WASM Clippy deny warnings通过；Trunk临时构建成功，pointer/hud/equipment/CSS产物与源码一致。
- node --test scripts/pointer-unit.test.cjs：12项全部通过，加载实际pointer.js，覆盖CSS/physical单位、父子样本、边界反向、
  裁剪/重置、按下顺序/chord、错配不重放和捕获/暂停/焦点门控。
- JS语法、git diffcheck及任务JSONL路径检查通过；独立完整代码/规范复核无未处理问题。

## 浏览器

| 入口 | 模式 | 实际结果 |
|---|---|---|
| pointer-response初版 | headless | native DPR1/1.5/2和physical单位DPR1.5四case通过；即时响应、正常gain、持有RAF、早先点击坐标、恢复居中 |
| smoke | headed/独占Xvfb | 实际WebGPU像素、指针、移动/攻击/技能、UI隔离、镜头/缩放/高DPI、失焦/暂停恢复通过 |
| pointer-response最终 | headed/独占Xvfb | 四case全部通过：native DPR1/1.5/2、physical DPR1.5 gain2/3；WeakMap同事件响应、labels证明WASM停帧、按下位置、恢复居中 |
| cursor | headless | 有界/越界回收/边缘镜头、Esc恢复、世界/技能按钮、高DPI resize拾取、捕获拒绝全部通过 |
| skills | headless | 三尺寸QWER、目标/控制/减速、击飞可见身体拾取、暂停冻结全部通过 |
| equipment | headless | 三尺寸装卸/满格/鼠标主动/被动/独立CD/Dash与Channel/旧实例/重开全部通过 |
| reselection | headless | 三尺寸整场重开、旧请求隔离、同角色/切角色和普通恢复全部通过 |
| characters | headless | 三尺寸及键盘、预览/实际值、无玩家冻结、捕获拒绝/延迟回调、真实失焦全部通过 |
| stats | headless | 三尺寸144px/七短文案无单位布局、真实值、UI隔离、暂停/移动全部通过 |
| healthbars | headless | 人物91/91帧、镜头49/49帧更新，maxStale均0，完整HUD限频保持 |
| startup | headless | WASM失败提示与重试通过 |
| smoke最终构建 | headed/独占Xvfb | 全部通过：真实像素/指针、移动/攻击/技能、UI/镜头/边缘/缩放/resize、失焦恢复和高DPI；截图目视正常 |

headless初版后补充了内部裁剪轴、HUD显隐唯一所有者及专项labels停帧计数/WeakMap证明，未改Rust。
最终构建/tmp/kiva-pointer-site。最终10个浏览器入口全部exit0（2 headed + 8 headless），所有原有断言保留。

有窗口专项第一次在DPR1通过后发生net::ERR_NETWORK_CHANGED，第二case未获得__kivaState、开局加载超时，
没有记为通过。增加请求URL失败日志后再次完整执行四case全通过，没有requestfailed；没有修改生产/断言或加入自动重试。
独占Xvfb及HTTP fixture在finally退出。原初版microtask观测不足的问题也未当作生产失败，正式红/绿证据均来自同事件capture/bubble。

## 最终状态

- 158项Rust、12项真实指针模块JS、最终10个浏览器suite均通过，WASM Clippy/Trunk/JS语法/JSON/本地链接/diff检查通过。
- 规范和架构明确Web唯一坐标所有者、Rust按下快照、未知单位默认CSS及正常位移校准、coalesced顺序和捕获排空。
- 独立复核/验收完成，无未处理发现。所有自建HTTP、Chromium、Xvfb和临时认证已退出。
- 验收环境Linux Chromium，包含DPR1/1.5/2和物理位移fixture；不宣称Windows实际硬件主观延迟已测量。
- 开发服务未接管；验收完成后，按用户提交/推送授权保存修复，再归档当前任务并记录会话。
