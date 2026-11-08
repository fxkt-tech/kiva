# 英雄联盟美术资源接入技术设计

继承[共同设计](../11-08-battle-presentation/design.md)。

本地 GLB 资源插件独立适配角色/兵种模型、动作、死亡和重生，保留 UnitVisual 世界根与低帧/暂停采样。源计划寒冰22008、皇子59000已有完整带骨骼动画GLB。蒙皮模型按每JointAabb/当前joint矩阵做真实拾取。按 AbilityId 图标，现有单中路地图只换纹理。资源异步加载与错误要明确，启动及Web发布复制本地 assets。

## 变更边界
新 client 模型/资源模块、必要 character 接口、src/client/picking.rs、world_scene.rs、unit_appearance.rs、assets/models/textures/icons及可复现导入/清单；公共 client/mod.rs、index.html、Cargo由主代理统一。生命周期任务先拥有scene/ui/hud的相关改动，尽量用插件与清晰接缝。

## 协作
读取本任务 research/ 中的已验证样例；公共接线由主代理负责，文件重叠先沟通。实现完成只报告实际验证，不能静默省略需求或伪造资源来源。
