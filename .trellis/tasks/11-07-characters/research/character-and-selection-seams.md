# Research: 角色与开局选择的接入位置

- Query：以最小、长期可维护的接口区分角色定义、本局技能装配和运行实例；开局选角如何与现有Pointer Lock、暂停、HUD及重生正确衔接？
- Scope：internal；仅调查当前仓库代码、测试、规范和已提供对话。
- Date：2026-11-07。
- Baseline：7eb69c0。本文是规划证据与建议，不表示已经实现。
- 操作范围：只读源码/规范；仅写本文，未运行测试、服务或Git操作。

## 1. 直接影响方案的事实

| 位置 | 当前事实 | 设计影响 |
|---|---|---|
| `src/simulation/model.rs:49` | UnitSpec同时传name、UnitStats、四槽AbilityId和重生延迟 | 角色系统可以在该入口上方解析定义与装配，NPC仍使用原入口 |
| `src/simulation/model.rs:58` | Unit拥有构造时stats、home和重生策略；没有角色ID | 增加独立只读CharacterIdentity即可，别把模板、运行时和NPC强行改为同一角色类型 |
| `src/simulation/model.rs:174` | AbilitySlots拥有唯一实际技能ID数组及各槽冷却 | 实际装配继续以它为唯一事实，不再给CharacterInstance保存另一份实际ID |
| `src/simulation/mod.rs:75` | spawn_unit完整校验数值、地图、技能引用后才分配私有来源并创建全部运行组件 | spawn_character应复用它，不另建第二套属性/生命/状态初始化 |
| `src/simulation/abilities.rs:115` | AbilityCatalog拒绝空名称、重复ID、非法参数，提供只读get | CharacterCatalog可沿用显式注册/只读查询，不需通用模板注册或trait |
| `src/simulation/combat.rs:157` | 死亡清理动作、指令、运动、状态及OnDeath属性来源 | 新角色身份不能参与另一个死亡清理循环 |
| `src/simulation/combat.rs:190` | 重生取Unit.stats/home，递增生命代次并只清AbilitySlots.cooldowns，保留ids | 角色和本局装配应跨重生保持，不能重生时重抽技能或重读默认装配 |
| `src/client/content.rs:217` | Startup直接创建旅者，使用默认属性和5/6/3/7技能 | 需要把地图/技能/NPC准备与玩家创建拆开 |
| `src/client/content.rs:272` | content::setup创建含确定Entity的PlayerController | “玩家尚不存在”目前没有表示法，不能用Entity::PLACEHOLDER假装有玩家 |
| `src/client/mod.rs:44` | Startup依次content/scene/camera，PostStartup发送ready | ready应继续只表示引擎准备，不代表已经选择/创建角色 |
| `src/client/mod.rs:49` | 输入在固定步前采样，再推进模拟 | 启动协调器应在该阶段、采样游戏输入之前完成校验/创建/绑定与暂停门控 |
| `src/client/controls.rs:325` | 根据焦点、捕获和DOM paused决定模拟是否暂停 | 新方案必须叠加“存在已提交对局”条件，浏览器捕获成功不能单独解暂停 |
| `src/simulation/model.rs:367` | SimulationControl默认paused=false | 仅应用选角阶段显式暂停；不要把模拟插件全局默认改为暂停，避免破坏无窗口调用者 |
| `web/hud.js:35` | enterGame在可信点击中立即请求Pointer Lock | 不能先await一轮Rust验证，再在非原点击上下文申请捕获 |
| `web/hud.js:79` | pointerlockchange把entered设true，并直接把data-paused设为!captured | 必须移除“曾经捕获=已经开局”的推断；选角拒绝时捕获成功也不代表已创建角色 |
| `src/client/ui.rs:100` | HudView.player_id为非空String | 无玩家阶段需null/Option，不能空串或假Entity |
| `src/client/ui.rs:117` | UnitView投影要求真实Unit、运行态、显示根和Pickable等 | 选角预览须独立定义投影，不能伪造UnitView或预先创建战斗实例 |
| `src/client/scene.rs:179` | 现有模型按Team选材质 | 第二角色可以先复用模型；若需要外观区分，应走客户端内容数据而非核心ID条件分支 |

## 2. 推荐的模拟接口与状态归属

