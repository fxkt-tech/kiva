# Kiva 开发规范

Rust / Bevy 单 crate，Trunk 构建 WebAssembly。这里只记录当前竹林动作战斗原型的事实；
未实现方案写在 `.trellis/tasks/`，旧 MOBA 规则查看 `lol` 分支。

| 层 | 入口 | 路径 |
|---|---|---|
| 模拟 | [simulation/index.md](simulation/index.md) | `src/simulation/` |
| 玩法 | [gameplay/index.md](gameplay/index.md) | `src/gameplay/` |
| 客户端 | [client/index.md](client/index.md) | `src/client/`、`web/`、`index.html` |
| 跨层检查 | [guides/index.md](guides/index.md) | 模块边界与输入 / 显示时序 |

## Pre-Development Checklist

1. 阅读当前任务的 `prd.md`、`design.md`、`implement.md`，再读涉及层的规范。
2. 用 `rg` 找接口和已有行为，保持 `client → gameplay → simulation` 依赖方向。
3. 以 [架构文档](../../docs/architecture.md) 为状态所有权依据；DOM、相机、模型和按键属于客户端。

## Quality Check

- 功能变更执行 `make check`：格式、原生 Clippy、Rust / Node 测试、WASM 检查。
- Web 专属 Rust 代码增加 `cargo clippy --locked --target wasm32-unknown-unknown -- -D warnings`。
- 发布入口变更执行临时目录 Trunk 构建；使用本机支持的参数，以 `CARGO_NET_OFFLINE=true trunk build --dist <临时目录>` 构建并验证 Cargo.lock 未变化。
  构建和纯代码测试不能宣称完成画面 / 手感验收。
- 文档检查链接、任务 JSON / JSONL 和 `git diff --check`。
- 用户自行启动和关闭开发服务。可以自建并回收测试专用临时 HTTP fixture，不能接管用户进程。
- 提交与推送按本次用户授权执行，任务记录关闭自动提交。
