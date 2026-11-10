# 近战模型接入验证

实施基线为 main 的 `f1da207`，用户明确要求本轮直接在 main 工作，不创建临时分支。
基线 Cargo.lock、旧五组运行资源树与两组 prepared 清单 SHA 保存在 baseline.json；验收时检查原件／旧输出未改动。

## 范围与检查计划

转换／实际源 LBS／静态选型由导出者验证，运行时通过真实 World 检查握持、同资产飞行、取消回收、暂停与 F3 隔离；独立复核见 independent-review.md。

| 运行资源 | 大小 | 顶点／三角形／部件 | SHA256 |
|---|---:|---|---|
| 伏浪锚 | 2,360,330 字节 | 5437／7838／2 | 104f01a3ced11c443ba2ea8f650d38fb9a92729bcea11c5e08ad7987f2f722c0 |
| 谪星摧山 | 1,321,418 字节 | 17504／22620／5 | 8828682dd2a818c37043d51b3ba166974acb7fd2e502f27e0e55e2b4d40c7dc1 |

两件采用 KIVAWP01、握柄0、长轴 +Y、总长 1.10m；源真实 LBS 与 f32 转换误差分别约 3.023e-8m／2.981e-8m。
伏浪锚保留主体5骨槽及附属448顶点的 implicit weight1；谪星保留主杆真实两骨 current-LBS 和四份互补原几何，排除两份 FX。
谪星逐部件组装为明确的 Kiva 适配，HeadPosition 仅作参考设计锚点，不声称源 Actor 矩阵或未知 Float lookup 已恢复。
导出报告检查实际 f32 部件在2mm容差下的连接图；主体／刃／背／头／柄在同一连通分量。
主会话查看 `/tmp/kiva-melee-static-assembly.png`，确认静态轮廓连贯与握柄位置；这只是原几何的 CPU 分区色投影，不是原贴图／着色器或真实 GPU 画面验收。

## 完整质量门槛

独立检查者执行并通过：

- `make check`：fmt、原生 all-target Clippy，211 个 Rust 单元＋9 个集成测试，65 个 Node 测试，WASM check。
- `cargo clippy --locked --target wasm32-unknown-unknown -- -D warnings`。
- 新近战导出器 `--verify`：共8文件逐字节一致，每件7种损坏输入被拒绝。
- 两组 prepared 原件 `--verify`：分别24／122文件，源 GLB／skin／PNG 校验通过。
- 离线 Trunk 构建到临时目录 `/tmp/kiva-melee-review-dist.NSta78`，供最终浏览器流程验证。
- 任务 JSON／JSONL／引用、文档链接和 `git diff --check`。

旧三组导出器 `--verify` 也全部通过，12文件逐字节一致；旧五组运行资源、prepared、旧转换器、simulation／gameplay、相机和 Cargo／lock 均未改动。
实施验收基线 branch／HEAD 为 main／f1da207，当时尚未提交／推送。

## 最终打包产物浏览器流程

Chromium／SwiftShader fixture 运行 exit0，page／console errors为空，failure为空。真实点击进入与中键锁定后，由输入桥 fixture 执行动作并读取最终 WASM／HUD；不称为全程真实键盘手动验收。
默认李火旺与伏浪锚／引冰落；Q 单枪、锤 E＋Tab 三枪、切谪星摧山／羿之影后的 Q＋Tab 三箭，目标生命依次1,000,000→999,960→999,856→999,767；两条联动结束恢复可用。
暂停通过外观按钮换为愁铃客后，谪星／羿之影和目标生命保持；暂停切实战清理旧目标。
完整十张截图、report.json、run.log、qa.cjs及 QA.md 保留在 `/tmp/kiva-melee-models-browser/`。
临时 fixture／浏览器由finally关闭，Trunk临时目录随后删除，用户常驻开发服务未改动。

## 画面条件

前轮 Chromium／SwiftShader 在重构前基线和当前代码均只有背景与 HUD。本轮查看07链斧联动截图仍相同，皮肤HUD正常，但3D可见性未通过验收。
真实代码／数值／流程检查已完成，CPU投影也检查了几何完整性；真实 GPU 下的外观／握持构图／穿插仍待复核，任务保留跟踪，不提前归档。
实现交付时改动直接保留在 main 工作区；后续用户明确要求“提交&推送”。

## Git 交付

按用户授权，代码直接在 main 提交为 `791c66d`（feat: integrate Fulangmao and Zhexing Cuishan models），并推送 origin/main。
提交前 fetch 确认 main 与远端一致，没有额外合并或临时分支。已通过的运行代码／资产未再修改，真实 GPU 画面验收继续跟踪，任务不提前归档。
