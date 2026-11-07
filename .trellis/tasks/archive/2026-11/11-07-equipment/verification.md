# 装备系统验收记录

基线`8f7107c`，用户授权方案完成后直接实施；验收完成后已明确授权提交与推送。
训练场直接装卸为充分等待可选偏好后公开采用的默认范围，不记作用户对该偏好的明确回复。
开发服务由用户管理，浏览器使用自有临时HTTP fixture与隔离显示。

## 代码质量

- `make check`通过：格式、native all-target Clippy（deny warnings）、158项Rust测试、WASM check。
- 158项为45客户端unit、6角色、12装备、2reset、56simulation、37status；与基线逐函数比对，原138项全部保留。
- `cargo clippy --locked --target wasm32-unknown-unknown -- -D warnings`通过。
- `node --check`检查web/equipment.js、web/hud.js、scripts/browser-equipment.cjs、scripts/browser-skills.cjs通过。
- 最终`trunk build --dist /tmp/kiva-equipment-site`通过，产物的equipment.js/hud.js/style.css与源文件一致。
- 独立全范围复核未发现需修正的生产代码问题；未引入运行依赖或第二套施法系统。
- 独立规范准确性复核提出三处措辞：开始失败与提交失败的动作区别、orders归一CastSource、适配层可先行拒绝卸装。
  已全部修正文档，没有修改生产代码或重跑无关门禁。
- Trellis implement/check JSONL各17项路径验证通过。

## 浏览器与实际渲染

Chromium软件WebGPU运行最终临时构建，真实Pointer Lock、键盘/鼠标和封闭DOM意图；
没有写World、HP、属性或运行态的测试后门。
有窗口检查使用测试独占的临时Xvfb，原有生产数值及断言保持。

| 入口 | 模式 | 实际结果 |
|---|---|---|
| browser-equipment | headless | 三尺寸通过；1280×800深度场景，800×600、1000×720 DPR2通用场景 |
| browser-equipment | headed | 同上三尺寸完整通过，额外产出装配/战斗6张实际WebGPU截图并逐张检查 |
| browser-stats | headed | 三尺寸144px/七短标签/无单位、实际属性、卡片/地图/技能无重叠、UI隔离和暂停恢复通过 |
| browser-smoke | headed | 实际渲染像素、移动/普攻后摇/技能/镜头/边缘/缩放、UI隔离、失焦恢复和高DPI通过 |
| browser-reselection | headless | 三尺寸通过：真实受伤/技能后整场重开、NPC与载体清理、旧请求隔离、切换/同角色及普通恢复 |
| browser-characters | headless | 三尺寸/两角色、无玩家冻结、预览/实际值、键盘选择/开始、非法/旧/重复请求、捕获拒绝重试/旧Promise及真实失焦通过 |
| browser-cursor | headless | 有界光标、越界回收、边缘滚屏、Esc恢复、技能按钮、高DPI缩放拾取及捕获拒绝全部通过 |
| browser-skills | headless | 三尺寸通过：真实Q/W/E/R、非法敌方瞄准、控制/减速恢复、活动击飞身体拾取、400ms测试时钟后的新快照冻结、恢复落地 |
| browser-healthbars | headless | 单位移动91/91帧更新、镜头移动49/49帧更新，maxStale均0；完整HUD仍80ms限频 |
| browser-startup | headless | WASM下载失败提示及重新加载重试通过 |

装备专项实际验证：空六格/9件目录；非法批次不执行合法前缀且UI可重试；重复锋刃80攻击、卸一件60；
满格禁用、七属性真实变化；捕获边界旧装备意图丢弃；数字6和真实HUD鼠标点选，非法地点保留瞄准；
法珠伤害90，角色Q冷却不变，暂停冻结装备CD；Q不触发装备被动；
一次普攻60+14.5附伤及Slow，卸护符后下一次只有60；实际装备Dash与自身Channel、卸装停止未来引导；
旧实例卸装拒绝；重新选角新Entity空六格且基础属性恢复。