建议新增`simulation/characters.rs`，通过`simulation/mod.rs`显式导出以下真实所需接口。
名称为方案协作基线，可在设计收敛时统一微调：

```text
CharacterId
CharacterDefinition { name, stats: UnitStats, default_loadout: AbilityLoadout }
AbilityLoadout = 完整四槽[Option<AbilityId>; 4]的定义值
LoadoutSelection = CharacterDefault | Explicit(AbilityLoadout)
CharacterSpawnSpec { character, team, position, loadout, respawn_delay }
CharacterIdentity { character_id } // 字段受保护，只读getter
CharacterCatalog::{register, get, iter}
spawn_character(&mut World, CharacterSpawnSpec) -> Result<Entity, CharacterSpawnError>
```

接口的价值是把定义查询、默认/显式装配解析、引用校验、角色身份和原子构造集中在一个位置，
不是把spawn_unit改名后留给所有调用方各自拼装UnitSpec。

### 所有者应保持清楚

- CharacterDefinition：不可变内容模板，包含基础战斗定义与默认技能ID；不含冷却、当前血量、Entity或浏览器信息。
- CharacterIdentity：实例的角色出处，只需要ID。实际Unit和已有组件已经是运行实例，不必再套一个包含所有运行态的CharacterInstance。
- Unit：仍持有本次构造的静态stats快照；属性运行态仍由attributes管理，修改模板不回写实例。
- AbilitySlots：唯一实际装配和冷却事实；不把它和角色默认技能双向同步。
- LoadoutSelection::Explicit：一次创建时替换完整四槽，不是和默认技能按None合并；None就是空槽。
- 当前spawn_unit允许重复技能ID，冷却按槽位保存。兼容初版可保留这一规则，不暗中新增“技能必须唯一”限制。
- CharacterCatalog存模板；AbilityCatalog存独立技能定义。角色仅引用技能ID，不能把技能定义复制到角色里，更不能把技能代码写进角色分支。
- 装备的主动能力以后有独立归属，不放入角色四槽默认装配数组；本期不为它增加抽象。

### 校验顺序与失败原子性

1. 注册角色时校验非空名称、重复角色ID、UnitStats以及默认装配的技能引用。
2. 明确注册依赖：先注册技能，再注册角色；CharacterCatalog::register可显式接收只读AbilityCatalog。
3. spawn_character解析角色与CharacterDefault/Explicit，再对最终装配作一次完整校验。
4. 组装UnitSpec并调用原spawn_unit，保留所有地图/数值/私有来源分配校验。
5. 成功后立即在同一独占World操作中附加CharacterIdentity，再向外返回Entity。身份附加不能再有可恢复失败分支。
6. 未知角色/技能、非法出生点/数值或来源耗尽都不创建半个角色，不绑定玩家，不启动对局。

可以用小型类型化错误区分UnknownCharacter、InvalidLoadout、InvalidSpawn；
不要把原spawn_unit已有校验复制一遍，必要时内部复用它的错误映射。
四槽默认数组可以全/部分为空；演示角色可以装满四槽，通用接口不需要为此新增玩法限制。

### 未来随机技能的真实接入位置

未来模式生成`Explicit(AbilityLoadout)`，通过相同spawn_character入口创建角色。
本期用公开接口测试非默认装配能正常施法、独立计冷却、重生保持即可证明接口成立，
不实现随机数、候选池、对局中换技能、装备主动技能或热重载。

## 3. 开局策略比较

| 方案 | 优点 | 代价/风险 | 建议 |
|---|---|---|---|
| 没有玩家直到确认开局 | 预览与实例清楚分离，只有最终选择创建战斗状态，没有隐藏的默认玩家 | 需一次性处理camera/controls/UI无玩家阶段 | 推荐 |
| 先创建旅者，选择后替换 | 少改部分非空Entity调用者 | 旧实体/显示子树/命令/来源/生命引用清理，可能残留已发射载体；把选角变成换角色机制 | 不采用 |
| 预先创建每个角色并只切换控制 | 表面切换简单 | 隐藏角色仍拥有运行状态、会被模拟/拾取/命中；角色预览与对局实例混合 | 不采用 |
| 给UnitSpec加角色ID并重构所有NPC/测试 | 统一字段位置 | 强迫NPC和109项旧场景参与不必要迁移，仍没有解决默认/实际装配和开局提交 | 不作为本期主路径 |

