# 角色 UV 与 PBR 制作源

使用 **`zhuque-v11-textured.blend`** 进行最终运行导出。它包含可编辑网格、兼容骨架与四种完整 ImageTexture 材质；16 张 2048² PNG 同时保存在 `textures/` 并打包进 `.blend`。前阶段 [灰模](../greybox/zhuque-v11-greybox.blend) 与 [细化模型](../refined/zhuque-v11-refined.blend) 均保留。

`zhuque-v11-uv.blend` 是烘焙前中间文件，便于检查 UV，不是最终纹理/运行资源。没有使用开发用临时投影 UV 或概念图作为模型贴图。

## 实际产物

| 材质 | 内容 | UV/表现 |
|---|---|---|
| 0 skin | 脸、手及身体、眼白 | 唯一打包 UV；真实皮肤色差/微法线/粗糙度烘焙 |
| 1 cloth | 灰色外衣、裤子、鞋部件及朱雀纹样 | 唯一打包 UV；朱雀铜色直接进入同一 Base 图集 |
| 2 charcoal | 黑灰头发、内袖、鞋、围巾、不透明基础披风、眼/眉 | 唯一打包 UV；材质区粗糙度不同，发梢有静态发光掩码 |
| 3 fire_fringe | 披风下缘前后各 15 条实体火焰带 | 周期 V 纹理有意复用；alpha 混合与 HDR 发光；只有该表面参与运行时流动 |

每种材质有 Base、切线 Normal、ORM、Emission 四图，共 16 张。ORM 的 R 为 AO、G 为粗糙度、B 为金属度；当前所有部分明确为非金属。Base/Emission 用 sRGB，Normal/ORM 用 Non-Color；图像直接连接 Principled，不需要运行端猜测程序节点。

最终火焰采用前后两层贴合实际披风曲面的铺设，修复早期直线插值卡片陷进布面的遮挡。图案为宽幅红橙主体和少量金色内芯，HDR 发光强度 5.5；没有改变不透明披风的图集或滚动它的 UV。`textured-back`、背面斜视和 `textured-fire-detail` 是修复后的实际 Blender 画面；历史修复记录单独命名，不当作最终画面。

前三组纹理通过 Blender COLOR/NORMAL/EMIT/AO/ROUGHNESS 实际烘焙得到。皮肤保留克制色差和微法线，布料仅作细微磨旧，不添加破洞；眼、发、布和橡胶的粗糙度不同。第四组火焰颜色/透明度/发光 tile 为原创周期函数生成，V 首尾连续；火焰薄带本身渐尖，不以滚动整张披风图片替代几何。火焰使用明确的平坦切线法线和常量非金属 ORM。

## 可验证数字

- 人物含火焰带共 **39,976 三角面**，最终四个材质网格。
- 三张不透明图集逐三角检查后 **零正面积 UV 重叠**；均在 0–1 内且面积非零。
- skin 11,172、cloth 13,484、charcoal 14,060 三角面。cloth 有 16 个局部冲突三角改为独立图岛后重打包；没有删除源几何。
- fire_fringe 1,260 三角面有意重复 UV，不套用不透明图集的唯一性声明。
- 原部件名称/角色保存在报告 JSON，并以面域 `kiva_component_id` 对应，合并材质不等于丢失来源标记。

详细检查、源 `.blend` SHA256、最终 PNG SHA256 在 `uv-report.json` 和 `texture-report.json`。实际运行导出/量化/全部动作兼容验证由独立导出报告继续记录，不能只凭烘焙图或 Blender 数值宣称完整游戏验收完成。

## 坐标、绑定与来源

保持 Blender 原 UV 约定，PNG 不翻转；运行导出器承担一次明确的 UV 坐标转换。新网格使用自身绑定，身体核心骨不改；火焰带沿用所在披风下缘的辅助骨权重。不透明披风 UV 固定，`kiva_surface_effect=1`、`kiva_alpha_mode=2` 仅标在第四种火焰材质。

服装、发、鞋、朱雀纹样、火焰带及所有纹理源函数为脚本创作。头部基础采用已记录来源的 Blender 官方 CC0 拓扑，骨架兼容参考来源见 [参考来源](../rig-reference-provenance.md)，脸部适配见 [细化模型说明](../refined/README.md)。没有读取旧游戏角色网格/纹理或使用未经说明的外部 PBR 图片。

这是实际可导出的第一轮纹理模型，不宣称已完成手工精绘皮肤、发丝各向异性、布料物理或与概念图逐像素一致。Blender 预览只展示静态火焰 tile；真正的时间流动与暂停行为由运行端明确实现并验证。

冻结制作源 SHA256：`b9847c7088bcb171e9dbccd8d13bdd42f1b26b7e3aff992a3e67a445ba17f59b`。只读渲染工具 `render_textured.py` 重开并补渲染放松姿态、火焰近景和真实攻击，检查该源文件 hash 不变。

复现命令见 [制作工具](../../../tools/wasteland/README.md)。
