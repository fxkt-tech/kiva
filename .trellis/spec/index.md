# Kiva 开发规范

本项目是 Rust / Bevy 单 crate 游戏，通过 Trunk 构建 WebAssembly。这里记录已经存在的代码约定；
待实现功能以 `.trellis/tasks/` 中的方案为准。[架构文档](../../docs/architecture.md)定义游戏规则与状态所有权。

## 规范入口

| 层 | 入口 | 涉及路径 |
|---|---|---|
| 模拟 | [simulation/index.md](simulation/index.md) | `src/simulation/`、`tests/simulation.rs` |
| 客户端 | [client/index.md](client/index.md) | `src/client/`、`web/`、`index.html`、`scripts/browser-*.cjs` |
| 跨层检查 | [guides/index.md](guides/index.md) | 模拟接口、HUD 数据和输入之间的改动 |

## Pre-Development Checklist

1. 阅读任务的 `prd.md`、`design.md`、`implement.md`，区分已批准范围与建议。
2. 按修改路径加载对应规范，先用 `rg` 找齐同一字段和接口的使用者。
3. 保持 `client → simulation` 的依赖方向；不将 DOM、按键或模型放入模拟规则。
4. 使用无窗口行为测试验证规则，浏览器测试验证实际交互和显示。

## Quality Check

- 功能改动执行 `make check`：格式、原生 Clippy、Rust 测试、WASM 检查。
- Web 专属 Rust 改动增加 `cargo clippy --locked --target wasm32-unknown-unknown -- -D warnings`。
- HUD 或交互改动执行对应浏览器回归；构建使用 `trunk build --dist <临时目录>`。
- 文档改动检查路径、链接、任务 JSON/JSONL 和 `git diff --check`，不为纯方案改动重跑游戏测试。
- 用户自行运行开发服务。测试自建临时 HTTP fixture 后自行关闭，不接管用户的 Trunk 进程。
- `.trellis/spec/` 写已实现约定，任务设计写拟议改动，二者不得混为既成事实。
