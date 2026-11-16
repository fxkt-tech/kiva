# Kiva 废土男主制作工具

当前提供**现有角色兼容性参考骨架**及 **concept-v11 灰模/蒙皮工具**。灰模已有新衣装、身体、手部、头发与披风网格，头部采用 Blender 官方 CC0 基础拓扑适配。文件不会进入 `assets/`，不随网页发布；尚未接入游戏或完成最终角色。

## 提取与复现

在仓库根目录执行，系统 Python 标准库即可：

```sh
python3 tools/wasteland/extract_reference.py
python3 tools/wasteland/extract_reference.py --verify
```

输入为随包的 `assets/characters/lihuowang.bin`、`assets/animations/naraka.bin`。提取器复用已有动画二进制验证器检查全部 44 个片段，并从默认男主实际 AV03 读取参考矩阵与原 IBM。JSON 不包含旧网格、纹理或动作轨。

在当前 WSL2 环境，用 Windows Blender 的**独立后台进程**生成和重新打开验证 `.blend`，不会触碰用户已打开的场景：

```sh
'/mnt/c/Program Files/Blender Foundation/Blender 5.3/blender.exe' \
  --background --factory-startup \
  --python "$(wslpath -w "$PWD/tools/wasteland/import_reference.py")" -- \
  --reference "$(wslpath -w "$PWD/art/wasteland/reference-rig.json")" \
  --output "$(wslpath -w "$PWD/art/wasteland/reference-rig.blend")"
```

在最后添加 `--verify` 可只打开已有文件并校验，不重建。其他平台直接用本机 `blender` 及本机路径。脚本拒绝在交互 Blender 会话中运行。

## 生成文件

- `art/wasteland/reference-rig.json`：52 个身体/手指变形关节、参考根、真实父关系、23 个结构驱动锚点、双手掌框与指链、原衣装各 palette 的独立 rest/IBM，以及输入 SHA256。
- `art/wasteland/reference-rig.blend`：53 骨参考 Armature，23 个独立锚点 Empty，以及完整 JSON 和说明 Text。没有 Mesh。
- `art/wasteland/reference-rig.report.json`：实际 Blender 版本、文件重新打开后的数值误差和阶段限制。

## 不能混用的数据

**结构锚点与实际皮肤 rest 不完全相同。** 现有男主的小腿皮肤关节与结构锚点相差约 31.108mm，脚/脚趾约 2.387mm。JSON 和 Blender 中两套数据都保留；不把它们“纠正”为同一个值。父关系只采用原动画中已明确的 52 个 `gMan` 关节；旧衣装额外 twist/饰物 palette 的矩阵保留为元数据，但其父关系不猜测、不创建。

**Blender EditBone 没有原源矩阵的统一缩放。** 原 `source_rest_game` 完整保留；转换到 Blender 的矩阵由 `bone.matrix_local @ kiva_source_correction_json` 重建。每骨保存原 rest JSON 和修正 JSON，原 IBM 在嵌入的完整参考中另外保存。骨尾沿原骨轴，仅作显示；它不是解剖链的子关节位置。

**新网格要有自己的绑定矩阵。** 原 IBM 只作来源证据，不能复制给新制作的顶点，不能以 `inverse(IBM)` 替代源参考姿态。后续离线导出必须显式处理该参考坐标、骨轴与修正；普通 Blender GLB 导出不会自动完成 KIVAAV03 的兼容映射。该参考文件本身没有新网格，参考往返数值通过不代表新角色游戏动作验收完成；灰模自身绑定的验证见下节。

来源与数值见 [rig-reference-provenance.md](../../art/wasteland/rig-reference-provenance.md)。

## concept-v11 灰模

已保存精简的 `art/wasteland/greybox/head-source.blend` 和来源报告，无需下载完整素材包即可复现灰模：

```sh
'/mnt/c/Program Files/Blender Foundation/Blender 5.3/blender.exe' \
  --background --factory-startup \
  --python "$(wslpath -w "$PWD/tools/wasteland/build_greybox.py")" -- \
  --reference "$(wslpath -w "$PWD/art/wasteland/reference-rig.json")" \
  --output "$(wslpath -w "$PWD/art/wasteland/greybox")" \
  --poses "$(wslpath -w "$PWD/art/wasteland/greybox/runtime-poses.json")"
```

`--skip-renders` 只构建和校验；`--head-source` 可明确指定精简头部来源文件。输出为 `zhuque-v11-greybox.blend`、中性正/侧/背/斜视、头部细节、两张独立压力姿势、四张真实共享游戏驱动姿势及报告。真实姿势的原名、时间点记录在 `runtime-render-index.json`；压力姿势是手工局部骨旋转示意，不能冒称游戏动作。

造型以腰际短外衣、每侧准确两层短外袖、纯色紧身内袖、无腰带连通阔腿裤、低帮鞋、围巾与不对称披风为灰模基准。衣装、头发、手部和身体为脚本原创曲面，不使用旧游戏人物网格。脸部来源、缩放/坐标转换与灰模眼部适配见 [灰模来源](../../art/wasteland/greybox/provenance.md)。

绑定采用新网格自身 Blender EditBone rest 的逆矩阵。每个真实姿势的目标骨模型矩阵 `A` 与原目标 rest `R` 经 `P=(B*A*B^-1)*(B*R*B^-1)^-1*M` 应用，`M` 是新绑定矩阵；不能把源骨缩放重复作用到新网格。脚本对全部 140 个快照检查实际 Blender 骨模型矩阵与预期值、用实际 palette 和自身绑定执行的 LBS 有限性；另检查中性姿态的零变形、权重上限/归一化和真实网格预算。

