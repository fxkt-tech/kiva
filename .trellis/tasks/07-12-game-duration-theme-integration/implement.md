# 游戏模式时长控制与剧本模式实施计划

## 1. 执行策略

本任务按三个可独立验收的子任务推进，当前目录作为集成父任务：

1. 游戏模式发言预算与上下文压缩；
2. 单局剧本核心、编写与规则模拟；
3. 剧本模式 New Game、审核、Editor 与视频集成。

先完成第 1 项，因为它直接解决样本局 76 分钟问题，也是剧本模式共享的时长合同。第 2 项只通过纯核心接口和 repository/server 测试证明剧本可执行。第 3 项最后接入跨层 UI、动态主理人语音和视频。

父任务最终负责双模式兼容、真实模型回放和 1920×1080 成片验收。Inline 模式不创建 `implement.jsonl/check.jsonl`。

## 2. 子任务一：游戏模式发言预算与上下文压缩

### 2.1 固化基线诊断

- [ ] 将样本局 `ae9b3a8a-fb8b-43cf-901b-252ad49b0341` 的统计过程整理为只读诊断脚本或测试 helper。
- [ ] 输出 public/director 场景数、各事件类型文本字符、真实语音时长、主理人时长和总片长。
- [ ] 固化当前导演版 `76.04` 分钟级基线，允许音频 metadata 的微小容差。
- [ ] 不修改或覆盖样本的 `record.json`、voice job 和 MP3；所有重放写入临时目录。

### 2.2 建立共享 `speech-budget` 模块

- [ ] 定义 `SpeechBudget`、`SpeechEvaluation`、`SpeechBudgetTier` 和单一 `spokenCharacterCount()`。
- [ ] 为五类 speech Draft（含首位/后置白天发言差异）定义目标区间、硬上限和最低有效结构。
- [ ] 集中实现文本预计语音时长、已承诺导演版时长和整局 pacing projection。
- [ ] 用样本真实语音校准字符/秒估算，测试中文、空白、标点、emoji 和换行。

### 2.3 把预算加入任务规格与 Prompt

- [ ] 扩展 `PromptTaskSpec`，由任务规格拥有长度合同 key/最低有效表达，而不是 Prompt builder 写死数字。
- [ ] 发言 Prompt 明确“一个结论 + 一个关键依据 + 一个后续验证点”等任务内压缩结构。
- [ ] 补回“不得复述整条时间线/完整票型/所有前置发言”，禁止 TTS 正文中的括号舞台动作。
- [ ] 人物风格只影响预算内句式和节奏，不能扩大硬上限。

### 2.4 校验与一次压缩 repair

- [ ] `parseAndValidateSpeechText` 在非空校验后执行统一硬上限校验。
- [ ] 超长错误进入现有唯一 repair；repair contract 只要求压缩原文，不重新分析整局。
- [ ] 第二次仍超长时保持 Draft 不变，并在 GenerationRecord attempts 中保留诊断。
- [ ] 明确不使用字符串 `slice()`、TTS 裁切或请求级 `max_tokens`。
- [ ] 人工 Draft text 编辑和确认入口复用同一个硬上限校验。

### 2.5 确定性上下文选择

- [ ] 用集中 selector 替换每区 `40 × 800` 的粗预算。
- [ ] 保留当前日、上一日结构化结算、每名玩家最新立场、仍未解决的公开身份/查验冲突和本人有效私有历史。
- [ ] 票型以结构化摘要提供，不要求每名玩家复述票表。
- [ ] 用 visibility 负向测试证明压缩没有重新引入 host-only、密票或他人私有事件。

### 2.6 游戏模式时长观测

- [ ] Editor/Preview 显示当前确认时长、预计导演版总时长和 `normal/compressed/critical` 档位。
- [ ] 生成前根据已承诺时长选择当前 SpeechBudget；达到 critical 时仍保留任务最低有效内容。
- [ ] 任意理论超长路径只提示风险，不在预算耗尽后静默裁切或改变游戏胜负规则。

### 2.7 回放与门禁 A

- [ ] 使用样本全部 57 个发言局面重放新 Prompt，检查每句硬上限合法率和一次 repair 率。
- [ ] 用重放文本和校准语速重新编排导演版，目标 `28–32` 分钟，超过 `35` 分钟不得进入下一子任务。
- [ ] 人工检查首位发言、真假预言家冲突、遗言和后期票型场景仍保留关键推理。

验证：

```bash
pnpm test src/core/__tests__/llm-task-specs.test.ts \
  src/core/__tests__/prompt-builders.test.ts \
  src/core/__tests__/speech-generation.test.ts \
  src/core/__tests__/player-context.test.ts \
  src/core/__tests__/playback.test.ts
pnpm typecheck
git diff --check
```

## 3. 子任务二：单局剧本核心、编写与规则模拟

### 3.1 领域合同与兼容读取

- [ ] 新增 `GameRunMode = "game" | "scripted"`，New Game 输入和 Game snapshot 显式携带。
- [ ] 新增版本化 `EpisodeScriptState/Snapshot`、`EpisodePlanStep`、`EpisodeSpeechBeat`、`ActorBrief` 和验证报告类型。
- [ ] Repository 从 `unknown` 集中验证；旧 Game 缺失字段时只在内存回退为游戏模式。
- [ ] 增加 cast/rules/theme input hash 和 compiler version，拒绝不匹配的批准/执行。

