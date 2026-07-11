# 提示词结构 v2 实施计划

## 1. 建立失败样例与任务矩阵测试

- [x] 为现有 12 种 LLM Draft 建立类型级测试工厂。
- [x] 把 `docs/examples/` 中的场景错乱、事实混淆、首位发言空洞、错误概率和女巫阵营假定转成 Prompt 结构回归断言。
- [x] 增加编译期穷举检查，新增 LLM Draft 时必须提供 Task Spec。
- [x] 保留现有 visibility-safe 和 legacy snapshot 回退测试。

验证：

```bash
pnpm test src/core/__tests__/prompt-builders.test.ts src/core/__tests__/action-generation.test.ts src/core/__tests__/player-context.test.ts
```

## 2. 统一合法候选来源并修正规则漂移

- [x] 抽取 Prompt 与输出 parser 共用的 `legalOptionsForDraft`。
- [x] 区分 exile 与 PK 投票目标；PK 只允许平票候选。
- [x] 按 `allowAbstainVote` 控制 Prompt 和 parser 是否接受 `null`。
- [x] 排除预言家历史已查验目标。
- [x] 明确女巫同夜双药禁止时的毒药行为，避免生成合法但无法结算的 Draft。
- [x] 按 `voteReveal` 修正单票事件的可见时间，保留密封票规范。

验证：

```bash
pnpm test src/core/__tests__/rules.test.ts src/core/__tests__/advance-planner.test.ts src/core/__tests__/action-generation.test.ts src/core/__tests__/player-context.test.ts
```

回滚点：该步骤会影响事件可见性和合法候选，必须在进入 Prompt v2 前独立通过完整事件链测试。

## 3. 建立语义化 Prompt Context

- [x] 扩展或拆分 `PlayerLlmContext`，加入当前场景、派生状态和资源信息。
- [x] 将可见事件确定性分类为公开事实、玩家主张、个人私有事实和阵营讨论。
- [x] 定义当前频道的披露策略；保持 `projectVisibleEvents` 为底层知识边界。
- [x] 去除可由场景表达的身份牌/阶段事件重复。
- [x] 集中实现长局上下文预算；Task Spec 不直接截断事件。

验证：

```bash
pnpm test src/core/__tests__/player-context.test.ts src/core/__tests__/visibility.test.ts
```

## 4. 实现穷举 Task Spec 注册表

- [x] 新增全部 12 种 Draft 的任务规格；`vote_cast` 再按 `voteType` 分派。
- [x] 每个规格定义自然语言场景、听众、任务、must-cover、must-not、相关规则键和输出种类。
- [x] 限制 `roleActionPromptSnapshot` 只用于匹配 mechanic 的私有技能行动。
- [x] 为无信息局面加入稳定候选轮换和明确的不确定性指令。

验证：

```bash
pnpm test src/core/__tests__/llm-task-specs.test.ts
```

## 5. 实现 Prompt v2 渲染器

- [x] 按“场景 → 任务 → 信息分区 → 候选/资源 → 相关规则 → 人物表达 → 输出”渲染。
- [x] System Prompt 建立优先级、事实纪律和披露纪律。
- [x] 避免重复注入 character system/persona/style；为旧游戏保留回退。
- [x] 发言输出改为 `text + decisionSummary`，必要时加入 `disclosure`。
- [x] 行动输出按必选目标/可选行动拆分 schema。
- [x] 升级 Prompt version 和 schemaName，保留 v1 builder 作为短期回滚路径。

验证：

```bash
pnpm test src/core/__tests__/prompt-builders.test.ts src/core/__tests__/speech-generation.test.ts src/core/__tests__/action-generation.test.ts
```

## 6. 加入一次最小修复请求

- [x] 区分前置规则失败与模型输出失败，只有后者允许 repair。
- [x] repair 只发送错误、原始输出、合法 schema 和候选集合。
- [x] GenerationRecord 保存 attempt 信息，同时保持旧记录可读。
- [x] 第二次失败后保持 Draft 不变。

验证：

```bash
pnpm test src/core/__tests__/llm.test.ts src/core/__tests__/generation-record.test.ts src/core/__tests__/action-generation.test.ts src/core/__tests__/speech-generation.test.ts
```

## 7. 更新调试展示与兼容

- [x] 编辑器优先显示 `decisionSummary`，继续兼容历史 `reasoning`。
- [x] Generation details 清楚显示 Prompt v2、schema 和 repair attempts。
- [x] 确认 v1 游戏 JSON 无需迁移即可载入和显示。

验证：

```bash
pnpm test src/components/editor/llm-generation-details.test.tsx src/components/editor/draft-panel.test.ts src/server/__tests__/game-repository.test.ts
```

## 8. 全量质量门与人工回放

- [x] 运行差异格式检查、类型检查、生产构建和全部测试（项目没有 lint script）。
- [x] 用 `docs/examples/` 对应局面重新生成 v2 日志。
- [x] 按设计文档七项评分表对照 v1/v2，记录每类失败是否消失。
- [x] 特别重复狼队四人密票，确认最终结构合法率 100%，且密票彼此不可见。
- [x] 检查公开狼人发言、神职披露一致性和延迟公开票型。

验证：

```bash
# 当前 package.json 没有 lint script
pnpm typecheck
pnpm test
```

## 9. Editor LLM 完成通知

- [x] 首次自动生成和手动 Regenerate 开始前记录当前 Draft 与旧 Generation ID。
- [x] 新 GenerationRecord 返回后发送成功或失败的浏览器通知，不误报历史记录。
- [x] Draft 标题栏提供通知授权入口；浏览器不支持或拒绝授权时静默降级。
- [x] 增加新旧 Generation ID、其他 Draft 和损坏标记的回归测试。

## Review Gates

1. 完成步骤 2 后，复核规则行为变化没有超出 Prompt 所需范围。
2. 完成步骤 5 后，先查看全部 12 类构造出的 Prompt，再接入真实模型。
3. 完成人工 v1/v2 回放后，才移除 v1 运行时回滚路径。

## Risky Files / Rollback Points

- `src/core/visibility.ts`、`src/core/advance-planner.ts`：投票可见性改变会影响 Preview/Editor，需遵守 private-event workflow。
- `src/core/action-generation.ts`：候选、解析和 repair 必须共用同一合法性来源。
- `src/core/player-context.ts`：任何新投影都必须先通过隐藏信息负向测试。
- `src/core/prompt-builders.ts`：保留 v1 入口直到真实回放通过，便于单点回滚。
- `src/core/generation-record.ts`：新增 attempt 信息必须向后兼容已有 JSON。
