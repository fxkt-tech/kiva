# 客户端规范

适用：`src/client/`、`web/`、`index.html`、浏览器脚本。

## Pre-Development Checklist

1. 阅读 [browser.md](browser.md)。
2. 找齐 Rust HUD 投影、JSON 字段、DOM 渲染及浏览器断言的使用者。
3. 规则留在模拟层；内容装配位于 `src/client/content.rs`，输入和展示不得判断具体技能 ID。
4. 区分模拟固定步、显示帧、完整 HUD 快照三个更新频率。
5. 修改属性面板时阅读 [attributes-hud.md](attributes-hud.md)。
6. 修改选角/开局/玩家引用时阅读 [character-selection.md](character-selection.md)。

## Quality Check

- `make check`，涉及 WASM 专属代码时增加 WASM Clippy。
- 修改 JavaScript 后用 `node --check <文件>` 检查语法。
- 修改返回选角/整场重开时执行 `scripts/browser-reselection.cjs`，覆盖不同角色、同角色及普通继续。
- 按变动运行 `scripts/browser-skills.cjs`、`browser-cursor.cjs`、`browser-healthbars.cjs` 或 `browser-smoke.cjs`。
- 在 Pointer Lock 下验证 HUD 交互；检查 800×600、高 DPI 和窗口缩放。
- 浏览器脚本使用临时构建和自有 fixture，结束时清理。截图使用已验证的有窗口 Chromium 环境。
