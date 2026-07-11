# 删除旧 Preview 页面及相关代码

## 目标

新的 HTML/Remotion Preview 已取代旧 Pixi Preview。删除旧实现及其专属代码，并让新实现成为唯一的 `/games/[gameId]/preview` 页面，减少重复维护成本；保留视频导出和新 Preview 依赖的共享逻辑。

## 已确认事实

- 旧 `/preview` 页面使用 `PlaybackStage` 和 Pixi 渲染链路。
- 新 Preview 原先位于 `/preview_v2`，editor 内嵌预览和视频导出都已使用新实现。
- `src/components/preview/` 不能整体删除：`preview-events.ts`、`preview-audio.ts` 以及部分 `shot-engine` 模块仍被新 Preview 或视频导出引用。
- `/api/**/preview-v2/**` 是视频导出 API，不是页面路由，本任务不对其重命名。

## 需求

1. 删除旧 Pixi Preview 页面实现，以新的 HTML/Remotion Preview 提供 `/games/[gameId]/preview`。
2. 删除 `/games/[gameId]/preview_v2` 页面路由，不保留重定向或兼容路由。
3. 首页每个游戏只保留一个 Preview 入口并指向 `/preview`，沿用旧 Preview 的眼睛图标。
4. editor 的内嵌和新窗口 Preview 指向 `/preview`；Preview 标题栏只保留右侧的新窗口入口。
5. Preview 页面不显示顶部 header、返回 editor 按钮和 `HTML video master` 标识；游戏标题和视频规格信息显示在右侧栏。
6. 游戏变更后的 preview 路径刷新使用 `/preview`。
7. 删除旧 PlaybackStage、Pixi renderer、DOM image renderer 契约、相关专属测试和 Pixi 依赖。
8. 保留新 Preview、视频导出仍使用的共享模块；不为了目录命名整洁而复制或移动共享代码。
9. 清除产品代码和测试中指向 `/preview_v2` 页面路由或已删除模块的引用。历史设计/计划文档保留原貌。

## 验收标准

- [x] `/games/[gameId]/preview` 运行新的 HTML/Remotion Preview 实现。
- [x] `src/app/games/[gameId]/preview_v2/page.tsx` 不再存在。
- [x] 首页只有一个 Preview 入口，使用眼睛图标并指向 `/games/[gameId]/preview`。
- [x] editor 的内嵌和新窗口 Preview 均指向 `/preview`，标题栏无重复入口。
- [x] Preview 页面无顶部 header、返回按钮和 master 标识，游戏标题及视频规格显示在右侧栏。
- [x] server actions 刷新 `/preview`。
- [x] 旧 Preview 专属组件、测试和依赖被删除，剩余 `src/components/preview/` 文件都有新 Preview、导出或自身测试的有效消费者。
- [x] 全局搜索确认产品代码不再引用 `/preview_v2` 页面路由或已删除模块。
- [x] 类型检查、相关测试和生产构建通过。

## 范围外

- 重命名 `/api/**/preview-v2/**` 视频导出 API。
- 重命名 `preview-v2` 组件目录、内部类型或 smoke 脚本。
- 重构新 Preview、视频导出或共享 shot/audio/event 模块。
- 修改历史 Trellis 任务及 `docs/superpowers` 中记录当时设计的文档。