开局前准备地图、技能、角色目录和训练NPC，但客户端显式冻结模拟。
初始预选当前旅者使原默认路径保持兼容；只有点击开始才创建实际玩家。
第二个使用不同基础值/现有技能装配的演示角色即可验证配置模型，不需要新效果或模型素材。

## 4. 客户端会话与捕获的唯一协调者

推荐新增`client/session.rs`拥有开局阶段和活动玩家Entity。PlayerController保留目标选择/瞄准，
不再保留另一份活动玩家身份。使用小型封闭状态和getter即可，不需要上来引入通用事务/请求框架。

```text
Selecting { selected_character, selection_revision, feedback }
Active { player_entity } // 暂停/捕获不改变这个身份
```

如果需要表示一次pending启动，用明确的本次请求数据即可，不为将来退出重开设计多层状态。
浏览器可拥有捕获申请的in-flight状态及对应选角快照；它不是模拟的活动玩家事实。

### 必须满足的门控公式

```text
Web允许固定步 = 已提交Active玩家 && 文档聚焦 && canvas捕获成功 && 无暂停请求
```

Selecting下即使data-paused误变false或有人单独捕获canvas，也不能运行模拟。
无玩家时，暂停只清输入/瞄准/队列，不向假Entity提交Stop。
死亡时仍有活动玩家Entity；重生走原流程，不能把“角色死亡”误判为回到选角。

### 推荐开局流程

1. `ready`只表示引擎资源和角色目录可用，显示选角；无玩家，模拟暂停。
2. 角色卡片发送选择意图给Rust；Rust验证ID并发布预览与版本。JS只渲染目录与确认结果。
3. 点击“开始”时锁住本次角色/版本并禁用选择控件，立即在可信点击内申请Pointer Lock。
4. 捕获成功时，从同一个pointerlockchange处理器发出带本次选角快照的Launch意图。
   再更新浏览器捕获/paused展示信息，避免先暴露unpaused、后补启动意图。
5. Rust在固定步前消费Launch，再核对阶段、ID/版本、实际捕获和焦点；成功后唯一创建/绑定玩家。
6. 绑定完成后才允许模拟固定步。当前GamePointer::sync的just_captured事件清空逻辑继续丢弃进入按钮的旧移动/点击。
7. 捕获失败不创建玩家、不丢失选角，保留重试；非法/过期启动请求保持Selecting并反馈、释放本次捕获。
8. Active时恢复按钮只申请捕获，不发送重新创建角色的请求。重复Launch/重复捕获事件不创建第二个玩家。
9. Esc/失焦/捕获丢失只暂停Active；恢复保持同Entity、角色、装配、冷却、生命和场内状态。

**关键异步陷阱**：若Launch只在requestPointerLock Promise的后续continuation发出，
而pointerlockchange先把DOM设为unpaused，则Rust可能看到“已捕获但没有启动意图”。
应把待启动快照放在可信点击时的单一pending数据中，由成功捕获事件在同一个回调内消费并发出；
失败分支只清本次pending，不清除较新的请求。禁用重复启动可减少并发分支，但Rust阶段校验仍不可省。

`web/hud.js:21`的entered当前表示“曾成功捕获”，不能继续用来判断显示选角还是继续试炼。
UI阶段应由Rust会话投影决定，浏览器继续拥有真实捕获/焦点事实。

## 5. 无玩家阶段必须检查的调用者

| 位置 | 必须改变的读取 |
|---|---|
| `controls.rs:51` | 暂停时直接取player并入队Stop；无玩家需清输入而不提交Stop |
| `controls.rs:134`、`:173`、`:209`、`:244` | 选技能/刷新瞄准/键盘/鼠标直接取活动Entity；Selecting仅消费会话/UI意图 |
| `controls.rs:301` | 先处理会话、再决定模拟门控；不能早return而遗留后续要重放的WindowEvent/技能请求 |
| `controls.rs:407` | 指令反馈只归因当前Active实体；Selecting不依赖PlayerController中的假ID |
| `camera.rs:47`、`:63` | 不存在玩家时仍允许背景相机成立，不跟随/边缘滚屏；Active再用真实实体 |
| `scene.rs:330`、`:403`、`:441` | 选中/hover与施法范围只在活动输入存在时显示；地图/NPC显示不要求玩家存在 |
| `ui.rs:191`、`:253`、`:266` | player_id为空、abilities为空；角色目录和预览必须仍然发布 |
| `ui.rs:319` | native HUD需明确无玩家文字或默认角色启动策略，不能因缺少Resource失败 |
| `web/hud.js:310` | 选角与Active HUD按phase投影；无玩家不要渲染写死的旅者血量/法力 |
| `scripts/browser-support.cjs:41`、`:48` | enter和resume区分；默认角色预选保留原测试入口，恢复不重新选角 |
| `controls/tests.rs:11` | fixture现在调用content::setup立即得到玩家；改成显式用会话启动入口创建，保留既有输入断言，避免替换为新的默认假设 |

