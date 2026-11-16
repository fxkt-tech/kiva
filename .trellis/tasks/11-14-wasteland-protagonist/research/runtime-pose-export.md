# 真实显示驱动姿态导出里程碑

## 变更边界
新增仅测试构建的 `src/client/avatar/wasteland_pose_tests.rs`，`avatar.rs` 增加 `#[cfg(test)]` 模块声明。没有修改运行时逻辑、动画、模拟、资产、默认外观或网页入口。工具文档为 `tools/wasteland/runtime-pose-export.md`。

## 交付
- `art/wasteland/greybox/runtime-poses.json`：140个姿态、52个无前缀gMan关节；原始sourceRestModel单独保留，输出矩阵均为行主序游戏模型空间、未乘IBM。
- `runtime-poses.report.json`：输入资产哈希、导出哈希、数值误差与验证记录。
- movement由现有PreviewRig，攻击由现有AttackReplay，均经真实avatar::sync读PaletteJoint Transform。两组武器三段普攻与近战技能、待机走跑、跳跃/二段跳、四向闪避、蹲伏、防御和受击均覆盖。首段每3帧保留短序列；每招从40帧待机独立开始，不冒称连续战斗录制。
- hand_grips与weapon_mount由共享挂点计算导出；握点原点与实际腕位置最大差为0m。

## 验证
- 显式ignored导出测试通过，常规完整palette/实际腕回归通过。
- make check通过：格式、原生全目标Clippy、Rust/Node测试与WASM check。
- WASM Clippy通过。
- 第二次导出逐字节相同；Python独立解析JSON，关节集与reference-rig.json一致，矩阵全有限。
- rest文本与参考JSON最大分量差5.93e-8（float32最短文本转换）；运行时Transform分解原始指节rest的最大分量差约1.1e-6，回归用2e-6阈值并保留原矩阵，未以inverse(IBM)替换。
- git diff --check通过。

## 限制
此交付只是真实已有目标palette姿态，未包含新灰模蒙皮、披风辅助骨、物理显示根位移或画面验收。灰模按 `P=B*A*inverse(R)*inverse(B)*M` 从游戏当前矩阵映射到Blender骨参考。不得将原目标矩阵直接复制为Blender局部四元数，也不得将本数值通过描述为完整新角色游戏兼容已通过。
