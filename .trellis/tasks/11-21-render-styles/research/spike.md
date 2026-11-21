# 阶段 0 技术验证：屏幕空间重着色（方案 A）

日期：2026-11-21

## 做了什么

临时模块 `src/client/yancai_spike.rs` 加着色器 `src/client/embedded/shaders/yancai_spike.wesl`，只装配在竹林。

- 后处理 pass 放在 `Core3dSystems::PostProcess`、tonemapping 之前（HDR），结构照搬 `bevy_post_process::motion_blur`。
- 同时绑定：HDR 场景颜色、prepass 深度、延迟 GBuffer（`Rgba32Uint`）、延迟光照 ID（`R8Uint`）、prepass 法线。
- F9 循环切换调试视图；原生加 `--yancai-spike-shots` 参数会自动截取每个视图后退出。
- 默认不挂载：只有第一次按 F9 或带截图参数时，才给相机加 `YancaiSpike` 和 `NormalPrepass`，正常游玩不受影响。

截图在 `research/spike/`，缩到 960 宽的 jpg，同一机位、同一次运行：

| 文件 | 内容 |
|---|---|
| `0-off.jpg` | 现在的写实画面（已挂 NormalPrepass，和不挂时的截图肉眼一致） |
| `1-classes.jpg` | 像素分类：绿 = 延迟，黄 = 前向但 GBuffer 非空，品红 = 前向且 GBuffer 为空，蓝 = 天空 |
| `2-bands.jpg` | 受光比三档分档，暗部偏石青 |
| `3-edges-gbuffer.jpg` | 用 GBuffer 法线（带法线贴图）加深度检测描边 |
| `4-normals.jpg` | GBuffer 法线可视化（前向像素为黑） |
| `5-edges-prepass.jpg` | 用 prepass 几何法线加深度检测描边 |
| `6-bands+edges.jpg` | 分档加 prepass 描边，岩彩雏形 |

## 环境

- 原生：WSL2 + WSLg，适配器是 **llvmpipe（软件渲染，Vulkan）**。功能正确性可信，**帧耗时没有参考价值**。
- 原生 X11 缺 `libxkbcommon-x11`，验证时用 `apt-get download` 把它和 `libxcb-xkb1` 解到 `/tmp/kiva-libs`，再通过 `LD_LIBRARY_PATH` 加载，没有改系统。
- Web：`cargo build --target wasm32-unknown-unknown` 编译通过；**WebGPU 运行时还没验证**，需要在用户启动的 trunk 服务上测（见文末）。

## 结论

### 1. 能拿到全部输入 ✅
深度、GBuffer、光照 ID、prepass 法线都能在 HDR 后处理里读到，固有色、自发光、法线、unlit 标志位按 Bevy 的打包格式解码正确（`4-normals` 里竹竿和角色的法线连续无噪点）。

### 2. 前向和延迟像素的区分：只能看光照 ID ⚠️
`1-classes` 里所有前向像素（地面草卡、镂空叶片）都是**黄色**：光照 ID 为 0，但 GBuffer **不是空的**，里面是后面那层地面的数据。所以：

- **必须用光照 ID 判别**（0 = 前向，非 0 = 延迟），**不能用 GBuffer 是否为空来判断**；
- 前向像素上的 GBuffer 固有色和法线都是错的，不能用来重着色。

竹竿是延迟渲染，可以正常重着色；需要材质自己处理的前向表面只有草卡、竹叶这类镂空和透光表面，和设计里"ForestMaterial 前向分支自己分档"的分工一致。

### 3. 分档在环境上效果好，在角色上太碎 ⚠️
- 地面和竹竿（`2-bands`）：阴影变成平涂的冷色色块，边缘清楚，已经接近目标效果。
- 角色：明暗被切得很碎、有黑斑。原因是受光比里混进了法线贴图和环境光遮蔽的细节，而且深色布料的固有色亮度很低，算出来的比值不稳定。
- 需要在后续阶段处理，可选手段（阶段 3 调参时验证）：
  1. 分档前对受光比做按深度保边的模糊，只保留大的明暗关系；
  2. 固有色很暗时退回到原始光照，或者放宽分档；
  3. 用 prepass 几何法线算一个平滑的 N·L，和受光比里的阴影项组合。

  如果这三种都压不住，角色再退回方案 B（角色材质单独做分档），环境继续用方案 A。

### 4. 描边要用几何法线 ✅（需要 NormalPrepass）
- GBuffer 法线带法线贴图，角色身上出现大量细碎的内部线条（`3-edges-gbuffer`）。
- prepass 法线是几何法线，角色轮廓干净，前向的草卡也能检测到（`5-edges-prepass`）。
- 新问题：草卡和竹叶被描成一团黑。对光照 ID 为 0 的区域只保留外轮廓（深度边缘），并且按距离减淡。
- 代价：要加 `NormalPrepass`。挂上以后写实画面没有可见变化（`0-off`），但会多一张渲染目标，多一些带宽。

### 5. 自发光和 unlit 可以识别 ✅
GBuffer 里的 emissive 和 unlit 标志位能读到，这类像素可以跳过分档，所以血条、刀光等特效保持原样（截图里的血条和刀光没有受影响）。

## 建议

**采用方案 A**，但对设计做三处修订：

1. 前向和延迟的判别**以光照 ID 为准**；前向像素不读 GBuffer。
2. 描边用**几何法线**，相机常驻 `NormalPrepass`。建议两种风格都挂，这样切换时不用重新编译管线；如果帧耗时不能接受，再改成只在岩彩时挂。
3. 角色的分档平滑作为阶段 3 的重点调参项。压不住就让角色单独走方案 B，环境保持方案 A。

## 待用户验证：Web（WebGPU）

1. 启动 trunk 服务，打开 `?level=bamboo`；
2. 点击画面进入捕获状态，按 F9 循环切换视图（窗口标题和控制台日志会显示 `spike <视图名>`）；
3. 重点看：画面没有报错或黑屏；视图 1 的颜色分类和原生一致；视图 6 的效果；顺便记录一下开关前后的帧率。

## 清理

`yancai_spike` 是临时代码，已在阶段 3 迁移进 `render_style/yancai_post.rs` 后删除，F9、`--yancai-spike-shots`、`--yancai-style-shots` 一并移除。
