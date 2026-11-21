# 技术设计：可切换渲染风格

## 1. 现状（竹林）

| 层 | 现在的实现 | 位置 |
|---|---|---|
| 渲染路径 | 延迟渲染，Hdr，Msaa 关闭，Fxaa，深度 prepass + 延迟 prepass | `client/rendering.rs` |
| 反射 | SSR，只对粗糙度约 0.08～0.24 的表面生效 | `client/rendering.rs` |
| 环境材质 | `ForestMaterial = ExtendedMaterial<StandardMaterial, ForestSurface>`，WESL 写了前向、延迟、prepass 三套片元；竹叶等镂空材质强制走前向 | `client/forest_material.rs`、`embedded/shaders/forest*.wesl` |
| 光照 | 暖色主光 4800 lux（2 级级联阴影）、冷色补光 850 lux、半球渐变环境贴图、全局环境光、清屏色 | `client/environment.rs:120` 起 |
| 雾 | 指数平方距离雾 | `environment::fog()` |
| 角色 / 武器 / 特效 | 普通 `StandardMaterial`；受击和蓄力靠替换 `MeshMaterial3d<StandardMaterial>` 句柄；`CharacterEffects` 改 emissive | `avatar.rs`、`combat.rs`、`character_effects.rs`、`weapons.rs`、`impact.rs` |
| 快捷键 | F2 敌人模式，F3 外观，F4 碰撞调试 | `enemies.rs`、`appearance.rs`、`hit_debug.rs` |

Bevy 0.20 自带的 `FullscreenMaterial` 只绑定屏幕颜色、采样器和一个 uniform，**拿不到深度和 GBuffer**，所以岩彩的后处理要照着它写一个自定义渲染系统。

## 2. 总体结构

一种风格由三部分组成：

```
RenderStyle（客户端 Resource）
 ├─ 表面着色    ：怎样把"受光量"变成颜色（连续 PBR / 分档冷暖）
 ├─ 场景光照配置：灯光、环境光、环境贴图强度、清屏色、雾、SSR 开关
 └─ 镜头后处理链：附加在 GameCamera 上的后处理组件（写实时为空）
```

### 新模块与依赖方向

- `client/render_style.rs`（叶子模块，不引用其它客户端模块）
  - `enum RenderStyle { Pbr, Yancai }`，带 `ALL`、`id()`、`label()`；以后加风格只要扩展这个枚举并登记配置。
  - `Resource LevelStyles(Vec<RenderStyle>)`：当前关卡支持的风格。竹林是 `[Pbr, Yancai]`，废站是 `[Pbr]`。
  - 输入采样 `sample`：F8 → 在 `LevelStyles` 里循环；只有一种风格时不切换，只提示。
  - Web 桥：`window.kivaRenderStyle.takeStyle(): number`（0 = 无请求，1 = 下一种）和 `render(id)`。
- `client/render_style/yancai_post.rs`：岩彩后处理的组件、渲染系统和 WESL 着色器。
- `environment.rs` / `rendering.rs` 各自读取 `RenderStyle` 的变化并应用自己那部分配置，这样依赖方向是 `environment → render_style`，不会出现环。

### 切换流程（一帧内完成）

```
F8 / Web 请求 → render_style::sample → ResMut<RenderStyle> 改变
  → environment::apply_style  ：改灯光、环境光、环境贴图强度、清屏色、雾
  → rendering::apply_style    ：插入或移除 SSR，插入或移除 YancaiPost 组件
  → forest_material::apply_style：改所有 ForestMaterial 的 mode uniform
  → hud / 窗口标题显示当前风格
```

不重建实体、不改相机位姿和投影、不碰模拟。

## 3. 场景光照配置

把 `environment::setup` 里写死的数值提取成 `SceneLighting` 结构体：

```rust
struct SceneLighting {
    clear: Color,
    ambient: (Color, f32),
    key: LightSpec,   // 颜色、照度、是否投影
    fill: LightSpec,
    environment_intensity: f32,
    fog: DistanceFog,
    ssr: bool,
}
fn bamboo_lighting(style: RenderStyle) -> SceneLighting
```

- `Pbr` 的配置**原样搬运**现在的数值，`setup` 也改成从这里读，保证切回写实时逐项一致（有回归测试）。
- 两盏方向光加上标记组件 `SceneLight::Key` / `SceneLight::Fill`，切换时就地改参数。
- 岩彩的初始取值（截图评审时再调）：
  - 主光改成偏暖的纸色，补光改成石青色；
  - 环境贴图强度大幅降低，避免写实的天光渐变；
  - 雾颜色改成纸白偏青，远处的竹子淡出成留白；
  - 关闭 SSR（湿地反光属于写实效果）。
