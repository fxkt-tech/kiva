# act 身体动画移植

## 目标

把 `origin/act`（`bd3e344`）的真实火柴人动画资产、骨骼采样和姿态过渡接入当前战斗 Demo，
替换静止的几何肢体；保留当前移动、碰撞、Shift、武器、伤害、联动和镜头规则。
继续不创建 Trellis 任务，不使用浏览器测试，不启动或关闭开发服务。

## 实现顺序与边界

1. 原样复制 578,598 字节的 `stickman.anim`、CC0 许可证及可复现导入脚本；移植 `AnimationLibrary`，保持原 23 骨骼 / 19 动画采样与朝向基变换。
2. 将火柴人杆件、关节、头部和眼睛绑定到骨骼；武器根绑定右手，不再独立固定在人物根旁边。
3. 用当前只读模拟快照和有明确 actor 的战斗反馈选择姿态，迁移 act 的循环、跳跃阶段、翻滚与平滑混合。
4. 在真实无窗口 ECS 调度下验证各动作改变骨骼、武器跟手、暂停不动、复活恢复及模拟位置 / 相机不受动画影响。

依赖保持 `client → gameplay → simulation`。`skeleton` 只解码采样，`animation` 选择并混合姿态，
`stickman` 把姿态变成几何 Transform。动作权威仍在模拟；根位移由资产导入阶段剥离，不能把动画位移叠加到 Body。

## 当前状态映射

| 状态 | 来源 / 映射 |
|---|---|
| 待机 / 行走 / 疾跑 | Sword_Idle / Walk_Loop / Jog_Fwd_Loop；步频匹配实际固定步位移，撞墙不原地奔跑 |
| 起跳 / 滞空 / 落地 | Jump_Start → Jump_Loop → Jump_Land；身体高度仅由 MotionState 负责 |
| 闪避 | Roll；进度和方向来自真实闪避状态 |
| 格挡 | Sword_Block；移动 / 空中下半身与上半身防御姿态组合 |
| 普攻 / 锤击 / 斧旋 / 射击 | act 中已有身体动作与手部姿态按当前武器和已提交反馈适配，不导入旧剑类战斗规则 |
| 受击 / 倒地 / 恢复 | Hit_Chest / Death01 / 清理姿态状态后恢复；尸体在复活前可见，死亡实体仍不可攻击 |

只有当前存在的游戏行为接入动画；不借此引入 act 的旧输入、蹲伏、滑铲、双跳、剑类连招或耐力系统。

## 接口约定

- `AnimationLibrary::{bone,duration,sample,model_pose}` / `blend` 保持原实现。
- `RigPose::new(&AnimationLibrary)`，`model` 是只读模型空间姿态快照。
- `animation::animate` 只写 RigPose；`stickman::sync_attachments` 只写肢体 / 武器 Transform，不写人物根。
- `Body::step_displacement()` 返回实际碰撞后固定步位移；MotionState 提供 `dodge_elapsed()` / `dodge_direction()` 只读接口。
- `CombatEvent.actor: Entity`：攻击为发起者，受击 / 防御 / 倒地 / 恢复为接受者；不按距离或中文状态字符串猜测归属。
- 更新顺序为人物与敌人显示根 → 武器可见性 → 姿态采样 → 骨骼附件 → HUD / 相机；SimulationActive=false 时动画时钟与游标冻结。

## 验证

执行 `make check`、WASM Clippy 和临时目录 Trunk 构建。验证资产层级、全部动画帧有限且无根位移、
真实走跑 / 跳跃 / 闪避 / 格挡 / 受击 / 倒地姿态变化、武器手部绑定、暂停与复活，以及不会修改权威位置。
不使用浏览器，画面与手感不宣称完成浏览器验收。

## 完成记录

- 原版二进制和许可证已原样迁入，导入脚本保留，动画资产共 23 骨骼 / 19 段动画。
- 已接入当前全部对应姿态、平滑过渡、实际位移驱动的步态、手腕武器挂点、倒地可见与复活恢复。
- `make check` 通过：67 项 Rust、24 项 Node、格式 / 原生 Clippy / WASM 检查。
- WASM Clippy 与临时 Trunk 构建通过，验证构建后的 WASM 包含动画数据，Cargo.lock 未变化；临时产物已清理。
- 代码审查未发现阻塞问题；未使用浏览器、未启动或关闭服务、未提交代码。
