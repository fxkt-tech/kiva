# 实施计划

- [x] 建立并运行红色反馈循环，最小化首次加载／首次ready／点击捕获现象，记录可证伪假设和证据。
- [x] 固定本机Trunk进度与Bevy首次渲染就绪接口，发布runtime-contract供两端实现。
- [x] Web显示真实下载进度与阶段，禁用首次按钮、捕获中反馈与重试，保留恢复和错误语义。
- [x] Rust首次渲染就绪再通知，覆盖真实渲染准备状态／失败／原生兼容回归。
- [x] 独立check、make check、WASM Clippy与临时Trunk构建；浏览器冷加载首击、缓存和暂停恢复验证。
- [x] 更新客户端启动规范、任务证据和日志，main未提交交付。

## 文件所有权

研究代理只写本任务research和临时诊断fixture，研究本机Trunk与Bevy实际接口及基线反馈。
Web实现者负责web/startup.js、input.js、style.css、可选专用loader、index.html与相关Node测试／必要Trunk入口配置。
引擎实现者负责src/client/mod.rs、web.rs及新的就绪模块／对应Rust测试。双方先读共享runtime-contract，不能自行发明互不兼容状态。
主会话负责文档／规范／任务集成，检查代理在交接后处理局部修复与完整门槛。工作区协作不回滚他人改动。
