# Research: 查看页入口、App 组合与渲染复用

- Query: 独立模型／动作查看页如何复用现有 Bevy/WASM、模型与启动加载，且不装配游戏世界；比较同 WASM 模式、独立 Rust bin/HTML、Three/glTF。
- Scope: internal，另核对本机 Trunk 0.22.1／Bevy 0.20.1 的版本固定源码证据。
- Date: 2026-11-10

## Findings

### 推荐结论与适用范围

推荐 MVP 采用 **A：同一 HTML/WASM 的显式 `?view=model` 入口，在创建 Bevy App 前选择独立组合**。这能提供可直接打开的专用页面，同时保持当前一次 Trunk 构建和静态发布方式。独立指 UI、输入和运行系统独立；它仍是同一下载包。采用这一方案不应先创建战斗世界再隐藏它。

主会话补充的已确认 MVP 是单角色动画片段／姿态、播放／暂停／时间轴；不包含完整战斗、技能、第二角色、目标和特效。因此没有理由注册 SimulationPlugin，也不需要模拟假敌人来触发现有游戏动作选择器。查看器应驱动共享姿态／蒙皮接口，具体采样契约由本任务动作研究收敛。

### 文件与函数证据

| 文件／锚点 | 现状与影响 |
|---|---|
| `Cargo.toml:1`、`Cargo.lock:434` | 单 crate；Bevy 0.20.1，Avian 0.8.0，WASM 专用 wasm-bindgen（锁定 0.2.130）。当前没有 glTF/Three 或相机控制新依赖需求。 |
| `src/lib.rs:2`、`:6`；`src/main.rs:1` | `client` 私有，公开入口只有 `kiva::run() → client::run()`。MVP 可保留此公开 API，在 client 内部分派，不必公开模型模块。 |
| `src/client/mod.rs:55` | `run()` 一次性建立 DefaultPlugins、ForestMaterialPlugin、StartupPlugin、SimulationPlugin，并注册全部游戏启动／更新系统；目前没有可直接重用的独立 renderer App builder。 |
| `src/client/mod.rs:80`、`:103` | 启动链包含 gameplay/scene/combat/companion/weapons/impact/camera/environment；更新链包含身体快照、敌人、HUD、声音和跟随相机。只删除地图或隐藏 HUD 并不能隔离这些系统。 |
| `src/simulation/mod.rs:52` | SimulationPlugin 安装真实 `PhysicsPlugins::new(FixedUpdate)`、重力、子步、战斗资源和固定步系统；viewer 不注册它。引用 WeaponPair/WeaponMode 等数据类型不等于注册物理。 |
| `src/client/scene.rs:12`、`:58` | scene::setup 先装配森林，再要求唯一真实 Player/Body；不适合作为 viewer 的启动入口。 |
| `src/client/avatar.rs:134`、`:692`、`:763` | AvatarAssets 按身体缓存，spawn_with_appearance 可为普通显示实体建立 RigPose/skin，replace 仅替换 AvatarPart。无需物理 actor。 |
| `src/client/avatar.rs:832` | avatar::sync 的 ActorView 可选；没有 actor/Health 时正常材质仍可用，可直接复用 palette 同步。 |
| `src/client/stickman.rs:44`、`:66` | 武器绑定／附件同步依赖 RigPose 与自身外观缓存，而不要求 SimulationPlugin；复用实际手腕适配。 |
| `src/client/weapon_models.rs:37`、`:132` | 四槽 pair+mode 缓存和按 Mount 的显示生成可复用，不要经战斗 sync_weapons 创建 viewer 武器。 |
| `src/client/animation.rs:45`、`:546` | RigPose 是显示数据；animate 是游戏状态驱动系统。viewer 应新增受控采样入口，不注册游戏 animate 后伪造 Body/Combatant。 |
| `index.html:10`、`:27` | 7 个 Web 桥同步安装后，只有 `<link data-trunk rel="rust" data-bin="kiva" />`；页面目前是战斗 UI。 |
| `Trunk.toml:1` | 单 `target="index.html"`，单 dist；literal pattern_script 调用 kivaLoader，保留 wasmBindings/Started。 |
| `web/startup.js:27`；`web/loader.js:5` | 可复用真实字节下载、阶段、失败与状态重放，不需要更换加载器。 |
| `web/input.js:31`、`:111`、`:269`、`:276` | input.js 控制 pointer lock、游戏进入卡片和输入 ready；不能在 viewer 页面照常安装它的监听器／绑定流程。 |
| `src/client/web.rs:14`、`:44` | Rust 最终 ready 固定调用 `window.kivaInput.ready()`；独立 viewer 必须提供明确 ready 通知路由。 |
| `src/client/loading.rs:143`、`:192` | 首次渲染观察只选择 GameCamera，并要求目标真实输出／可见 mesh specialization／管线就绪；没有 GameCamera 将永久等待。 |
| `src/client/rendering.rs:11` | deferred/HDR/FXAA/prepass/SSR 也只作用于 GameCamera；只新建 ViewerCamera 会漏掉现有渲染设置。 |
| `src/client/environment.rs:79`、`:95` | 雾／环境照明与 Level 森林装配耦合，不能为了灯光而调用 environment::setup。 |
| `README.md:68` | 发布就是 HTTPS 静态 dist，子路径使用 `--public-url /子路径/`，无服务端路由约定。 |

