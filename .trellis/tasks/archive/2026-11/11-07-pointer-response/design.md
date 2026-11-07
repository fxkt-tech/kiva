# 实现边界

新增web/pointer.js拥有Web捕获指针的CSS位置、边界、位移比例和鼠标按下位置队列。
使用进入前可信pointermove的client差值与movement差值学习raw→CSS比例，忽略首样本/零位移/失焦与边界中断。
无样本时使用浏览器CSS单位，后续未捕获移动校准；不根据DPR再次盲除。DPR变化时重置校准。
锁定中在document capture阶段处理pointermove，立即更新DOM；同一事件内记录按下快照（含多键chord）。
setActive由Rust完成会话/捕获门控后调用，首次启用居中。JS还复核实际锁定、事件确认、DOM暂停和焦点。
失锁/暂停/失焦立即隐藏/清队列。尺寸变化夹紧；每个移动只更新DOM/坐标，不逐样本调用Rust射线拾取。

浏览器桥公开sync(active)、position()、takePress(button)及clearPresses()，仅JS输入适配，不能写World。
GamePointer在Web仅镜像读取的坐标，Native保留原逻辑。WindowEvent仍是键鼠有序输入通道：
MouseMotion在Web不再积分；MouseButtonInput先消费对应的按下坐标，再走原UI/世界指令；末尾读取最新坐标供镜头/hover。
just_captured/no-player/paused清旧press，并保留原窗口/DOM技能丢弃边界。按下队列不对缺失样本退回最新坐标。
Rust移除DOM指针绘制，CSS和虚拟hover由pointer.js拥有，HUD/装备/捕获会话仍由原模块拥有。

文件：web/pointer.js、index.html加载、client/web.rs桥、pointer.rs镜像、controls.rs接入，
browser-support按正常CSS/CDP位移驱动并修正physical误名，新增browser-pointer-response专项。
规范/架构只同步实际落地契约。本次不重构输入总线、不改变战斗模块或相机算法。