无玩家不是异常，也不是等待生成的fake Unit。HUD新增独立SelectionView/CharacterPreview，
保持UnitView只表示真实战斗实体；`player_id:null`、`abilities:[]`是明确可测契约。
会话/选择修订发生时建议立即发布一次完整投影，不必等80ms限频；正常对局仍保持原频率，
逐帧kiva-labels不承担角色目录或状态机逻辑。

## 6. 预览属性与表现的边界

`attributes.rs:49`允许有限合法基础值，而`:233`在生成最终属性时对正攻速做技术范围归一化。
因此预览不要在JS复制最终属性算法，也不能为了预览spawn一个隐藏玩家。

两种可明确选择的实现：

- 显示“角色基础属性”：直接投影已验证的定义值，明确不把它叫“当前最终属性”。
- 若要求卡片与开局HUD七项数值精确一致：增加一个真正复用attributes::compute的纯只读初始属性投影，
  让正常构造和预览共用计算。不要在client重写攻速归一化/来源运算。

角色显示文案/色板/头像或模型引用属于客户端内容；CharacterIdentity用于连接显示数据，
不要把Bevy材质、DOM或按角色ID决定的战斗逻辑放进CharacterDefinition。
两个演示角色可以复用现有模型；若加入颜色区分，使用客户端映射数据即可。

## 7. 验证矩阵

| 场景 | 主要断言 |
|---|---|
| 目录合法性 | 重复ID/空名/非法数值/未知默认技能拒绝，原目录保持 |
| 创建原子性 | 未知角色、Explicit未知技能、非法出生点不增加Unit/CharacterIdentity或绑定玩家 |
| 同角色多实例 | 基础值来源相同，但生命/冷却/状态/属性来源彼此独立 |
| Explicit装配 | 完整替换；None为空；默认目录不被修改；实际技能只从AbilitySlots读取 |
| 技能调用 | 同一角色使用非默认装配可通过原CastAbility按槽位施法；无角色ID规则分支 |
| 死亡/重生 | CharacterIdentity与实际ids保留，epoch递增、状态/运动清理、既有冷却重置行为保持 |
| 初始Selecting | tick不动，无玩家，NPC不巡游，原始游戏输入不积压 |
| 开始/失败 | 成功后唯一实例；拒绝捕获、失焦、非法或过期选角均保持未开局，不出现部分绑定 |
| 重复事件 | 连点、重复Launch、重复capture通知不额外创建；多固定步不重复消费 |
| 输入边界 | 选角操作不穿透；捕获后光标居中，不重放之前鼠标/键盘输入 |
| 恢复 | Esc/失焦后重新捕获，保留同实体和实际装配，不回到角色默认状态 |
| UI正确性 | 选择卡片/开局HUD匹配，第二角色的方向Q和自身R按定义运行；空槽元数据正确 |
| 显示回归 | 144px无单位七属性，状态/击飞/血条/后摇/缩放和DPR2不回退 |

保留109项基线测试；新增角色核心行为通过公开目录/创建/命令接口，不借浏览器改World。
客户端会话测试可把平台捕获/焦点/请求事实作为小型输入值来驱动私有协调函数，
不为唯一Web实现新建trait/mock框架。浏览器使用真实选角按钮、Pointer Lock和既有临时fixture。

## 8. 外部资料与相关规范

本调查不依赖外部库新功能，只使用当前代码中既有的捕获、ECS调度和目录模式，因此未查询外部资料。

相关规范：

