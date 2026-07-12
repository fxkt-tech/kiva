# 游戏模式时长控制与剧本模式技术设计

## 1. 设计结论

本任务引入两种明确不同的运行模式，但不复制狼人杀规则引擎：

- `game`：现有事件推进器和玩家自主决策保持权威，只在发言生成入口加入统一的长度合同、上下文压缩和时长观测。
- `scripted`：分配角色与隐藏身份后，先生成一份结构化单局剧本；同一个现有推进器在内存中完整模拟并编译为可执行步骤。导演批准后，运行时逐步把已批准计划绑定到当前 Draft，行动和票型不再交给玩家模型自行选择，最终台词仍逐场生成。

核心 seam 放在新的 `episode-script` 深模块。New Game、Editor、Prompt、规则模拟和 Preview 不得各自解析原始单局剧本；它们只消费该模块返回的已验证快照、Actor Projection、当前执行步骤和验证报告。

## 2. 领域词汇

| 名称 | 含义 | 当前/新增所有者 |
|---|---|---|
| 主题模板 | 可跨局复用的世界观、叙事语法和视觉包，例如《未明档案》 | 扩展现有 `GameScriptDefinition/Snapshot` |
| 游戏模式 | 玩家自主生成发言、行动和投票的现有玩法 | `Game.runMode = "game"` |
| 剧本模式 | 先编写并批准单局剧本，再按计划演出的玩法 | `Game.runMode = "scripted"` |
| 单局剧本 | 绑定一局角色、隐藏身份、规则、主题、剧情与时长的不可变执行合同 | 新 `EpisodeScriptSnapshot` |
| 分场 | 一个或一组连续 Draft 的剧情目的、行动计划和表达预算 | `EpisodeScene` |
| 计划步骤 | 与推进器将产生的某一个 Draft 一一对应的编译结果 | `EpisodePlanStep` |
| Actor Projection | 从完整单局剧本投影给当前玩家的最小安全指令 | `ActorBrief` |
| 发言预算 | 某个发言 Draft 的目标区间、硬上限和预计语音时长 | `SpeechBudget` |
| 偏航 | 当前 Draft 的结构签名与批准的下一计划步骤不一致 | `script_diverged` |

现有代码中的 `GameScript` 类型暂不做破坏性重命名；UI 和新文档使用“主题模板”，避免与 `EpisodeScript` 混淆。

## 3. 不变量

1. `advance-planner.ts`、`rules.ts` 和 `state.ts` 继续是行动合法性、结算与胜负的唯一权威。
2. 单局剧本不能直接写入结算结果并绕过规则；它只能给可编辑 Draft 提供计划值，再让现有推进器推导结算。
3. 实际 `GameEvent[]` 继续是已发生事实的唯一来源。计划步骤不是事实，只有确认后的事件才进入状态、Prompt 和回放。
4. 运行时计划游标从 active event 数量/事件索引派生，不另存一个可能漂移的 mutable cursor。
5. 完整单局剧本是导演信息。玩家只获取从其可见事件前缀和当前计划步骤构造的 Actor Projection。
6. 时长限制发生在文本生成与校验阶段；视频层不截断台词、不倍速、不隐藏已经选择收入导演版的事件。
7. 历史游戏缺少新字段时规范化为 `runMode: "game"`，不回写磁盘，不要求迁移。

## 4. 总体数据流

### 4.1 游戏模式

```text
Game + active events + Draft
        │
        ├─ projectVisibleEvents()       知识边界
        ├─ selectPromptKnowledge()      相关上下文预算
        └─ speechBudgetForDraft()       节目长度合同
                    │
                    ▼
              buildSpeechPrompt()
                    │
                    ▼
           LLM JSON → 字数校验
                    │
        超长 ───────┴──── 一次语义压缩修复
                    │
                    ▼
          完整限长 text → Draft → Event
```

行动类 Draft 仍使用现有 LLM 合法候选流程；投票和技能选择仍由玩家自主生成。

### 4.2 剧本模式：编写