### A 的具体改动边界与可达 URL（提案，尚未实施）

1. **入口选择**：在 index 的同步脚本最前面解析唯一受支持值 `new URLSearchParams(location.search).get("view") === "model"`，形成只读页面模式；无值／未知值保持游戏。通过现有 wasm-bindgen 增加一个小型模式读取桥即可，不需要 web-sys/js-sys。模式在建立 App 前固定，切换页使用普通链接整页导航，MVP 不在同一运行中销毁／重建世界。
2. **App 组合**：client::run 先选模式，再调用 game builder 或新的 viewer builder；公共部分仅包括 Window/DefaultPlugins、StartupPlugin、共享 renderer 设置、AnimationLibrary/SourceAnimationLibrary，以及模型所需资产。viewer 启动创建自身显示根、相机、地面和灯光；更新为 viewer 命令／播放采样 → avatar::sync → stickman::sync_attachments → viewer camera。查看器根不用 ActorView/PlayerView，也不需要 Body/Combatant/Level。
3. **Web 页面**：同一 index 提供独立 viewer shell（选择、播放、时间轴、视角）与游戏 shell，最早模式决定显示；game input/hud/enemies/appearance/audio 脚本在 viewer 模式必须在注册任何 listener／DOM bind 前退出，或由显式启动入口只安装所选页面桥。startup/loader 为公共桥，新 viewer 桥独自拥有 DOM 命令与画布输入。仅 CSS 隐藏原 HUD 不足以隔离监听器和进入卡片。
4. **ready**：把首帧观察改成共享主视图标记／显式目标选择，在 GameCamera 和 ViewerCamera 各自创建时设置；保留当前真实 GPU 条件，不给 viewer 假装 GameCamera 以获取 ready。web::ready 的通知目标须随启动模式路由到 viewer 的就绪适配器（触发公共 kiva-ready/startup 状态）或提取公共 startup.ready API，并保留游戏 kivaInput.ready 兼容入口。只有 StartupSignal 能授权最终就绪，JS 不在下载完成后自行 ready。
5. **主体准备**：viewer 首次 ready 除共享渲染条件外，要确认选定模型已经创建并至少有其可见主体网格完成准备；中性地板独自可见不能证明模型就绪。当前全局 ready 是一次性信号，后续模型替换的“正在切换／当前选择”不能假装重新使用它；若 UI 声称已显示，需要对应选择版本的确认，迟到旧确认不可覆盖新选择。

开发时直接打开 `http://localhost:3221/?view=model`；根部署用 `https://host/?view=model`，子路径部署用 `https://host/kiva/?view=model`。请求始终命中已存在的 index 文档，query 不要求静态服务器 rewrite。构建仍为项目现行 `trunk build --release`，子路径加 `--public-url /kiva/`。`/model-viewer` 漂亮路径没有实际文件／服务端回退证据，MVP 不承诺这种地址。

### 渲染、地面与自由相机

