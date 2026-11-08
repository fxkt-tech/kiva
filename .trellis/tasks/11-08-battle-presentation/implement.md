# 执行与验收

- [x] 阅读 TODO、架构、生命周期/表现/拾取/浏览器规范，调研并下载音频样本。
- [x] 创建父任务和三个独立交付子任务，记录用户自主实施授权与范围。
- [x] hero-lifecycle：真实死亡事实和只读倒计时、尸体表现/HUD、暂停/重生/重开回归。
- [x] combat-audio：确定源计划/原皮 cue、离线处理和来源清单、事件适配/播放/浏览器解锁及清理。
- [x] league-assets：验证模型/动画样例，再接图标、小兵、地图纹理及本地发布。
- [x] 联合检查死亡模型/声音/HUD、技能装配、真实模型拾取、加载和整局清理；更新架构与已实现规范。
- [x] CARGO_BUILD_JOBS=2 make check（343 Rust通过，2既有soak忽略）；WASM Clippy；40 Node测试；trunk临时构建。
- [x] 有窗口 Chromium 独占 Xvfb 验证800×600、常规窗口、DPR2，保存截图和模型/音频证据。原生 healthbars 检查真实显示跟随，虚拟帧钟不冒充性能证据。
- [x] 独立 Trellis 检查、修复发现项，记录最终验证和不足，任务记录不自动提交。

不启动/停止用户开发服务，不提交或推送。可创建临时构建及测试自有 HTTP fixture，结束清理。主代理统一公共接线和构建，避免互相覆盖文件。

最终结果与源/构建指纹见 [verification.md](verification.md)，完整审查见 [review.md](review.md)。全部实现和质量门已完成；Trellis状态保留in_progress，等待用户后续提交授权再做提交/归档收尾。