```text
角色/身份/规则快照 + 主题模板
                │
                ▼
       LLM 生成结构化故事大纲
                │
                ▼
     按昼夜分批生成分场与计划选择
                │
                ▼
 episode-script 模块驱动现有 planNextDraft()
        ├─ 应用计划 edit
        ├─ confirmDraftEvent()
        ├─ appendEvent()
        └─ deriveGameState()/rules
                │
                ▼
      完整 dry-run 到 game_ended
                │
        ├─ 规则/知识/剧情校验
        ├─ 计划步骤编译
        └─ 时长预算报告
                │
                ▼
        导演 Review → Approve/Regenerate
```

### 4.3 剧本模式：执行

```text
episode.planNextDraft(real events, eventCount)
        ├─ 需要时先发出计划内剧情旁白 Draft
        ├─ 否则调用现有 planNextDraft()
        ├─ 对比结构签名
        ├─ 应用锁定行动/投票值
        ├─ 或返回安全 ActorBrief + SpeechBudget
        └─ 重新校验当前合法候选
                │
                ▼
行动直接进入只读 Draft / 发言逐场请求 LLM
                │
                ▼
confirmDraftEvent() → 下一计划步骤
```

## 5. 持久化合同

### 5.1 Game

```ts
type GameRunMode = "game" | "scripted";

type Game = {
  // existing fields
  readonly runMode: GameRunMode;
};
```

`runMode` 在 New Game 创建时确定并随 Game 快照保存。旧记录回退为 `game`。

### 5.2 GameRecord 中的剧本状态

单局剧本在批准前可替换、批准后不可变，因此放在 `GameRecord` 而不是可复用主题库：

```ts
type EpisodeScriptState =
  | { readonly status: "idle" }
  | { readonly status: "generating"; readonly jobId: string }
  | {
      readonly status: "review";
      readonly candidate: EpisodeScriptSnapshot;
      readonly report: EpisodeScriptValidationReport;
    }
  | {
      readonly status: "approved";
      readonly script: EpisodeScriptSnapshot;
      readonly report: EpisodeScriptValidationReport;
      readonly approvedAt: string;
    }
  | { readonly status: "failed"; readonly error: ScriptAuthoringErrorSnapshot };
```

游戏模式规范化为 `episodeScript: null`。剧本模式在 `approved` 前必须保持 `events: []`，`continueGame`、配音和导出统一拒绝。

### 5.3 主题模板扩展

`GameScriptDefinition/Snapshot` 增加主题级而非角色级的叙事语法：

```ts
type ScriptNarrativeGrammar = {
  readonly premise: string;
  readonly stakes: string;
  readonly roleAliases: Readonly<Record<GameRole, string>>;
  readonly actionAliases: Readonly<Record<NarrativeActionKey, string>>;
  readonly publicBriefing: string;
  readonly toneRules: readonly string[];
  readonly forbiddenShortcuts: readonly string[];
  readonly openingFrame: string;
  readonly goodEndingFrame: string;
  readonly wolvesEndingFrame: string;
};
```

它只定义《未明档案》的世界语法，例如“狼人｜篡改者”“预言家查验｜鉴真”；具体玩家命运、身份声明和事件顺序只存在于单局剧本。

### 5.4 单局剧本

```ts
type EpisodeScriptSnapshot = {
  readonly schemaVersion: 1;
  readonly id: string;
  readonly gameId: GameId;
  readonly compilerVersion: string;
  readonly inputHash: string; // cast + roles + rules + theme snapshot
  readonly title: string;
  readonly logline: string;
  readonly plannedWinner: Faction;
  readonly plannedDayCount: number;
  readonly targetDurationMs: number;
  readonly outline: readonly EpisodeAct[];
  readonly scenes: readonly EpisodeScene[];
  readonly steps: readonly EpisodePlanStep[];
  readonly expectedTrace: readonly ExpectedEventSummary[];
  readonly authoring: EpisodeAuthoringMetadata;
};
```

`steps` 是模拟编译结果，运行时按 active event index 匹配；`expectedTrace` 供 Review、测试和偏航诊断，不作为实际状态输入。

### 5.5 计划步骤与结构签名

