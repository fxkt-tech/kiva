# 角色离线骨架与姿态验证

## Scope / Trigger

修改 `tools/wasteland/{extract_reference,import_reference,build_greybox,prepare_head_reference,refine_cape,finish_cape,export_runtime}.py`、测试专用
`avatar/wasteland_pose_tests.rs`，或把真实游戏关节姿态传给 Blender 时阅读。
这些工具生成离线制作/验收数据；运行接入另由严格导出器发布独立外观资源。
美术原型与尚未落地的方案仍放任务目录。

## Signatures

仓库根目录执行：

```sh
python3 tools/wasteland/extract_reference.py --verify
blender --background --factory-startup --python tools/wasteland/import_reference.py -- \
  --reference art/wasteland/reference-rig.json \
  --output art/wasteland/reference-rig.blend --verify
KIVA_POSE_EXPORT=art/wasteland/greybox/runtime-poses.json \
  cargo test --locked --lib \
  client::avatar::wasteland_pose_tests::export_runtime_poses \
  -- --ignored --exact --nocapture
```

灰模创建与可选头部来源准备：

```sh
blender --background --factory-startup --python tools/wasteland/build_greybox.py -- \
  --reference art/wasteland/reference-rig.json --output art/wasteland/greybox \
  --head-source art/wasteland/greybox/head-source.blend \
  --poses art/wasteland/greybox/runtime-poses.json
blender --background --factory-startup --python tools/wasteland/prepare_head_reference.py -- \
  --source human_base_meshes_bundle.blend --output head-source.blend \
  --archive human-base-meshes-bundle-v1.4.1.zip
```

`build_greybox --skip-renders` 只生成模型和数值报告；`--poses`、`--head-source`
为可选参数，头部默认从输出目录的 `head-source.blend` 读取。
灰模文件只在 `art/`，不进入运行 `assets/`。

生成参考文件时去掉 `--verify`。Windows Blender 从 WSL 调用时显式转换
文件参数为 Windows/UNC 路径；命令例子见 [工具说明](../../../tools/wasteland/README.md)。
Blender 导入器只允许独立后台进程，不能在用户交互场景中清空对象。
`export_runtime_poses` 为 ignored 测试，普通 `cargo test` 不写导出文件。

## Contracts

- `kiva-compatibility-reference-v1` 保留 52 个实际身体/手指 `gMan` 关节、
  合成参考根和 23 个独立结构锚点。父关系来自源动画明确层级，位置/轴来自
  目标自身 `sourceRestModel`，不能拿源动画 rest 替代目标 rest。
- AV03 不含完整部件骨父关系；没有证据的 twist/附件链只留元数据，不能猜链。
  结构锚点与皮肤关节不是同一份数据，当前小腿约有 3.1cm 差异，必须分别保留。
- JSON 矩阵是 4×4 行主序嵌套数组；原二进制矩阵是列主序。
  游戏模型空间：米、+Y 向上、−Z 向前；Blender：米、+Z 向上、+Y 向前。
- `kiva-runtime-palette-poses-v1` 的 `rest`、`poses[].joints` 使用相同的
  52 个无前缀关节名，不含合成根或披风辅助骨。`joints` 是当前绝对模型矩阵，
  **未乘 IBM**；`rest` 为原参考矩阵，与 IBM 分开。
- `poses[]` 记录 `name/pair/track/time/duration/source_clip/source_time`。
  业务信息读取独立字段，不解析展示名称。
  `hand_grips.R/L` 来自当前真实腕/MCP；`weapon_mount` 来自共享附件适配，
  不能当作原生腕轴。
- 走跑跳/防御场景复用 `PreviewRig`；攻击复用 `AttackReplay`；都执行真实
  `avatar::sync` 后读取直接挂在显示根下的 `PaletteJoint.Transform`。
  不是原始源动画采样，也不是读取未传播的 `GlobalTransform`。

设游戏参考 `R`、当前关节 `A`、游戏到 Blender 基变换 `B`、Blender 自身骨 rest `M`：

```text
B = [[1,0,0,0],[0,0,-1,0],[0,1,0,0],[0,0,0,1]]
P = B * A * inverse(R) * inverse(B) * M
B * R * inverse(B) = M * correction
```

EditBone 不保留原参考统一缩放，`correction` 必须显式保留和求值。
新网格使用自身绑定矩阵；旧 IBM 只作证据，不复制给新顶点。
普通 GLB 导出不会自动完成这套 rest-frame 映射。

