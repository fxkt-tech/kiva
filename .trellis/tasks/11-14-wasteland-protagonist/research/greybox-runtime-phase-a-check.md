# 灰模阶段A：实际运行姿态导出审查

范围：`src/client/avatar/wasteland_pose_tests.rs`、`avatar.rs`测试模块声明、`art/wasteland/greybox/runtime-poses.json`。未检查制作中的灰模脚本/网格；完整里程碑审查另行进行。

## 结论
无本阶段阻断项，无需修改。avatar.rs仅增加`#[cfg(test)]`模块；导出测试为显式忽略的离线工具，正常cargo test不写文件，产品构建不包含该模块。

## 路径与声明
- 通过已有PreviewRig或AttackReplay获得RigPose，将其写入实际显示owner，再执行avatar::sync；读取已生成PaletteJoint实体上的实际Transform。
- 用共享joint_pose结果交叉断言；挂点采用attachment_pose与attachment_transform；检查手掌挂点位置与实际腕关节一致。
- 以完整SourceBone身份选择palette槽并保留first outfit slot语义，rest不替换成inverse(IBM)。
- JSON明确仅含现有李火旺目标palette的模型空间矩阵，使用合成只读显示场景，不声称模拟回放、新网格兼容或披风完成；跳跃actor root高度不在文件中。

## 独立数据检查
- JSON schema为kiva-runtime-palette-poses-v1；140个唯一命名快照，每帧52个完整同名关节。
- 两组武器、六个近战段、两个近战技能均有快照；另含12种移动/防御/受击场景及参考姿态。
- 所有矩阵4x4、有限、affine最后一行正确；时间有限并位于各自声明区间。
- rest与reference-rig.json同名源参考比较，最大分量误差5.9326171975015995e-8（十进制float32表示差异）。
- BTreeMap/BTreeSet及固定帧步使导出字段/样本顺序稳定。

Rust编译/Clippy/测试由运行姿态实现代理统一执行，未并发重复运行。本记录不冒称这些检查已通过；完整报告以该代理检查结果为准。
