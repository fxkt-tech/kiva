# 永劫无间资源准备

这些资源仅供后续接入使用，**不加入当前游戏的角色选择、武器玩法或运行时加载**。当前已接入的愁铃客、李火旺和惘心笠保持原样；这里的新文件不会通过 `include_bytes!` 编入现有角色资产。

## 资源目标

| 资源 | 原配置身份 | 准备目录 |
| --- | --- | --- |
| 玉玲珑「道诡异仙·白灵淼」衣装 | FashionBody `3402139` → ActorTemp `1210776` → ActorVisualPart `11602` → `ch_f_ming_yulinglong_lv_s16`；不是「新喜」`3402140` | `bailingmiao/` |
| 「道诡异仙·莲母」发型 | HairItem `100408` → 原女性发型 `female_hair_yulinglong_lv_s19` | `bailingmiao/` |
| 鸟铳「引冰落」 | WeaponSkin `2311228` → WeaponTemp `3110620` → `wp_flintlock_bingxueniaochong` | `yinbingluo/` |
| 弓箭「羿之影」 | WeaponSkin `2310557` → WeaponTemp `3112195 / 3112196 / 3112197` → `wp_bow_wuzhengong`、`_quiver`、`_arrowbag` | `yizhiying/` |
| 阔刀「伏浪锚」 | WeaponSkin `2310478` → WeaponTemp `3112349` → `wp_blade_chuanmaolv1` | `fulangmao/` |
| 斩马刀「谪星·摧山」 | WeaponSkin `2311638` → WeaponTemp `3112012` → `wp_saber_xingtian` | `zhexingcuishan/` |

准备工作包括原模型、UV、骨序、逆绑定矩阵、实际绑定的原尺寸 PNG 及来源记录。弓的原骨骼和弦部件保留；独立附件不在缺少原装配信息时强拼为一件模型。原 ID / 流向 / packed 图按原通道保存，未证明其用途时不当作颜色或 ORM。

白灵淼使用其自身的标准女性体型和女性源骨架；通用女性脸与专属面容参数分别说明。交付保留源单位与姿态的 GLB，没有生成 AV03，没有接入当前动作、武器挂点或角色入口，也不复用李火旺的男性比例、面容或骨索引。

源文件读取与转换在离线工具中完成，游戏安装目录只读。整包和原 CAB 保留在项目外研究目录，普通贴图不重复复制 GPU 压缩数据；谪星形变所需浮点纹理与实际绑定的 Cubemap 则保留原精度 / 原始数据及布局证据，不能用 8 位 PNG 展示图替代。本目录只存目标可用资源和可核对的来源信息。精确面容、原各向异性发材质、动态布料 / 发丝、开火 / 拉弓动画和原 FX 等未恢复部分，在各资源说明中明确列出，不能把静态模型准备称为完整玩法或原作效果复刻。

按本轮要求不启动浏览器、不截图、不启动游戏或常驻开发服务。验收限定为文件与来源哈希、原骨骼 / LBS 数值、权重 / 索引 / 材质引用、PNG 通道和离线格式完整性。

## 文件与使用

- [白灵淼 / 莲母说明](bailingmiao/README.md)：原衣装、莲母及独立通用女性脸三个蒙皮 GLB；27 张原材质图、3 张原妆容图和来源证据。专属脸型及原染色 shader 未复刻，原图与中性 alpha 派生图分别存放。
- [引冰落说明](yinbingluo/README.md)：鸟铳原层级、两套蒙皮和 9 张原材质图。原单索引、隐式单权重格式显式展开为 glTF 表示，保留真实 joint1，不能称为原四权重数据。
- [羿之影说明](yizhiying/README.md)：原配置中的弓体、quiver、arrowbag 三个部件，保留九骨弓与原静态附件、9 张原材质图。另附 `related/` 中的独立箭几何，材质及 Actor 装配未恢复，不当作第四个已配置装备部件。
- [伏浪锚说明](fulangmao/README.md)：两个原最高 LOD 蒙皮、原贴图与来源数据；主体原四权重与附属单影响展开分别说明，仅资源准备。
- [谪星·摧山说明](zhexingcuishan/README.md)：基础部件与六组独立刀刃 / 特效输入分开保存，另附原控制规则、浮点形变纹理和 Cubemap 证据。没有把互斥输入叠成默认外观，中性材质表示不等同于已恢复星格外观、变式选择或原动态效果。NPY 按位保留源数据，包括原有的非有限值，不能直接当作已解码的位移坐标；6 张展示 PNG 不替代数值数据。
- [总索引](catalog.json)：各组入口清单的 SHA、资源路径及准备状态。各组 `portable_manifest.json` 覆盖交付文件的相对路径、字节数和 SHA。

模型可供后续标准 glTF 工具导入；当前 GLB 中性 PBR / alpha 表示不等同于原游戏完整着色器。`source-evidence/` 中的历史绝对路径只表示来源，不是校验依赖。目录内 `.gitattributes` 禁止自动换行转换，避免跨机器检出破坏原件和清单的字节哈希。

在仓库根目录，用带 NumPy / Pillow 的 Python 离线校验：

```sh
python scripts/import-bailingmiao.py --verify assets/prepared/naraka/bailingmiao
python scripts/prepare-naraka-weapons.py --verify assets/prepared/naraka/yinbingluo
python scripts/prepare-naraka-weapons.py --verify assets/prepared/naraka/yizhiying
python scripts/prepare-naraka-weapons.py --verify assets/prepared/naraka/fulangmao
python scripts/prepare-naraka-weapons.py --verify assets/prepared/naraka/zhexingcuishan
```

校验已准备的文件不需要游戏安装或本机研究目录。重新整理时使用各组说明中的 `--source` 命令，输入是已提取的研究成果目录，不能直接用游戏安装目录代替。
