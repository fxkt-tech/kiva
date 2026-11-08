# 英雄联盟美术资源接入

## 目标与需求
对应父任务 [需求](../11-08-battle-presentation/prd.md) 的 R4 外观、R5，沿用其用户授权、范围与验收；用户明确要求方案后立即实现，无需再次确认。

## 验收
本地 GLB 资源插件独立适配角色/兵种模型、动作、死亡和重生，保留 UnitVisual 世界根与低帧/暂停采样。源计划寒冰22008、皇子59000已有完整带骨骼动画GLB。蒙皮模型按每JointAabb/当前joint矩阵做真实拾取。按 AbilityId 图标，现有单中路地图只换纹理。资源异步加载与错误要明确，启动及Web发布复制本地 assets。

功能保持既有输入、模拟和整局生命周期；提供真实行为测试与集成证据，不以仅能编译视为完成。

## 边界
新 client 模型/资源模块、必要 character 接口、src/client/picking.rs、world_scene.rs、unit_appearance.rs、assets/models/textures/icons及可复现导入/清单；公共 client/mod.rs、index.html、Cargo由主代理统一。生命周期任务先拥有scene/ui/hud的相关改动，尽量用插件与清晰接缝。
不提交/推送，不接管开发服务。
