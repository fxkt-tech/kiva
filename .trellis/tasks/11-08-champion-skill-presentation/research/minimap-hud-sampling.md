# 小地图拖拽断言的采样层次

391项生产代码最终门禁之后，浏览器小地图回归在beginFrameDrag失败：grabbing inside the rectangle must not jump the camera to the pointer。

反馈命令为KIVA_MINIMAP_CASE=1280x800@1的browser-minimap.cjs，使用自有临时站点/隔离Xvfb。原完整矩阵与追加同条件诊断均实际红灯。真实WebGPU启动和完整HUD/独立相机通道是必要条件，不能以纯JSmock或无窗口相机数学替代此反馈环。

按顺序核对：完整HUD旧采样；方向键松开后的残留一帧；真实拖拽提交偏移。只在beginFrameDrag加入带DEBUG-minimap-grab标签的前后记录，保持原红灯断言和0.02阈值。

关键诊断：

- before旧HUD = (0.8954852, -0.23023793)
- 同时probe.live = (1.1638134, -0.36440203)，probe.hud仍等于before
- 按下并等待6真实tick后，after.live = (1.1638134, -0.36440203)，dragging=true，完整HUD此时才追上

因此逐帧真实镜头前后差为0；误差来自约80ms完整HUD的前一次平移样本，不是拖拽跳镜头，也不是按住后继续平移。原功能没有修改。

修复仅将该手势测试的before/after都改读window.__minimapCamera.center，保留原阈值与真实按住/拖动/六tick等待；若真实镜头发生跳动，仍会失败。此值本就是视野框消费的独立逐帧通道。删除所有临时DEBUG记录；完整同场景回放及三窗口联合结果由父任务verification.md记录。

同条件修正回放已通过1280完整场景，日志/tmp/kiva-skills-minimap-corrected.log，覆盖真实点击/框内拖动、chord释放、UI隔离、边界/滚轮、逐帧框、暂停/恢复与重选。旧红灯诊断保留；没有放宽0.02阈值。

## 捕获恢复的绝对定位夹具

800窗口后续在captureWithButton最后movePointer(center)超时。追加原断言诊断：target=(400,300)，生产CSS位置before=(400,260.491)，WeakMap last=(400,263.140625)，合成next=(400,302.649625)，after=(400,302.1356349172458)。请求都在画布内、捕获/暂停状态正常，仅2.135635残差刚超过原<2阈值。前序移动也表现为整数合成事件与当前已学习gain下的小残差，而非严格1:1。

仅修改绝对定位测试helper：每次读取完整精度生产kivaPointer.position，以残差继续正常CDP输入，最多6次；失锁、非法位置、次数耗尽仍失败，最后保留原DOM每轴<2硬断言。moveRelative与鼠标单位/响应的单次输入路径未改，不通过放宽阈值或改pointer.js掩盖问题。临时DEBUG日志和短超时均已移除。

同一800完整回放已实际通过，日志/tmp/kiva-skills-minimap-800-corrected.log。其余DPR/原生输入回归由父任务继续收敛。
