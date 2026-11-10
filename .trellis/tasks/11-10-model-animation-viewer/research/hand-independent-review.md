# 手部重定向修复独立审查

日期：2026-11-11。审查基线为`2f49537`，范围是用户确认默认李火旺手部异常后明确授权的手臂／手指／物理握点修复（V12／AC09）。未把此前查看页一致性测试当作解剖正确性证明。生产修复由hand_repro实施；本审查仅新增一条独立reference握轴回归，未改运行时实现。

## Findings (fixed)

### 1. 不同参考臂向被错误保留到实际目标姿态

- 位置：`src/client/avatar.rs`的`driver_pose`、`source_driver_pose`、`calibrate_arm_directions`。
- 问题：源rest相对旋转直接作用于目标自己的参考段向量，保留约49°／57°的A/T参考方向差；骨长不变和游戏／查看一致不能检出该错误。
- 修复：在原旋转上作最小摆动，使目标自身臂段朝向实际采样解剖方向，再按自身长度FK。CC0／燕云结构输入共用校准，显式妹妹握点IK仍在后。
- 复核：新增回归读取实际palette关节／锚点，覆盖两Naraka身体、双臂、默认持握、原CC0行走和真实燕云样本；分别断言方向与目标自身段长。不是只比较两次调用同一helper的结果。

### 2. 有真实源指轨，但目标指骨只跟腕刚性搬动

- 位置：`src/client/avatar/hand.rs`与`avatar.rs::joint_pose`。
- 问题：真实源手指可转约91.6°，目标对应骨相对手腕为0°。40源节点中30个带权重、10个为无权重Nub；目标是30个带权重的三节指链。
- 修复：按明确名称匹配前三节，Nub仅提供源末端方向。源／目标掌部与原骨轴分别校准，传递当前源手形；局部旋转先按源权重混合，再从当前实际腕骨按目标自身局部平移／尺度FK。部分skin缺少未加权祖先时，只合并一致的同名rest元数据，不串用外观或palette索引。
- 复核：全部实际目标指骨均有绑定（含李火旺不完整左拇指palette）；两身体、双手、五指、三时刻的实际child／Nub方向在各自当前掌基准一致，末节用已核实的native+X。权重0／0.25／0.5／0.75／1保持自身段长，0权重等于fallback；真实四权重LBS顶点相对腕发生变化。原参考关节保持原矩阵，愁铃客全源同名palette路径在1e-5矩阵容差内保持原样。

### 3. 旧语义腕轴与真实手掌不一致

- 位置：`hand.rs::GripReference`、`current_grip`、`avatar.rs::apply_hand_frames`。
- 问题：默认李火旺腕→中指根与旧握点偏移方向有约63°偏差；保持原右腕四元数等式会继续自证错误。愁铃客中间源权重下，原全局关节位置混合也不严格等于单个腕骨的刚性变形。
- 修复：明确语义+Y沿掌长、+Z沿小指→食指的有向杆轴；默认皮肤变形接收已有CC0声明握持frame，源层接收当前源掌frame。最终挂接读取实际palette腕／MCP位置，保留0.055m偏移和正交单位尺度。不能仅翻最终武器或增加查看页补偿。
- 复核：实际掌长与有向指侧分别对齐attachment的Y／Z，不能使用绝对点积掩盖倒握。三角色、双手和五档源权重通过；近战偏移仍为0.055m，scale为ONE。正向刀头朝食指／拇指侧是本次明确的Kiva握持约定，未冒称恢复未知原作反握装配。

### 4. Reference提前返回分支缺少几何握轴回归

- 位置：`src/client/avatar/hand_tests.rs::asset_reference_grips_use_actual_rest_palms_after_source_sampling`。
- 问题：原新增reference测试只验证palette未改，未锁定提前返回分支仍把真实参考掌轴交给武器。
- 本审查补充：先建立实际源姿态再启用reference，读取三角色真实参考palette腕／MCP，独立断言握点原点、掌长／有向杆轴、单位尺度及0.055m偏移。定向与完整质量门槛通过。该文件仅在cfg(test)编译，未改变最终WASM运行代码。

## Findings (not fixed)

未发现尚未处理的本轮范围内代码错误。以下边界保留并已与主会话确认：

- 未重写原愁铃客全身的全局Transform混合；中间权重的当前掌几何由实际palette读取，避免把它当作严格刚性腕变形。完整原生源骨数据及原件保持。
- 未猜测新增独立twist曲线：当前35片段没有相关8个Upper／ForeArm Twist节点的直接轨；目标分段骨仍是已知结构驱动适配，不宣称完整原作控制器恢复。
- 硬件GPU画面、最终固定Trunk／浏览器及手部近景由主会话另行验收。本报告不以数值或软件GPU条件宣称原模型全部美术细节正确。

## Verification

| 检查 | 结果 |
|---|---|
| `make check`最终完整执行 | PASS |
| 格式／原生all-target Clippy `-D warnings` | PASS |
| Rust单元 | PASS，251项，0ignored |
| Rust集成 | PASS，9项 |
| Node真实桥脚本 | PASS，94项，0failed／0skipped |
| WASM `cargo check --locked` | PASS |
| WASM Clippy `-D warnings` | PASS；运行时代码随后未变，仅追加cfg(test)回归 |
| `git diff --check` | PASS |
| 资产、Cargo.toml／Cargo.lock、simulation／gameplay、既有导入器对基线 | 差异为空 |

最终合计260项Rust＋94项Node。日志为`/tmp/kiva-hand-check.log`、`/tmp/kiva-hand-wasm-clippy.log`及定向reference日志`/tmp/kiva-hand-reference-check.log`。临时ignored导出测试、HAND-DIAG／println／dbg均已清理；调查数据保存在明确标记的任务research目录。没有启动／关闭用户服务、提交、推送或归档任务。
