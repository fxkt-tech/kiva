# 剧本模式产品集成设计

创建 action 持久化 scripted Game 后立即调用 `generateEpisodeScript`。该服务先在 game lock 内写 generating/jobId，再在锁外执行 author，最后只在 jobId 仍当前时写 review/failed。Regenerate 复用相同入口。Approve 携带 jobId+scriptId，在锁内复验空事件、input hash、报告和 candidate identity。

Script 页面按持久化状态渲染。Editor 只接受 approved；planner 使用 `planNextEpisodeDraft`，Prompt 使用当前 `EpisodeActorBrief`，结构编辑在 UI 和 server 两层阻止。现有 Preview/voice/export 只消费确认事件，因此未批准空事件自然不可输出；页面继续显示 blocker。

Script Author 使用一次精简 outline 请求和最多 12 个 speech beat 的分批请求。每批只携带附近结构窗口；Headers Timeout 自动重试一次，仍失败则进入 failed，避免重试同一个超大输出。