第一次连续运行在已通过equipment/stats/smoke后、reselection首尺寸通过时以SIGTERM（143）结束；
该次不计为reselection全量通过，剩余入口改为独立命令运行；原1280首尺寸及新完整三尺寸均有输出证据。
进程检查确认隔离显示/Chromium/HTTP fixture无残留，用户Trunk服务保持运行。

截图中目录因真实点击自动滚到后排，内部可滚回六格库存；固定关闭按钮和反馈在三尺寸均可达。
战斗截图装备2×3完整显示，原四技能和144px属性保留，小窗口布局无横向溢出。

## 短击飞自动化时序

本轮原browser-skills严格断言发现既有fixture的长帧竞争：pick-ready时tick382、Airborne余0.25s，
真实pointerdown仍观察该帧，下一标签通知已追赶到tick397，选中目标正确但控制已结束。
没有延长生产0.8s控制或放宽can_cast/活动Airborne/选中/205HP/正高度/4单位落点断言。

先前尝试Playwright完整fake timers与逐步pump，实际选角/进入的locator稳定等待不通过；这些运行不计通过。
最终仅在该脚本导航前安装原生RAF/performance.now包装，R前切换到每显示帧1000/60ms，
定时器和真实输入保持原生，同批RAF不重复计时且该context内不跳回墙钟。
暂停检查等待时钟+400ms后的新kiva-state，避免时钟未推进或读旧快照造成假通过。
最终三尺寸一次完整运行全部通过。独立只读复核确认单调、保留原生事件和严格断言、暂停证据充分。
这是帧归一的交互/姿态验收，墙钟时长和长帧性能不由此测试证明；原生时钟smoke已通过。

## 验收映射

| 验收 | 证据 |
|---|---|
| AC1 | equipment公开注册场景：重复/非有限/被动/未知主动拒绝，目录保持且按ID排序 |
| AC2 | 公开六格原子场景：边界/占用/单位/装备/实例失败保留；管理批次解析与满格E2E |
| AC3 | 独立属性来源、重复/百分比/Slow公开场景；浏览器80→60及七实际属性 |
| AC4 | 公开重复被动、顺序/正损血/目标生命/无递归；浏览器Q无被动、普攻附伤/Slow |
| AC5 | 出手快照/来源移除/卸装公开场景，极值溢出无部分出手；浏览器卸护符后无附伤 |
| AC6 | 同技能角色/多件装备CD和Stun来源分离公开场景；浏览器装备不消耗角色Q/E冷却 |
| AC7 | 同一施法目标/权限/忙碌/原子失败公开场景；真实普通主动/Dash/Channel |
| AC8 | 未提交取消/旧实例、已提交弹丸/区域/Dash存续和未来Channel停止；浏览器卸装Channel及旧实例拒绝 |
| AC9 | 公开死亡保留/死后CD/重生不刷新/暂停/reset；浏览器暂停和重开恢复空六格 |
| AC10 | 客户端运行/捕获/旧玩家/实例/严格批次场景；浏览器非法批次可重试 |
| AC11 | 三尺寸P打开、满格/卸装反馈、1–6与鼠标、普通/Dash/自身引导；原UI隔离回归 |
| AC12 | 客户端无玩家/捕获窗口顺序/旧装备点击排空，浏览器实际恢复不释放旧点击 |
| AC13 | 三尺寸有窗口装备/属性布局断言和6张截图；小窗口目录可滚动、关闭/操作可达 |
| AC14 | 共享AbilityDescription与实际slots投影单测；一次静态目录缓存及JS只读展示复核 |
| AC15 | 158项/原138保留、WASM Clippy、JS/Trunk通过；九个浏览器入口最终均通过，装备额外有窗口三尺寸验收 |
| AC16 | 独立代码/规范/测试时钟复核通过；架构、README、两层契约、TODO及任务验收同步，无新增运行依赖 |

## 收尾

九个浏览器入口全部通过。最终文档本地链接、JSON/JSONL、17项上下文及git diff检查通过。
自有HTTP/Xvfb/Chromium fixture均已退出，用户开发服务未接管。
实现与验收完成后，按用户提交/推送授权保存功能提交，再归档当前任务并记录Trellis日志。
