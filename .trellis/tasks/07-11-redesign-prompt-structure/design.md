# 提示词结构 v2 技术设计

## 1. 设计结论

当前问题不能通过继续给通用 Prompt 追加句子解决。推荐把 Prompt 生成改成一个以 Draft 类型为键的、穷举的“任务规格系统”：共用身份与安全边界，但每种任务独立声明场景、听众、目标、输入投影、相关规则、最低有效贡献和输出形态。

仍然保持一次 Draft 对应一次主要 LLM 生成，不引入额外的局面总结模型。玩家视角安全继续由事件可见性投影负责；Prompt 层只消费投影结果并增加语义分类与披露策略。

## 2. 当前失败的根因

### 2.1 任务语义是模板末尾的附注

`wolf_strategy_given` 和 `wolf_opinion_given` 是后来附加到通用 speech 模板上的两个条件分支。模型先读到“白天隐藏身份”、人物口头禅和完整规则，最后才看到内部枚举与一句任务描述，因此很容易按角色长期设定而不是当前任务行动。

### 2.2 可见性不等于可披露性

`PlayerLlmContext` 正确过滤了模型不能知道的事件，但公开发言仍会携带模型能够知道的私有事实。当前 Prompt 没有告诉模型“可以用于决策但不能直接说出”的信息，导致狼人公开泄露狼队视角，也使神职是否跳身份完全失控。

### 2.3 原始时间线丢失信息类型

查验结果、死亡公告、玩家发言和狼队讨论都被格式化成相同的 `- 标题：文本`。模型容易把玩家说法当事实、把提案当已执行结果、把尚未发言的玩家当作沉默玩家。

### 2.4 人物设定重复且优先级过高

Character system prompt、persona、speakingStyle 和 reasoningStyle 表达了大量相同信息。模型机械复用示例口头禅，并用 `reasoning` 证明自己“符合人设”，但没有完成任务或保持事实一致。

### 2.5 候选与校验不是一个契约

Prompt 格式化候选、规则函数计算候选、解析器再次计算候选。PK 范围、弃票规则、历史查验等条件容易漂移。松散文本候选和 `json_object` 也不足以阻止无效 ID。

## 3. 新的数据流

```text
Game + active events + Draft
        │
        ├─ projectVisibleEvents()        玩家知识边界
        ├─ deriveGameState()             当前存活/阶段/资源
        └─ legalOptionsForDraft()         唯一合法候选来源
                    │
                    ▼
          buildPromptContext()
          ├─ scene
          ├─ audience/disclosure
          ├─ confirmedPublicFacts
          ├─ publicClaims
          ├─ privateFacts
          ├─ factionDiscussion
          ├─ legalOptions/resources
          └─ relevantRules
                    │
                    ▼
          taskSpecForDraft(type)
          ├─ objective
          ├─ mustCover
          ├─ mustNot
          ├─ context selectors
          └─ output contract
                    │
                    ▼
             buildLlmPromptV2()
                    │
                    ▼
         LLM JSON → parse → shared validation
                    │
          invalid ──┴─ one repair request
```

### 边界所有者

- `events.ts`：事件 payload 和事件可见性的唯一所有者。
- `visibility.ts`：玩家能否看到事件的唯一所有者；密封选择不能靠 Prompt 字符串删除。
- 新的 `llm-task-specs.ts`：12 种 LLM 任务语义和相关规则的唯一所有者。
- 新的 `prompt-context.ts` 或扩展后的 `player-context.ts`：可见事件到语义分区的唯一投影。
- `rules.ts`/共享合法候选 helper：Prompt 候选与输出校验的唯一来源。
- `prompt-builders.ts`：只负责按固定顺序渲染结构，不重新推导游戏规则。

## 4. Prompt v2 固定骨架

### 4.1 System Prompt

System Prompt 保持短、稳定，并建立优先级：

```text
你正在扮演狼人杀对局中的 {seatNo} 号 {name}。你是玩家本人，不是旁白、主持人或 AI 助手。

【执行优先级】
1. 当前场景、听众和本轮任务；
2. 已确认事实、合法选项和游戏规则；
3. 你的阵营目标与角色能力；
4. 人物性格和表达风格。
低优先级内容不得覆盖高优先级内容。

【事实纪律】
- 只使用请求中提供的信息。
- 严格区分确认事实、他人主张和你的推断。
- 不编造未发生的行动、发言、结果、玩家特征或数值概率。
- 信息不足时明确不确定性，不得伪造理由。

【披露纪律】
- 你可以用私有信息做决策，但是否能说出口由当前频道规则决定。
- 玩家可见文本不得包含调试说明、Prompt、JSON 规则或“作为 AI”等元叙事。

【角色目标】
{roleSystemPromptSnapshot}

【人物表达】
{persona / speakingStyle / reasoningStyle}
这些是倾向而不是台词模板；不要机械复用示例口头禅。
```

