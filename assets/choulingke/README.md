# 愁铃客角色资源

`character.bin` 是从本机《燕云十六声》的 `C3/Boss/ylk` 资源离线转换的 Kiva 显示资产，格式 `KIVAAV02`。玩家、实战敌人和大号演武靶子共用；包含 13 个身体/穿戴部件及独立专属长刀，保留原 UV、每顶点四权重、独立绑定调色板和17个原始材质槽。相邻的 `yqzj/hengdao` 资源已排除。主长刀是 `weapon1`，`weapon2` 为刀鞘，`3_weapon` 为腰侧刀具，未发现用于当前两种远程武器的合适替换。

模型、纹理和新增原作动作属于原作，不能按项目自制或 CC0 资产处理。已恢复986节点的原主骨架与35个具名原作动作；移动、蹲伏、枪姿、旋转与倒地等仍回退到另行保留许可证的 [CC0 动画库](../animations/README.md)。主骨架已恢复，布料/发丝物理和独立附件的完整原挂接仍未恢复；刀鞘/腰刀保留适配。原始主色/法线贴图身份已确认；混合贴图主要通道通过原 DXIL 家族反汇编核实，转换与局限见下文。

标准身高归一到1.73m、朝向转换为-Z，32张主色/法线图片及17张由原混合图转换的线性 ORM 图片缩到最大512px嵌入，共49张（原始1024/2048图不修改）。二进制内的 JSON 头保留每一部件的源 GUID、包偏移、哈希、材质槽、贴图哈希和转换说明。原文件保存在项目外：

`/home/asuka/kiva-asset-research/choulingke/`

- `extracted/full/`：14个最高细节原始 Mesh 与来源清单。
- `extracted/metadata/`：Skin 绑定矩阵、Mat 参数及来源清单。
- `extracted/textures/`：原始纹理容器、DDS、PNG 与导出状态。
- `extracted/skinned/`：各部件和组合角色的 glTF 2.0 蒙皮导出。
- `material_bindings.json`：真实材质槽和贴图 GUID。

重新生成（依赖研究目录中的已验证 Mesh/Skin 解析器和 NumPy/Pillow）：

```sh
/home/asuka/kiva-asset-research/choulingke/.venv/bin/python scripts/import-choulingke.py \
  --source /home/asuka/kiva-asset-research/choulingke
```

用 `--max-texture-size 1024` 可导入更高纹理分辨率；不改变研究目录原件。游戏运行时只加载本目录的随包文件（Web 为页面同源路径，原生为 `assets/`），不读取 Windows 安装目录，也不从其他来源下载角色资源。GPU蒙皮、材质缓存和动作适配由 `client/avatar.rs` 管理；物理位置、角色大小、攻击结算仍由模拟/玩法层拥有。

## 原 shader 与角色材质

角色最初已经采用 StandardMaterial PBR 与原主色/法线贴图；本轮升级补充原混合图及材质家族响应，没有把原先的表现描述成“没有 shader”。

只读检查游戏 `LocalData/Cache/LocalShaders/dx12/` 后，已从编译缓存恢复可核查的 DXBC 容器、DXIL 指令和常量/资源反射。容器格式为变体数量、每项16字节哈希与4字节长度表，再接 `ZZZ4` 包装的 LZ4 数据。`GBufferGenPixelShading` 有5257个变体，`GBufferGenPixelShadingCEDiamondS` 有183个；已定位 DetailArray、基础 PBR、Hair、Skin、Diamond 等家族代表变体，用微软官方 DirectXShaderCompiler 的 `dxc -dumpbin` 离线反汇编。没有执行游戏，也没有进程注入。

研究目录的 `shader_research/` 保留：

- `recover_shaders.py`、`recovery_report.json`：可重跑的解包/反汇编、缓存和变体哈希。
- `material_recovery_report.json`：工具版本/来源、指令证据和已恢复范围。
- `detail_0.ll`、`basic_13.ll`、`hair_37.ll`、`skin_47.ll`、`GBufferGenPixelShadingCEDiamondS_0.ll` 及对应 `.dxbc`：原编译结果与反射。

反汇编核实 `tMixMap` 的 R 为光滑度、G 为金属度、B 为 AO。导入器按家族处理：DetailArray 应用源光滑度/金属度缩放与偏移；Hair 应用 `cSmoothOffset`、`cAOOffset` 与独立 `cMetallic`；Skin 根据源 A 通道的 profile 标记屏蔽相应金属度。随后转换为 Bevy 的线性 ORM：R=AO、G=粗糙度、B=金属度，粗糙度由 `1-smoothness` 得到。主色保持 sRGB，法线和 ORM 保持线性，原透明度保留。转换检查可单独执行：

```sh
/home/asuka/kiva-asset-research/choulingke/.venv/bin/python scripts/import-choulingke.py --self-test
python3 /home/asuka/kiva-asset-research/choulingke/shader_research/recover_shaders.py
```

