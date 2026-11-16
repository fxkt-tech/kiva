# concept-v11 灰模来源与阶段边界

权威美术原型为 [concept-v11.png](../concept-v11.png)。本目录是可编辑灰模，不是最终游戏角色；没有最终纹理、面部表情、实时布料或火焰特效。

## 原创部分

`tools/wasteland/build_greybox.py` 以实际参考骨位置创建原创新网格：躯干、手掌和三节手指、连通的腰头/裤裆/双腿曲面、短款上衣、每侧两层外袖、纯色内袖、鞋、曲面发束、围巾和不对称披风，以及原创朱雀线条的低浮雕占位。没有复制李火旺或其他旧游戏的网格、纹理或 IBM。骨架参考的既有来源单独见 [骨架来源](../rig-reference-provenance.md)。

## CC0 头部基础

为避免用简单球体冒充可审阅人脸，头部基础拓扑来自 [Blender 官方 Demo Files / Human Base Meshes](https://www.blender.org/download/demo-files/) 的 `Human Base Meshes v1.4.1`，许可 CC0。精简源为 `head-source.blend`，原完整文件/下载包的不同 SHA256、提取对象清单见 [head-source.provenance.json](head-source.provenance.json)。

只保留真实头部和左右眼白/虹膜的基础网格，移除原 Multires 与材质。精简源统一烘焙到原头部局部坐标：源 +Z 向上、-Y 面朝前。人物适配明确采用 `game=(-0.86*source.x, 1.29+0.94*source.z, 0.76*source.y+0.028)`；下脸轻微收窄，裁去低于原 Z=0.16 的肩颈扩展，让其与本角色领口结合，不改变核心骨长。

原素材的透明角膜位于虹膜前面；四材质灰模不用透明角膜 shader，因此虹膜在人物适配中向前移动 3mm 以便看清朝向。这是灰模展示适配，不是恢复原素材着色，也不是新增表情系统。

## 绑定与动作证据

新网格使用自己的 Blender bind，原资产 rest 与 IBM 只作为兼容参考分开保存。实际目标骨模型矩阵从共享 Kiva `RigPose/AttackReplay → avatar::sync` 导出，见同目录 `runtime-poses.json` 及其报告。灰模按目标当前/rest 的增量驱动，不直接把来源 TRS 当作新骨局部姿态。

`greybox-report.json` 记录全部 140 个实际快照的骨模型矩阵误差、用实际 Blender palette 与自身绑定计算的顶点 LBS 有限性、中性绑定零变形误差、四权重上限及归一化、基础和 evaluated 人物三角面。`runtime-render-index.json` 记录四张真实动作预览的片段身份/时间：跑步、蹲伏、锤狙组和链斧弓组攻击。`stress-*` 图片则是手工骨旋转压力姿势，来源明确不同。

数值有限和姿势导入通过不等于无穿模。肩/髋/膝、宽裤与披风仍需要通过灰模审阅决定后续拓扑和权重细化；披风三根辅助骨目前继承躯干，没有布料模拟或最终游戏驱动。完整武器刀刃轨迹比较、游戏接入、最终贴图和实际战斗验收仍在后续阶段。

复现命令见 [制作工具](../../../tools/wasteland/README.md)。
