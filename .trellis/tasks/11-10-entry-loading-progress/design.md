# 技术设计

## 边界

入口现状只提供loading/failed/ready，input.js在收到Rust Startup末尾的ready后显示可点击；没有下载进度或捕获进行中反馈。
最小改动位于启动桥、Trunk加载接口、入口DOM/CSS和客户端ready通知。保留输入采样／动作规则与资源格式。

## 调查与接口

先以实际桥脚本／浏览器建立可运行反馈循环，记录下载完成、引擎Startup、首次渲染准备和捕获的时间顺序，再确认根因。具体Trunk加载接口和Bevy渲染阶段须以本机版本源代码／产物证实，写research/startup-evidence.md和research/runtime-contract.md。
启动状态由单一startup桥保存并向订阅者重放；兼容__kivaStartup既有字符串和真实ready优先语义。下载可测量则使用真实字节，其他阶段明确indeterminate。不得全局拦截不相关音频或使用假计时条。
Rust侧仅通知客户端就绪，不改变模拟；首次渲染必要管线准备后经现有桥放行。WebGPU/初始化错误可回报失败，不无限等待未知资源；隐藏妹妹等后续技能资源不应成为首次进入的无界门槛。

## 兼容与验证

入口按钮在加载时可见但disabled，进度／阶段与无障碍状态一致；ready后沿原用户手势申请pointer lock，捕获中禁重入并给状态，失败可重试。
保留源游戏资源、Cargo依赖与按钮以外的选择卡片功能。原生不依赖JS；浏览器用隔离fixture／实际WASM冷加载验证首击、缓存和恢复，全部临时进程回收。