- 级联阴影保留，阴影的软边会在分档时被压成硬边。

## 4. 表面着色：采用方案 A（阶段 0 已验证）

> 阶段 0 结论见 `research/spike.md`：方案 A 可行，按下文三处修订执行。

难点是：现在大部分不透明表面走**延迟渲染**，光照是在 Bevy 通用的延迟光照 pass 里算的，**材质自己的着色器里拿不到最终光照**。所以"在材质里加一个岩彩分支"只对前向渲染的表面有效。

### 方案 A（采用）：屏幕空间重着色

在后处理里读取 HDR 光照结果和 GBuffer 里的固有色（base color），按像素算出"受光比"：

```
shade = 亮度(lit) / max(亮度(albedo), ε)        // 包含阴影、所有灯光、环境光
band  = 色阶(shade)                                // 2～3 档硬边
color = 色板映射(albedo) × 冷暖色(band)            // 暗部偏石青，亮部偏暖
```

- 优点：角色、武器、敌人、环境中所有延迟渲染的表面**一次生效**，不用迁移任何材质类型，受击变色和特效的句柄替换逻辑完全不用改；也不会改渲染管线，切换时不会因为编译着色器而卡顿。
- 高光：受光比大于 1 的部分直接截断，镜面高光和清漆自然消失。
- **前向与延迟的判别只看延迟光照 ID**（`R8Uint`，0 = 前向，非 0 = 延迟）。验证发现前向像素下的 GBuffer **不是空的**，存的是被遮住的那层延迟表面的数据，所以前向像素一律不读 GBuffer。
- 前向渲染的像素（地面草卡、镂空竹叶、薄片透光）由 `ForestMaterial` 的前向分支自己做同样的分档（见下），后处理原样保留。
- **角色分档平滑**：验证中角色的受光比混入了法线贴图和遮蔽细节，深色布料的比值也不稳定，分档后很碎。阶段 3 依次尝试：按深度保边模糊受光比后再分档；固有色亮度过低时放宽分档或退回原始光照；用几何法线算平滑 N·L 与受光比中的阴影项组合。三种都压不住时，**只让角色**改走方案 B，环境保持方案 A。
- 自发光：GBuffer 里有 emissive 信息，自发光像素跳过分档，保证技能特效和受击闪光的亮度。

### 方案 B（后备，仅在角色分档压不住时用于角色）：材质分支 + 强制前向

所有场景材质迁移到 `KivaMaterial = ExtendedMaterial<StandardMaterial, StyledSurface>`。岩彩下把材质的 `opaque_render_method` 改成前向，在材质着色器里调用 `apply_pbr_lighting`（粗糙度 1、无镜面）后再分档。

- 缺点：要改 `avatar`、`model_asset`、`combat`、`character_effects`、`weapons`、`weapon_models`、`companion`、`appearance`、`viewer` 里的材质类型；切换时会改管线，第一次切换可能有几帧物体缺失，需要预热；岩彩下 SSR 不可用（本来也要关）。
- 只在阶段 3 角色分档平滑失败时启用，并且只迁移角色相关材质。

### ForestMaterial 的岩彩分支（两个方案都需要）

- 在 `ForestSurface` 里加一个 `mode` uniform（现有的 `style: Vec4` 四个分量都已经占用，单独加一个字段）。
- 前向分支：岩彩下对镂空竹叶做平涂剪影和分档。
- 延迟分支：岩彩下关掉程序化湿度和清漆，改成矿物颗粒噪声，并把固有色往对应矿物色推：竹竿推向石绿、叶子推向石青和石绿、泥土推向赭石、石头推向青灰。
- 切换时遍历 `Assets<ForestMaterial>` 改 `mode`，只会重新上传 uniform，不重新编译管线。

## 5. 岩彩后处理链

`YancaiPost` 组件挂在 GameCamera 上；写实风格下移除这个组件，管线就和现在完全一样。

