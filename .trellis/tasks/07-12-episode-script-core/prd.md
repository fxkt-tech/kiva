# 单局剧本核心与规则模拟

## Goal

建立可持久化、可审核、可批准并能按同一规则引擎执行的结构化单局剧本。首版先由规则推进器确定性生成完整合法 trace，再让 Script Author 为合法 trace 编写主题化叙事和逐场 Speech Beat。

## Requirements

- Episode snapshot 版本化并绑定 Game cast/rules/theme input hash。
- dry-run 循环调用 `planNextDraft`、`confirmDraftEvent`、event log 与 state，最多 240 步且必须到 `game_ended`。
- 每个 step 保存稳定 Draft slot、计划 payload 和 speech beat；不保存模拟 Draft/Event ID。
- Script Author 只能装饰已确定的合法 trace，不得改变结构、行动、票型或胜负。
- Actor Projection 只暴露当前 beat，不提供完整剧本或未来 step。
- 运行时按 active event index 匹配批准 step，slot/payload/hash 不一致时停止。

## Acceptance Criteria

- [x] 同一 Game 两次 dry-run 产生相同 slot/payload trace 并到终局。
- [x] Snapshot/schema/hash/step/beat 都有集中验证。
- [x] 运行时 Draft 绑定复用现有 planner，并锁定结构 payload。
- [x] Actor Projection 只返回当前玩家当前场景信息。
