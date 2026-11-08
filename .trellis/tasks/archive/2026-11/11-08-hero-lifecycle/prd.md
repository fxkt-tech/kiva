# 英雄尸体与复活倒计时

## 目标与需求
对应父任务 [需求](../11-08-battle-presentation/prd.md) 的 R1、R2，沿用其用户授权、范围与验收；用户明确要求方案后立即实现，无需再次确认。

## 验收
模拟只增加 death_tick()/respawn_remaining() 等必需只读事实，死亡与复活维护时间；尸体、显示姿态与HUD各用现有所有者。与资源任务约定 UnitState::death_tick() -> Option<u64>、respawn_remaining() -> Option<f32>，不额外创建模拟实体，不修改 Died 既有结构（音频在每固定步后采集事件）。

功能保持既有输入、模拟和整局生命周期；提供真实行为测试与集成证据，不以仅能编译视为完成。

## 边界
src/simulation/model.rs、combat.rs、mod.rs；src/client/scene.rs、unit_appearance.rs；现有 character 的死亡姿态；src/client/ui.rs、web/hud.js、web/hud.css、必要测试。公共 client/mod.rs 与 index.html 由主代理接线。
不提交/推送，不接管开发服务。
