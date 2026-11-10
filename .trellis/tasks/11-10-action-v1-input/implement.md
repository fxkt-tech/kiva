# 子任务执行清单

- [ ] 阅读父任务 PRD／design／implement、相关 specs 和 research。
- [ ] 明确并遵守唯一写入范围：src/client/controls.rs、src/client/web.rs、web/input.js、scripts/input-unit.test.cjs。
- [ ] 依赖模拟子任务发布的 runtime-contract.md；可先实现独立浏览器事件队列，Rust调用在接口固化后接入。
- [ ] 按负责验收实现并更新有意义的真实行为回归。
- [ ] 执行所属层的格式、编译／Node检查；集成前不因其他代理进行中的接口变化删除有效测试。
- [ ] 向主会话报告文件、验证结果和剩余风险，不提交／推送／归档，不派下一层实施或检查代理。