布壳使用少量真实边界翻边，不用 Solidify 将全部三角面翻倍；报告同时记录基础与 evaluated 三角面。四种人物灰模材质仅用于辨认层次，不是最终贴图/发光。三根披风辅助骨仅供静态继承与预览，不宣称布料物理或游戏中的新骨驱动已完成。

若需要从 Blender 官方完整 `Human Base Meshes v1.4.1` 重提精简头部：

```sh
blender --background --factory-startup --python tools/wasteland/prepare_head_reference.py -- \
  --source /path/to/human_base_meshes_bundle.blend \
  --archive /path/to/human-base-meshes-v1.4.1.zip \
  --output art/wasteland/greybox/head-source.blend
```

只提取 `GEO-head_animation_realistic` 与眼部四网格；去掉 Multires/原材质，将眼部变换烘焙到头部局部坐标。`source_blend_sha256` 和可选的 `source_archive_sha256` 分开记录，不混淆 ZIP 与 `.blend` 哈希。

## 独立细化构建

`--stage refined` 使用同一兼容骨架与验证管线，输出到独立目录；不覆盖灰模交付：

```sh
'/mnt/c/Program Files/Blender Foundation/Blender 5.3/blender.exe' \
  --background --factory-startup \
  --python "$(wslpath -w "$PWD/tools/wasteland/build_greybox.py")" -- \
  --stage refined \
  --reference "$(wslpath -w "$PWD/art/wasteland/reference-rig.json")" \
  --head-source "$(wslpath -w "$PWD/art/wasteland/greybox/head-source.blend")" \
  --output "$(wslpath -w "$PWD/art/wasteland/refined")" \
  --poses "$(wslpath -w "$PWD/art/wasteland/greybox/runtime-poses.json")"
```

`refined_surfaces.py` 创建实体发束、软布外上衣/双层袖、披巾叠褶、朱雀浅浮雕与鞋部件；`refine_head.py` 通过同一个平滑映射处理 CC0 头部与眼部，添加投射在实际脸表面的静态眉形，不增加面部骨或表情系统。细化阶段仅对高密度眼球网格做 50% 减面，将预算用于人物可读形体；身体骨长与参考矩阵不改。

新增 `authored-relaxed-review.png` 是手工放松双臂的审阅姿态，便于与概念图比较，不能当作游戏原动作。中性 `.blend` 仍保存原绑定姿势。`shoe-structure-detail.png` 单独展示鞋头、鞋舌、鞋带、后跟与抓地结构。

`--quick-preview` 缩短视觉迭代，只渲染正面/斜视/细节/放松审阅，但保留完整数值检查；正式一批不传该参数，生成正侧背、压力及四个实际游戏驱动快照。细化材质里的细微程序纹理仅标为 `PREVIEW ONLY`，不是最终 UV/PBR 贴图。后续纹理制作需要另行生成真实 UV0 和 PNG 图集。

## UV 与实际 PBR 图集

`texture_character.py` 读取已细化文件，另存 `textured/`，不会覆盖前两阶段：

```sh
'/mnt/c/Program Files/Blender Foundation/Blender 5.3/blender.exe' \
  --background --factory-startup \
  --python "$(wslpath -w "$PWD/tools/wasteland/texture_character.py")" -- \
  --source "$(wslpath -w "$PWD/art/wasteland/refined/zhuque-v11-refined.blend")" \
  --output "$(wslpath -w "$PWD/art/wasteland/textured")" \
  --resolution 2048 --samples 8 \
  --poses "$(wslpath -w "$PWD/art/wasteland/greybox/runtime-poses.json")"
```

`zhuque-v11-textured.blend` 是最终纹理制作源，包含四种实际 ImageTexture 材质与打包 PNG；外部同名 PNG 也保留在 `textures/`。`zhuque-v11-uv.blend` 只是在烘焙前保存的中间文件，不能作为最终运行导出源。`--unwrap-only` 用于单独验证 UV，不执行 PBR 烘焙。

三个不透明材质先按角色合并为皮肤、灰布/朱雀纹、黑灰发/鞋/内袖/不透明披风。原部件名保存为 JSON 对照及面域 `kiva_component_id`；合并不会丢掉 Armature 或原顶点组权重。保留真实身体骨架与独立披风辅助骨。第四个材质只作用于新增的前后各 15 条贴合披风曲面的下缘火焰带（共 1,260 三角面），不让基础披风或头发 UV 滚动。

不透明网格三角化后做 Smart UV 和图岛打包，逐个检测归一化范围、非零三角面积及正面积重叠。少量冲突三角独立切图岛后重打包，不能跳过检测。当前三张不透明图集重叠为零；火焰带明确共用周期 V 平铺图，属于有意重复，不冒称唯一图集。

实际纹理来自 Blender COLOR、NORMAL、EMIT、AO、ROUGHNESS 烘焙，所有最终 PNG 为 2048²。ORM 按 R=AO、G=粗糙度、B=金属度打包，当前所有源面明确为非金属。皮肤细微色差/微法线、布料克制磨旧与发/眼/布不同粗糙度进入图集，不依赖程序预览节点运行。火焰 tile 是代码创作的周期噪声颜色/透明度/发光图；其法线使用已明确的平坦切线法线，ORM 为常量非金属。

最终材质直接连接 Base、NormalMap、ORM 分通道和 Emission 图像；火焰另将图像 alpha 接入 Principled Alpha，HDR 发光强度为 5.5。`kiva_surface_effect=1`、`kiva_alpha_mode=2` 只标在火焰材质，其他材质均为 0；实际时间驱动由运行端负责。UV 保持 Blender 原约定，纹理不手动翻转，运行导出器负责其已验证的一次坐标转换。

结果见 `uv-report.json`、`texture-report.json` 与 [纹理制作记录](../../art/wasteland/textured/README.md)。