### 3.2 扩展主题模板叙事语法

- [ ] 为 `GameScriptDefinition/Snapshot` 增加 premise、stakes、role aliases、action aliases、public briefing、tone/forbidden rules 和双结局 frame。
- [ ] 更新《未明档案》seed/runtime 数据与校验，不添加角色专属秘密或改变规则候选。
- [ ] 旧主题快照通过明确 legacy narrative grammar 兼容。

### 3.3 Script Author 生成与记录

- [ ] 为 Script Author 建立独立 Prompt 版本、模型绑定配置和 generation metadata，不伪造 Player Draft ID。
- [ ] 第一阶段生成结构化大纲；第二阶段按夜间、白天、PK/遗言批次生成计划。
- [ ] 每批只暴露当前模拟状态、合法目标、可见事实摘要、未完成大纲目标和剩余时长。
- [ ] 保存 token usage、attempt、错误和 author 模型，聚合 Game 成本时计入。

### 3.4 深模块：dry-run compiler

- [ ] 实现 `authorEpisodeScript()`，内部循环调用现有 `planNextDraft()`。
- [ ] 集中实现 `draftSlotFor()`；计划匹配不依赖随机 Draft/Event ID 或时间戳。
- [ ] 计划 edit 通过 `applyDraftPayloadEdit()` 和共享合法候选应用，再用现有 confirm/event/state/rule 函数模拟。
- [ ] 将模拟路径编译为 steps 和 expected trace，游标使用 expected/active event index。
- [ ] 对最大事件数、最大昼夜、最大 repair 批次和 35 分钟预算设失败保护。
- [ ] 覆盖平安夜、死亡、女巫双药限制、守卫连续守护、预言家历史查验、平票 PK、猎人、双方胜负。

### 3.5 Actor Projection

- [ ] 计划保存稳定 evidence step index，运行时映射为实际已确认 Event ID。
- [ ] 使用 `projectVisibleEvents` 验证 ActorBrief 依据，拒绝未来 step 和不可见事件。
- [ ] 玩家只获得当前 scene、计划立场/披露、可见依据和 SpeechBudget；不获得完整剧本、计划胜方或未来命运。
- [ ] 公开狼人发言继续隐藏队友/刀口；狼队私聊可使用合法阵营信息。

### 3.6 深模块：运行时 Draft 绑定

- [ ] 实现 `planNextEpisodeDraft()`，内部委托现有推进器并对比批准 step。
- [ ] 行动/投票应用只读计划值后重新计算合法性；不得相信旧 dry-run 结果直接确认。
- [ ] 发言返回 ActorBrief 并走共享限长生成；台词不改变计划 edit。
- [ ] slot、合法目标或 input hash 不一致时返回结构化 `script_diverged`，停止推进。
- [ ] Rollback 后按 active event index 从同一计划重放，不维护独立 cursor。

### 3.7 单局剧本状态与批准

- [ ] `generating/review/approved/failed` 状态在 game lock 内原子更新。
- [ ] Approve 同时验证 `events.length === 0`、candidate hash、报告成功和预计时长。
- [ ] 批准后 snapshot 深拷贝锁定；Regenerate 只在未批准/未执行时替换 candidate。
- [ ] 所有非执行就绪状态统一阻止 continue/confirm/voice/export。

### 3.8 门禁 B

- [ ] 确定性 fake Author 生成至少一局好人胜、一局狼人胜、一局 PK、一局猎人链，并全部 dry-run 到终局。
- [ ] 破坏 step 目标、actor、day、round、证据引用和 input hash，验证编译/运行拒绝。
- [ ] 同一批准剧本两次重放产生相同结构 trace；不同台词不改变事件路径。

验证：

```bash
pnpm test src/core/__tests__/game-creation.test.ts \
  src/core/__tests__/advance-planner.test.ts \
  src/core/__tests__/rules.test.ts \
  src/core/__tests__/visibility.test.ts \
  src/core/__tests__/game-script.test.ts \
  src/server/__tests__/game-repository.test.ts \
  src/server/__tests__/game-actions.test.ts
pnpm typecheck
git diff --check
```

## 4. 子任务三：剧本模式产品与视频集成

### 4.1 New Game 双模式

- [ ] 在主题模板和座位选择之前/旁边提供清晰的游戏模式、剧本模式卡片。
- [ ] 两种模式都要求主题模板，且与 Preset/随机座位独立组合。
- [ ] 游戏模式创建后进入 Editor；剧本模式创建空事件记录后进入 Authoring。
- [ ] Home 卡片显示模式和剧本状态，旧局显示游戏模式。

### 4.2 持久化 Authoring Job

- [ ] 复用 voice/video job 的状态、取消、retry、轮询和原子文件模式，避免一个长 Server Action。
- [ ] 增加 queued/outline/planning/validating/completed/failed 进度和安全错误响应。
- [ ] 同一 Game 同时只允许一个当前 authoring job；过期 job 不能覆盖新 candidate。
- [ ] 开发启动命令纳入 worker，并测试重启后 job 状态可恢复或明确失败。

