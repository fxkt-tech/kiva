# 选角、会话和无玩家阶段

## 1. 范围

修改选角、开局/恢复、玩家引用、菜单输入或角色预览时，必须同时检查会话、捕获和固定步门控。
核心角色构造见[characters.md](../simulation/characters.md)，鼠标/显示规范见[browser.md](browser.md)。

## 2. 接口与投影

`ClientSession`只存在Selecting或Active{player,selection_revision}，player()返回Option<Entity>。
它是客户端活动玩家引用唯一所有者；PlayerController仅保留selected/aim。
Web桥接读取并移除`data-session-requests`，JSON数组最多32条，严格type标签：

```json
[
  {"type":"SelectCharacter","character_id":2,"request_id":1},
  {"type":"Launch","character_id":2,"selection_revision":2}
]
```

character_id是u16，request_id是u32关联值，selection_revision是Rust分配的u64修订。
角色请求不能携带属性、技能数组或额外字段。JS回显修订，不计算角色规则。
暂停菜单还可提交`{"type":"ReturnToSelection","player_id":"4294966763"}`。
player_id回显Rust的Entity字符串，避免在JS中将实体身份转成有精度限制的数字。

HUD保持原字段并增加session：

- player_id为null或真实Entity字符串，UnitView.character_id为null或角色ID。
- session.phase使用Selecting/Active；Active的selection=null。
- Selecting的selection包含choices摘要、selected纯预览、selection_revision、selection_ack{request_id,accepted}、can_start、error。
- 实际abilities在无玩家时为空，纯预览另用静态AbilityMetadata；实际AbilityView扁平字段继续兼容。

## 3. 执行契约

Startup准备地图/技能/角色/NPC，Web不创建玩家且显式暂停；默认预选旅者，初始回执request0已接受。
Native通过相同创建入口自动采用默认角色，不等待DOM；SimulationPlugin默认暂停语义保持。
固定步前按`session::process_requests → collect_input → patrol_units`运行，会话处理本身不受暂停阻断。

每次接受Select（含同角色）推进修订并回显request_id。失败选择保留旧预览但撤销开始资格，
直到新的选择被接受。Launch同时核对阶段、最新资格、ID/修订、实际捕获和文档焦点，再唯一创建绑定。
捕获/创建失败保留有效选择以便重试；Active阶段拒绝后续选角/启动而不改变实例。
整个批次先解析结构/长度，结构错误不执行任何前缀；结构合法后语义请求FIFO独立处理，不回滚已提交Active。

### 重新选择与整场重开

暂停菜单“重新选择角色”会重开整场试炼，旁边明确说明重置本局进度；普通“继续试炼”仍保留当前局。
ReturnToSelection只接受当前玩家身份、实际未捕获鼠标且已经暂停/请求暂停/失焦的状态。
成功返回是本批次边界，丢弃剩余旧局意图；Selecting重复返回无副作用，旧玩家返回不能重置新玩家。
SelectCharacter是当前Selecting界面的同步意图，不携带局代次；其旧局隔离保证为返回批次尾部丢弃。
后续若新增异步发送，需要扩展上下文标识；不能假定无上下文的Select可识别其创建于哪一局。
Active保留最后接受的selection_revision，返回时checked_add推进版本，并预选上局实际角色。
不能调用setup把版本重置到1，否则旧Launch会再次合法。选择失败仍保留已有版本，不回收版本空间。

session协调`reset_simulation`与`content::spawn_training_units`，只重建NPC，不重复注册目录、地图或静态场景。
模拟清理契约见[runtime.md](../simulation/runtime.md)。Selecting时controls清目标/瞄准/反馈和输入，
camera复位现有CameraRig，patrol暂停时清Local tick缓存，防止tick归零后的旧缓存影响新局。
模拟tick归零，应用墙钟/显示调度不重建。

DOM等待Rust的Selecting投影后才切换界面，重置本地request_id=0关联、旧捕获attempt和技能意图。
返回进行中禁用继续/重复返回，拒绝的Active快照解除等待并提示重试；不能靠刷新页面实现重开。

开始按钮须等最新已接受回执与预览匹配。它在点击内直接请求捕获，成功pointerlockchange的同一回调
先提交冻结角色/修订的Launch，再更新DOM暂停展示；RustActive投影才表示已开局。
浏览器pending尝试按对象身份隔离，旧Promise回调不能清掉新尝试；恢复只重新捕获，不重新Launch。

