# 子任务执行清单

- [x] 阅读父任务 PRD／design／implement、相关 specs 和 research。
- [x] 明确并遵守唯一写入范围：src/client/**（除 controls.rs、web.rs）、web/hud.js、web/style.css、index.html、scripts/hud-unit.test.cjs、scripts/appearance-unit.test.cjs、scripts/audio-unit.test.cjs。
- [x] 依赖模拟子任务发布 runtime-contract.md；不得另造规则时钟。与输入子任务文件不重叠，集成遵循父任务A→B→C检查点。
- [x] 按负责验收实现并更新有意义的真实行为回归。
- [x] 执行所属层的格式、编译／Node检查；集成前不因其他代理进行中的接口变化删除有效测试。
- [x] 向主会话报告文件、验证结果和剩余风险，不提交／推送／归档，不派下一层实施或检查代理。

## 实施结果（2026-11-10）

- 新增 client/companion.rs 与 weapons.rs：独立蓝衣双马尾临时妹妹、人形骨架、锤／鸟铳／链斧／弓；复用共享几何与材质。妹妹枪／弓口直接读取 CompanionState.source_position，手部姿态与各自武器握柄同步。
- 哥哥继续使用原身体／服饰和源骨骼动画，F3 不再替换战斗武器；敌人原长刀保持。动作归属读取 actor，射击声音只消费 Shot，武器类别读取不可变事件 pair。
- 两类镜头使用同一模拟序列，保留插值与球体净空；仅实际锤鸟发射产生镜头后坐。取消／死亡／F2（包括暂停中发射后切模式）有普通镜头恢复路径。
- 链斧离手／连线／回收与三箭效果读取动作快照；无伤害、物理和输入权限。
- HUD 为生命1000、耐力、E／Q双冷却、当前绑定组、右键状态、Tab前置条件；进入／暂停卡片、中键锁定、F3身体外观及敌人模式提示同步更新。JS无规则计时器且只更新安全文字／属性。
- 检查：cargo test --locked --lib client:: 95项通过；随后新增真实链斧显示／回收／资源复用测试，单独1项通过。Node HUD／appearance／audio／enemies共26项通过。rustfmt指定拥有文件、git diff --check通过。
- 原生Clippy已无本子任务拥有文件问题；最后一次全库Clippy仍报告模拟combat与输入controls的进行中问题，已交回对应负责人。make check、WASMClippy、打包与浏览器画面验收由父任务统一集成，不将代码测试等同画面验收。
- 音效沿用既有程序枪声、共享attack及hit／deflect适配，无新增音频资产；临时妹妹和武器不宣称正式美术。未提交、推送或启动服务。
