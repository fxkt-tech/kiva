# 朱雀：原创废土主角与精修披风

`assets/characters/zhuque.bin` 是严格模式导出的 KIVAAV04：**139,152三角形 / 显式预算150,000 / 7显示部件 / 5材质 / 20张实际2048² PNG**。运行来源为 `art/wasteland/cape-refined/zhuque-cape-runtime-source.blend`，交换件为 `art/wasteland/runtime/zhuque.glb`；未使用任何development参数。

高精编辑源 `art/wasteland/cape-refined/zhuque-cape-detail.blend` 为430,128三角形，保留围巾堆叠、肩部连接、非周期褶皱、错落羽尾及真实厚度。它不是直接运行资源：运行制作源取消披风Subsurf、保留Solidify；高源到运行表面的最大点距约2.351毫米，三块布面RMS分别约0.117/0.146/0.153毫米。准确路径、哈希、面数与差异记录于manifest和`cape-authoring-report.json`。

原39,976面首版来源 `art/wasteland/textured/zhuque-v11-textured.blend` 保留。`baseline-v11-*.json`是此前发布的历史记录，其运行路径现已由新版覆盖。非披风部件几何/UV/权重指纹仍由制作报告逐项保持；原三身体材质及火焰材质共16个贴图通道的导出PNG哈希与历史记录逐个相同。新增独立披风PBR使用4张实际FILE/packed贴图，Normal Strength1、无不支持的Sheen。三块布面的图集矩形互不重叠；Solidify内外/厚度rim有意共享无差异布纹UV，ORM.R=1，不冒称全局唯一图集或烘焙了内外不同AO。

身体参考现有李火旺真实gMan骨结构与source-rest，运行保留52个核心骨。没有复制其可见网格或旧inverse-bind，也没有新增攻击时钟。Blender/GLB另保留3个披风辅助骨，运行权重显式合并到`gMan Spine2`，全部140个实际目标姿态验证delta等价。新模型自身绑定采用参考R，**新IBM=inverse(R)**，实际当前矩阵A的变形为`A*inverse(R)`；Blender自身参考M使用`P=B*A*inverse(R)*inverse(B)*M`。

严格导出独立读取实际Blender求值顶点和GLB节点/skin/IBM/UV：

- 140姿态关节矩阵最大分量误差2.61e-6，辅助骨合并7.75e-7。
- 460,110次真实Blender顶点对照，量化运行LBS最大误差约0.613毫米。
- 实际GLB92,022顶点，绑定几何误差约0.000571毫米，UV误差2.98e-8。
- 5个姿态（参考、跑、跳、右闪避、首段实际攻击tick024）共460,110次实际GLB顶点对照，运行LBS最大误差约0.613毫米。
- GLB显式应用非Armature修改器，实际交换几何包含Solidify厚度。五材质AO/MR共用声明ORM，图像索引/采样器及完整内嵌PNG独立读回见`glb-material-report.json`。

原始头部拓扑来自Blender官方Human Base Meshes 1.4.1的CC0基础头，经过本任务比例/脸型适配；出处哈希在manifest和`head-source.provenance.json`。衣服、发型、围巾披风、鞋与朱雀纹样为本任务作者网格和纹理。

只有独立AlphaBlend火焰边缘带有Ember标签，进行确定性UV流动和发光脉动；不透明布面UV保持固定。没有实时布料物理或面部表情。数值证明不代替真实浏览器/游戏画面验收。

浏览器稳定选择ID3，查看页ID `zhuque`；默认李火旺和妹妹保持原行为。游戏启动仍按既有清单要求全部外观，新资源随发行版加载；查看页沿用所选资源待就绪门槛。复现命令/格式见`tools/wasteland/runtime-export.md`，准确来源和误差见本目录manifest与conversion-report。