实际pointerLockElement可能先更新，pointerlockchange稍后才分发，Bevy可在中间运行一帧。
每次开始/恢复申请前清除data-pointer-locked确认，事件回调再写入实际捕获结果。
适配层区分实际捕获与本次事件已确认的捕获：Selecting不能提前释放等待事件的锁，
事件已确认、处理完请求仍无玩家时才释放无效/单独捕获。Launch仍复核实际捕获和焦点。

controls集中处理运行时暂停与输入：Web须Active+真实聚焦+事件已确认的canvas捕获+无暂停请求才跑固定步。
Selecting排空窗口/技能意图和游戏命令，不向假Entity发Stop；Active暂停保留原清队列/一次Stop规则。
GamePointer仅在已确认且允许运行时视为捕获；暂停清除其捕获边界，恢复重新居中。
just_captured不仅丢旧WindowEvent，也要排空并忽略旧DOM技能请求，防止启动/恢复当帧误瞄准或施法。
无玩家是合法阶段，camera不跟随空玩家，scene不画玩家瞄准，HUD不伪造旅者生命和技能。

## 4. 错误与边界矩阵

| 情形 | 结果 |
|---|---|
| 批次非法JSON/字段/长度>32 | 整批拒绝并消费，已选择状态不部分更改 |
| 非法Select后接旧Launch | 最新资格为拒绝，旧预览保留，不能创建玩家 |
| 同ID重新选择 | 新修订，旧Launch仍失效 |
| 过期Launch/失焦/捕获失败/创建失败 | 不部分绑定；有效选择可重试 |
| 单独捕获但无有效Launch | Selecting继续冻结，释放捕获 |
| 实际捕获成功、通知尚未分发 | 等待事件确认，保持暂停，不抢先释放或消耗just_captured |
| 重复启动/局内Select | 不多创建、不换人、不刷新生命/冷却/NPC |
| Esc/真实失焦/丢捕获 | 暂停并清输入；恢复仍为同Entity/角色/实际装配 |
| 死亡/重生 | 仍为Active，不显示选角，不重套默认技能 |
| 明确点击重新选择 | 整场清理、重建NPC、tick0冻结，预选上局角色；下次开局创建新玩家 |
| 旧玩家返回请求/旧版本Launch | 不影响新玩家/不创建旧角色 |

## 5. 示例

正常：先选择秘术师，等最新回执后进入，实际30攻击/40法强与预览一致；Q方向瞄准、R自身引导。
默认：旅者预选但尚无玩家，tick/NPC保持不动，实际英雄HUD隐藏。
错误：data-paused=false就开始模拟，或失败选角后用旧A/rev1偷偷创建A，或恢复时重建玩家。

## 6. 验证

session/controls/UI测试不创建GPU窗口；用同一协调函数和捕获事实验证资格、版本、批次、重试、重复及纯投影。
原16项客户端行为保留，角色公开测试及旧simulation/status场景继续执行。
`browser-characters.cjs`验证真实按钮/捕获、三分辨率、键盘、过期/非法混合意图、旧Promise、真实失焦和同实例恢复。
其中延迟真实捕获通知跨四个显示帧，验证开始/恢复等待期间不丢锁、不推进tick，通知后不重放旧技能。
浏览器只可通过现有意图通道构造非法请求，不设置World/运行态；core显式技能装配通过公开Rust创建接口测试。
`browser-reselection.cjs`覆盖三尺寸反复重开、同角色/切角色、旧版本与旧玩家请求、NPC生命/位置和普通恢复，
使用实际受损NPC和已提交延迟技能验证新局没有旧伤害；performance.timeOrigin保持证明没有刷新页面。

`#character-select`/`[data-character-id]`/`#character-preview [data-preview-stat]`是稳定选择器，
预览不使用实际HUD的[data-stat]。错误分别在#selection-error和#pointer-error，#enter保持开局/恢复入口。
800×600可滚动选择区且开始可达；实际HUD仍144px七项短标签无单位。
复用browser-support的openSelection/selectCharacter/enterGame/resumeGame，恢复辅助函数强制校验玩家ID不变，
保留合成鼠标校准前的真实居中断言。构建到临时目录、fixture在finally清理，不接管用户开发服务。

## 7. 错误与正确

错误：用占位Entity或隐藏默认玩家满足非空引用，另设JS entered布尔量当开局事实，UI重新算属性和技能规则。

正确：唯一ClientSession保存合法阶段/玩家，预览来自核心纯计算，实际状态来自实例；异步捕获只产生经Rust复核的意图。
