# Preview 舞台事件结果可视化 — Implementation Plan

## 1. Structured playback data

- [x] 在 `src/core/playback.ts` 定义封闭的 `StagePresentation` 判别联合。
- [x] 实现 `stagePresentationForEvent`，覆盖首批事件与所有空结果分支。
- [x] 从 `exile_resolved.voteTable` 确定性聚合候选票数和弃票数。
- [x] 调整 audience 过滤：director 包含 `night_resolved`，public 继续隐藏；密票保持隐藏。
- [x] 将 `stage` 写入 `PlaybackItem`，不解析展示文案。
- [x] 扩展 playback tests，先覆盖映射和隐私矩阵。

## 2. Composition validation

- [x] 在 composition decoder 中验证 `stage` 判别联合。
- [x] 保持旧 composition 缺失 `stage` 时合法并渲染空舞台，避免 schema version 迁移。
- [x] 增加合法、非法和缺失 `stage` 的 decoder tests。

## 3. Stage view model

- [x] 在共享 stage 组件中以纯投影解析 `StagePresentation + scene.players`。
- [x] 对玩家缺失、目标为空、多死亡、多候选和零票做安全降级。
- [x] 通过共享舞台 render tests 覆盖 participant/result 投影。

## 4. Shared rendering

- [x] 用 `StageEventVisual` 替换 `HtmlPlaybackStage` 的空中央 `<section>`。
- [x] 实现共享容器与 action、night_result、vote_result、game_result 四种布局。
- [x] 复用 `StagePalette` 和现有玩家头像资产解析，不引入异步请求。
- [x] 实现基于 `sceneMs/progress` 的淡入、连线、结果印章动画。
- [x] 对没有 `stage` 的普通播报/发言保持中央舞台为空。

## 5. Timing and composition fixtures

- [x] 检查各类事件现有持续时间容纳 1050ms 入场，无需改变场景时长。
- [x] 保持测试 fixtures/builders 对可选 `stage` 的兼容。
- [x] Presenter/玩家配音时长逻辑未修改，字幕同步保持不变。

## 6. Verification

- [x] `pnpm typecheck`
- [x] `pnpm test -- --run`
- [x] `pnpm build`
- [x] 浏览器检查 director Preview 的实际守卫场景，组件测试覆盖其余变体与空结果。
- [x] 检查 public audience 不出现守卫、狼刀、查验、女巫等私密舞台数据。
- [x] 导出短视频 smoke，确认共享舞台组件可进入 Remotion 渲染链路。
- [x] `git diff --check`

## Risk and rollback points

- 最大风险是错误放宽 `night_resolved` 可见性；必须以 audience 单测作为第一道门。
- 第二风险是 `PlaybackItem` decoder 破坏历史 export；缺失字段兼容测试必须在 UI 开发前完成。
- 舞台渲染应作为可移除的叶子组件；若视觉实现出现问题，可回滚 `StageEventVisual` 而保留结构化数据和原有播放。