- **相同显示实现**：复用 AV02/03、KIVAWP01 解码、StandardMaterial、PNG 色彩空间、原逆绑定、GPU skin、身体 retarget 和手腕挂点；不复制一套模型转换。`model_asset.rs:97` 保留真实纹理／ORM／cutoff／双面等设置。
- **共享相机配置 seam**：把 rendering::setup 中按 GameCamera 查找的设置改为显式相机／共享配置 helper。viewer 保持同一 PBR/deferred/HDR/FXAA 基础；湿地 SSR 可继续属于游戏配置，不能为无湿地的查看场景强行装配整个森林。中性背景、普通 rough 非金属地面、柔和主光／填光和可控环境光属于 viewer；不装森林雾、风动、地形资源和音频。
- **视角数据**：viewer-only 模块保存 target、yaw/pitch、distance 和拖动状态；实现环绕／平移／缩放、前后左右／俯视／三分之四预设、重置／适配。不调用 game camera::setup/follow（其 setup 在 `camera.rs:96` 要求 PlayerView，其 follow 有 Avian SpatialQuery 依赖）。无需新增 orbit 插件。
- **输入唯一所有权**：选择 JS 或 Bevy 作为 viewer 指针增量的唯一消费者。若沿现有 Web 边界用 viewer.js，则仅画布起始的 pointer drag 使用 pointer capture，滚轮仅在画布区域消费，表单／滑块不触发相机；pointercancel、blur 和 release 清拖动。pointer capture 不等于 pointer lock。Rust 只消费一次命令／累计增量，暂停仅冻结播放时钟，相机持续可用。
- **适配边界**：当前 avatar 已生成 skinned_mesh_bounds（`avatar.rs:328`）并挂 DynamicSkinnedMeshBounds（`:746`）。Bevy 0.20.1 本机 `bevy_camera/src/visibility/mod.rs:320` 明确动态 Aabb 随当前 skin 姿态更新；可汇总所选显示树当前世界 bounds（包含武器，排除地板）实现 fit。必须在骨骼／GlobalTransform／bounds 更新后读取，不能只取原始 mesh rest bounds；缩放／俯仰／近平面设有限值和范围。fit 为用户操作或选择／窗口变化触发，避免每帧自动取景破坏观察角度。蒙皮 bounds 是保守界，未覆盖任意额外 shader 位移，不能承诺精确轮廓包围。
- **真实性描述**：相同材质／蒙皮路径可避免第二套实现漂移，但中性灯光不同于竹林，颜色／高光不可能保证相同。原作完整 BRDF、发丝／布料物理和武器源动作仍有限制（model-assets spec、各资产 README）；不得把“复用当前渲染”写成“原作画面已还原”或画面验收通过。

### 方案比较与包体成本

| 方案 | 优点 | 成本／限制 | 判断 |
|---|---|---|---|
| A 同 WASM + 显式模式 | 一次构建、一个静态 dist、现有 loader，复用私有 client 代码；直接 query URL | 同一 WASM 的游戏代码和嵌入资源仍被下载；JS/HTML 模式与 App 组合要严格同步 | 推荐当前 MVP |
| B 独立 bin + HTML | 真正独立 artifact；未可达游戏／森林代码有机会被 linker 裁掉；页面脚本天然分离 | 第二 Rust 入口／HTML／构建和发布流程；每个包仍带 Bevy 与所用全部嵌入角色；不能承诺大小／速度降幅 | 有明确包体／发布隔离需求后采用 |
| C Three/glTF | 可作为 prepared 源 GLB 的独立离线审查工具 | 第二 renderer／材质／骨架 retarget／武器挂点实现；prepared 源姿态与当前 Kiva 烘焙、适配并不等价；需新增运行依赖和分发资源 | 不适合此“观察当前游戏模型与动作”的 MVP |

只读测量现有 `dist/kiva-b5d10f034fa45658_bg.wasm` 为 **216,837,106 bytes（206.8 MiB）**；这是已有开发产物，未为本任务重建，不能称为未来 release 包体。11-10 加载任务也记录相同量级，且完整大 WASM 再次访问实际重新传输，不能承诺浏览器缓存消除成本。

A 在 viewer 模式不调用 environment/gameplay/combat，能省掉关卡实体、环境 PNG 解码、网格／材质生成、GPU 上传和运行系统；**不能通过 runtime if 避免同包 include_bytes 的下载／实例数据成本**。身体／武器缓存本来按需解码（avatar.rs:134、weapon_models.rs:37）；保持这个策略，初始仅创建当前角色和选定武器，后续切换复用句柄，不预建所有身体／武器。AnimationLibrary/SourceAnimationLibrary 有嵌入数据解码；具体是否按需分拆另测，不凭规划声称零初始化成本。

