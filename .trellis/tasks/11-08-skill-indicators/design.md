# 设计

本子任务遵循[父方案](../11-08-champion-skill-presentation/design.md)。

独占src/client/skill_indicators/、src/client/ability_targeting.rs、src/client/abilities.rs（CastInfo ability字段）、controls.rs和其必要测试；公共mod/scene/ui由root接线。

技术依据见仓库路径`.trellis/tasks/11-08-champion-skill-presentation/research/skill-presentation-contracts.md`，已取得实际样本/代码证据。跨模块接口改动必须先通知root及另一个renderer所有者；不要加第二套规则/位置/时钟，不在未落地规范里写计划。
