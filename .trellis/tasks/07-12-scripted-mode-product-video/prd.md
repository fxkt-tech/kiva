# 剧本模式产品与视频集成

## Goal

完成从 New Game 到可执行 Editor 的完整剧本模式链路：自动生成、持久化状态、失败重试、导演 Review、Regenerate、Approve、结构锁定、逐场主题化台词与 Preview/Export 门禁。

## Requirements

- Scripted New Game 自动开始 authoring；已有 idle/failed 游戏可手动生成或重试。
- 状态持久化为 idle/generating/review/approved/failed；job identity 防止旧结果和旧审批覆盖。
- Review 展示标题、logline、幕、胜方、天数、时长、完整 trace、发言节拍和报告。
- Approve 后跳转 Editor；结构 action 只读，speech text 可在 beat 和预算内生成/编辑。
- 未批准时 Editor、配音、Preview 导出不得开始正式执行。
- Home 显示 run mode 与 script state。

## Acceptance Criteria

- [x] 创建剧本模式游戏后自动得到 Review 或明确失败状态。
- [x] Regenerate/Approve/过期审批行为正确。
- [x] 批准后完整 trace 可执行到终局，结构字段不可编辑。
- [x] 玩家 Prompt 包含当前 Actor Brief，不包含完整或未来剧本。
- [x] 全量测试与生产构建通过。

## Out of Scope

- 独立进程 worker、动态主理人 TTS 和真实 30 分钟 MP4 留作视频增强；首版 authoring 在持久化 job identity 下由 Server Action 执行。