| Pass | 位置 | 输入 | 作用 |
|---|---|---|---|
| 1. 重着色 + 描边 | `Core3dSystems::PostProcess`，tonemapping 之前（HDR） | 场景颜色、prepass 深度、GBuffer（固有色、自发光、unlit 标志）、延迟光照 ID、prepass 几何法线 | 方案 A 的分档；用深度和几何法线的不连续检测出轮廓，画成墨线，线宽随距离变细、远处变淡；光照 ID 为 0 的前向区域只保留深度外轮廓，避免草卡和竹叶被描成一团黑 |
| 2. 纸纹 + 色板 | tonemapping 之后、FXAA 之前 | 色调映射后的颜色、纸纹贴图、色板查找表 | 映射到矿物色板（石青、石绿、朱砂、赭石、金、墨、纸白）；乘上麻纸纹理；颜色块边缘积色 |

- 自定义渲染系统参照 `bevy_core_pipeline::fullscreen_material` 的写法，额外绑定 `ViewPrepassTextures` 里的深度和延迟纹理。
- **描边用 prepass 几何法线**：GBuffer 里的法线带法线贴图，会在角色身上描出大量细碎内线。几何法线需要 `NormalPrepass`，它**常驻在游戏相机上，两种风格都挂**，这样切换风格不会重新编译管线；验证中挂上后写实画面没有可见变化。若阶段 4 测出帧耗时不可接受，再改成只在岩彩时挂，并接受首次切换的管线编译。
- 因此第 9 节"写实路径不变"的含义修订为：写实的**画面**和光照参数不变，相机多一个 `NormalPrepass` 组件。
- FXAA 放在最后，正好给墨线抗锯齿。
- 纸纹和色板在着色器里程序化生成（阶段 3 决定），不新增资源，资源清单不变。
- pass 2 只需要屏幕颜色，直接用 Bevy 的 `FullscreenMaterial`；pass 1 需要深度、GBuffer 和法线，用自定义渲染系统。
- 渲染世界的视图实体跨帧保留：pass 1 的 pipeline id 必须随 uniform 一起移除，否则切回写实后 pass 会用过期的 uniform 继续运行。

## 6. 输入与 HUD

- **F8**：F4 继续给碰撞调试；F5 在 Web 上会刷新页面，F6 会聚焦地址栏，F7 会触发光标浏览提示，所以选 F8。
- 原生：在 `BeforeFixedMainLoop` 和 F2 / F3 一起采样，要求窗口有焦点并且 `SimulationActive`（与 F3 一致）。暂停时可以通过 Web 桥切换。
- Web：新增 `web/render_style.js`，照抄 `web/enemies.js`：只处理可信、非重复的 F8 按键，在捕获状态下 `preventDefault`，请求由 Rust 单次消费，HUD 文字由 Rust 的 `render(id)` 确认。
- `index.html`：HUD 增加 `#hud-render-style`，操作说明表增加「渲染风格 · 写实 / 岩彩」一行。
- 窗口标题追加当前风格。

## 7. 废站与模型查看器

- 废站：`LevelStyles = [Pbr]`，F8 不切换，HUD 提示「当前关卡仅写实」。框架已经就位，以后给废站做岩彩只需要登记风格、补光照配置、处理 `Water` 和 `paint` 材质。
- 模型查看器：不插入 `RenderStyle`，所有 `apply_style` 系统都用 `run_if(resource_exists::<RenderStyle>)` 守卫。

## 8. 风险

| 风险 | 应对 |
|---|---|
| 方案 A 在 WebGPU 上读不到 GBuffer | 原生已验证、wasm 编译通过；WebGPU 运行时待用户在 trunk 服务上确认，失败时改用方案 B |
| 前向像素误用 GBuffer 数据 | 已确认只能用光照 ID 判别，并写成契约 |
| 角色分档太碎 | 见第 4 节的三步平滑；不行则角色单独走方案 B |
| 常驻 `NormalPrepass` 的额外开销 | 阶段 4 记录两端帧耗时；不可接受时改为只在岩彩时挂 |
| 竹林细节太多，描边过密 | 按距离衰减线宽和透明度，用深度阈值过滤小的不连续 |
| 级联阴影的软边在分档后出现锯齿 | 分档用 smoothstep 窄过渡，不用硬阶跃 |
| 两个全屏 pass 在 Web 上的开销 | 两端分别记录帧耗时；必要时把两个 pass 合并 |
| 自发光特效被分档压暗 | 自发光像素跳过分档 |

## 9. 回退

所有岩彩逻辑都在 `RenderStyle::Yancai` 分支后面。写实路径只做了三处改动：光照数值提取到 `SceneLighting`，两盏灯加了标记组件，相机常驻 `NormalPrepass`（不改变画面）；光照参数有逐项一致的回归测试。出问题时，把 `LevelStyles` 改回 `[Pbr]` 就能整体关掉岩彩。