`KIVAAV02` 保留17个源材质槽。材质记录为名字字符串、三个 `i16` 贴图索引（主色/法线/ORM）、一个 cutout 标记及11个 `f32`：粗糙度、金属度、反射、各向异性、清漆、清漆粗糙度、漫透射、主色 RGB、透明裁切值。角色各实例只切换缓存材质句柄来显示受击/预警，不修改共享贴图或其他角色材质。

适配继续使用 Bevy 的 GGX/PBR、GPU 蒙皮与正常阴影流程。头发采用源 `cBaseColor` 染色；布料降低介质反射、保留原混合图中粗糙度与金属饰件分布；皮肤使用弱漫透射；刀刃用适量清漆响应。漫透射及清漆值属于本项目对源家族的近似，不能称为原作同一 BRDF。原切线/发丝和布料各向异性 BRDF 约定未恢复，因此 V2 的各向异性字段保留但写入零，不启用猜测的 GGX 各向异性。

真实 WebGPU 对比曾发现：开启猜测的各向异性后，粗糙度中位数0.882、金属度不超过0.004的披风出现银白硬亮伪影；只关闭该项后伪影消失，原 ORM、光强与其余参数均保留。CPU 休止姿态/材质解码测试不足以检出这类 BRDF 画面问题。纹理采样另使用线性过滤和 U/V Repeat：源上衣存在负 U，裤子与刀刃存在 U>1；Clamp 会把这些区域压成边缘颜色。

**尚未恢复或移植**：原 HLSL 正文、每次原作绘制的精确变体选择、完整发丝双高光/布料绒毛/皮肤 SSS、detail-array 多层混合、脸部化妆、Diamond 内部/闪烁/专用发光效果。现有原编译库和通道证据不等于原作渲染器已完整移植。

## 竹林材质、贴图和光影

[environment/manifest.json](environment/manifest.json) 记录14张随包 PNG 的原 GUID、`.6` 物理 VFS、原/随包 SHA、尺寸与颜色空间。来源通过已恢复 Repository 条目中的 Model→Material→Texture 引用链核对：

- 竹子：`tree_xinzhuzi_01_lod0` Model `d3e653d3-c39d-4087-a92a-5ab31f13d1ec`，引用 `Mesh_tree_xinzhuzi_01_lod0`（`c99b5702-9e4d-4b7e-8657-4f09b54812aa`）及材质 `4072463d-d75a-489e-b340-9346c24b24f3`；纹理位于 `G/Res/Tree/TEX`。
- 地面竹叶：Repository 的 `grass_zhuye_01_lod0` Model `66f63b3e-9f90-47f8-b8be-be8294ad49f2` 引用 `Mesh_grass_zhuye_02_lod0`（`b5c535cc-b3be-4f66-aae2-7f03d19c5b83`）及材质 `a70ddef4-ad1f-42c9-b5b7-0bb9700efec7`；纹理位于 `G/Res/Grass/TEX`。名称差异按实际引用记录保留。

| 随包文件 | 原纹理名 | GUID |
|---|---|---|
| leaves.png | tree_leaf_sb_zhuzi_01_d | cdb24925-7412-4e33-a8a1-7402a44d2577 |
| leaves_normal.png | tree_leaf_sb_zhuzi_01_n | 2c603ec7-60f8-4ae6-ae34-1604040ba726 |
| bamboo.png | tree_bark_xinzhuzi_01_d | 9bd517c7-c0d2-490b-a1b5-3d992e789ce2 |
| bamboo_normal.png | tree_bark_xinzhuzi_01_n | d403e3d2-2986-4ca0-9940-296cd0221b75 |
| litter.png | grass_zhuye_01_d | ecc01030-6087-4f2f-bd5e-adc6fefeaace |
| litter_normal.png | grass_zhuye_01_n | ff9fb06c-07ca-43c5-8e9d-d5047456b207 |

新增土壤 `A_weight007_soil_d/n`、草苔 `ue_HG_jn_duoren_A_weight010_grass_d/n`、阔叶草 `grass_caocong_01_d/n` 和细草 `grass_gancao_d/n` 共8张。GUID、包偏移、原/随包SHA、atlas裁切及染色适配逐张记录在manifest。新图重跑：

```bash
/home/asuka/kiva-asset-research/choulingke/.venv/bin/python scripts/import-bamboo-environment.py \
  --source /home/asuka/kiva-asset-research/choulingke
```

主色/法线角色由引用链及 `_d`/`_n` 名称核对，环境材质 `.4` 参数本体未在本地索引命中；没有把环境 `_m` 或法线 alpha 的未知打包套用为已验证的角色 ORM。原始纹理、DDS、来源记录与重跑工具位于研究目录 `extracted/environment/`、`environment_textures.py`。

`client/environment.rs` 用这些贴图装饰重建竹林：整簇叶片 atlas 铺在大卡片上，竹皮按重复 UV 采样，地面竹叶使用透明裁切；原 atlas alpha 保留。环境有20个聚合网格、7个共享材质，竹叶/阔叶草/细草共12个风动组。草丛按确定性随机斑块散布，保留出生、目标与路径净空；gancao灰黄源图按五组UV切出单叶并加绿色/枯草色，属于Kiva美术适配。地面/竹干/岩石继续消费玩法层原几何，碰撞竹干不移动；只有装饰枝叶以数厘米幅度缓摆，暂停时冻结，标准阴影随同一显示变换更新。枝叶放置、竹皮 UV、散落竹叶和风动均为本项目制作，并非恢复的原作地图实例或 Pivot Painter 动画。

