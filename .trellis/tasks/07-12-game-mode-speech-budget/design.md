# 游戏模式发言预算技术设计

## 1. 设计结论

新增 speech-budget 深模块作为 Prompt、解析、人工编辑、Editor 观测和后续剧本 Author 的共同 seam。调用方只获取预算和评估结果，不自行维护字数常量。

主要接口：

- speechBudgetForDraft(input) -> SpeechBudget
- evaluateSpeech(text, budget) -> SpeechEvaluation

## 2. 初始预算

| Draft | 目标非空白字符 | 硬上限 |
|---|---:|---:|
| 首夜狼队战术 | 110–150 | 180 |
| 狼队意见 | 70–100 | 120 |
| 当日首位发言 | 100–140 | 170 |
| 普通公开发言 | 140–180 | 220 |
| 遗言 | 160–200 | 240 |
| PK 发言 | 140–180 | 220 |

最终值由样本真实 TTS 的约 6.1 非空白字符/秒回放校准。

## 3. 数据流

Draft 和 active events 先产生 SpeechBudget 与压缩后的可见上下文，再构建 Prompt。LLM 返回完整 JSON 后统一评估 text；超过硬上限进入现有一次 repair，仍失败则保留 Draft。

## 4. 预算策略

- Base contract 由 Draft 任务类型拥有。
- 游戏模式结合已承诺时长与 pacing envelope 选择 normal、compressed 或 critical，但不能低于最低有效表达。
- spokenCharacterCount 统一按 Unicode code point 统计非空白字符。
- 预计语音使用样本校准模型；实际 Preview 仍以 TTS artifact duration 为权威。
- 35 分钟是标准局/验收长局目标，不是理论无限平票路径的数学保证。

## 5. 上下文选择

- 当前日发言与结构化结算保留。
- 上一日保留结算、每名玩家最新立场和仍有效冲突。
- 更早历史只保留死亡、放逐、公开身份声明、未解决查验冲突和存活玩家最近立场。
- 票型生成结构化摘要；本人仍有效的私有历史始终保留。
- 选择发生在 visibility 投影之后，不改变知识集合。

## 6. 失败与兼容

- 超长是结构合同错误，可使用现有一次 repair；不新增关键词语义拦截。
- repair 只携带原始输出、字数错误和压缩合同。
- v1 历史记录和旧游戏继续读取。
- 不发送角色级 max_tokens，避免 JSON 未闭合。

## 7. 验证

- 单元：预算矩阵、字符计数、档位、估时、超长 repair。
- 回归：Prompt、speech generation、人工编辑、visibility、playback。
- 真实：57 个样本局面回放与导演版时长重编排。
