# Independent review: 模型、武器与动作验收页

审查日期：2026-11-11。当前任务为 `in_progress`；审查时用户已批准实施，尚未授权提交或推送；后续交付状态以task.json和会话记录为准。审查覆盖本任务完整客户端／Web差异、PRD／设计／实施清单、实际共享计算与新规范，未修改资产或模拟规则。

## Findings (fixed)

### 1. 子部件显式 Visible 绕过源模式隐藏

- 位置：`src/client/viewer/mod.rs:360`、`:436`。
- 问题：切换配置恢复部件或重新显示部件时使用 `Visibility::Visible`。Bevy 的 Visible 会绕过 Hidden 祖先，因此把手持根隐藏并不能可靠隐藏子Mesh；源片段对照可能继续显示武器，联合bounds也可能包含它。
- 修复：显示子部件改用 `Visibility::Inherited`，隐藏仍用Hidden。手持／脊柱根继续由当前模式和装备控制，未增加查看页独立姿态修正。
- 回归：`src/client/viewer/tests.rs:193` 使用真正 `VisibilityPlugin` 传播，覆盖源模式中换远程武器，以及部件隐藏后重新显示；断言实际子Mesh的InheritedVisibility为隐藏，同时角色主体仍可见。原配置重选／旧部件ID回归同步保留。

### 2. 完整 Clippy 门槛发现局部嵌套条件

- 位置：`src/client/viewer/mod.rs:540` 的bounds更新。
- 问题：本机Clippy以 `-D warnings` 拒绝可合并的嵌套if。
- 修复：合并为带let条件的单个if；未改变更新条件、取景或缓存行为。

### 3. 检查标记缺少颜色说明

- 问题：紫色手腕、青色挂点／发射源、黄色武器握点及RGB轴的意义未在页面说明。
- 处理：已向主会话报告，主会话批准按既有“分别标明关节与挂点”的需求增加简短图例，交原Web实现者完成。该UI补充由主会话浏览器验收，不改变数值计算。

## Findings (not fixed)

未发现尚未修复的代码／范围／接口问题。真实GPU画面和浏览器固定产物检查不由本报告宣告通过：当前已探测浏览器只有SwiftShader回退适配器，主会话继续执行Trunk与浏览器验证，真实GPU模型／武器／动作验收须单列结果。

## Review evidence

- `animation.rs:133` 的单帧入口被真实game animate和PreviewRig共用；动作通知／事件去重顺序和原裁剪、混合、IK计算保留。重置／倒拖／前向增量、54个真实片段及末帧检查均有回归。
- `viewer/tests.rs:281` 通过实际游戏适配器与预览检查三种外观、两组近战的真实palette和武器变换。`avatar.rs:1229` 检查源权重0/0.5/1的左右手／脊柱与皮肤一致，保留原右手数学结果。
- `companion.rs:38` 的局部发射源／朝向／握点helper由游戏和验收共同使用；检查页只演示无射击标记的持握／瞄准。
- 独立App无SimulationPlugin、Body或ActorView；游戏专属Web桥在注册监听器前退出，Rust持有唯一时钟，页面按确认快照显示。准备确认绑定当前主体、真实渲染提交及revision，故障保留重载恢复和原RenderErrorPolicy。
- `viewer/mod.rs:501` 联合当前动态skin和刚性附件bounds，排除地板；`:540` 位于传播／bounds后、视锥更新前，暂停相机操作不重算皮肤与bounds。换装／部件生命周期和资产复用有真实World回归。
- 新 `model-viewer.md` 规范和client index／startup／architecture／用户说明与实际签名、运行边界一致；未来设计未冒充已实现功能。

## Verification

| 检查 | 结果 |
|---|---|
| `cargo fmt --all -- --check` | PASS（make check内） |
| 原生all-target Clippy `-D warnings` | PASS（make check内） |
| Rust单元测试 | PASS，241项 |
| Rust集成测试 | PASS，9项 |
| Node实际脚本测试 | PASS，94项 |
| WASM `cargo check --locked` | PASS（make check内） |
| WASM Clippy `-D warnings` | PASS |
| `git diff --check` | PASS |
| task validate | PASS，implement/check清单各12项 |
| 资产／Cargo.toml／Cargo.lock／simulation／gameplay／既有导入器基线 | 差异为空 |
| Trunk／固定浏览器产物／真实GPU | 主会话负责，未在本报告宣称通过 |

完整执行日志：`/tmp/kiva-viewer-check.log`、`/tmp/kiva-viewer-wasm-clippy.log`。本审查没有启动或关闭用户服务，没有提交、推送或归档任务。
