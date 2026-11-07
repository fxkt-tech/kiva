# 定时兵线、小兵AI与表现设计

共同边界和时序见[总体设计](../11-07-single-lane-match/design.md)。以下只记录本切片的模块和不变量。

- gameplay/waves使用SimulationTick安排期限，生成前再次检查Running，全部6个出生配置预校验并原子提交。
- gameplay/ai/minions保存目标UnitLifeRef与中路进度，不把玩家AttackMove改造成兵线专用规则。
- simulation/movement基于一致上一tick快照实现有限软避让，稳定排序并继续经过NavigationMap sweep。
- client低成本共享小兵几何/材质/武器与专属拾取标签配置；不将完整英雄骨架无差别复制给所有单位。

恢复/结构受击等拟议规则采用父任务最终确认结果，实施前刷新相关规范和真实前置接口。
