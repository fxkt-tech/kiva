<!-- TRELLIS:START -->
# Trellis Instructions

These instructions are for AI assistants working in this project.

This project is managed by Trellis. The working knowledge you need lives under `.trellis/`:

- `.trellis/workflow.md` — development phases, when to create tasks, skill routing
- `.trellis/spec/` — package- and layer-scoped coding guidelines (read before writing code in a given layer)
- `.trellis/workspace/` — per-developer journals and session traces
- `.trellis/tasks/` — active and archived tasks (PRDs, research, jsonl context)

If a Trellis command is available on your platform (e.g. `/trellis:finish-work`, `/trellis:continue`), prefer it over manual steps. Not every platform exposes every command.

If you're using Codex or another agent-capable tool, additional project-scoped helpers may live in:
- `.agents/skills/` — reusable Trellis skills
- `.codex/agents/` — optional custom subagents

Managed by Trellis. Edits outside this block are preserved; edits inside may be overwritten by a future `trellis update`.

<!-- TRELLIS:END -->

## Kiva 项目约定

- 架构依据：[docs/architecture.md](docs/architecture.md)。项目是单 crate 的 Bevy 游戏，模拟规则与客户端单向依赖。
- Trellis 规范入口：[.trellis/spec/index.md](.trellis/spec/index.md)。功能方案写在对应任务目录，未落地的设计不能写成现有规范。
- 用户自行启动开发服务。构建、测试可以执行；不要启动常驻开发服务，也不要关闭用户的服务。
- 提交与推送按用户当前授权执行；Trellis 的任务归档和日志记录关闭自动提交。
