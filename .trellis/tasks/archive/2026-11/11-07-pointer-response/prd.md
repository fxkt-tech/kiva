# Web鼠标速度与响应修复

用户报告游戏内速度与系统指针不同、指针不跟手；环境Windows11/Edge/系统150%/网页100%。
基线18e3589。工作区起始干净；开发服务由用户管理，实现验收完成后，用户已明确授权提交与推送。

## 需求与验收

- 锁定与未锁定的浏览器正常指针位移一致，覆盖DPR1/1.5/2及浏览器不同movement单位。
- 浏览器移动事件结束即可观察到光标CSS位置变化，不等待Bevy或完整HUD；模拟渲染停帧时同样成立。
- 点击/按键仍按既有事件顺序执行，点击绑定按下时坐标，不能使用之后移动到的新位置。
- 有界、越界立即回收、进入居中、暂停/失焦/失锁隐藏并排空旧点击、捕获事件确认保持。
- 实际技能/装备HUD鼠标、世界移动/拾取、镜头边缘与高DPI缩放继续可用。
- 指针坐标只有一个Web所有者；Rust不另积相对量或覆写DOM位置。Native原绝对指针路径保持。
- 不加入手工灵敏度设置、不改模拟规则/时间/技能数值、不新增运行依赖。

## 复现证据

/tmp/kiva-pointer-probe.cjs在已验收构建运行：非零mousemove后microtask中CSS位置仍旧，下一帧才更新5–10ms后。
严格即时响应断言失败。DPR1.5时movementX12只移动8CSSpx，DPR2时只移动6。
这是Linux Chromium的DPR模拟，不当作Windows原生硬件复现；实际单位需按未捕获位移测量。
Winit0.30.13对Chromium按physical解释movement；controls再除window.scale_factor，存在浏览器单位假设。
官方资料：https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent/movementX
以及https://www.w3.org/TR/pointerlock-2/，默认保留系统鼠标加速。
