# Kiva Trellis 规范初始化

## 目标

响应用户“使用 Trellis 先出详细方案”的要求，为本项目建立可用的本地任务工作流和真实代码规范。

## 完成情况

- [x] 使用本机 Trellis 0.6.17 初始化 Codex 项目文件与 asuka 工作区。
- [x] 从 `docs/architecture.md`、模拟和客户端源码、测试中提取已有约定。
- [x] 将不适用的 backend/数据库模板替换为 simulation/client 规范。
- [x] 给出公开构造、指令、生命周期、HUD 与浏览器验证的实际路径和例子。
- [x] 调整思考指南为 Kiva 数据流，去除 Trellis 自身开发的无关模板内容。
- [x] 关闭任务归档/日志自动提交，保留用户自行启停开发服务的约定。

## 验收

规范入口与相对链接有效，没有待填写段落；初始化不改动游戏业务代码，
不修改用户全局 Codex 设置，不启停开发服务。功能任务另建为 `11-07-base-stats` 并保持 planning。

## 主要产物

- `.trellis/spec/index.md`
- `.trellis/spec/simulation/`
- `.trellis/spec/client/`
- `.trellis/spec/guides/`
- `AGENTS.md`、`.trellis/config.yaml`
