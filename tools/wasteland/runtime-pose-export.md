# 真实显示驱动姿态导出

此工具仅存在于 Rust 测试构建中，用于 Blender 灰模变形检查；不会改变游戏行为、默认外观或命中轨迹。

在仓库根目录执行：

```sh
KIVA_POSE_EXPORT=art/wasteland/greybox/runtime-poses.json \
  cargo test --locked --lib \
  client::avatar::wasteland_pose_tests::export_runtime_poses \
  -- --ignored --exact --nocapture
```

普通 `cargo test` 不写文件。单独运行数值回归：

```sh
cargo test --locked --lib \
  client::avatar::wasteland_pose_tests::runtime_snapshots_use_complete_palettes_and_real_wrist_grips \
  -- --exact
```

## 数据来源

- 身体参考为当前随包李火旺的 outfit palette：与 `extract_reference.py` 一样，按部件/骨槽顺序取每个实际 `gMan` 关节的第一个 slot，并核对重复名称参考一致。
- 待机、走跑、跳跃、四向闪避、蹲伏、防御与受击通过现有 `PreviewRig`，与游戏验收页共享 `RigPose` 状态驱动。
- 两组武器的三段普攻和近战技能通过现有 `AttackReplay`，与离线命中烘焙共享 `AttackClock` 采样。每招从 40 帧待机开始，按 60Hz 逐帧推进；导出若干时间点，第一段另保留每三帧的短序列。不是连续战斗录制或完整连段回放。
- 每个姿态实际运行 `avatar::sync`，读取 `PaletteJoint` 实体的 `Transform`。这些实体直接挂在角色显示根下，因此是角色模型空间矩阵，无需读取可能尚未传播的 `GlobalTransform`。

## JSON 合同

`schema = kiva-runtime-palette-poses-v1`，矩阵全部为 **4×4 行主序嵌套数组**，使用游戏模型空间、米、+Y 向上、−Z 朝前。

- `rest`：52 个原始 `sourceRestModel`，不是 inverse bind，也不以 `inverse(IBM)` 替代。名称为无前缀 `gMan ...`；不含 Blender 合成参考根和披风辅助骨。
- `poses[]`：`name`、`pair`、可空的 `track`、场景/攻击 `time` 与 `duration`，实际 `source_clip` 与 `source_time`。
- `poses[].joints`：同一组 52 个关节的**当前绝对模型空间矩阵，未乘 IBM**。
- `poses[].hand_grips.R/L`：由实际腕和 MCP 掌框计算的语义握持坐标，原点经断言与对应实际腕位置一致。
- `poses[].weapon_mount`：共享 `attachment_transform` 的近战武器局部模型矩阵，包含已存在的挂接适配；不是原生腕轴。

例子：`HammerMusket:game:run:0.50`、`HammerMusket:game:jump:0.20`、`ChainAxeBow:PlayerMelee { pair: ChainAxeBow, stage: Two }:tick012`。名称供审阅选择，业务信息应读取独立字段。

## Blender 映射

设游戏参考矩阵为 `R`、当前姿态为 `A`、游戏到 Blender 的基变换为 `B`、Blender 骨自身静止矩阵为 `M`，则所需 Blender 姿态绝对矩阵为：

```text
P = B * A * inverse(R) * inverse(B) * M
```

这与参考骨架保存的 `B * R * inverse(B) = M * correction` 相符。不能直接把游戏四元数赋给 Blender 姿态骨，不能将 `A` 当成蒙皮矩阵，也不能复制旧模型的 IBM 给新网格。

## 验证与限制

导出过程中检查矩阵有限/可逆、52 关节完整、实际同步结果与共享关节求值一致、握持原点来自真实腕。参考姿态经运行时 `Transform` 分解后，原始 float32 指节参考的微小非正交性产生约 `1.1e-6` 最大分量差；回归阈值为 `2e-6`，原始 `rest` 仍单独保留。

本阶段输出 140 个既有目标皮肤姿态。它证明共享显示驱动的数据来源，不证明新灰模的权重、穿模、披风运动或完整游戏画面已经验收。跳跃的物理显示根高度不在这里输出，不可再次叠加源根运动。
