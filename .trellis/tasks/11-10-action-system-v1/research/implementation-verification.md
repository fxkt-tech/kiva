# 第一版实施与验证记录

日期：2026-11-10。实施分支：feat/action-system-v1。代码提交：79b3538（feat: implement action system v1）。

## 已实现范围

- 哥哥常驻近战操控；逻辑妹妹拥有独立演出身份、RigPose 和临时蓝色人形；锤／鸟铳／链斧／弓为独立程序几何。
- 左键近战、E／Q 分离、中键显式锁定、1／2 绑定组；删除旧近远人格切换、火炮／旋刀／Leap 及其专用接口。
- 锤鸟击飞与三枪、链斧弓减速／投斧／回收／拉近三箭；每发独立固定步标记、原目标与来源验证。
- 有序右键拆招／防御／弹反、共同来袭结算、耐力消耗与安静恢复，无能量／耐久／破防。
- 新 HUD、实际演出者动画与音频、两个联动镜头、F2／F3／暂停／死亡清理。

## 完整自动检查

最终 Trellis check 代理 full-scope 审查无未解决代码发现。

| 检查 | 结果 |
|---|---|
| cargo fmt --all -- --check | PASS |
| cargo clippy --locked --all-targets -- -D warnings | PASS |
| cargo test --locked | PASS：201 单元测试＋9 真实地形集成测试 |
| node --test scripts/*-unit.test.cjs | PASS：64 测试 |
| cargo check --locked --target wasm32-unknown-unknown | PASS |
| cargo clippy --locked --target wasm32-unknown-unknown -- -D warnings | PASS |
| git diff --check、任务 JSON／JSONL、文档链接 | PASS |

全量日志：/tmp/kiva-action-v1-make-check.log。定向回归涵盖独立发射时间点、实际自然落地后继续三枪、来源僵直、Boss 控制效果清理、缩放敌人／高台边界范围、拉近制动／最小间隔／不推走目标、防御恢复与输入释放事实。

## 审查中修正的实际问题

- 动作／命中标记后可用性快照需要在同一步更新，避免 HUD 短暂仍显示可再次起手。
- 单步跨过结束时间仍需按顺序结算跨过的标记，不能提前回收动作。
- 链斧 Q 的发射与 Shot／声音在 0.15s，伤害／僵直在 0.30s，不能把发射挪到命中时。
- 保存事件的原始 pair，避免切组后重放历史声音时选错武器。
- 暂停后 F2 清理当前动作时也清理其后坐，避免相机永久偏移。
- 拉近按真实 motor 制动预算限速，真实物理测试确认接近后不推走原目标。
- 起手与命中统一使用双方脚底三维距离，枪口／胸口仅参与真实视线和弹道；移除额外范围补丁。
- 所有方向键／Shift 松开又按下，即使最终 held 相同也锁存 release 位，防止旧疾跑与长按错误延续。

## 打包

两次使用 CARGO_NET_OFFLINE=true trunk build --dist <自建临时目录> 成功；本次本机 Trunk 0.22.1。
最终精确源码包：/tmp/kiva-action-v1-final.xnP3a9，包含最后输入 release 位与检查修正。
Cargo.lock 与 HEAD 完全一致，SHA256 f582ba6e933a38619a2f5d29eb5410492b24e4ce7d674d753026b1acdc0d5955。
临时构建目录在浏览器使用结束后回收；不修改用户 dist 或开发服务。

## 浏览器观察（记录与验收分开）

使用已有 Playwright／Chromium、一次性本地 HTTP 测试夹具运行真实 WASM，没有启动／关闭用户3221开发服务。
已确认加载、可信进入／鼠标捕获、中键锁定、1000生命／耐力／E-Q HUD，以及实际 Q40、锤 E20、三枪28×3和对应音效次数。
最终精确源码包复核了 Q40／锤 E20／三枪28×3／链斧 Q35／三箭18×3、各次真实声音、可信中键锁定、F3 保留武器组、暂停 F2 清目标且保留生命／耐力。
软件 WebGPU 下整个三维画面只呈清屏色；320×240 物理分辨率和现有 DISPLAY 的 headed 尝试均未确认模型可见，不能把“模型／武器／镜头可见”和实际手感记为验收通过。
已用 git archive 隔离构建重构前 HEAD e7fed32 对照：旧版生命100／自动锁定正常，同配置进入画面仍同样只呈背景。旧版与最终版本中心区域像素逐字节相同（主色 RGB17,26,33；占比0.86646），两者均有真实 GPU render pass／draw 提交且无页面错误。因此此可见性限制可在重构前复现，不能算本次新机制的画面通过或新引入回归。
浏览器证据：/tmp/kiva-action-v1-browser（含 pixel-comparison.json）；所有测试浏览器与 HTTP 夹具均已关闭，临时构建／基线副本随后回收。

## 授权与交付

用户随后明确授权提交、合并到 main、删除本次临时分支并推送；保留用户源需求文档的术语修改及重构前 HTML 图谱。
第一版正式美术后置，图谱保留重构前快照标识。代码交付按本轮授权进行；任务暂留真实 GPU 画面复核跟踪，不把尚未观察到的画面验收标记为通过。
