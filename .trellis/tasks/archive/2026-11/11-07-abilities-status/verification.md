# 集成验收记录

方案、实现与验收完成。基线`d078410`；工作提交`ef5fac6`，任务已按Trellis流程归档。推送已获用户授权。
开发服务仍由用户管理。构建使用`/tmp/kiva-status-site`，浏览器均自建并关闭临时HTTP fixture。

## 验收归属

| 验收 | 已通过证据 |
|---|---|
| AC1 / AC3 / AC17 | tests/status_effects.rs：投递复用、多效果顺序/致死/零伤害、整组快照原子提交、注册矩阵与数值边界 |
| AC2 | controls测试空地/友方保留瞄准；模拟端友方/非法目标拒绝；browser-skills真实鼠标完整路径 |
| AC4 / AC6 / AC11–AC13 | 状态测试精确1tick/100ms、权限叠加/刷新、中断/Stop、持续意图恢复与不重放 |
| AC5 | 状态测试0.84乘算/最强接替/零倍/饱和/非法系数；浏览器状态与3.85最终移速同快照 |
| AC7 / AC8 | 位移墙/边界/极低速/零行程/替换/拉拽/方向/Stop/混合控制测试 |
| AC9 / AC10 | 死亡清理、Explicit保留、重生生命代次、旧施法/跟踪目标拒绝、来源移除快照存续测试 |
| AC14 | 原引导分数脉冲/快照、控制/运动暂停；scene实际击飞及后摇姿态暂停；浏览器正高度冻结 |
| AC15 | 可见GlobalTransform拾取/根与底座测试；三分辨率击飞实际点选与投影；逐帧血条检查 |
| AC16 | 装配元数据/空槽/普通与跟踪弹丸描述；HUD状态/动态技能；144px无单位七属性；捕获和UI隔离 |
| AC18 | 原65项保留，新增44项；完整质量入口和浏览器回归全部通过 |

## 完整质量入口

- `make check`：通过格式检查、原生全target Clippy、109项Rust测试、WASM cargo check。
- 测试构成：16项unit（新增7）、56项旧模拟、37项状态（35项功能+2项独立复核边界）。
- `cargo clippy --locked --target wasm32-unknown-unknown -- -D warnings`：通过。
- 修改的JavaScript均通过`node --check`；任务JSONL、Markdown本地链接和`git diff --check`通过。
- 最终Rust/DOM版本`trunk build --dist /tmp/kiva-status-site`：通过。
- 独立Trellis全范围复核通过，无未解决的架构或规则问题。

## 浏览器结果

统一环境为Playwright Chromium软件WebGPU；截图/技能/布局/光标/smoke使用有窗口模式。
逐帧血条使用无头模式采样真实RAF与DOM位置。

| 入口 | 实际结果 |
|---|---|
| browser-skills | 最终版本三组通过：1280×800、800×600、1000×720 DPR2；Q/W/E/R、非法点选、权限/减速、击飞身体选择、正高度/状态/坐标暂停冻结和恢复 |
| browser-smoke | 真实WebGPU像素、移动/替换、Q/W、E/R、普攻后摇/停止/攻击移动、UI、镜头/缩放/重设尺寸、暂停/真实焦点丢失、DPR2全部通过 |
| browser-stats | 三组通过；七项数值/短标签、144px宽度、无单位、DOM边界、输入隔离和暂停恢复 |
| browser-cursor | 捕获/拒绝捕获、越界回拉、边缘滚屏恢复、Esc重入、技能点击、高DPI/重设尺寸通过 |
| browser-healthbars | 人物移动90/90帧更新，镜头移动48/48帧更新，maxStale=0；完整HUD保持约80ms限频 |

截图位于`/tmp/kiva-browser-tests/artifacts/`：proving-grounds、stats三组、airborne三组。
击飞截图仅临时隐藏暂停覆盖层以检查冻结场景，随后立即恢复；不修改模拟状态。
已目视检查实际场景、800×600击飞人物/地面底座/状态与紧凑HUD。

## 复核修正与限制

复核修正了无法按一步递减的f32技能时钟、无法表示位移的极小速度锁、空槽位图标缓存及状态徽章点击隔离。
时钟/速度新增公开行为回归。短击飞浏览器测试的CDP时序处理见[debug-retrospective.md](debug-retrospective.md)。

本期没有新增装备、角色选择、韧性/免控/净化或主动攻击玩家的AI；对应TODO继续未完成。
演示数值可以在内容层调整，技能/状态/位移规则中没有具体英雄或技能ID分支。