Draft ID 和时间戳在真实执行时不同，计划不能用它们匹配。核心模块集中生成结构签名：

```ts
type DraftSlot = {
  readonly type: DraftEvent["type"];
  readonly phase: Phase;
  readonly actorPlayerId: PlayerId | null;
  readonly dayNumber: number | null;
  readonly round: number | null;
  readonly voteType: VoteType | null;
};

type EpisodePlanStep = {
  readonly index: number; // expected confirmed event index
  readonly slot: DraftSlot;
  readonly plannedEdit: DraftPayloadEdit | null;
  readonly speechBeat: EpisodeSpeechBeat | null;
  readonly expectedEvent: ExpectedEventSummary;
};
```

一个共享的 `draftSlotFor(draft)` 是所有编译、运行和 UI 展示的唯一结构投影；不得在多个调用方拼接字符串 key。

同理，预生成 Speech Beat 不能保存模拟时产生的随机 Event ID。它保存稳定的 `evidenceStepIndexes`；运行时 Actor Projection 将已经确认的对应索引映射为真实 Event ID，再执行可见性校验。尚未确认的 step 永远不能成为当前依据。

## 6. 深模块与接口

### 6.1 `speech-budget` 模块

该模块隐藏字符计数、模式/阶段策略、语音估算与总时长计算。调用方只学习两个主要接口：

```ts
speechBudgetForDraft(input: SpeechBudgetInput): SpeechBudget
evaluateSpeech(text: string, budget: SpeechBudget): SpeechEvaluation
```

Preview、Prompt、解析器和 Review 都消费返回值，不各自维护字数常量。

### 6.2 `episode-script` 模块

外部 seam 保持小：

```ts
authorEpisodeScript(input: AuthorEpisodeScriptInput): Promise<ReviewableEpisodeScript>

planNextEpisodeDraft(input: {
  game: Game;
  events: readonly GameEvent[];
  script: EpisodeScriptSnapshot;
  draftId: DraftId;
  createdAt: string;
}): EpisodeDraftResolution
```

`authorEpisodeScript` 内部拥有大纲生成、逐阶段编写、共享合法候选、模拟、编译、时长估算、信息边界检查和报告。`planNextEpisodeDraft` 内部调用现有 `planNextDraft`，并隐藏计划内旁白插入、结构匹配、Actor Projection 和计划 edit。调用方不得读取原始 step 后自行套字段。

纯函数 `validateEpisodeScriptSnapshot(unknown)` 仍作为持久化入口；repository 只调用它，不复制内部校验。

## 7. 单局剧本编写算法

不要求模型一次输出约百个 Draft 的巨大 JSON。采用分层、逐阶段的离线编写，但全部在开始对局前完成。

### 7.1 大纲

专用 Script Author 模型读取导演视角的角色、真实身份、规则、人物稳定核心、主题叙事语法和 30 分钟预算，输出：

- 标题、logline、计划胜方与预计天数；
- 2–4 条主要冲突线程；
- 关键身份声明、误导、反转与回收；
- 主要角色的戏剧功能；
- 每幕的目标而非最终对白。

首版由 Author 自动选择胜方；导演在 Review 不满意时整份 Regenerate。暂不增加“指定胜方”配置，避免 New Game 选项膨胀。

### 7.2 分阶段计划

编写器在内存事件前缀上按阶段请求结构化计划：

- 夜间批次：狼队方向/刀票、守卫、预言家、女巫及可能的猎人计划；
- 白天批次：存活玩家的 Speech Beat、披露计划和放逐票；
- PK/遗言批次：只在模拟实际产生时编写。

每次请求只提供当前合法目标、玩家当时可见的结构化事实和大纲中仍未完成的剧情目标。返回计划立即通过共享规则应用并模拟；错误修复只携带失败字段和合法选项。

### 7.3 终止条件

编写必须在现有 `checkWinCondition` 产生 `game_ended` 时结束，同时设置保护阈值：

- 最大计划事件数；
- 最大昼夜数；
- 最大生成/修复批次；
- 目标时长超过 35 分钟立即拒绝该 candidate。

