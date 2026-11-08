# 设计

本子任务遵循[父方案](../11-08-champion-skill-presentation/design.md)。

独占assets/vfx/、scripts/import-league-vfx.py、src/client/skill_assets/，必要时提供独立通用chunk获取器，不动renderer、mod/scene/ui或模拟。

技术依据见仓库路径`.trellis/tasks/11-08-champion-skill-presentation/research/league-vfx-assets.md`，已取得实际样本/代码证据。跨模块接口改动必须先通知root及另一个renderer所有者；不要加第二套规则/位置/时钟，不在未落地规范里写计划。
