# 指定模型接入验证

## 已验证的冻结源

2026-11-10，在仓库根目录运行以下离线检查，全部 exit 0／passed=true：

```sh
uv run --offline --with numpy --with pillow python scripts/import-bailingmiao.py --verify assets/prepared/naraka/bailingmiao
uv run --offline --with numpy --with pillow python scripts/prepare-naraka-weapons.py --verify assets/prepared/naraka/yinbingluo
uv run --offline --with numpy --with pillow python scripts/prepare-naraka-weapons.py --verify assets/prepared/naraka/yizhiying
```

源清单分别覆盖 76／20／33 个文件。白灵淼三个 GLB 共 43,801 顶点；引冰落两套真实蒙皮共 5,395 顶点；羿之影弓体为 6,218 顶点／9 个有效骨骼槽，另含原配置的 quiver 和 arrowbag。
这些结果只验证准备资源的完整性、真实原骨骼／LBS 与来源，不证明运行时挂接或画面已经正确。

## 实施与后续检查

当前分支：`feat/action-v1-models`。源资源保持冻结，转换输出另存三个运行时资源目录。
导出者完成真实源／量化 LBS／PNG／有界格式检查与临时目录重建，全部 12 个输出逐字节一致：

```sh
uv run --offline --with numpy --with pillow python scripts/import-prepared-models.py --verify
```

| 输出 | 字节 | 几何 | SHA256 |
|---|---:|---|---|
| 白灵淼 character.bin | 7,207,149 | 43,801 顶点／58,524 三角形／22 部件 | adaeb80895ade1234677fd3b024dba459621b395e8d478ff222b74f28402408b |
| 引冰落 weapon.bin | 1,577,483 | 5,395 顶点／7,106 三角形 | 27ce590db7dfd294656f6a1052410b24e07c75b52580ae8a3cda939f7307172b |
| 羿之影 weapon.bin | 2,026,175 | 10,228 顶点／11,674 三角形（含两个配置附件） | 124e496ff8ce56a88ff122c60cad94d063584958efc33b8d8b82c6944641ea9a |

白灵淼实际 f32／量化权重的最大顶点误差约 1.118mm；引冰落 joint1 的 135 个顶点和羿之影九个有效槽保留真实当前 LBS 后再烘焙。
损坏输入检查覆盖截断、尾随字节、错误 magic／过长头／非有限顶点／非法索引，以及女性 current matrix／joint／weight 错误。prepared 的所有原文件在生成前后哈希校验通过，无源目录修改。

## 运行时复核

独立复核记录见 [independent-review.md](independent-review.md)。复核发现并交由运行时修正：高角度枪柄超出女性臂长、世界空间最短弧造成的 yaw 相关武器翻转；修正采用女性自己的肢体 IK 和局部瞄准框架，不改变模拟源或动作时间线。
附加连续性检查发现锁骨在可达阈值可能突跳，现改为余弦定理求最小连续转角；腿部使用原膝盖弯曲平面，陡角度俯射允许在自身腿长范围内从放松站姿升起。
最终客户端聚焦检查 `cargo test --locked client:: -- --test-threads=2` 104／104 通过；对应 Node 外观／HUD 10／10 通过。新增真实 palette 握点与女性上／下肢长、脚掌／脚趾落点及旋转、±1rad 连续扫描、四个 yaw 的枪／弓上轴、两条真实联动每步三发、完整显示树回收和换装隔离检查。

## 完整质量门槛

独立检查者在最终代码上执行并通过：

- `make check`：格式、原生 all-target Clippy（warnings=deny）、206 个单元测试、9 个集成测试、65 个 Node 测试、WASM check。
- `cargo clippy --locked --target wasm32-unknown-unknown -- -D warnings`。
- 离线运行资源 `--verify` 与三组冻结原件 `--verify`；所有语义损坏输入拒绝检查通过。
- Trunk 0.22.1 离线构建到 `/tmp/kiva-model-gate-Mb9D8P`，供后续独立浏览器检查。
- 文档链接、任务 JSON／JSONL、`git diff --check` 通过。

`Cargo.lock` 未变，SHA256 为 `f582ba6e933a38619a2f5d29eb5410492b24e4ce7d674d753026b1acdc0d5955`。
simulation／gameplay 与 prepared 无修改；实际二进制字节数／SHA 与各 conversion-report 一致。
独立复核未留下代码问题。

## 打包产物浏览器流程

最终 Trunk 产物通过隔离 Chromium／SwiftShader 流程（exit 0）：ready 默认李火旺／引冰落、真实进入与中键锁定、单枪 Q、锤 E＋Tab 三枪、链斧弓 Q＋Tab 三箭、恢复、暂停换为愁铃客、暂停切模式。动作步骤在真实捕获后由输入桥 fixture 驱动，消费最终 WASM 与 HUD，不冒称全程真实键盘手动验收。
目标生命依次为 1,000,000 → 999,960 → 999,856 → 999,767，与 40、20＋3×28、35＋3×18 相符；远程武器文字从引冰落变为羿之影，暂停换身体不改变武器和目标血量。
无 page／console 错误、无测试失败。完整记录、十张截图与报告保留 `/tmp/kiva-models-browser/QA.md`／`report.json`。
临时 fixture 和浏览器由 finally 关闭，Trunk 临时目录检查后删除；用户常驻服务未改动。

## 画面验收边界

上一轮第一版使用本机 Chromium／SwiftShader 时，现有场景与重构前基线均只得到背景画面，具体证据见父任务的 implementation-verification.md 和 `/tmp/kiva-action-v1-browser/QA.md`。
本轮查看单枪出场截图仍只有背景和正常 HUD，实际 GPU 提交不作为模型已经可见的证据。
真实数据／World／浏览器流程已通过，真实 GPU 下的模型可见性、发片、握持构图及穿插仍待复核。因此当前任务继续保留验收记录，不提前归档或宣称视觉完成。

## 提交与合并交付

用户明确要求提交、合并到 main、删除临时分支并推送。
模型代码提交 `9073157`，合并提交 `8ee266e`；先同步并保留远端 `e702745` 的独立武器资源准备结果，再合入本次代码，无冲突。
合并后的 src／web／入口／Cargo／三组运行资产与已通过完整门槛的功能提交逐文件一致；离线运行导出 `--verify` 再次通过，全部 12 文件逐字节一致，冻结源文件校验通过。
本次临时分支 `feat/action-v1-models` 已删除，交付分支为 main；其它 worktree 和分支保留。真实 GPU 验收继续跟踪。