阈值是作者任务的失败保护，不改变游戏模式规则。

### 7.4 模拟与编译

模拟循环只复用现有函数：

1. `planNextDraft()` 生成下一 Draft；
2. 当前分阶段计划通过 `applyDraftPayloadEdit()` 写入可编辑字段；
3. 发言使用不进入最终成片的短占位文本，状态机只消费其事件存在性；
4. 重新运行 `legalActionOptions` / 规则级校验；
5. `confirmDraftEvent()` + `appendEvent()`；
6. 将 Draft slot、planned edit 和 expected event 编译为 step；
7. 直到 `game_ended`。

模拟器不复制 `advance-planner` 的阶段机，也不直接手造夜间/投票结算。

## 8. Actor Projection 与信息边界

Script Author 知道全局真相，但玩家 Prompt 不能读取完整剧本。`planNextEpisodeDraft` 为发言返回：

```ts
type ActorBrief = {
  readonly sceneTitle: string;
  readonly publicStoryContext: string;
  readonly objective: string;
  readonly stance: string;
  readonly mustCover: readonly string[];
  readonly mustNot: readonly string[];
  readonly evidenceEventIds: readonly EventId[];
  readonly disclosurePlan: "conceal" | "claim" | null;
  readonly targetPlayerIds: readonly PlayerId[];
  readonly budget: SpeechBudget;
};
```

生成 ActorBrief 时必须验证：

- `evidenceEventIds` 全部存在于该玩家此刻的 `projectVisibleEvents()`；
- 计划引用的私有事实属于本人或合法阵营频道；
- 公开发言中的狼人同伴/夜间行动不作为可披露证据；
- 未来步骤、计划死亡、计划胜方和其他玩家真实身份不进入 Projection；
- 剧本可以要求玩家撒谎或提出指控，但不能把导演真相伪装成该玩家已经知道的事实依据。

最终 Prompt 的优先级调整为：规则/事实与信息边界 > 已批准分场目标 > 阵营目标 > 人物表达。剧本模式允许分场决定投票与行动，但不允许改写事实或合法候选。

## 9. 发言时长合同

### 9.1 样本校准

样本中文语音约为每秒 `6.1` 个非空白字符。当前 57 段 LLM 发言的语音约 69.9 分钟；导演版其他场景和交接约 6.1 分钟。把玩家语音降到约 22–24 分钟，可得到 28–32 分钟成片。

首轮建议值如下，最终以真实模型回放校准：

| Draft | 目标非空白字符 | 硬上限 |
|---|---:|---:|
| 首夜狼队战术 | 110–150 | 180 |
| 狼队意见 | 70–100 | 120 |
| 当日首位公开发言 | 100–140 | 170 |
| 普通公开发言 | 140–180 | 220 |
| 遗言 | 160–200 | 240 |
| PK 发言 | 140–180 | 220 |

人物可以在目标区间内形成快慢、句式和重点差异，不能拥有独立的无限字数配置。括号舞台动作、重复报座位和完整前情复述不计作有效贡献，Prompt 明确禁止把它们塞入 TTS 文本。

### 9.2 游戏模式预算

`speechBudgetForDraft` 综合 Draft 类型、是否本日首位、当前天数、已承诺播放时长和 30 分钟 pacing envelope，返回 `normal | compressed | critical` 档位。档位只能在任务最低有效内容允许范围内收紧。

游戏模式无法预知任意长对局的终局，因此 35 分钟是标准局与验收长局的节目上限，不是对理论上无限平票/平安夜路径的数学承诺。运行时必须展示当前已承诺时长和投影警告，不能在预算耗尽后静默裁切。

### 9.3 剧本模式预算

剧本模式已知完整 step 数，可以先预留：

- 主理人固定流程与动态章节旁白；
- 每段 Speech Beat 的目标语音；
- 交接、结算和终局镜头。

Author 分配的全部 budget 经过总和校验，candidate 目标为 28–32 分钟、绝不批准预计超过 35 分钟的剧本。

### 9.4 生成与失败恢复

