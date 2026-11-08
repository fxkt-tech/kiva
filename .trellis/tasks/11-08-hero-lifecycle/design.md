# 英雄尸体与复活倒计时技术设计

继承[共同设计](../11-08-battle-presentation/design.md)。

模拟只增加 death_tick()/respawn_remaining() 等必需只读事实，死亡与复活维护时间；尸体、显示姿态与HUD各用现有所有者。与资源任务约定 UnitState::death_tick() -> Option<u64>、respawn_remaining() -> Option<f32>，不额外创建模拟实体，不修改 Died 既有结构（音频在每固定步后采集事件）。

## 变更边界
src/simulation/model.rs、combat.rs、mod.rs；src/client/scene.rs、unit_appearance.rs；现有 character 的死亡姿态；src/client/ui.rs、web/hud.js、web/hud.css、必要测试。公共 client/mod.rs 与 index.html 由主代理接线。

## 协作
读取本任务 research/ 中的已验证样例；公共接线由主代理负责，文件重叠先沟通。实现完成只报告实际验证，不能静默省略需求或伪造资源来源。