- `.trellis/spec/simulation/runtime.md`
- `.trellis/spec/simulation/attributes-and-damage.md`
- `.trellis/spec/simulation/abilities-and-statuses.md`
- `.trellis/spec/client/browser.md`
- `.trellis/spec/client/attributes-hud.md`
- `.trellis/spec/guides/cross-layer-thinking-guide.md`
- `docs/architecture.md`

## Caveats / Not Found

- 未发现现成角色目录、角色身份、选角状态机或本局技能换装入口；这是新增接口，不是已有功能命名整理。
- 本报告未运行编译/测试，也没有复现运行故障；上述时序风险来自当前路径和拟议改动的交互。
- 开局选角、默认/实际装配分离、对局中不切角色/不做随机生成已由前序推荐及本轮规划范围确定，
  无需重新询问用户这些问题。
- 演示角色数量/名称/数值、卡片精确视觉样式是最终方案的内容建议，不声称用户逐项指定。
- Native需要保留编译和测试；若不扩展native选角UI，可通过相同启动接口自动采用默认角色，
  应在方案中明确，避免native等待永远不会到达的DOM请求。
- 装备、局内换角、重开/重置地图、随机算法、角色独有被动、继承/脚本/通用插件体系均不在本期调查建议范围。

## 9. 最终规划复核

已阅读本任务完整prd/design/implement，未加载JSONL，未执行实现或测试。
父代理选择的两个演示角色、默认/显式创建、共享初始属性计算、Native默认路径以及单一会话所有者与调查相符。

### 9.1 已收敛的接口与职责

- CharacterDefinition.initial_attributes复用属性模块的初始计算，解决基础攻速归一化预览差异；
  无World入参本身也限制它不能创建单位/分配World中的修正来源，无需为测试增加来源计数写口。
- AbilityLoadout使用同形四槽别名、Explicit完整替换，保留原spawn_unit和每槽冷却，未增加不必要的容器抽象。
- CharacterIdentity仅存角色出处，实际ID仍只在AbilitySlots，重生无须重写原combat策略。
- ClientSession唯一持有阶段和活动Entity；PlayerController仅交互状态；session不反向依赖controls，能避免新的模块依赖环。
- Native经相同创建路径自动进入默认角色，未引入无实际需求的第二套选择界面。
- 两类32条有界意图只解决选择/开局，当前方案没有扩成通用请求框架、局内换角或地图重置。

### 9.2 一个必须补严的启动资格条件

发现的反例：当前已接受A/rev1后，语义失败的Select(未知B,request2)保留A预览/rev1。
若接着收到旧Launch(A,rev1)，只核对阶段/ID/revision/捕获/焦点仍然会通过，
与“失败选择不能隐式启动旧角色”的需求不一致。UI禁用按钮不能替代Rust校验。

父代理已接受以下设计修订，实施前须在正式设计与测试矩阵中保留：

- Launch还必须核对最新选择回执的accepted资格（session.can_start），不能只检查旧预览版本。
- 被拒绝的Select保留原预览，但撤销启动资格，直到下一条合法Select被接受。
- 每次合法Select，包括重新选择同一ID，都推进Rust revision。
- 捕获/创建/Launch失败保留已经合法接受的选择资格，允许正常重试；不能把它与Select失败混为一类。
- 测试精确覆盖`[非法SelectB,旧LaunchA]`不创建，以及再次合法SelectA获得新revision后可以重试。

这是协议的启动资格补充，不要求新产品决策或新增通用事务模型。

### 9.3 批次失败语义应继续保持明确

完整批次先做结构校验/条数限制；结构失败时不执行任何条目。
结构合法后按FIFO逐条执行语义请求，后续语义失败不回滚前一成功请求。
因此`[SelectB成功,旧LaunchA失败]`应保留B；
`[合法LaunchA成功,局内SelectB失败]`应保持已创建的A，不回滚实例或重置场景。
这和“开始后不换角色”的阶段约束一致，不能把批次原子性误写成整个语义列表回滚。

### 9.4 复核结论

除上述已被父代理接受的启动资格补充外，没有发现新的阻塞产品问题或需要扩大范围的架构问题。
开局选择、暂停恢复、同Entity重生、实际装配与预览计算在现有方案中已经分工明确。
本结论是规划复核，不代表角色代码或验收已运行通过；后续仍需用户审阅后才开始实施。
