# 客户端规范

适用：`src/client/`、`web/`、`index.html`、浏览器脚本。

## Pre-Development Checklist

1. 阅读 [browser.md](browser.md)。
2. 找齐 Rust HUD 投影、JSON 字段、DOM 渲染及浏览器断言的使用者。
3. 通用规则留在模拟层，对局/内容装配位于 `src/gameplay/`；`client/content.rs`只准备客户端上下文，输入和展示不得判断具体技能 ID。
4. 区分模拟固定步、显示帧、完整 HUD 快照三个更新频率。
5. 修改属性面板时阅读 [attributes-hud.md](attributes-hud.md)。
6. 修改选角/开局/玩家引用时阅读 [character-selection.md](character-selection.md)。
7. 修改装备管理、主动输入或六格HUD时阅读 [equipment.md](equipment.md)。
8. 修改鼠标位移、光标时序或点击位置时阅读 [pointer.md](pointer.md)。
9. 修改地图、基地、波次、终局/重开、角色类型表现时阅读 [lane-match.md](lane-match.md)。
10. 修改角色套件、视野、护盾、充能或重施时阅读 [champion-presentation.md](champion-presentation.md)。
11. 修改角色模型、骨骼、动作或其生命周期时阅读 [character-animation.md](character-animation.md)。
12. 修改HUD布局、默认镜头/人物屏幕比例或小地图取景框时阅读 [hud-layout.md](hud-layout.md)。
13. 修改模型悬停、鼠标命中或右键/指向技能选目标时阅读 [unit-picking.md](unit-picking.md)。
14. 修改导入 GLB、源动作、模型比例、地图纹理或图标映射时阅读 [imported-models.md](imported-models.md)。
15. 修改战斗音效、手势解锁、音频队列或音量控制时阅读 [combat-audio.md](combat-audio.md)。
16. 修改技能原素材、特效生命周期、瞄准几何或共享目标资格时阅读 [skill-presentation.md](skill-presentation.md)。
17. 修改施法模式、技能快捷键或按键目标快照时阅读 [casting.md](casting.md)。
18. 修改设置弹窗、Esc或模拟运行/输入门控时阅读 [settings.md](settings.md)。

## Quality Check

- `make check`，涉及 WASM 专属代码时增加 WASM Clippy。
- 角色表现变更运行 `cargo test --locked --lib client::character`，保留scene击飞根测试，验证有窗口截图和技能/血条/重开回归。
- 修改 JavaScript 后用 `node --check <文件>` 检查语法。
- 启动加载/DOM顺序变更运行 `node --test scripts/startup-unit.test.cjs`，并在重建后串行执行 `scripts/browser-startup.cjs` 与最终smoke。
- 修改返回选角/整场重开时执行 `scripts/browser-reselection.cjs`，覆盖不同角色、同角色及普通继续。
- 装备变更执行 `scripts/browser-equipment.cjs`，并按影响回归属性、技能、捕获及重开入口。
- 按变动运行 `scripts/browser-skills.cjs`、`browser-cursor.cjs`、`browser-healthbars.cjs` 或 `browser-smoke.cjs`。
- 在 Pointer Lock 下验证 HUD 交互；检查 800×600、高 DPI 和窗口缩放。
- 小地图改动运行 `node --test scripts/minimap-unit.test.cjs` 与 `scripts/browser-minimap.cjs`，覆盖真实镜头点击/拖动、完整地图、逐帧矩形、输入隔离和清理。
- 模型命中改动运行 `cargo test --locked --lib client::picking` 和 `scripts/browser-picking.cjs`，保留视野/击飞/原生输入回归，检查纯模型检测与剑形光标。
- 指针变更运行 `node --test scripts/pointer-unit.test.cjs` 和 `scripts/browser-pointer-response.cjs`，独立对比正常移动比例及同事件响应。
- 导入模型/死亡展示变更运行 `scripts/browser-presentation.cjs`，覆盖本地模型和图标、尸体/真实倒计时、复活与重选；保留原生帧钟血条回归。
- 音频变更运行 `node --test scripts/audio-unit.test.cjs`、Rust音频行为回归与 `scripts/browser-audio.cjs`，使用真实资源解码和音频节点输出证据。
- 浏览器脚本使用临时构建和自有 fixture，结束时清理。截图使用已验证的有窗口 Chromium 环境。
