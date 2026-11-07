# 验收记录

## 来源与边界

- 参考 act 提交：`bd3e3443cfb2922c5cbf6a682cb75bb1091fea12`。
- `stickman.anim` 与参考逐字节一致：578598 字节，SHA-256 前缀 `b2116f1ee9dc527d`。
- `LICENSE.txt` 与参考逐字节一致：332 字节，SHA-256 前缀 `6d01f55c6e4c49a2`。
- Python 导入脚本语法已通过 `compile` 检查，无缓存产物。
- 用户自行管理开发服务，本任务只运行自有临时构建和测试 fixture。

## 外观基线

已使用旧构建 `/tmp/kiva-dash-site`，通过真实输入让玩家面向镜头，锁定相机并放大到现有限制。
临时有窗口 Chromium/WebGPU 截图成功，无 pageerror；自有 Xvfb、浏览器和 HTTP fixture 均已关闭。

- `/tmp/kiva-browser-tests/artifacts/before-skeletal-scene.png`
- `/tmp/kiva-browser-tests/artifacts/before-skeletal-character.png`

该基线截图确认旧外观为胶囊身体、球头和固定侧边武器。

## 当前实现验收

- 最终 `make check`：173项Rust测试通过（56客户端、6角色、13装备、2整局重置、59模拟、37状态）。包含12项骨骼专项测试。
- 最终WASM Clippy `-D warnings`通过。
- 日志：`/tmp/kiva-skeletal-check.log`、`/tmp/kiva-skeletal-wasm-clippy.log`。
- 最终 `trunk build --locked --dist /tmp/kiva-skeletal-site`通过，构建Web静态资源与当前源码逐字节一致。
- 生产输入哈希记录：`/tmp/kiva-skeletal-source.json`；最终构建后生产源未变化。
- 有窗口Chromium/WebGPU smoke全部通过：实际几何、移动、技能、前后摇、取消、UI隔离、镜头、暂停恢复及高DPI拾取。
- 最终smoke截图：`/tmp/kiva-browser-tests/artifacts/skeletal-final/proving-grounds.png`，已人工检查玩家/NPC分段四肢、头眼、手剑及血条；放大对比截图为`/tmp/kiva-browser-tests/artifacts/skeletal-character.png`。
- 有窗口技能回归三尺寸通过：1280×800 DPR1、800×600 DPR1、1000×720 DPR2，包含Q/W/E/R、无效瞄准、组合控制、减速恢复、击飞拾取/暂停与超距冲刺。
- 血条按脚本默认无头模式通过：移动单位91/91帧更新、镜头移动48/48帧更新，maxStale均0，完整HUD分别18/10次发布，保持80ms节流。
- 血条有窗口SwiftShader首次测量未达到20帧最小样本（新构建18帧/10266.3ms、旧构建同环境19帧/10232.9ms），两者实际更新率均100%、maxStale为0；这两次只记录为采样不足，不计通过。无头复验没有修改脚本、时钟或断言，不用于声称有窗口墙钟性能。
- 整场重开回归三尺寸通过：1280×800 DPR1、800×600 DPR1、1000×720 DPR2，覆盖更换/相同角色、NPC/效果/时间重置、旧请求隔离与普通继续；模型层级无残留。

## 独立复核

全范围代码/规范独立复核发现并修复两处普攻采样接缝：前摇末段从LOADED连续挥到CONTACT；后摇首段采样到Attack实际末帧后接Recover。新增测试同时验证全部模型骨骼和实际剑socket在接缝两侧的位置/旋转连续性。
复核确认模型最大头顶约1.94，现有2.2标签锚点和0..2拾取高度足够，无需改HUD协议。源代码和规范最终复核无未解决问题。

## 文档与清单

本任务JSON/JSONL有效；所有变更Markdown本地链接和源代码/文档空白检查通过。
暂存检查中，第三方LICENSE.txt保留act原件的CRLF、原有行尾空格及空行，因此不改写该文件；其逐字节来源校验通过，git diff --cached --check对该许可之外的全部改动通过。
架构文档已超32KiB自动注入单文件阈值，复核者已显式读取实际完整文档；没有调整全局注入配置。

## 最终状态

所有验收项完成，生产源码保持与最终Web构建一致。两处动作接缝已修复，独立代码/规范复核无未解决问题。验收完成后用户已明确授权提交/推送；保存实现后归档任务并记录会话。
