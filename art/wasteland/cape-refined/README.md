# 披风模型精修

此目录是新披风的可编辑制作源与实际 Blender 审阅图。依据 concept-v11 保留黑灰围巾、人物右肩覆布、不对称宽披风和火焰羽尾；修正旧模型左右镜像、矩形直垂、圆管褶皱和过于规则的尾缘。人物脸、头发、上衣、裤鞋及核心骨架未重做。

## 正式文件

- `zhuque-cape-detail.blend`：高精作者源，保留四边形控制笼、Subsurf1、真实 Solidify 厚度与绑定。
- `zhuque-cape-runtime-source.blend`：同一形体控制笼，只有新披风的 Subsurf 为0，保留厚度、材质、权重和原骨架。用于后续严格运行导出，不能把高模文件改名当作已验证运行资源。
- `cape-report.json`：保留部件指纹、真实求值面数、高/低表面距离、实际姿态 LBS 检查、来源和纹理哈希。
- `textures/cape_detail_{base,normal,orm,emission}.png`：4张实际2048² PNG。原人物三材质的12张图保持；火焰复用原四张PNG字节。

| 范围 | 三角面 |
|---|---:|
| 新厚布控制笼，未求值 | 48,128 |
| 新厚布高精求值（含真实厚度） | 389,440 |
| 新厚布运行求值（细分0，含真实厚度） | 98,464 |
| 独立火焰边缘与四根宽焰舌 | 7,360 |
| 原人物其余部件 | 33,328 |
| 高精全身求值 | 430,128 |
| 运行制作源全身求值 | 139,152 |

两个源均为7个角色网格、5种实际材质。高精源不是把旧披风简单细分：原披风/围巾/火焰面域被明确移除，新建3块带有大主褶、斜向压褶、软卷边和错落尖尾的四边形布面。164个保留来源部件的逐面位置、UV、命名骨权重指纹与原 b984 源匹配。

## 看图

- `cape-clay-back.png` / `cape-clay-neck.png`：新几何的中性审阅，不含火焰。与 `cape-before-clay-back.png` / `cape-before-clay-neck.png` 使用相同中性材质和灯光；只为旧披风面域换同种clay，不重做旧几何。
- `cape-geometry-comparison.png`：上述实际渲染的旧/新背面并排，不是生成概念图。
- `cape-final-{front,back,side,back-three-quarter,neck,hem}.png`：实际纹理/火焰审阅。
- `cape-runtime-hammer-attack.png` / `cape-runtime-crouch.png`：由原有真实目标姿态快照驱动的 Blender 显示，不是连续游戏录像，也不是布料物理。

## 复现两步

从仓库根目录执行。以下为实际使用的 Windows Blender 5.3 + WSL 路径；文件参数必须传 Windows/UNC 路径。

```sh
cape_blender_bin='/mnt/c/Program Files/Blender Foundation/Blender 5.3/blender.exe'
cape_project_win=$(wslpath -w "$PWD")

"$cape_blender_bin" --background --factory-startup --python-exit-code 1 \
  --python "$cape_project_win\tools\wasteland\refine_cape.py" -- \
  --source "$cape_project_win\art\wasteland\textured\zhuque-v11-textured.blend" \
  --output "$cape_project_win\art\wasteland\cape-refined" --preview-only

"$cape_blender_bin" --background --factory-startup --python-exit-code 1 \
  --python "$cape_project_win\tools\wasteland\finish_cape.py" -- \
  --source "$cape_project_win\art\wasteland\cape-refined\zhuque-cape-detail.blend" \
  --original "$cape_project_win\art\wasteland\textured\zhuque-v11-textured.blend" \
  --output "$cape_project_win\art\wasteland\cape-refined" \
  --poses "$cape_project_win\art\wasteland\greybox\runtime-poses.json"
```

第一步先做中性几何、保留域校验和两张短预览；第二步生成实际PNG、匹配新轮廓的火焰层、直接图像材质和独立高/低源。第二步会验证自身绑定与代表姿态、记录高/低差异，再保存中性源；后续渲染变姿不重新保存源。`review_cape.py --output ... --original ...` 可单独生成旧几何的匹配clay图；`--extra-renders --poses ...` 只读正式高模补正面/羽尾/攻击图，不修改源。`--finalize` 仅用于已有工作文件规范化材质并清理本工具未用数据，使用后必须重新记录来源哈希，不能对已冻结发布源无记录复跑。

## 材质、UV与验证边界

新cloth使用自己的atlas区域，不挤占旧头发/鞋/内袖UV。3块新布面的UV矩形互不重叠；**Solidify内外成对面及厚度rim有意共享相同布纹texel**。这是明确的复用例外，不宣称整个披风唯一UV或零重叠。ORM.R=1为明确未遮蔽常量，不在共享内外面上分别烘不同AO；若后续需要不同AO，应分开这些面域后烘焙。

基础色由线性约0.015–0.025的黑灰布色编码成sRGB PNG；生成后按FILE重新读入并打包，避免GENERATED缓冲与导出PNG显示不一致。法线/ORM为Non-Color；基础色/发光为sRGB。正常接入Base、NormalMap（Strength1）、ORM分通道与Emission，额外Sheen为0，使用运行支持的标准PBR子集。织纹为实际图像像素；宏观褶皱与厚度是真实几何。火焰只在独立AlphaBlend/Ember材质上流动，布面仍然不透明且UV固定；宽火边覆盖下部约23.5%的布长，并有4根非等宽焰舌。

高模顶点到低模表面的单向最近距离，长披布最大2.351mm/RMS0.153mm；围巾最大0.785mm/RMS0.146mm；覆肩片最大0.578mm/RMS0.117mm。这不是连续双向Hausdorff距离。运行制作源3个实际姿态（跑、锤普攻tick024、蹲伏）共160,404个真实求值顶点对自身绑定LBS，最大约3.91e-7m；严格GLB/AV04导出与实际游戏验收由运行导出流程另行证明。

下侧cowl有少量藏入原衣领的连接内层，包含约12.7mm局部负净空，实际前/侧/后近景中由覆肩/主布遮接；不宣称全局无交叉。没有实时布料碰撞或新的攻击规则。人物其他部分仍为此前第一版，未随本次披风工作变成精修人物。
