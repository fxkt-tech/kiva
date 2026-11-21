# 渲染风格

## 入口和登记

`client/render_style.rs` 拥有 `RenderStyle`（`Pbr` 写实、`Yancai` 岩彩）和 `LevelStyles`。风格只属于客户端表现，模拟不读取。

| 关卡 | 登记的风格 |
|---|---|
| 竹林 | 写实、岩彩 |
| 废站 | 写实 |

模型查看器不插入这两个资源，所有风格系统都不会运行。

## 切换

- 捕获期间按 F8 请求下一种风格（F4 仍是判定显示）。原生要求窗口有焦点且 `SimulationActive`，与 F3 一致。
- Web：`web/render_style.js` 只接受可信、非重复的 F8，捕获状态下 `preventDefault`；请求经 `window.kivaRenderStyle.takeStyle()` 由 Rust 单次消费，HUD 的 `#hud-render-style` 由 Rust `render(style, count)` 确认，文本不变时不写 DOM。
- 单一风格的关卡：请求被消费但不切换，HUD 显示「本关仅此一种」，窗口标题不显示 `[F8]`。
- `render_style::switched` 只在真实切换时为真，首次插入不算；`environment::apply_style` 和 `rendering::apply_style` 都用它守卫。切换不重建实体、不动相机和模拟。

## 每种风格各模块负责的部分

| 模块 | 写实 | 岩彩 |
|---|---|---|
| `environment::bamboo_lighting` | 原有的灯光、环境光、清屏色、雾、环境贴图强度（`setup` 也从这里读） | 纸色主光、石青补光、弱天光、纸色远雾 |
| `environment::forest_style` → `ForestSurface.yancai`（binding 106） | 全零，着色器走原路径 | x=1、y=主光照度/π、z=分档数；矿物色、颗粒、去高光和清漆、减弱法线贴图；前向的草卡和竹叶在材质里分档 |
| `rendering` 相机组件 | SSR | 去掉 SSR，挂 `YancaiShade` 和 `YancaiPaper` |

- 可切换关卡的游戏相机常驻 `NormalPrepass`（描边用几何法线），两种风格都挂，切换时不重编管线。单一风格关卡不挂，相机与原来完全一致。
- `SceneLight::{Key,Fill}` 标记竹林的两盏方向光，切换时就地改颜色和照度。

## 岩彩后处理（`render_style/yancai_post.rs`）

1. `YancaiShade`：自定义渲染系统，`Core3dSystems::PostProcess`、tonemapping 之前（HDR）。读取场景颜色、prepass 深度、延迟 GBuffer、延迟光照 ID、prepass 法线。
   - 前向和延迟**只按延迟光照 ID 区分**（0 = 前向）。前向像素下的 GBuffer 存的是被遮住的那层表面，不能读。
   - 延迟像素：去雾 → 受光比与 8 个同表面邻点平均 → 分档 → 再加雾。unlit、自发光（受击、蓄力、特效）和固有色过暗的像素保持原色。
   - 墨线：深度和几何法线的不连续。涉及前向区域时只描深度外轮廓；距离和雾都会淡化墨线。
   - 只撤销指数平方雾；其他雾形保持原样。
2. `YancaiPaper`：Bevy `FullscreenMaterial`，tonemapping 之后、FXAA 之前。矿物色板拉色、颜色交界积色、程序化纸纹，不使用贴图。

渲染世界的视图实体跨帧保留：`prepare_pipelines` 必须给没有 `YancaiShadeUniform` 的视图移除 `ShadePipelineId`，`shade_pass` 也要求 uniform 存在。否则相机切回写实后，pass 1 会用过期的 uniform 继续运行。

## 开发截图

原生 `--render-style-shots`：场景稳定后依次截写实、写实受击、写实蓄力、岩彩、岩彩受击、岩彩蓄力、写实，保存到 `target/render-style/` 后退出，并在日志里打印每段的平均帧耗时（窗口无焦点时 Bevy 会限帧，耗时不可用）。第一张和最后一张应当一致，只差风吹和待机动画。单一风格关卡带此参数会报错退出。

## 测试

- `environment` 往返测试：写实 → 岩彩 → 写实后，材质参数、灯光、环境光、清屏色、雾、环境贴图、SSR、后处理组件、`NormalPrepass` 与初始逐项相同；实体和资源数量不增长。
- `rendering`：只有可切换关卡挂 `NormalPrepass`；写实没有后处理组件。
- `render_style`：关卡登记、桥 ID、原生 F8 的焦点和模拟门控、废站不切换。
- `forest_material` 与 `yancai_post`：所有 WESL 在无 GPU 下组合，覆盖前向和延迟分支。
- `scripts/render-style-unit.test.cjs`：Web 桥的单次消费、按键过滤、HUD 文本和 DOM 写入。