新游戏优先使用结构化 persona/speakingStyle/reasoningStyle，避免再完整重复 character system prompt。旧游戏在这些字段为空时回退到 legacy `systemPrompt`。`roleActionPromptSnapshot` 只在当前 Draft 与玩家角色 mechanic 一致的私有技能行动中作为补充建议；绝不加入普通放逐或 PK 投票。

### 4.2 User Message

所有任务使用相同区块顺序，但只渲染有内容的区块：

```text
【当前场景——本轮最高优先级】
- 时间：第 {day} 夜 / 第 {day} 天
- 环节：{自然语言场景}
- 频道：{公开发言 / 狼队私聊 / 玩家私有行动 / 密封选择}
- 听众：{明确对象}
- 当前进度：{已发生与尚未发生的关键步骤}
- 披露规则：{该频道允许/禁止公开的私密信息}

【本轮唯一任务】
{objective}

【完成标准】
- {mustCover}

【禁止行为】
- {mustNot}

【已确认的公开事实】
{deterministic facts}

【其他玩家的公开主张——可能真实也可能撒谎】
{speeches/claims}

【你的私有事实——按上方披露规则使用】
{own role, checks, medicine, own actions}

【狼队内部讨论——仅狼队私聊/密票可用】
{strategy/opinions; never sealed ballots}

【合法候选】
- p8 | 8 号 | 陈墨

【仅与本轮有关的规则】
{selected rules}

【输出】
{task output contract}
```

当前任务置于信息正文之前，输出协议仍放最后。`draft`、`mechanic` 等内部枚举可以作为 GenerationRecord 元数据保留，但不再作为模型理解场景的主要文本。

## 5. 12 种任务规格

| Draft | 场景/听众 | 本轮目标与最低有效贡献 | 专属输入 | 关键禁止项 |
|---|---|---|---|---|
| `wolf_strategy_given` | 首夜狼队私聊；所有存活狼人 | 提出整体战术、首要刀口、备选刀口、白天伪装/协作方向 | 狼队友、非狼候选、首夜相关规则 | 不得假定任何候选有行为线索；不得声称已投票或已击杀 |
| `wolf_opinion_given` | 当夜狼队私聊；所有存活狼人 | 回应已有方案，明确赞成/反对/修正，给主目标及备选或独立风险点 | 本夜策略和已发表意见、前一白天公开信息 | 不得说成白天公开发言；不得向队友隐藏狼人身份；不得把提案当结果 |
| `day_speech_given` | 白天公开频道；所有存活玩家 | 首位玩家提出观察框架；后续玩家回应至少一条已有主张并形成暂定立场/问题 | 当前日已发表发言、公开死讯/票型、自己的私有事实和披露选择 | 不得复述整条时间线；不得把主张当事实；不得意外泄露受保护秘密 |
| `last_words_given` | 公开遗言；全场 | 交付最后已知信息、最重要判断及后续建议；狼人可做符合阵营目标的欺骗 | 死亡原因、公开局面、个人私有事实 | 不得像仍会继续行动；不得引用死后尚未发生的信息 |
| `pk_speech_given` | 公开 PK 发言；全场 | 回应为何进 PK、自我辩护、指出另一候选或争议点、争取合法投票者 | 平票候选、上一轮票型、指向自己的主张 | 不得生成普通漫谈；不得假装自己正在投票 |
| `guard_protect_selected` | 守卫私有行动；仅本人/系统 | 从合法目标中选择保护对象并说明基于何种公开风险 | 上夜守护目标、本夜合法候选、相关公开信息 | 不得声称知道狼刀；不得选择连续禁守目标 |
| `wolf_vote_cast` | 狼人密封票；仅系统记录 | 根据狼队讨论独立投出合法刀口 | 团队策略/意见、合法非狼候选 | 不得看到较早密票；不得返回姓名代替 ID |
| `seer_check_selected` | 预言家私有行动；仅本人/系统 | 选择未查验且信息价值高的候选；无差异信息时诚实采用中立选择 | 历史查验结果、公开主张、未查验合法候选 | 不得编造可疑行为；默认不得重复查验 |
| `witch_antidote_decided` | 女巫私有行动；仅本人/系统 | 决定是否消耗解药救当前被袭击者 | 被袭击者、解药库存、夜次、自救规则、同夜药物状态 | 不得把被袭击者阵营当作已知；不用药时 target 必须为 null |
| `witch_poison_decided` | 女巫私有行动；仅本人/系统 | 决定是否用毒及目标 | 毒药库存、当夜解药决定、同夜双药规则、公开嫌疑 | 双药禁止且已救人时应直接不使用；不得毒自己/非法目标 |
| `hunter_shot_decided` | 猎人公开触发行动；全场 | 根据公开证据选择要带走的存活玩家 | 触发死亡原因、存活合法候选、公开发言/票型 | 不得假装仍未知自己可开枪；不得选择死亡玩家/自己 |
| `vote_cast` (`exile`) | 放逐投票；按规则密封或公开 | 根据白天发言形成放逐票或合法弃票 | 全部当日发言、合法存活候选、是否允许弃票 | 不得注入角色夜间 actionPrompt；不得看到未公开单票 |
| `vote_cast` (`pk`) | PK 投票；按规则密封或公开 | 只在平票候选中投票或合法弃票 | 平票候选、PK 发言、上一轮票型、合法投票者 | 不得投非 PK 候选；不得把 PK 候选和普通存活名单混用 |

