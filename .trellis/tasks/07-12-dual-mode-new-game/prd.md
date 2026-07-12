# New Game 双模式选择与持久化

## Goal

重构 New Game 弹窗，让导演先选择“游戏模式”或“剧本模式”，并把选择作为不可变的 Game 快照持久化和正确路由。

## Requirements

- `GameRunMode` 只有 `game` 与 `scripted`；历史记录缺失时只读规范化为 `game`。
- 模式选择与现有“预设/随机座位”是两个不同维度，不复用同一个 `mode` 状态。
- 游戏模式保留当前创建后进入 Editor 的行为。
- 剧本模式创建相同角色、规则和主题快照，但进入剧本准备页；剧本核心上线前不得推进事件、生成配音或导出。
- 两条创建 action（预设、随机座位）都必须传递并验证 `runMode`，无效值必须拒绝。
- 首页游戏列表展示模式，避免导演混淆。

## Acceptance Criteria

- [x] New Game 弹窗明确展示两个模式的区别、流程和时长目标。
- [x] 预设与随机座位创建都能持久化所选模式。
- [x] 游戏模式路由到 Editor，剧本模式路由到剧本准备页。
- [x] 旧 Game JSON 加载后得到 `runMode: "game"`，无需迁移或回写。
- [x] 剧本模式在剧本未批准前不能通过 `continueGame` 推进。
- [x] 单元、server action、repository 兼容和构建检查通过。

## Out of Scope

- 生成、Review、Approve 或执行单局剧本。
- 修改狼人杀规则、角色分配或现有游戏模式推进逻辑。
