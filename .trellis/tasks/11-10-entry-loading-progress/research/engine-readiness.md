# Bevy 0.20.1 首次画面就绪门槛

## 本机源码证据

以下均查自本机 Cargo.lock 对应源码：
`/home/asuka/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/`，不是其他 Bevy 版本的接口推测。

| 源码 | 事实与实现含义 |
| --- | --- |
| `bevy_render-0.20.1/src/lib.rs:469` | `PipelineCache::process_pipeline_queue_system` 与 `renderer::render_system` 在 `RenderSystems::Render` 中 chain。Startup 装配结束不包含这些工作。 |
| `bevy_render-0.20.1/src/renderer/mod.rs:94`、`:106`、`:123` | render_system 执行 RenderGraph、提交命令、按 view 是否写入输出决定 window.present。观察系统放在整个 Render set 之后、Cleanup 之前。 |
| `bevy_render-0.20.1/src/texture/texture_attachment.rs:443` | `OutputColorAttachment::needs_present` 表示输出附件确实被 render pass 使用；`ViewTarget::needs_present` 转发它。单纯分配纹理／任意空 GPU submit 不足以放行。 |
| `bevy_pbr-0.20.1/src/material.rs:895` | `SpecializedMaterialPipelineCache` 是按 RetainedViewEntity → MainEntity 的非泛型聚合缓存，包含 StandardMaterial 和 ForestMaterial，并非某一材质专用 cache。 |
| `bevy_pbr-0.20.1/src/material.rs:1079` 起 | specialize_material_meshes 在 material instance、GPU mesh instance、RenderMesh、PreparedMaterial 缺失时跳过／重试，完成后才写入该 mesh 的 pipeline entry。 |
| `bevy_render-0.20.1/src/view/visibility/mod.rs:169`、`:279` | RenderVisibleEntities.get::<Mesh3d>().iter_visible() 同时提供 CPU 可见与 NoCpuCulling 的 MainEntity；与材质缓存使用相同 ID。隐藏的妹妹／未来技能变体不加入首次可见网格门槛。 |
| `bevy_render-0.20.1/src/render_resource/pipeline_cache.rs:237`、`:863`、`:911` | pipelines() 可查看实际状态；waiting_pipelines() 为空既可能尚未入队，也可能因永久失败被移出，不能独自作为 ready。process_queue 先收取新 pipeline 再编译。 |
| 同文件 `:913`、`:920`、`:930` | ShaderNotLoaded／ShaderImportNotYetAvailable 是 Bevy 自身重试状态；ProcessShaderError／CreateShaderModule 是终止错误，必须发 failed，不能永远显示准备中。 |
| `bevy_render-0.20.1/src/error_handler.rs:69`、`:189` | 设备／验证错误处理在 extract 边界、可访问 main/render 两世界，默认 handler 写 AppExit 并停止渲染。新 wrapper 先通知入口失败，再保留原 handler 的策略。 |
| `bevy_render-0.20.1/src/pipelined_rendering.rs:219` | 原生流水渲染可把 RenderApp 移至另一个线程，不能在渲染观察器直接调用浏览器桥。 |

## 最终条件与通知

新 `client/loading.rs` 的观察器只在以下条件同时满足时报告 Ready：

1. 本次 Render schedule 中所有当前 pipeline 实际状态均为 Ok，且 pipeline 集合非空。
2. 已提取项目真正的 GameCamera；用其 MainEntity 匹配 ExtractedView，其他空视图不替代游戏画面。
3. 这个 view 的输出附件被 render pass 写入，且 render_system 已返回（命令已提交，窗口 present 已调用）。
4. 该 view 至少有一个可见 Mesh3d，且每个可见 mesh 都有真实材质 pipeline specialization entry，证明其必要 GPU mesh/material 已准备。

使用共享 `Arc<Mutex<Readiness>>` 将结果交给主世界，独占 First 系统只调用一次最终 `window.kivaInput.ready()`。普通 Send 主世界系统仍可能在原生工作线程运行，独占系统明确固定通知到主线程。失败只通知一次，真正 ready 可以覆盖失败；ready 后迟到失败不降级。

Startup 首部通知 `kivaStartup.progress("initialize",0,0)`，装配末尾通知 `progress("render",0,0)`。pipeline 集合会随 specialization 增长，故不把动态数量伪装为固定进度分母。终止 pipeline 错误和 RenderErrorHandler 发送 `fail("画面准备失败，请重新加载后重试")`；技术错误保留 Bevy 自身日志。不等待音频、隐藏妹妹或整个 AssetServer 的后续资源。

原生桥为空操作，原输入／SimulationActive 门控未改；没有 RenderApp 的原生无窗口测试照常运行。有 RenderApp 但无 GPU／PBR 资源的测试场景保持 Preparing，不访问缺失资源、不 panic。WASM 缺少渲染后端时装配末尾报告失败，不误报 ready。

## 回归与证据边界

`cargo test --locked client::loading::tests -- --nocapture` 覆盖实际 Bevy 状态枚举与可见网格／材质缓存数据、空集合、重试与永久错误、普通与 forest 两类材质、隐藏网格不入门槛、render→main 通知一次、失败恢复、已有设备错误策略，以及无 GPU Render schedule。测试桥计数直接记录真实 web::ready／fail 调用。

首次测试运行 4 通过／1 失败：普通 notifier 在原生工作线程执行，主线程未收到测试桥计数。改为独占主世界系统后通过。这是实现时线程边界的红绿反馈；原入口过早 ready 的基线来自旧 `mod.rs` 的 Startup 直接调用与独立浏览器调查，不能把这个单元测试失败描述成已经复现用户首击硬件问题。

实施者最终针对性检查：6 个 loading 测试通过；`cargo clippy --locked --all-targets -- -D warnings`、`cargo clippy --locked --target wasm32-unknown-unknown -- -D warnings`、`cargo fmt --all`、`git diff --check` 通过。完整 make check／Trunk／浏览器门槛由集成检查单独记录。

真实 GPU 的首次场景／冷下载／首击／暂停恢复仍由主会话在固定 Trunk 产物中独立验证。无窗口测试和源码分析不能替代画面或输入手感验收。