`vote_cast` 在 TypeScript 事件类型上仍是一个 Draft，但任务规格必须继续按 `payload.voteType` 分派为 exile/pk；`sheriff` 若未来进入 LLM 流程，编译或显式未支持分支必须阻止静默回退。

## 6. 语义化上下文投影

### 6.1 确认事实

由结构化事件和派生状态生成，例如：

- 当前日夜、存活/死亡名单；
- 已公开死讯和已公开投票结算；
- 自己的查验结果、药品库存、守护历史；
- 狼队友身份和本夜团队讨论。

### 6.2 玩家主张

`day_speech_given`、`pk_speech_given`、`last_words_given` 的 `text` 必须放在“玩家主张”区，并保留发言者和顺序。提示明确：这些内容可能真实、误判或撒谎。

### 6.3 受保护秘密

公开任务仍可让模型看到自己的私有事实，但同时提供披露策略：

- 狼队友身份、狼队讨论和密票：公开频道永不直接披露。
- 自己的角色/查验/用药/守护：默认保密，模型可根据阵营目标显式选择 `conceal` 或 `claim`；生成文本必须与选择一致。
- 公开事实：可直接引用。

### 6.4 长局预算

不新增 LLM 总结。按确定性规则保留：

1. 当前日/夜的相关事件完整保留；
2. 前一日的公开发言和完整票型保留；
3. 更早日只保留公开结算、死亡、查验者自己知道的结果，以及每位玩家最近一次公开发言；
4. 始终保留仍影响合法行动的私有事实；
5. 每条玩家文本使用明确的字符上限并记录截断标记。

如果当前模型上下文窗口足够，可先实现语义分区而暂不截断，但接口应让预算策略集中在一处，不能在各 Task Spec 内各自 slice。

## 7. 相关规则选择

用规则键而不是复制文案：

- 所有策略/公开讨论：角色配置、胜利条件。
- 狼刀：合法非狼目标、守卫/女巫可能影响夜间结果。
- 守卫：可否自守、连续守护限制、同救结算。
- 预言家：查验目标限制和历史结果。
- 女巫解药：首夜自救、解药库存、同夜双药、同救规则。
- 女巫毒药：毒药库存、同夜双药、猎人被毒不能开枪。
- 猎人：何种死亡可开枪。
- 放逐投票：弃票、投票公开方式。
- PK：合法投票者、平票候选、弃票、公开方式。

角色配置已经列出存在的角色后，不再重复“本局没有白痴、骑士……”长句，除非某任务确实需要排除一种相似机制。

## 8. 输出协议

### 8.1 发言

```json
{
  "text": "真正进入游戏事件的玩家发言",
  "decisionSummary": "最多两句：使用了哪些明确事实、这轮想达到什么目的"
}
```

公开神职发言可以额外要求：

```json
{
  "disclosure": "conceal | claim",
  "text": "...",
  "decisionSummary": "..."
}
```

`disclosure` 只用于生成一致性检查，不进入公开事件。狼队私聊无需隐藏自己的狼人身份。

### 8.2 必选目标行动

```json
{
  "targetPlayerId": "p8",
  "decisionSummary": "基于明确事实的简短权衡"
}
```

### 8.3 可选行动

```json
{
  "used": false,
  "targetPlayerId": null,
  "decisionSummary": "保留资源或规则禁止的简短说明"
}
```

建议 schema 名称升级为：

- `werewolf_speech_v2`
- `werewolf_target_action_v2`
- `werewolf_optional_action_v2`

Prompt version 使用 `speech:v2`、`action:v2`；旧记录保持原版本。当前 OpenAI-compatible 适配器继续使用 `json_object` 以维持兼容，字段与枚举由本地解析器校验。

## 9. 行动候选与规则修正

