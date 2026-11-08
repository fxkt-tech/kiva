# 英雄联盟美术资源接入执行

- [x] 需求与变更边界收敛，用户授权连续方案与实现。
- [x] 阅读关联规范与 research/，核对现有使用者。
- [x] 按 design.md 实现，公共接线由主代理协调。
- [x] 资源/蒙皮拾取/死亡与攻击采样行为回归、格式及17文件哈希校验；主代理共享 make check 已通过340 Rust场景与2个既有忽略场景。
- [x] 记录来源/验证/清理/限制并交付集成。
- [x] 最终源快照（含比例/UV/暂停Root修正）重建后，主代理完成三种窗口真实WebGPU模型/地形、尸体/复活、拾取和既有交互验收。

依赖：生命周期只读 getter 由 hero-lifecycle 提供；音频和资源可并行准备，最终用真实尸体/重生联调。最终质量门见父任务 implement.md。

## 交付快照

最终运行/资源交付文件列表：`/tmp/kiva-league-assets-final-files.json`（28文件，含独立审查新增回归文件）；按排序key紧凑JSON的SHA-256：
`6463bdc65192b4234dfd6acf50ec1d9da8e90369d489ffe9dd6bdc96c44a27a2`。
主代理接收后运行码冻结，仅具体浏览器/独立审查失败允许修正；规范和记录可以继续补充。

资源产物与源哈希在 `assets/league-models.json`，重建/来源说明在 `assets/models/README.md`。
新增已实现规范 `.trellis/spec/client/imported-models.md`；索引及总架构由主代理统一接入。

首轮真实 1280×800 WebGPU 已展示导入英雄/小兵、HUD 图标与尸体；截图暴露原草地/石路非无缝图集重复产生硬接缝。
已按实际图像修正为相邻格UV交替镜像、扩大草地采样范围并提高世界纹理密度，增加共享边UV相等回归；最终重建截图已实际查看通过。
独立审查已修复跨一整个高速普攻后旧施法记忆、暂停R时源Root曲线重复叠高两个表现边界。
CommunityDragon地表源URL已核实固定16.22与原latest字节相同，仅更新来源记录，不改资源产物。

主代理最终make check通过343 Rust场景（2既有soak忽略），WASM Clippy/40 Node和最终Trunk构建通过。
有窗口浏览器完成1280×800、800×600、DPR2的实际模型/图标/头像/模型选中/重选，以及真实尸体/暂停倒计时/复活；独立picking、skills、reselection、startup、pointer、smoke和原生帧钟healthbars均通过。证据见父任务 [verification.md](../11-08-battle-presentation/verification.md)。
