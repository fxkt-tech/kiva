# 单中路地图、基地与胜负闭环设计

共同边界和时序见[总体设计](../11-07-single-lane-match/design.md)。以下只记录本切片的模块和不变量。

- 新增无窗口gameplay的definition/map/match_state/spawn组合；SimulationPlugin只导出Step调度集合。
- MapDefinition同时派生NavigationMap、场景和小地图；基地采用可导航低台，避免封死目标中心。
- Outcome在Step之后、Wave之前；MatchPhase不可逆Finished，controls合并有效冻结不允许恢复旧局。
- client按显式表现描述适配基地模型/拾取/标签；全程共用UnitVisual显示根。
- 选择会话只协调公开start/reset，不在session里实现对局规则；新增结果请求关联match_id。

恢复/结构受击等拟议规则采用父任务最终确认结果，实施前刷新相关规范和真实前置接口。
