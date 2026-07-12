# New Game 双模式技术设计

## Domain contract

`Game.runMode: "game" | "scripted"` 是创建时确定的快照。`createGameFromPreset` 接收显式 `runMode`，兼容调用方可省略并默认 `game`；repository 对历史 JSON 采用相同默认值。

## Creation flow

弹窗拥有 `runMode` 与 `seatMode` 两个独立状态。两个 form 都提交隐藏字段 `runMode`。Server Action 使用共享 parser 严格验证，然后透传 library-actions/game-actions。创建后根据已持久化的 `record.game.runMode` 决定路由。

## Scripted guard

本子任务尚无 EpisodeScriptState，因此任何 `scripted` Game 都被视为未批准。`continueGame` 在计划 Draft 前拒绝，并提供确定错误。准备页只读取 Game 快照并说明下一步，不伪造剧本状态。

## Compatibility

旧记录在 `normalizeRecord` 中注入 `runMode: "game"`。不修改磁盘，不批量迁移。现有测试和 seed helper 未传模式时继续产生游戏模式。
