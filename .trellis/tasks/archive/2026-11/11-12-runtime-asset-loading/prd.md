# 运行时资源加载：资产移出二进制

状态：用户于 2026-11-12 确认三项决定并授权“方案后直接实现”：建 Trellis 任务；使用 Bevy AssetServer 异步加载；原生版从磁盘 `assets/` 读取。提交与推送未授权。

## Goal

把角色、动画、武器和环境贴图从 `include_bytes!` 改为运行时加载，使 WASM 不再携带约 77MB 资源字节：浏览器不必整包下载并编译后才出首帧，原始字节不再常驻线性内存，改代码不再重新链接资源。

## 现状依据

- 26 处 `include_bytes!`，去重 24 个文件约 77MB；`dist/` 现有 WASM 201MB。
- 使用方（`avatar`、`weapon_models`、`environment`、`ground_cover`、`source_animation`）都在 Startup 或独占系统里同步解码，解码结果已有缓存。
- 约 190 个客户端单元测试在没有 `AssetPlugin` 的 `App` / `World` 里同步调用这些入口。
- 启动契约见 `.trellis/spec/client/startup.md`：下载 → 初始化 → 画面准备，最终 ready 由真实渲染提交决定。

## Requirements

| ID | 需求 |
|---|---|
| R01 | 角色 `character.bin`×3、动画 `animations.bin`×2、武器 `weapon.bin`×4、环境 PNG×14 不再编入二进制，由 Bevy `AssetServer` 加载 |
| R02 | Web 从页面同源路径拉取；原生从可执行文件旁或仓库的 `assets/` 读取；两端使用同一套接口 |
| R03 | 游戏模式在所需资源全部到齐后才装配场景；到齐前不运行输入采样、表现同步和模拟规则 |
| R04 | 验收页（`?view=model`）启动只加载动画库与默认配置，其余角色／武器在选中时按需加载 |
| R05 | 加载期间向 `kivaStartup` 报告真实已完成／总文件数；任何文件失败都报告失败，不能悬挂或带缺失资源进入 |
| R06 | 原始字节解码后释放，不重复解码；既有“重复技能／换装不增长资产”保持 |
| R07 | 最终 ready 仍由真实渲染提交决定，资源到齐不能替代 |
| R08 | 既有无窗口测试继续同步可用，不需要启动 AssetServer |

## 不在范围

- 游戏内分批装配（先出首帧再补妹妹／第二组武器）。隐藏实体的延迟装配要让 companion／weapons／combat 全部容忍“显示根暂缺”，本任务不做。
- `assets/animations/stickman.anim`（0.6MB，CC0 结构参考骨架）和 16KB shader 继续嵌入：它们是所有 `RigPose` 的同步前提。
- 字节级资源进度、`<link rel=preload>` 并行预取、资源压缩格式。

## Acceptance Criteria

- [x] `src/` 中除 `stickman.anim` 外没有资源 `include_bytes!`
- [x] `make check`（fmt、clippy、Rust／Node 测试、WASM check）通过
- [x] WASM 目标 Clippy 通过
- [x] 新增测试覆盖：清单中的文件都存在于 `assets/`；未到齐／到齐／失败三种状态；字节取走后释放；验收页选择未加载模型时不重建、不确认 revision
- [x] 临时目录 Trunk 构建产物包含资源文件，WASM 体积明显下降
- [x] 架构文档、启动规范、客户端规范同步更新

浏览器内的冷加载、画面与手感验收需要用户自己的开发服务，不在本次自动验证内。
