# 设计

## 字节来源：`client/bundle.rs`

- `Blob(Vec<u8>)`：唯一的原始字节 Asset；`BlobLoader` 把整个文件读成 `Blob`。按资产类型请求（`load::<Blob>(path)`），所以 `.png` 也走它而不触发 Bevy 的图像 loader——环境贴图仍由现有 `texture()` 解码并生成自定义 mip。
- `Bundle` 资源：`path → Loading(Handle<Blob>) | Taken`。
  - `request(world, paths)`：幂等地开始加载。
  - `status(world, paths) → Pending{loaded,total} | Ready | Failed(path)`；`Taken` 视为已就绪。
  - `take(world, path) → Vec<u8>`：从 `Assets<Blob>` 移出字节并丢弃句柄，标记 `Taken`。每个文件只有一个解码方且解码结果已有缓存，取走即释放。
- 测试构建下，`take` 在没有已加载 `Blob` 时直接读 `CARGO_MANIFEST_DIR/assets/<path>`。这样既有无窗口测试保持同步，不需要 AssetServer。非测试构建下取未就绪资源是程序错误，直接 panic 并带路径。
- 路径归各自模块所有：`Appearance::asset_path()`、`weapon_models::asset_path(pair, mode)`、`source_animation::PATHS`、`environment::TEXTURES`。`bundle::game()` 汇总成游戏清单。

## 使用方

只替换字节来源，解码逻辑不动：

| 模块 | 变化 |
|---|---|
| `source_animation` | `Default` 改为 `FromWorld`（经 `bundle::take`）；测试用 `SourceAnimationLibrary::bundled()` |
| `avatar` | `AvatarAssets::shared` 用 `bundle::take(world, appearance.asset_path())` |
| `weapon_models` | `WeaponModel::shared` 同上 |
| `environment`、`ground_cover` | `texture(world, path, …)` 取路径而非 `&'static [u8]` |
| `skeleton` | 不变，`stickman.anim` 继续嵌入 |

## 装配门槛

新增 `Assemble` 调度和 `Assembled` 标记资源（`client/mod.rs`）。

- Startup 只保留 `loading::begin` 和资源请求。
- `First` 中的独占系统 `assemble`：未装配且清单 `Ready` 时运行一次 `Assemble`，然后插入 `Assembled`。`Pending` 时报告 `progress("initialize", loaded, total)`；`Failed` 时报告一次失败。
- 原 Startup 链（玩法装配 → 显示根 → 战斗表现 → 相机 → `scene_ready`）整体移入 `Assemble`，顺序不变。
- `RunFixedMainLoop` 的采样和 `Update` 表现链加 `run_if(resource_exists::<Assembled>)`。
- `SimulationActive` 在装配前置为 false，装配后由 `controls::sample` 按捕获状态接管（原有行为）。
- 资源事件在 `PreUpdate` 落入 `Assets<Blob>`，`First` 在下一帧看到，装配与同帧的采样／Update 顺序等同于原来 Startup 后的首帧。

渲染就绪门槛不变：装配前没有 `MainView`，`observe_rendered_scene` 自然保持 Preparing。

## 验收页按需加载

- 启动清单：两份动画库 + 默认外观（李火旺）+ 默认武器（伏浪锚）。同样经 `Assemble` 门槛。
- `update` 中，选择变化时先 `request` 所选外观／武器的文件：
  - `Pending`：不重建、不注册新 revision、不求值新姿态，状态文字“正在加载所选模型…”；由于 revision 未确认，页面保持既有的“正在准备所选模型”。
  - `Failed`：快照 `failed=true`，状态文字说明失败；页面沿用既有失败面板。
  - `Ready`：走原有重建路径。
- `finish_view`、`attachments` 改读 `scene.shown_appearance` / `scene.shown_weapon`，不能在加载期间按未到齐的选择取缓存。

## 平台

- `AssetPlugin { meta_check: Never, file_path }`：Web 没有 `.meta`，不请求它。
- 原生 `file_path = "assets"`（Bevy 解析为 `BEVY_ASSET_ROOT` / `CARGO_MANIFEST_DIR` / 可执行文件目录）。
- Web `file_path = "."`：Trunk `copy-dir` 把 `assets/<dir>` 复制为 `dist/<dir>`，与既有 `assets/audio → dist/audio` 一致，不依赖较新 Trunk 的 `data-target-path`（本机 Trunk 为 0.17.5）。目录内的 README／manifest 会随之发布，体积很小，与 audio 的先例相同。

## 进度与失败

- 复用 `initialize` 阶段，带真实计数（规范已允许“真实字节／计数”）；`input.js` 在该阶段有分母时显示“正在加载资源 n / m”。不新增阶段，阶段单向顺序不变。
- 失败文字“资源加载失败，请重新加载后重试”，经 `web::fail` 一次性上报；`startup.js` 的 failed 语义不变。

## 取舍

- 游戏仍然“全部到齐再装配”：首帧所需本来就覆盖了除妹妹和第二组武器以外的全部文件，分批装配的收益约 10MB，代价是表现层要处处容忍缺失。
- 资源下载在 WASM 初始化之后才开始（AssetServer 在 Rust 里发起）。并行预取可以后续用 `<link rel=preload>` 补，需要浏览器验证，不在本次范围。
