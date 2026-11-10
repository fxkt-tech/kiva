# 李火旺与烛泪：Kiva 随包适配

发布版 `character.bin` 是 `KIVAAV03` 离线资产，包含红斗笠，共 22 个显示部件、51414 个顶点、66268 个三角形、23 个结构锚点、22 个材质与 9 张内嵌 PNG（最大 1024²）。它由已校验的原资源 GLB 转换，不在运行时读取游戏目录或下载资源。

衣装身份有原配置完整链：FashionBody **3401149 道诡异仙·李火旺** → ActorTemp **1210779** → ActorVisualPart **10447** → `ch_m_ming_wuchen_lv_s20`。发型为 HairItem **100407 道诡异仙·心素**；武器为 WeaponSkin **2312007 烛泪** → `wp_hengdao_manjianghong`，配原独立刀鞘。原诡道主件作为来源证据记录，因动态 FX 材质未移植，不默认加入角色。

红斗笠为独立挂饰 **3411276 道诡异仙·惘心笠** → OrnamentConfig **1102706** → `ch_ornament_hat_wuchen_s_lhwdouli.prefab`。OrnamentType **1013** 明确标记 `Head`，原 ActorBodyPartWeaponSlot **1200066** 标记“头部”；源包 key 为 `8/a/8a90c33afec6e79e`。四个原 LOD0 renderer 保留主体、附片及 cb/cf 的各自原调色板（10/7/10/10 骨），无材质的模拟代理不作为可见网格。

## 明确的适配边界

- 衣装保留原 FBX 骨架调色板、UV、四权重、源导入姿态和逆绑定矩阵。每个骨槽另外保存源当前模型矩阵；**源导入姿态可能不是绑定姿态，不能用 inverse(IBM) 替代当前 rest**。
- 心素原预制的 `m_Bones` 为空。30 个骨槽由原配置指定的 `l22_male_skeletonAvatar`、真实发丝节点原 local SRT 和原 IBMs 唯一数值匹配；6 条外部父链接为离线重建，非原序列化骨列表。逐槽候选数、误差阈值、原节点和重建链接保留在 `source-manifest.json`。
- 当前模型包含原模板的**通用基脸**，没有恢复李火旺 FacePreset 1415608 面容参数或 Cosmetic 1400595 眼妆。脸部肤色取原手/前臂实际 UV 对应主图的中值，解码为线性颜色；眼球沿原 UV 烘焙 Kiva 中性棕色虹膜与瞳孔，睫毛为深色。它们是明确的 Kiva 外观适配，不声称完整面容复刻。
- 发片与头皮的 RGB 是原 shader 数据，未当作颜色。仅保留原 alpha/cutoff，用中性黑材质检查；原红色发饰使用原主图。各向异性、SH、流向、染发、物理发丝与完整原专用 shader 未移植。
- 只接实际原主色/alpha，未猜接 normal、`mrav`、`skin`、mask 等图为 glTF/Bevy 法线或 ORM。材质粗糙度、皮肤透光、刀金属响应采用 Kiva 中性 PBR 参数。原正反面实例的绕序、源双面标记保留；睫毛双面为适配。
- 烛泪原 4 骨当前姿态先按原 LBS 完整烘焙，再把长轴从源 +Z 转为局部 +Y。柄中心 `[0,0,-0.395]` 位于原窄柄区域，是明确的 Kiva 手持校正；穗子随刚性武器运动。原 4 骨源 rest/IBMs 仍保存于来源清单，未声称原骨架不存在。
- 刀鞘为原静态源几何，左腰位姿由 Kiva 明确设置，跟随骨盆结构增量，不冒称原游戏装配偏移。
- 斗笠按已确认的 Head 装配槽挂到真实 l22 头骨，保留原帽子局部 Transform、骨序、UV、权重和 IBMs，无额外自制帽模型或猜测偏移。使用原主图/alpha、原 Cull 和 cutoff 的 masked PBR 适配；黑纱半透明、周期发光和布料物理未恢复为原 shader。
- 原帽 `HairLimitData` 包含 `m_HairHideMask=4`、`m_BoneHideThreshold=0.5`、310 字节空间数据。心素 `L_hairA_01` 的 mask4 与其对应，但隐藏粒度/空间变形算法尚未恢复。当前保留心素的全部原三角形，不按一个 mask 删掉整发型；可能存在局部发片与帽子穿插，来源清单保留原限制数据与未应用说明。

## 坐标与动作

高度以**衣装、完整通用头部和心素发型的实际源 LBS 联合边界**计算，统一缩放到 1.73m，脚底为零。GLB 已做过 Unity 基变换，本次只追加 Y 轴 180° 转向，不再重复 X 反射；左手锚点 X<0、右手 X>0、脚趾前向为游戏 -Z。

加帽保持先前发布的比例 `0.9128533132643297` 和原 C 矩阵，帽子不参与本体重新归一化。离线单独记录帽顶、戴帽总顶高与超出本体高度；当前原 Head 装配姿态下帽顶约 1.664m，最高心素发片仍为 1.73m，不为帽子改变角色、物理或相机尺寸。

23 个驱动名称和顺序沿用既有动画库；锚点取明确原 l22 骨位置。骨槽选择其真实结构祖先的驱动增量，原衣装/发型/脸的当前模型矩阵留作姿态基准。原比例与休息姿态不同于愁铃客/CC0，这属于结构重定向适配，不是原动作恢复。所有 Naraka 骨名使用 `naraka::` 前缀，避免误配燕云骨骼源动画。

设统一模型变换为 C，则转换严格保持：

```text
position' = C * position
inverseBind' = inverseBind * inverse(C)
sourceRestModel' = C * originalBoneWorld
```

运行时使用结构增量乘 `sourceRestModel'`，不要求 `sourceRestModel' * inverseBind'` 为单位阵。所有源顶点都做了转换前后 LBS 比较，并另检查实际 float32 与 weights/255 的序列化误差。

## 格式与复现

AV03 完全沿 AV02 小端字段顺序，只增加两处：材质 `hhhB11f` 后的 `u8 doubleSided`，以及每骨 `name + u16 driver + 16f inverseBind` 后的 `16f sourceRestModel`。矩阵按列主序写入；每顶点仍为 `8f8B`，四个权重字节和必须为 255。

```sh
/path/to/naraka/.venv/bin/python scripts/import-naraka.py --source /path/to/naraka --red-hat
/path/to/naraka/.venv/bin/python scripts/import-naraka.py --verify assets/lihuowang/character.bin
/path/to/naraka/.venv/bin/python scripts/import-naraka.py --self-test --verify assets/lihuowang/character.bin
```

解释器需要 NumPy 与 Pillow，与既有离线导入管线一致。导入器固定核对六件原 GLB SHA256，拒绝其他样本；源资源与身份记录的哈希、所用/未用部件及适配见 `source-manifest.json`。完整读回、原 LBS 数值误差、实际高度、材质/纹理/索引/矩阵/权重/UV/握柄轴和 EOF 验收见 `conversion-report.json`。

正式复现须带 `--red-hat`，该选项另核对原帽 GLB 的固定 SHA 和四条原配置 record 的 SHA/ID/UTF16 字段。不带此选项用于第一批无帽对照，不是当前发布版。AV03 布局没有为帽子增加任何字段，新增头饰仍为普通显示 part，全部骨槽跟随实际 Head 结构祖先。

本轮验收仅为离线导入和无窗口数值检查，按用户要求没有浏览器、截图或游戏运行验收。
