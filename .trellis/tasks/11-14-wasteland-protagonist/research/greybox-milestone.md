# 三维灰模与兼容蒙皮里程碑

用户已确认concept-v11为原型。当前完成三维灰模、绑定、可复现工具和真实姿态初验，下一步是用户审阅比例及轮廓。

## 文件
- art/wasteland/greybox/zhuque-v11-greybox.blend：可编辑灰模，已在Windows工作窗口打开。
- 同目录front/back/side/three-quarter/head-and-sleeve-detail.png：真实Blender渲染，无生成图替代。
- runtime-HammerMusket_game_run_0.50.png、runtime-HammerMusket_game_crouch_0.50.png及两组攻击预览，来源/时间见runtime-render-index.json。
- runtime-poses.json和两份报告保存真实现有目标驱动数据与灰模数值检查。
- head-source.blend/head-source.provenance.json/provenance.md记录CC0头部来源及角色适配。

## 验证与当前边界
最终实际求值后三角面38,954，4材质；不是只计未应用修改器的基础面数。140关节姿态映射通过，独立保存重开验证了权重、自有绑定、实际求值与独立LBS。详细结果见greybox-milestone-check.md和greybox-independent-report.json。

原生/WASM与新姿态测试已由实现代理通过；Python工具Lint/编译和文档/差异检查通过。数值通过不等于最终造型或游戏画面验收。当前面部/发型/胸纹为灰模；衣物在极限姿态的肩袖/髋部仍需细化，披风仅辅助骨预览，无布料物理或最终火焰效果。未导出KIVAAV03，未替换游戏外观，不修改模拟、旧资产或命中轨迹。

全任务尚未完成，保持in_progress，不提交或归档；等待用户灰模反馈后继续细化。
