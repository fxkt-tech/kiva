# 实施计划

- [x] 研究两件冻结 GLB 的 current-LBS、原材料／实际轴与握柄；核对谪星主件／变式／FX 的控制器证据。
- [x] 导出者先发布 runtime-contract，再生成可复现的新运行资产及来源／数值报告；不得改 prepared 和旧运行资产。
- [x] 运行时复用有界武器解码及缓存，替换两组近战和 FlyingAxe；保持远程接口／动作和绳索规则。
- [x] 更新皮肤身份文字；实际手腕、两身体外观、离手回收、暂停／死亡／F2／F3、资产复用回归。
- [x] 独立 Trellis check：来源／静态选型／姿态与跨层边界；运行 make check、WASM Clippy、临时 Trunk、离线重建与 lock 未变检查。
- [x] 记录浏览器／画面条件与剩余限制，更新现行模型规范、架构和会话日志；全部在 main，保留未提交工作区。

自动质量门槛与浏览器功能流程全部通过；真实GPU画面验收继续跟踪，不提前归档。

## 所有权

研究代理仅写本任务 research/source-evidence.md；导出代理负责新 scripts/import-melee-models.py 与 assets/{fulangmao,zhexingcuishan}，以及 research/runtime-contract.md，不改旧转换器／旧输出／prepared。
运行时代理负责 src/client 内武器模块及必要调用方、web/hud.js、index.html、对应 Node 测试；不得写 simulation／gameplay／导出资产。
主会话负责任务文档、规范、架构与集成；先固定导出合同，再对接，避免重复格式／坐标判断。每个实现者仅在自己所有权内编辑，最终检查者在交接后接管修复。
