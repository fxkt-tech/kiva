# 模型接入实施计划

- [x] 核对现有 AV03／动作驱动与准备资源的层级、单位、材料、骨序和源数据证据。
- [x] 固定 model-runtime-contract，转换生成可复现随包身体和武器资源，保持 prepared 原件／清单哈希不变。
- [x] 玩家默认李火旺；妹妹替换为白灵淼源蒙皮；引冰落、羿之影替换相应远程几何。
- [x] 对接女性骨架参考与身体挂点，保持模拟 source_position、动作时间线和模型射击源一致；配置相关独立附件。
- [x] 更新最小 UI 身份说明及有意义的真实资产／蒙皮／生命周期回归。
- [x] Trellis check 完成全范围检查；make check、WASM Clippy、临时 Trunk构建与 Cargo.lock 不变检查。
- [x] 按实际条件记录画面验证；更新架构／规范／资产来源与当前任务记录，不改第一版战斗规则。

代码与自动检查完成，浏览器功能流程通过；真实 GPU 画面验收继续跟踪，详见 research/integration-verification.md，任务暂不归档。

## 文件责任与依赖

研究代理仅写本任务 research；导出代理负责 scripts/import-prepared-models.py 与新的 assets/bailingmiao、assets/yinbingluo、assets/yizhiying（不改 prepared 原件）；运行时代理负责 src/client、web相关身份显示、index及对应Node测试。
先由导出代理固化文件／二进制合同；运行时实现随后对接。主会话负责文档、规范、集成和质量检查协调。