- Prompt 同时给目标区间、硬上限、允许保留的最小信息结构。
- 本地按 Unicode code point 的统一 `spokenCharacterCount` 校验硬上限；这是结构合同，不是关键词语义审查。
- 超长使用现有唯一 repair 机会，要求保留结论、一个依据和一个验证点，删除复述与舞台动作。
- repair 仍超长时 Draft 保持原占位文本并记录失败；不得 `slice()`、不得确认半句。
- 继续保持 HTTP 请求不发送角色级 `max_tokens`，避免推理模型在 JSON 闭合前被截断。

## 10. 长局上下文选择

当前“每区 40 条、每条 800 字”会让后期 Prompt 和输出同时膨胀。改为一个集中策略：

1. 当前日的结构化结算、发言和有效主张；
2. 上一日完整结算与每名玩家最新立场，不重复整段全文；
3. 更早日只保留死亡、放逐、公开身份声明、仍未解决的查验冲突和每名存活玩家最近立场；
4. 票型使用结构化摘要，不依赖后续玩家逐字复述；
5. 本人仍有效的私有行动历史始终保留；
6. 剧本模式额外加入一个短 ActorBrief，不加入完整单局剧本。

选择策略只删减上下文，不改变 `projectVisibleEvents` 的知识权威，也不使用额外总结 LLM。

## 11. 主题剧情与视频

### 11.1 剧本模式的主题深度

单局剧本用主题模板的角色/行动映射组织实际剧情，例如《未明档案》中：

- 狼人是篡改者；查验是鉴真；守护是保护性封存；放逐是永久封存；
- 每个 Act 必须围绕本局真实计划事件，而不是反复说“档案、红印”；
- 终局身份公开后，Closing Beat 才能用全局真相回收此前误导与伏笔。

### 11.2 动态章节旁白

仅靠玩家台词无法让节目结构稳定呈现剧情。剧本编译可插入新的计划内公开事件 `narrative_beat_announced`：

```ts
type NarrativeBeatAnnouncedEvent = {
  type: "narrative_beat_announced";
  payload: {
    beatId: string;
    title: string;
    text: string;
    sourceStepIds: readonly string[];
  };
};
```

它只使用批准剧本中已经到达该位置的叙事，不改变规则状态。Player Context 将它分类为节目包装而不是游戏证据；Prompt 明确不可把旁白当身份证据。

动态旁白使用闻舟声音，但不能伪装成预制 Presenter manifest 片段。语音任务为这类确认事件生成不可变的事件级主理人音频；Playback 的 audio timeline 统一消费类型化 voice artifact。预计旁白时长在剧本批准前预留。

游戏模式不强制插入动态章节旁白，保持当前事件逻辑与主题包装。

## 12. 生命周期与 UI

### 12.1 New Game

选择顺序：

1. 运行模式：游戏模式 / 剧本模式；
2. 主题模板；
3. Preset 或随机座位；
4. 创建。

游戏模式直接保存并进入 Editor。剧本模式保存角色/身份/规则/主题快照和空事件，进入 Authoring 页面。

### 12.2 Authoring Job

单局剧本可能需要多次模型调用和模拟，不能依赖一个不可恢复的长 Server Action。沿用 voice/video job 的持久化模式：

- queued → generating outline → planning phases → validating → completed/failed；
- 页面轮询进度；
- job 结果通过 game lock 原子写入 `EpisodeScriptState.review`；
- Regenerate 创建新 job，旧 candidate 保留在 job 历史但不再是当前候选；
- 记录 author 模型、Prompt 版本、token usage 和失败诊断。

### 12.3 Review

导演 Review 展示：

- 标题、logline、计划胜方、预计天数和预计时长；
- 每幕/分场、主要冲突、身份披露、行动、票型和角色命运；
- 完整模拟 trace 的可折叠视图；
- 规则、信息边界、时长校验报告；
- Regenerate / Approve。

Approve 要求 candidate input hash 仍匹配 Game，且 `events.length === 0`。批准后 snapshot 不再引用可变主题库或 job candidate。

### 12.4 Editor

