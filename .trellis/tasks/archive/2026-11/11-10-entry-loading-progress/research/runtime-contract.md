# 启动进度与最终就绪合同

## JS／Rust 边界

- `window.kivaStartup.progress(phase,loaded,total)`：phase 为 `download`／`initialize`／`render`；数字为真实字节或真实已准备／总计数。`0/0` 表示不可计量，不是假百分比。Rust 侧使用 `&str,u32,u32`，下载由 Trunk 加载路径报告。
- `window.kivaStartup.fail(reason)`：有界的可解释错误原因，不能把任意 GPU 调试栈直接塞进产品文字。
- `window.kivaInput.ready()`：最终客户端就绪的唯一放行入口，继续触发 `kiva-ready`；Rust 在源代码已证实的首次必要渲染准备完成后调用一次，不能再从 Startup 装配末尾直接调用。
- `window.kivaStartup.subscribe(callback)` 兼容原接口，第一参数仍为 `loading`／`failed`／`ready`，允许第二参数附带当前进度快照；订阅立即重放。
- `window.__kivaStartup` 继续保留原状态字符串，已有桥和测试不依赖新的字段猜状态。

## 状态语义

阶段向下载→初始化→画面准备前进，迟到的前一阶段进度不能覆盖当前阶段；最终 ready 不被任何迟到进度／错误降级。
failed 保留失败与重载入口，真实引擎 ready 仍可恢复。进度不会自行触发 ready；可计量的下载100%不等于可进入。
下载总量缺失／无可靠解码字节总量时显示不定进度与已收数据；初始化／渲染无法给可信分母时显示阶段，不用计时器补造百分比。

## 入口与捕获

初次按钮始终显示，但 ready 前为原生 disabled，并有可访问的加载状态；按钮和画布都必须由同一 ready 门槛拒绝进入。
ready 后需要新用户点击请求 pointer lock；请求期间显示捕获中并防止重复，失败／失焦清理后可以重试。不能让 Promise resolve 代替 pointerlockchange＋焦点确认。
Esc／失焦恢复保留已完成 ready，不重复加载；不改变模拟动作、已承诺技能、资源或相机。

## 实现待证据化部分

研究者负责本机 Trunk 0.22.1 实际进度回调和下载流机制，写 startup-evidence.md；不猜 API 或全局拦截无关 fetch。
引擎实现者负责本机 Bevy 0.20.1 的 RenderApp／PipelineCache 调度与失败边界，写 engine-readiness.md；不能以任意帧数／等待时长代替准备完成。
具体报告源与门槛在这两份研究中固化，UI 和 Rust 分工不增加第二个 ready 权威。
