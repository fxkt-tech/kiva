# 实施步骤

1. `client/bundle.rs`：`Blob`、loader、`Bundle`、`request/status/take`、测试磁盘回退及单元测试。
2. 使用方切换字节来源：`source_animation`、`avatar`、`weapon_models`、`environment`、`ground_cover`；测试里的 `SourceAnimationLibrary::default()` 改为 `bundled()`。
3. `client/mod.rs`：`AssetPlugin` 配置、`Assemble` 调度、`Assembled` 门控、进度／失败上报。
4. `client/viewer`：启动门槛与按需加载；快照失败标记。
5. Web：`index.html` 的 `copy-dir`、`input.js`／`viewer.js` 文案与计数显示、Node 测试。
6. 验证：`make check`、WASM Clippy、临时目录 Trunk 构建并核对产物与体积。
7. 文档：`docs/architecture.md`、`.trellis/spec/client/{startup,index,model-assets}.md`、README。