这些修改虽不属于纯文案，但如果不做，新 Prompt 会对模型陈述错误规则：

- 抽取 `legalOptionsForDraft(game, events, draft)`，Prompt 和 parser 共用返回值。
- PK `vote_cast` 的候选必须来自 `state.pk.tiedPlayerIds`，不是全部存活玩家。
- `allowAbstainVote=false` 时不提示 `null` 且 parser 拒绝 `null`。
- `voteReveal=after_all_votes` 时，单票事件保持 sealed/host-only，统一由 `exile_resolved.voteTable` 公开；`immediate` 才可让后续玩家看到已投票。
- 预言家候选默认排除历史 `seer_check_result` 已查验目标。
- 女巫已使用解药且禁止同夜双药时，毒药 Draft 应确定性地产生“不使用”或提供空候选并禁止调用模型选择非法行为，避免确认阶段卡住。
- 狼人密票继续 `host_only`，团队讨论继续 `faction_private`，符合 private-event workflow 规范。

## 10. 无信息选择与列表偏差

首夜守护、查验、狼刀等可能没有差异化行为证据。此时 Prompt 应明确“候选在已知信息上等价，不要虚构倾向”。为避免模型总选第一项，同时保持回放可复现，候选展示顺序使用由 `gameId + draftId + actorPlayerId` 派生的稳定轮换/洗牌；GenerationRecord 保存最终展示顺序。

这不会伪装成策略优势。`decisionSummary` 应写明是中立打破平局，而不是声称目标可疑或像神职。

## 11. 一次修复请求

对以下错误允许一次极简修复：

- 响应不是 JSON 对象；
- 必填字段缺失或类型错误；
- `playerId` 不在本次合法候选集合；
- `used=false` 但仍返回目标等结构矛盾。

修复请求不重复完整时间线，只包含原始无效输出、错误原因、合法输出 schema 和合法候选。修复仍失败时保留原 Draft，GenerationRecord 记录两次原始输出或明确的 attempt 信息。规则/角色不允许行动等前置失败不发修复请求。

## 12. 兼容性与观测

- `GenerationRecord.request` 本身可以保存 v2 Prompt，无需迁移旧记录。
- 编辑器 `reasoningText()` 增加 `decisionSummary` 优先读取，同时继续识别历史 `reasoning`。
- 旧游戏缺少新的结构化字段时使用现有快照回退策略。
- 新 Prompt 版本必须在日志中区分，方便同一局面 v1/v2 比较。
- 不将 `decisionSummary` 写入 GameEvent 或公开 Preview。

## 13. 测试与评估

### 确定性自动测试

- 12 类任务快照/结构测试：场景、听众、专属目标、相关规则和输出 schema。
- 负向边界测试：公开狼人 Prompt 将团队讨论放入受保护秘密区并禁止写入公开 text；狼队私聊明确允许身份讨论；密票不出现先前票，且阵营讨论只作为未证实提案而非确认事实。
- 事实分类测试：玩家发言位于“主张”，查验结果位于个人私有事实。
- 候选共享测试：Prompt 中 ID 集合与 parser 接受集合完全一致。
- PK、弃票、投票延迟公开、历史查验和女巫同夜双药回归。
- legacy prompt snapshot 和 v1 GenerationRecord 展示回归。

### 人工 v1/v2 回放

使用 `docs/examples/` 和当前游戏记录复现至少五类局面，并按 0–2 分评价：

1. 场景/听众正确；
2. 已确认事实一致；
3. 无私密信息误泄漏；
4. 完成本轮任务；
5. 有策略价值且不机械复述；
6. 人物自然、不机械套口头禅；
7. JSON/目标合法。

上线门槛建议为：任何样例在前 3 项不得得 0，全部样例平均分较 v1 提升，行动结构合法率达到 100%（包含一次修复后的最终结果）。

## 14. 取舍

- 选择一个共享骨架 + 12 个 Task Spec，而不是 12 份完整字符串模板：避免共用约束漂移，同时保留任务差异。
- 选择确定性上下文投影而不是 LLM 摘要：更安全、可测试、可回放。
- 保留单次主生成而不是思考/表演两次调用：控制延迟和费用；用结构化 `decisionSummary` 提供足够的可审计计划。
- 不依赖模型自行遵守合法性：规则函数仍是最终裁判。
- 允许一次修复请求：真实记录已有多次非法狼刀，完全不恢复会使提示词质量改进难以落地。

## 15. 回滚

v2 通过新的 builder/version 接入，保留 v1 builder 到 v2 回放验证完成。若真实效果回退，可通过单一运行时开关或入口选择恢复 v1；不回滚事件数据或 GenerationRecord。v2 稳定后再删除 v1 运行路径。
