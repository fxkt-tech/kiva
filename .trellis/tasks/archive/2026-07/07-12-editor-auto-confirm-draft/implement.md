# Editor 自动确认 Draft 开关 — Implementation Plan

## 1. Add Preference and Timer Contracts

- 在现有 LLM generation 客户端模块中定义命名空间化的 `localStorage` key。
- 添加纯函数解析持久化值，缺失/异常值默认关闭。
- 添加纯延迟常量与就绪状态契约；自动确认不依赖 LLM pending request。
- 先扩展 `llm-generation-notification.test.ts` 覆盖上述边界。

## 2. Add the Toggle UI

- 新增自动确认 toggle client component，挂载后读取浏览器偏好。
- 点击时同步更新 React 状态和 `localStorage`。
- 使用现有 icon button 样式、明确的开/关视觉状态、动态 `aria-label`/`title`。
- 将按钮放入 Draft panel header，保留当前 notification permission button 和生成状态布局。
- 扩展 `draft-panel.test.ts` 验证可访问入口。
- 移除初始 LLM 生成遮罩，将 pending 状态放到 Generation 按钮；隐藏自动提交表单继续负责发起请求。

## 3. Wire Ready Drafts to Existing Confirmation

- 保持 generation completion effect 只处理浏览器通知。
- 自动确认 control 在任何 Draft `ready` 且开关开启时启动 1 秒 timer，并在关闭、Draft 变化或卸载时清理。
- 将手动确认 form 和自动确认都改为传递当前 Draft ID。
- 扩展 `confirmDraftAction` / `gameActions.confirmDraft` 契约，在 game lock 内只确认与 expected Draft ID 匹配的当前 Draft；不匹配时 no-op，避免手动与自动提交竞态推进两个 Draft。
- timer 到期后通过 React transition 调用 `confirmDraftAction(gameId, draftId)`。
- 普通 Draft 立即 ready；LLM 生成结束（成功或失败）后 ready，生成中不确认。
- 不修改 repository 数据格式或 core Draft 事件语义。

## 4. Verify Continuous and Stop Behavior

- 验证开启：成功 generation 自动确认，随后下一 LLM Draft 继续自动生成与确认。
- 验证请求中关闭：当前请求继续，但完成后停留在 Draft。
- 验证请求中开启：当前请求完成后自动确认。
- 验证失败、刷新已有成功 Draft、非 LLM Draft不会误确认。
- 验证刷新或切换游戏后偏好仍保留，首次使用默认关闭。

## Validation Commands

```bash
pnpm test -- src/components/editor/llm-generation-notification.test.ts src/components/editor/draft-panel.test.ts
pnpm test -- src/server/__tests__/game-actions.test.ts
pnpm typecheck
pnpm test
```

## Risk and Rollback Points

- 风险最高的文件是 `src/components/editor/llm-generation-notification.tsx`：pending 消费顺序错误可能重复确认或误确认。
- `confirmDraft` 签名会影响现有调用与测试；实施时必须先搜索所有调用点，并确保 expected Draft ID 校验发生在 game lock 内。
- `src/components/editor/draft-panel.tsx` 等相关文件已有用户未提交修改；实施前后必须用限定路径的 `git diff` 核对，只增量保留本任务变化。
- 若自动确认导致 UI transition 错误，可先移除 completion effect 中的 action 调用，同时保留无害的 toggle；完整回滚无需迁移数据。

## Review Gate Before Activation

- PRD 中无未决产品问题。
- toggle 默认关闭、浏览器级持久化和连续模式已获用户确认。
- 实施严格复用现有 `confirmDraftAction`，自动确认与通知用的 pending request matcher 解耦。
- 用户审核本方案后，才运行 `task.py start` 进入实现阶段。