B 的已核实可行路径是：新增 `src/bin/model-viewer.rs` → 公共 `kiva::run_model_viewer()`（内部仍复用 client 模块），根目录 `viewer.html` 使用 `data-bin="model-viewer"`。保留 index 的 `data-bin="kiva"`，并在 Cargo.toml 设置 `default-run="kiva"`，避免新增第二 bin 后破坏原生 `cargo run` 的默认选择。native viewer 并非此次已确认需求。

本机 `trunk build --help` 明确支持单 `[TARGET]`、`--html-output`、`--dist`、`--public-url`；0.22.1 Rust pipeline 源码也确认 data-bin。可设计两次隔离 staging 构建：

```sh
# 仅方案示例，本轮未执行；最终发布脚本还需实现与验证。
trunk build --release --dist <main-stage> --public-url /kiva/ index.html
trunk build --release --dist <viewer-stage> --html-output index.html --public-url /kiva/model-viewer/ viewer.html
```

在最终发布目录先放 main-stage，再将 viewer-stage 作为 `model-viewer/` 子目录，地址为 `/kiva/model-viewer/`。不要连续向同一 dist 构建并假定产物会追加保留；不要把两个 main Rust link 放进一个 HTML 并假定会按页面模式自动选一个。当前没有内建一次多 HTML 构建／发布任务，需显式新增组合脚本与检查。没有运行以上提案，B 的实际 linker 裁剪／bundle 大小仍未测量；若继续复用动态模式总入口，未可达裁剪收益还会丢失。

### 外部参考与相关规范

- 本机命令证据：`trunk --version → trunk 0.22.1`；`trunk build --help` 的 TARGET/html-output/dist/public-url 参数（本轮只读运行）。
- [Trunk v0.22.1 Rust pipeline](https://github.com/trunk-rs/trunk/blob/v0.22.1/src/pipelines/rust/mod.rs)：本机前次研究快照 `/tmp/kiva-startup-evidence/trunk-rust-v0.22.1.rs:138`（manifest 路径）、`:153`（data-bin）；本轮已读。
- [Trunk v0.22.1 output](https://github.com/trunk-rs/trunk/blob/v0.22.1/src/pipelines/rust/output.rs)：本机快照 `trunk-output-v0.22.1.rs:69` 的 public base/hashed js/wasm 和 pattern_script 插值，本轮已读；保留 `{ wasm }` 事件对象空格以避免 `{wasm}` 文件占位符替换。
- 本机 Cargo registry `bevy_camera-0.20.1/src/visibility/mod.rs:320`、`:594` 的 DynamicSkinnedMeshBounds 文档和实际 Aabb 更新实现；版本以 Cargo.lock 为准。
- `.trellis/spec/index.md`；`.trellis/spec/client/{index,startup,input,model-assets}.md`；`.trellis/spec/gameplay/index.md`；`docs/architecture.md:3` 的单向层次／所有权。
- 历史启动研究：`.trellis/tasks/archive/2026-11/11-10-entry-loading-progress/research/{startup-evidence,runtime-contract,integration-verification}.md`，其中 integration-verification.md:40–50 记载完整包体、真实首次画面与大文件缓存限制。

## Caveats / Not Found

- 本轮只研究与保存方案；未编辑产品代码、规范／构建配置，未运行构建／测试／浏览器／服务，也未执行 git 操作。所有代码符号为当前源码，拟新增 API／模块不是已存在事实。
- Trunk 本机和历史版本固定源可证实单 target、data-bin、loader；没有证据支持把 B 写成“Trunk 原生一次多页构建”。源码快照在 /tmp，持久结论和版本固定链接已记于此。
- 206.8 MiB 是现有开发产物，不是 viewer 或 release 实测；A 无包体缩减承诺，B 也只有裁剪机会。模型切换和实际性能／外观需实施后按固定构建另验收。
- 当前客户端规范对白灵淼写的是游戏 Companion 限制；查看器可否作为独立可选展示对象是本任务产品范围，不应为此改动游戏 F3 枚举。不要让 viewer 选择行为反写 AppearanceSelection 的游戏含义。
- 源 clip 与 Kiva 姿态／附件适配的可播放列表、根运动与时间轴采样契约由另一份动作研究覆盖；本文件仅确定其运行入口不需要物理世界。