灰模衣物厚度使用边界窄壁，不用整个面片复制来掩盖真实面数。
灰模报告同时计基础网格和求值后网格三角面，均须满足灰模工具的 40,000 上限；
不能只用未应用修改器前的数量宣称满足预算。辅助披风骨与身体核心骨分开，
灰模没有布料物理或最终火焰材质。

`head-source.blend` 保存官方 Blender CC0 头部拓扑及嵌入来源信息。
`source_blend_sha256` 与 `source_archive_sha256` 是不同来源层级，不能混称。
保留官方来源 URL、许可和角色适配记录，不把引用拓扑宣称为全部原创顶点。

## UV / PBR / Runtime Export

`texture_character.py` 从细化源另存纹理源；`export_runtime.py` 在独立后台
Blender 进程读取纹理源，输出 GLB 与 AV04。完整命令和材质节点约束见
[纹理流程](../../../tools/wasteland/README.md)及
[运行导出](../../../tools/wasteland/runtime-export.md)。

- 不透明图集必须有真实 UV0、非零面积、归一化范围和无正面积重叠；
  火焰周期贴图允许显式记录的重复 UV，不得冒称唯一图集。
- 最终发布不得使用 development 投影 UV 或常量材质绕过贴图。
  PNG（含 ORM）须校验文件头、尺寸不超过 2048 和大小不超过 16MiB。
  ORM 为 R=AO、G=roughness、B=metallic；基础色/发光按 sRGB，法线/ORM 为线性。
- 运行顶点使用游戏参考 R 为新 bind，新 IBM 为 inverse(R)；不是旧网格 IBM。
  来源中的静态披风辅助骨显式合并到 Spine2，并证明所有导出姿态的 delta 等价。
- UV 从 Blender 到 GLB/运行资源只翻一次 V，PNG 不翻。必须比较实际 GLB UV，
  不能仅用同一份中间数据互证。实际 GLB skin 与 Blender 求值顶点分别和量化运行 LBS 比较。
- 只有骨矩阵、辅助映射及实际顶点证明均通过，才原子替换运行二进制。

## 高精披风与运行 LOD

`refine_cape.py --source <原纹理源> --output <目录> [--preview-only]` 先建立独立
四边面制作源；`finish_cape.py --source <新制作源> --original <原纹理源>
--output <目录> --poses <runtime-poses.json> [--skip-renders]` 生成实际布料图、
贴合下缘的火边、高精源和 `zhuque-cape-runtime-source.blend`。
两步命令和文件用途见 [披风说明](../../../art/wasteland/cape-refined/README.md)。

- 按明确 component 面域替换旧披风/围巾/火边；保留域逐角点位置、UV、
  命名骨权重和 rig/sourceRest 指纹必须不变，不能镜像整个人修披风方向。
- 高精源保留新披风 Subdivision；运行源仅关闭该细分，仍保留 Solidify 厚度、
  同一控制笼和骨架。分别报告控制笼、高精求值、运行求值及完整人物面数，
  并比较高精顶点到运行表面的实际距离；不能把细分本身当作形体改善。
- `export_runtime.py --triangle-budget N` 为正式预算参数，默认 40000，
  仅允许整数 1..250000。预算始终检查整人求值后三角面，写入资产来源头、
  authored manifest 和转换报告。本次披风运行 LOD 显式使用 150000，
  高精作者源不能直接当作该 LOD 导出。
- 原三个不透明图集仍无正面积重叠。新披风三块外面板在不同图区；
  同布内外及薄 rim 可明确共享纹理，须记录范围，不能称整体唯一无重叠。
  共享区使用常量未遮蔽 AO=1；若要烘焙不同内外遮蔽，先分开展开。
- 新生成基色/发光图必须明确线性与 sRGB 编码，在写盘后作为 FILE 图重新加载、
  设置正确颜色空间并 pack，再做最终渲染和导出，核对真实 PNG 采样。
  不用 GENERATED 预览缓冲代替文件一致性证据。
- 制作预览只依赖实际运行支持的 PBR 子集：新布 Sheen=0、NormalMap Strength=1。
  不能靠未导出的 Sheen 抬亮黑布，然后宣称与运行材质相同。
- GLB 导出显式 `export_apply=True`，求值非 Armature 修改器、保留 skin。
  默认不应用修改器会丢 Solidify 内面/厚度；必须验证实际 GLB 三角面、
  skin/IBM/UV 和运行 LBS，不能仅证明导出器共享的输入数据。

