# New Game 双模式实施计划

1. 新增 GameRunMode、创建参数与 repository 兼容规范化测试。
2. 贯通 game-actions、library-actions 和两条 Server Action。
3. 重构 NewGameDialog：模式卡片优先，预设/随机作为座位配置。
4. 新增剧本准备页与 `continueGame` 防误推进门禁。
5. 首页显示模式；补齐组件、action 与路由测试。
6. 运行 targeted tests、全量 test、typecheck、build、diff-check。

## Rollback

删除 UI 模式选择并让所有创建调用省略 `runMode` 即回退为游戏模式；历史兼容默认保持安全。