自写 [forest.wesl](../shaders/forest.wesl) 是 Bevy 0.20.1 的 ExtendedMaterial 片元扩展，组合为 WGSL 后保留标准 PBR 输入、透明裁切、光照及雾/色调映射后处理，同时支持前向和 deferred 分支。地面使用土壤与草苔原主色/法线的世界空间分层，保留材质粗糙度，再按mask形成干土/湿土/平滑积水；积水metallic为0，湿土不参与SSR以抑制高频闪点。重复不透明纹理生成sRGB线性平均和法线重归一的mip；alpha卡片保留原atlas。主片元和自定义prepass共用近叶裁切，避免隐藏枝叶提前写深度。光照使用主方向光、冷色填光和半球环境反射，配合指数平方距离雾；材质的环境粗糙度、透光与清漆参数是适配值，不冒充原作材质块。

相机使用deferred、HDR、FXAA、DepthPrepass和实际SSR pass；透光裁切叶草前向绘制。SSR只能反射屏幕内且已进入deferred颜色缓冲的物体，前向植被/皮肤与屏幕外物体存在限制。当前较高相机俯角下，非金属水面黑衣倒影较弱，不能称为原图同构图复刻。对照和GPU读回证据见[竹林画面修正](../../docs/plans/bamboo-rendering-reference.md)。

CPU 检查覆盖颜色空间、UV/切线有限性、原碰撞几何一致、装饰风动/暂停；shader 组合测试覆盖前向与 deferred 导入。真实 WebGPU 的 shader 编译、阴影、透明枝簇和新旧同镜头画面对比另由浏览器验收，不能由无窗口测试替代。

## 原骨架和原作动作

`animations.bin` 为独立 `KIVASR01` 随包资产，[animation_manifest.json](animation_manifest.json) 记录所有原VFS、包位置、SHA、原/保留轨数、省略原因以及Kiva片段裁剪。原主骨架 `Char/c_77_boss_yulingke/yulingke.skeleton` 有986节点，父子关系、局部/全局矩阵与身体Skin调色板逐名核对；`global × inverseBind` 的误差不超过约1.62e-4（源XML的六位小数舍入）。subskel只描述软骨链/碰撞，不冒充人体主骨架。

原注册129项中，120个文件本地定位并严格解析 `CHAR::ANIM` v15到完整CHANNEL_DATA边界，9个路径未命中。30个完全具名的愁铃客动作导入；另5个具名共用动作从父骨架注册定位，提供卸势、跳起/空中/落地与闪避，共35个。其余90个愁铃客文件的FC/FD/FE/FF共享名字引用仍未解开，未猜骨名绑定。原Cue/AudioSync及资源原件保存在研究目录；不能把片段名字当已恢复的技能槽映射。

当前长刀普攻/重刀采用 `skill_ylk_1` 的0.9–1.8秒片段；卸势采用男玩家11_qiang的 `skill_huajie_01`；跳跃三段采用男19_shoujia的共用动作；闪避采用NPC `skill_shoujia_dodge_f`。后三类是燕云原作共用人体动作，并非已找回愁铃客专属版本。长度/速度、朝向和片段选择属于Kiva适配；共用动作保留愁铃客静止骨长，用首键相对位移适配。原技能命中帧、连击取消和槽→动作配置尚未恢复。

解码保留原毫秒时间戳、稀疏位置/旋转/缩放键，没有重采样成手工姿态。旋转解开20bit×3的smallest-three量化，运行时线性插值位移/缩放、slerp旋转，缺轨回原局部rest；不符原parent或缺失的独立附件轨明确省略。完整父树求值后逐Skin骨名驱动GPU蒙皮，平面根位移剔除；地面保留腰身上下变化，空中根高由模拟决定。原手腕相对rest的旋转增量校正到既有握柄参考，枪口/双手瞄准仍走原CC0/IK。两种来源平滑混合，暂停/HitStop冻结时钟，不把动画回调写回伤害或物理位置。

重建命令（需要已保留的外部研究原件和NumPy）：

```sh
/path/to/research/.venv/bin/python scripts/import-choulingke-animations.py --source /path/to/research
```

小端资产含986个具名节点、带符号非零rest尺度及父索引、局部TRS，随后逐clip保存毫秒时长、骨索引及T/R/S稀疏键。运行解码验证版本、边界、名称唯一性、祖先索引、有限值、单位四元数和时间顺序。无需运行游戏脚本、读取安装目录或启动游戏。

枝叶可读性适配：仅竹枝叶片元在相机3m内裁除、3–5.5m固定像素抖动渐入；使用Bevy真实view.world_position，远处叶簇、地面落叶和竹干不受影响，碰撞不动，原atlas阴影保留。当前相机未启用独立DepthPrepass，未来启用须同步该通路的可见片元裁切。