### 4.3 导演 Review

- [ ] 展示 logline、计划胜方、预计天数/时长、acts/scenes、角色命运、行动/票型、验证报告和可折叠 trace。
- [ ] 提供 Regenerate 和 Approve；批准请求携带 candidate/job identity 防止竞态。
- [ ] 失败 candidate 展示可操作诊断，不泄漏文件路径、Prompt 密钥或 stack。

### 4.4 Editor 锁定与台词生成

- [ ] 未批准剧本局不能进入普通 Draft Panel。
- [ ] 批准后结构性字段只读，显示来自计划的 scene objective、stance、披露和预算。
- [ ] Regenerate 只重写当前 text；人工 text 编辑/确认复用硬上限。
- [ ] 自动确认继续携带 expected Draft ID；计划绑定不能削弱现有竞态保护。
- [ ] `script_diverged` 有专用阻断 UI，不自动切回游戏模式。

### 4.5 动态剧情节拍事件与声音

- [ ] 把 `narrative_beat_announced` 加入中央事件 taxonomy、格式化、visibility、Playback 和测试。
- [ ] Episode 模块在计划位置产生该 Draft/Event；它不改变 DerivedGameState 或合法候选。
- [ ] Player Context 将其分类为节目包装而非公开游戏证据。
- [ ] 为动态闻舟旁白增加事件级不可变 voice artifact，统一进入 Preview/Export audio timeline。
- [ ] 旁白 MP3 文件名/路由/Range/导出快照遵守现有 programmatic-video 规范。

### 4.6 Preview 与 Export

- [ ] Preview/Export 使用批准的单局剧本快照和确认事件，不读取当前 candidate 或主题库。
- [ ] 标题、章节、关键冲突和终局回收进入舞台；玩家 speech 仍由玩家拥有 transcript identity。
- [ ] Script Authoring/Review 状态不能导出；缺少动态主理人或玩家声音时按现有 blocker 失败。
- [ ] 预计时长、实际 TTS 时长和最终 MP4 时长在 Review/Preview/Export 可对照。

### 4.7 跨模式与成片门禁 C

- [ ] 浏览器测试覆盖两种 New Game、Review/Approve、结构锁定、台词重生成、偏航阻断和旧局兼容。
- [ ] 游戏模式完整回归证明没有单局剧本依赖和行动行为变化。
- [ ] 剧本模式真实 Author 生成一局《未明档案》，人工审核主题因果链、信息边界和角色辨识度。
- [ ] 两种模式分别生成代表性 1920×1080 MP4；目标 28–32 分钟，均不得超过 35 分钟。
- [ ] 对照音频 cue 总时长和 ffprobe MP4 duration，误差在一帧/编码允许范围内。

验证：

```bash
pnpm test
pnpm typecheck
pnpm build
pnpm video:smoke
git diff --check
```

## 5. Review Gates

1. **Gate A — 时长**：没有真实样本重放与 28–32 分钟投影，不进入剧本模式核心。
2. **Gate B — 可执行性**：没有纯核心 dry-run、Actor Projection 负向测试和同剧本 trace 稳定性，不接 UI。
3. **Gate C — 产品**：没有审核门和结构锁定，不开放剧本模式 New Game 入口。
4. **Gate D — 视频**：没有真实 TTS/MP4 时长与动态旁白资产快照验证，不宣布任务完成。

## 6. 风险文件与回滚点

- `src/core/advance-planner.ts`、`rules.ts`、`state.ts`：不为剧本模式复制或分叉规则；若需要大面积 mode 分支，回退重新设计 seam。
- `src/core/events.ts`、`visibility.ts`：动态剧情事件不得进入身份事实或泄露导演信息。
- `src/core/player-context.ts`、`prompt-builders.ts`：上下文压缩和 ActorBrief 都必须保留现有负向可见性测试。
- `src/core/speech-generation.ts`、`validated-generation.ts`：长度失败复用唯一 repair，不增加隐式多次调用。
- `src/server/game-repository.ts`：所有新字段集中 normalize/validate，旧记录只读兼容。
- `src/server/game-actions.ts`：游戏/剧本模式通过明确 dispatcher 选择 planner，不能在 confirm 末端偷偷覆盖 payload。
- `src/core/playback.ts` 与 video export：动态旁白使用确认事件和不可变音频，不读取未批准 candidate。
- New Game/Editor：保留 feature flag，可在不破坏游戏模式和旧剧本读取的前提下关闭新入口。

## 7. 最终验收材料

- 样本局原始 76.04 分钟构成报告；
- 游戏模式新版全局发言重放、字符/repair/预计时长统计；
- 一份批准的《未明档案》单局剧本 JSON、验证报告和 dry-run trace；
- Actor Projection 信息边界负向测试报告；
- 游戏模式与剧本模式各一份 MP4 及 ffprobe 信息；
- 人工评分：规则正确、事实一致、剧情因果、主题深度、角色自然度、无机械复读、观看节奏。
