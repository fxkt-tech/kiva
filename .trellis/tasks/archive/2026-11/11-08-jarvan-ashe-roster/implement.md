# 实施顺序
1. 钉住一级数据与换算；定义通用模拟接口和只读客户端/AI查询，保持旧字段默认语义。
2. simulation 实现被动/护盾/场上对象与光环/几何技能/视野/临时地形，通过统一动作和命中链集成。
3. gameplay 并行接入新角色/小兵数值、自然恢复、实际技能装配与语义AI；client 并行准备弓/枪、对象显示、技能状态与团队可见性。
4. 对接公开API后修复所有枚举/构造/HUD使用者；更新基于旧内容的断言，保留通用规则原测试。
5. make check、WASM Clippy、浏览器回归、独立 Trellis check；更新实际架构文档和验证记录，不自动提交或接管服务。

分工：模拟代理独占 src/simulation 与新 tests/champion_mechanics.rs；玩法代理独占 src/gameplay 和 gameplay 相关 tests；客户端代理独占 src/client/web/scripts 浏览器相关文件；主代理负责任务/研究/文档、接口协调和最终集成验证。交叉文件先沟通，不抢写。
