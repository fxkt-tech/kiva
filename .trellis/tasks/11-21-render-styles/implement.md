# 实施计划

每个阶段结束都要能编译、测试通过，写实风格的画面不变。

## 阶段 0：技术验证（已完成，采用方案 A，见 research/spike.md）

- [x] 写一个最小的自定义后处理系统，在竹林相机上同时绑定场景颜色、深度和延迟 GBuffer，输出"受光比分档"的调试画面。
- [x] 确认能区分前向像素（镂空竹叶）和延迟像素，并读到 emissive。
- [ ] 原生和 Web（WebGPU）都要跑通。（原生 ✅；Web 编译 ✅，运行时待用户确认）
- [x] 结论写进 `research/spike.md`：采用方案 A 或 B，附截图。
- **评审点**：和用户确认方案以后再进入阶段 1。

## 阶段 1：框架与切换（写实不变）—— 已完成

原生实机：`--yancai-style-shots` 截图写实 → 岩彩 → 写实，往返前后亮度差均值 0.003/255（最大 13，来自风吹草动），见 `research/spike/style-*.jpg`。岩彩此时只有光照配置，效果要到阶段 2、3 才出来。规范已同步 `spec/client/index.md`（F8、`kivaRenderStyle`）。

- [x] `client/render_style.rs`：`RenderStyle`、`LevelStyles`、F8 采样、Web 桥。
- [x] `environment.rs`：提取 `SceneLighting` 并加 `bamboo_lighting(style)`，`setup` 改成读配置，两盏灯加 `SceneLight` 标记，实现 `apply_style`。
- [x] `rendering.rs`：游戏相机常驻 `NormalPrepass`；`apply_style` 根据风格插入或移除 SSR。
- [x] `web/render_style.js`、`index.html` 的 HUD 和操作说明、窗口标题。
- [x] 测试：
  - World 测试：写实 → 岩彩 → 写实之后，灯光、环境光、清屏色、雾、SSR 和相机组件与初始状态逐项相等；反复切换后实体、材质、贴图数量稳定。
  - 废站：F8 不改变 `RenderStyle`。
  - 原生 F8 需要窗口焦点并且模拟激活。
  - Node 测试 `scripts/render-style-unit.test.cjs`：Web 桥单次消费、可信按键和重复按键过滤、捕获状态门控（参照 `scripts/enemies-unit.test.cjs`）。

## 阶段 2：岩彩环境 —— 已完成

`ForestSurface.yancai`（binding 106）由 `environment::forest_style` 给出，setup 和切换时统一写入全部林地材质。岩彩下：固有色按种类改成矿物色（地面稀释赭石，苔藓处石绿；竹竿石绿；石头青灰；叶和草卡石绿 / 石青），保留明度；加矿物颗粒；去掉湿度高光、清漆和镜面反射；法线贴图减弱到 35%。前向表面（草卡、竹叶）在材质里按主光照度分三档，暗部偏石青。雾可见距离调到 72。截图：`research/spike/style-1-yancai.jpg`（只有材质和光照）、`style-3-yancai-preview.jpg`（叠加阶段 0 的后处理预览）；写实往返差异与阶段 1 相同（均值 0.003/255）。

阶段 3 要处理的问题（从预览图看到的）：
- 草卡和竹叶被描边成黑团：前向区域只保留深度外轮廓。
- 预览的后处理分档丢掉了雾：重着色要保留原像素里的雾，或者在分档后重新套雾。
- 角色明暗依旧太碎（阶段 0 已知）。


- [x] `ForestSurface` 加 `yancai`（原计划叫 `mode`）；`forest.wesl` 的前向和延迟分支实现岩彩；`apply_style` 遍历材质改 `mode`。
- [x] 扩展现有的 WESL 无 GPU 组合测试，覆盖岩彩分支。
- [x] 岩彩光照配置初始值。

## 阶段 3：岩彩后处理链 —— 已完成

- `client/render_style/yancai_post.rs`：pass 1 `YancaiShade`（自定义渲染系统，HDR、tonemapping 前）做去雾 → 受光比平滑（8 个同表面邻点）→ 分档 → 重新加雾，再按几何法线和深度描墨线；前向区域只描深度外轮廓，雾和距离都会淡化墨线。pass 2 `YancaiPaper`（Bevy `FullscreenMaterial`，tonemapping 后、FXAA 前）做矿物色板拉色、颜色交界积色、程序化纸纹。
- 纸纹和色板改为**着色器内程序化生成**，没有新增贴图资源，资源清单不变。
- 角色分档平滑成功（`research/stage3/1-yancai-hero-crop.jpg`），**不需要方案 B**。
- 修了一个实机才出现的问题：渲染世界的视图实体跨帧保留，相机移除 `YancaiShade` 后 pipeline id 还在，pass 继续用过期的 uniform 跑，切回写实画面错乱（亮度差均值 40/255）。现在没有 uniform 的视图会移除 pipeline id，pass 也要求 uniform 存在；修复后往返差异回到 0.003/255。
- 临时 `yancai_spike` 已删除；原生开发截图改为 `--render-style-shots`（`render_style/shots.rs`，输出 `target/render-style/`）。


- [x] `render_style/yancai_post.rs` 和 `embedded/shaders/yancai_{shade,paper}.wesl`：pass 1 重着色和描边，pass 2 纸纹和色板。
- [x] ~~纸纹贴图、色板查找表加进竹林资源清单~~ → 改为程序化，不加资源。
- [x] 着色器组合测试；测试写实风格下相机上没有 `YancaiPost`。
- [x] 删除临时的 `yancai_spike`。
- [x] ~~（仅在角色分档平滑失败时）迁移角色材质~~ → 平滑成功，不需要。

## 阶段 4：调参与验收 —— 本机能做的部分已完成

- 截图：`research/stage3/0-pbr.jpg`、`3-yancai.jpg`、`6-pbr-again.jpg`（同机位全图），`0-pbr-hero-crop.jpg` / `1-yancai-hero-crop.jpg`（主角局部），`tints-grid.jpg`（上排写实、下排岩彩；从左到右为常态、受击、蓄力）。
- 受击和蓄力是自发光，岩彩下跳过分档、保持原色，并加上墨线；两种状态颜色可区分。
- 废站：只登记写实，不挂 `NormalPrepass`，相机与原来一致；`--station` 运行 100 秒无报错。
- 帧耗时：本机是 llvmpipe 软件渲染，并且窗口无焦点时 Bevy 限帧（各段都是约 250 ms），**没有有效数据**，需在有显卡的机器上用 `--render-style-shots` 重测。
- 调参：试过一轮提亮（亮部色调 ×1.22、色板 0.45、纸纹 0.12），截图几乎看不出差别，已撤回，等美术给方向。
- 规范：新增 `.trellis/spec/client/render-style.md`，`index.md` 改为链接。

待用户：Web（WebGPU）实测 F8 和 HUD、有显卡机器的帧耗时、美术评审。


- [x] 同一机位截写实和岩彩各一张，放到任务目录，交给美术评审。
- [x] 检查受击变色、蓄力提示、角色特效在岩彩下的可读性。
- [ ] 记录原生和 Web 的帧耗时。
- [x] 更新规范：`.trellis/spec/client/index.md` 的快捷键和桥签名，新增 `render-style.md`（风格框架契约）。

## 验证命令

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --target wasm32-unknown-unknown   # Web 构建
node scripts/render-style-unit.test.cjs           # 新增，写法参照 scripts/enemies-unit.test.cjs
```

开发服务由用户自行启动，截图和帧耗时在用户的服务上验收。