AV04 保留 AV03 几何/骨架布局。材质 doubleSided 后新增 20 字节：
alphaMode:u8、opacity:f32、linear emissive:3f32、emissiveTexture:i16、
surfaceEffect:u8。alphaMode=0/1/2 对应 opaque/mask/blend；effect=0/1 对应
None/Ember。验证有限性、范围、索引与未知标签；旧 AV02/03/WP01 兼容测试必须保留。

Ember 只用于独立 AlphaBlend 发光火焰材质，不标在不透明披风/静态发梢上。
`character_effects` 缓存各反馈材质自身 UV/发光基线，禁止逐帧分配资产。
游戏只用有效模拟 delta；暂停不积累补帧。查看器按 Session.time 绝对求值，
暂停/拖动时间轴可复现。专用火焰图像 Clamp U、Repeat V；当前翻 V 后沿 V 正方向
偏移采样使图案由根流向尖端，改变 UV 约定时需同步验证方向。

朱雀浏览器 ID=3、查看器 ID=zhuque，默认李火旺及妹妹保持原选择行为。
游戏沿用全清单启动加载；查看器沿用按所选资源加载的门槛，二者不能混称延迟加载。

## Validation & Error Matrix

| 情况 | 行为 |
|---|---|
| 错误 magic、截断、非法索引或多余尾字节 | 提取失败，不造几何补齐 |
| 参考 JSON 与重新提取的字节不同 | `--verify` 失败，检查来源/转换差异 |
| Blender 重开后父关系、原 rest 元数据或锚点变化 | 验证失败 |
| 姿态缺关节、非有限/不可逆矩阵、握持原点不在实际腕 | 导出/回归失败 |
| 当前参考经 float32 Transform 分解与原 rest 存在微差 | 保留原 rest；使用已记录的 `2e-6` 分量容差，不强改 IBM |
| 灰模基础面数或求值后面数超 40,000 | 创建失败；降低冗余细分，不绕过求值后统计 |
| 有姿态数值/来源证明，但未检查新蒙皮和画面 | 只报告姿态导出通过，不声称新角色动画验收完成 |
| 正式预算非整数、≤0、>250000，或整人求值后超过显式预算 | 参数解析/导出失败；不发布运行资源 |
| GLB 缺少非 Armature 修改器结果/实际厚度 | 顶点证明失败；保留旧运行资产 |

## Good / Base / Bad Cases

Good：导出同一攻击时刻的实际目标 palette，按显式 rest 差映射到新绑定，
检查新网格、握点和刀刃轨迹；参考文件与姿态文件可重复生成。

Base：52 个原身体关节加独立结构锚点，保留原参考/IBM/修正，Blender 保存再打开验证。

Bad：只因骨名相同就复制四元数，或只看 Blender 摆姿后称游戏动作已兼容。

披风 Good：先看中性几何，再比较材质/运行 LOD；真实厚度进入 GLB，
藏在衣领内的接触与可见穿模分别记录。Bad：只加细分或用火光遮盖旧管状褶皱，
以面数增加代替美术审阅。

## Tests Required

- 提取结果逐字节复现；拒绝损坏输入。
- Blender 文件重新打开，验证原参考重建、关节头和结构锚点位置。
- `runtime_snapshots_use_complete_palettes_and_real_wrist_grips` 检查完整 palette、
  实际同步关节/握点，参考分解容差和动作覆盖。
- 显式导出重复运行，文件逐字节一致；当前导出 140 个姿态，包含两武器组的
  普攻三段/近战技能及移动防御场景，不等同连续战斗录制。
- 新蒙皮必须另查权重、真实变形、衣物穿模和游戏画面；姿态数据不含物理根高度，
  不能重复叠加动画根运动。
- 测试源码变化按客户端规范运行 `make check` 与 WASM Clippy。
- 灰模保存再重开后检查自有绑定、权重和求值后预算；用真实目标姿态验证
  Blender 实际关节矩阵，并将求值顶点与自有绑定 LBS 独立比较。
- 披风检查保留域/rig 指纹、实际厚布闭合与法线/零面积/权重，
  核对高精与 LOD 的实际表面距离和代表动作的求值顶点。
  用保留身体/外衣 BVH 量化净空，并结合实际前后/侧面/攻击图判读；
  隐藏下领接触不能冒称为全身零交叠，也不能直接当作可见外衣穿模。

## Wrong vs Correct

错误：`pose = source_quaternion`，或把 `A * IBM` 当作关节绝对姿态。

正确：保留 `R/A/M` 的含义与坐标系，按上述 `P` 公式求新姿态；
用新网格自身绑定变形，并将数值证明和画面验收分开记录。
