# 角色运行资源导出

`export_runtime.py` 只在独立 Blender 后台进程运行，只读来源 `.blend`，生成 GLB 交换件、KIVAAV04 运行资源、材质 PNG、紧凑网格 JSON 和转换报告。

```sh
blender --background --factory-startup --python-exit-code 1 \
  --python tools/wasteland/export_runtime.py -- \
  --blend art/wasteland/textured/zhuque-v11-textured.blend \
  --reference art/wasteland/reference-rig.json \
  --poses art/wasteland/greybox/runtime-poses.json \
  --output art/wasteland/runtime \
  --runtime assets/characters/zhuque.bin
```

WSL 调 Windows Blender 时将文件参数转成 Windows/UNC 路径，见已有工具说明。`--python-exit-code 1` 确保 Blender 不会在 Python 导出失败时返回成功状态。

## 显式三角面预算

`--triangle-budget N` 是正式导出的预算参数，默认40000，接受1..250000的整数。
限制始终作用于**完整人物的实际求值后三角面总数**；超过指定值即失败，不发布运行资源。
`triangle_budget` 随来源信息写入AV04头、中间网格manifest与转换报告，报告同时保留实际`triangles`。
高精披风必须先完成真实几何审阅和稳定源确认，再显式选择足够的上限；例如经过确认的整人预算可传`--triangle-budget 120000`。
此参数不关闭真实UV、实际PNG、140姿态、Blender/GLB变形或AO验证，也不属于development标志。

当前已批准的精修披风运行源命令与首版命令相同，但替换以下参数：

```sh
--blend art/wasteland/cape-refined/zhuque-cape-runtime-source.blend \
--triangle-budget 150000
```

该运行制作源为139152三角形；`zhuque-cape-detail.blend`为430128三角形的高精编辑源，不能混称本次运行导出输入。Solidify内外/rim共享无差异布纹UV为记录在manifest的明确例外，ORM.R=1；身体其余图集合同不变。

材质/贴图列表按实际来源读取，不固定为首版的4材质/16贴图；新增独立披风材质及真实贴图沿用同一PBR合同。
每张PNG的2K/16MiB上限与运行二进制64MiB上限保持，不能借增加材质绕过这些限制。

## 制作输入

- 核心 `gMan` 骨来自已验证参考。每个来源网格有自己的 Armature 修改器与权重，使用自身 Blender bind。
- 必须有可用 UV_0，材质为一个 Principled BSDF。Base Color、Normal（可经 Normal Map 节点）、Emission Color 支持直接 Image Texture；复杂过程节点必须先烘焙，严格模式不猜纹理。GLB交换件显式将occlusionTexture与metallicRoughnessTexture绑定到同一ORM纹理索引/UV；保留导出器的图像索引和采样器，内嵌声明的完整ORM PNG，避免MR重打包丢失R通道AO。
- ORM 通过 `mat['kiva_orm_image'] = image.name` 明确声明，不猜任意图的通道。所有 PNG，包括 ORM，均验证尺寸不超过 2048、文件头和大小不超过 16MiB。基础色/发光按 sRGB，法线/ORM 按线性读取。
- `mat['kiva_alpha_mode']`：0 opaque、1 mask、2 blend；`kiva_opacity` 可选，默认 Principled Alpha。`kiva_alpha_cutoff` 默认0.5。
- `mat['kiva_surface_effect']`：0无动态效果、1 Ember，仅允许独立 AlphaBlend 火焰边缘材质；基础披风和发梢静态发光不打此标签。
- GLB显式应用非Armature修改器（例如Solidify厚度），保留Armature蒙皮；不把控制笼交换件冒充求值后的运行几何。
- 顶点按 loop 法线/UV 分裂并去重，最多四个权重按最大余数法量化为和255的字节。每个部件仅保存实际需要的调色板，第一部件另保留完整52骨元数据供手部语义绑定，避免每个小网格复制52个实体。

`--development-projected-uv` 和 `--development-material-factors` 仅用于早期接入检查，前者明确生成临时主轴投影UV，后者只输出Principled常量而跳过过程纹理。报告会记录这些标志；最终交付不得以它们绕过UV/贴图制作。

## 新绑定合同

运行顶点位置为 `inverse(B) * authored_world_position`。每个身体骨 `sourceRest = R` 使用现有实际目标参考，**新模型的 inverseBind 为 inverse(R)**。这不是复制旧网格的IBM。

```text
Runtime vertex = sum(weight * A * inverse(R) * game_vertex)
Blender vertex = sum(weight * P * inverse(M) * blender_vertex)
P = B * A * inverse(R) * inverse(B) * M
```

声明辅助骨映射：`cape_L/cape_C/cape_R -> gMan Spine2`。它们在来源中是该骨的静态子骨；导出合并权重，并在全部140实际姿态验证其变形delta等价。来源/GLB保留作者辅助骨，运行列表明确为52个身体骨。没有恢复或猜造不存在的原骨骼，也没有布料物理。

## KIVAAV04

沿用 AV03 顶点、部件、23结构锚点、原参考与inverse-bind布局。每个材质在 AV03 `doubleSided:u8` 后追加：

```text
alphaMode:u8 (0 opaque / 1 mask / 2 blend)
opacity:f32 (0..1)
emissive:3f32 (linear HDR, each 0..1000)
emissiveTexture:i16 (-1 or valid texture index)
surfaceEffect:u8 (0 None / 1 Ember)
```

显式小端，新增尾部20字节。旧mask标志与新alphaMode保持一致。旧AV02/AV03与KIVAWP01布局/读取不改变；新字段与效果标签有边界/有限性/错误值回归。

## 证明与发布

转换报告记录来源哈希、面数、材质/贴图、辅助映射与验证误差。全部140姿态检查真实 Blender 骨矩阵与辅助骨合并；代表性姿态比较实际 Blender 求值顶点与float32/byte权重运行LBS。另独立读取实际GLB节点/蒙皮/IBM及顶点，核对其绑定空间几何、UV约定、AO/MR绑定与声明ORM图的实际内嵌字节，并在五个已存在的真实姿态（含实际攻击时钟帧）比较GLB实际蒙皮与运行LBS；按来源物体和UV匹配缝顶点，避免只比较共享输入。

只有两项证明通过后，临时二进制才以原子替换发布到运行路径，避免用户正在运行的构建器读取半成品或失败导出。

运行选择新增 `Zhuque`，浏览器稳定ID3、查看页ID `zhuque`。ID1/2与默认李火旺不变；妹妹保持独立。游戏启动清单仍然要求加载全部外观，包括新增朱雀资源；默认李火旺不等于朱雀资源延迟下载。换装复用缓存并保留当前动作/物理状态，查看页沿用已有异步所选资源门槛。