- `game`：保持当前行为，显示时长预算/预计时长。
- `scripted + !approved`：禁止 Advance，跳转 Review/Authoring。
- `scripted + approved`：行动、票型与计划披露元数据只读；发言 text 可生成、重生成和人工修改，但保存/确认前校验硬上限。
- Rollback 可回到已批准计划中的较早 event index，并从同一 plan step 重放；不能改结构。
- `script_diverged` 时停止推进并显示期望/实际 slot，不自动跳过或回退游戏模式。

## 13. 错误与恢复

| 条件 | 行为 |
|---|---|
| Author transport 失败 | job 失败，可整份 retry；不创建事件 |
| 大纲 JSON 非法 | 一次结构修复；仍失败则 job 失败 |
| 某阶段计划目标非法 | 用共享合法候选做一次局部修复；仍失败则 candidate 失败 |
| dry-run 未结束或胜方不符 | candidate 失败，不进入 Review |
| 预计时长 > 35 分钟 | candidate 失败或在 authoring 内重新压缩 Speech Beat；不能批准 |
| ActorBrief 引用不可见事件 | candidate 失败；不交给玩家模型 |
| 运行 Draft slot 与计划不符 | `script_diverged`，停止 Advance |
| 已批准行动在当前规则下不再合法 | `script_diverged`，停止确认；不默默选其他目标 |
| 发言首次超长 | 一次压缩 repair |
| repair 仍超长 | 保留 Draft，记录失败，允许人工修改/重新生成 |
| 批准时已有事件 | 拒绝批准，避免计划绑定错误初始状态 |

## 14. 兼容性、版本与回滚

- Repository 对旧 Game 注入 `runMode: "game"` 和 `episodeScript: null`，只在内存中规范化。
- `EpisodeScriptSnapshot.schemaVersion`、`compilerVersion` 和 input hash 必须验证；未知版本不可执行，但不影响读取游戏模式记录。
- Script Author Prompt 与玩家 Speech Prompt 分别版本化，Generation 记录能区分。
- `KIVA_LLM_PROMPT_VERSION=v1` 只影响旧玩家生成回滚，不可用于剧本模式；剧本模式要求支持预算和 ActorBrief 的新 Prompt 版本。
- 可用 `KIVA_SCRIPTED_MODE_ENABLED` 隐藏 New Game 的剧本模式入口；已存在的批准剧本仍需可读，禁用时不可继续执行。
- 不覆盖样本局 `ae9b3a8a-fb8b-43cf-901b-252ad49b0341`。验收使用临时复制/重放和新建剧本局。

## 15. 任务拆分

当前任务作为集成父任务，建议批准后建立三个可独立验收的子任务：

1. **游戏模式发言预算与上下文压缩**：先完成共享 `speech-budget`、超长 repair、时长投影和样本重放。
2. **单局剧本核心与规则模拟**：完成领域合同、Authoring、dry-run compiler、Actor Projection、批准快照与运行时 Draft 绑定。
3. **剧本模式产品与视频集成**：完成 New Game、Authoring/Review、Editor 锁定、动态章节事件/语音、Preview/Export。

第 2 项依赖第 1 项的预算合同；第 3 项依赖第 2 项的已验证接口。父任务负责最终双模式回归和 30 分钟成片验收，不直接承担重复实现。

## 16. 被拒绝的方案

- **只在现有 Prompt 加“请简短”**：没有硬上限、校验和总预算，样本已经证明不足。
- **视频导出时裁切或倍速**：破坏推理和观看节奏，且用户明确要求生成阶段控制。
- **一次生成整部最终对白**：输出巨大、局部修改会牵连全局，也难以对每句做独立超长修复。
- **另写一套剧本模式规则推进器**：会与 `advance-planner/rules/state` 漂移，无法证明剧本可执行。
- **把完整导演剧本塞给每个玩家**：泄漏未来和隐藏身份，使 Prompt 信息边界失效。
- **批准后动态重写剩余剧情**：第一版复杂度过高，会让伏笔、技能资源、Actor 知识和时长预算同时失效。
- **让调用方自行解析 plan step**：形成浅模块和跨层重复；所有匹配与投影必须留在 `episode-script` 深模块内。
