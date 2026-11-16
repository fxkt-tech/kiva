# 最终制作源与运行资产独立检查

## 结论与范围
冻结制作源、16张2K贴图、实际AV04与修复后的GLB检查通过，无此范围内的交付阻断项。浏览器真实GPU、全量Rust/Node/WASM和最终游戏动作验收由主会话/运行实现代理分别记录；本报告不替代这些验收，不宣称真实布料物理、面部表情或参考图照片级外观。

## 最终文件身份
- 制作源 `.blend` SHA256：b9847c7088bcb171e9dbccd8d13bdd42f1b26b7e3aff992a3e67a445ba17f59b。
- AV04 SHA256：2c15b4cde4bc5d7b4c97cb9dab27f7f24f0195fcc668903f81af02932f63959f；19,498,421字节。
- GLB SHA256：54f24775258b3dd0b4a7b2bc6f98fa416cd2664d0d8725dcfc9a1229f30b4d82。

## 源文件重开与差异检查
独立Windows Blender5.3后台读取原UV检查点与冻结源，不重建、不保存源、不操作用户服务。
- 最终4网格/4材质、21,310顶点、39,976求值后三角面，预算<=40k；最多2有效权重、归一化最大误差2.98e-8，自有绑定中性求值最大位移1.80e-7m，原52身体/手指关节与参考根/3披风辅助骨的合同保持。
- 三份不透明网格的坐标、面、权重、loopUV与原UV检查点指纹逐字节相同，复用此前独立231,042候选三角对的凸多边形相交审查：无正面积重叠。新的火焰1,260三角为明确允许的重复UV例外。
- 12张不透明PNG未改变；火normal/ORM也未改变，仅火base/emission两张色图更新。16张实际图均2048²、已打包、磁盘哈希和报告一致；base/emission标sRGB，normal/ORM标Non-Color。
- 通过来源component属性定位1,352个内袖面心样本，保持深色无发光，没有胸纹串入内袖。
- 查看新版火焰细节渲染，宽红橙焰羽已替代原细线；最终动态流向/透明叠加/暂停画面由主会话GPU检查。

## 实际AV04/GLB独立读回
- AV04严格模式：projected_uv为空、material_factors_only=false，头部记录的源/参考/姿态哈希与实际文件匹配。
- 独立按字段解析至精确EOF：23锚点、4材质/部件、16纹理、44,748序列化顶点、39,976三角面；索引、单位法线、UV、字节权重和自己的rest/inverse-bind有效，与authored-mesh真实字段逐项对应。
- 最大rest*ownIBM分量误差1.82e-7；法线长度最大误差1.16e-7。16张内嵌PNG与源PNG逐像素一致，最大8位通道差0。
- 仅火焰材质为AlphaBlend/SurfaceEffect1，HDR发光因子5.5；其他3材质为Opaque/Effect0。颜色空间标记符合4种图用途。
- 审查发现初始GLB没有occlusionTexture，导出代理已修复。独立读取最终GLB的JSON/BIN：4材质的AO与MR共用正确纹理索引/UV，strength=1；嵌入ORM payload逐字节等于真实源PNG，包含AO红通道，不仅保留G/B。

## 动作证明的准确范围
运行导出代理已报告全140快照的实际骨矩阵与辅助骨合并等价验证，并在明确列出的5代表帧对223,740实际Blender顶点/GLB顶点做量化LBS比较，最大约0.437mm。这里区分140帧矩阵检查与5帧逐顶点检查，不混称140帧逐点全量证明。独立资产读回核对该报告文件/哈希，不并发重复全量Rust或整个导出。

## 检查与证据
- `uvx ruff check tools/wasteland`、`python3 -m py_compile tools/wasteland/*.py`、`git diff --check`通过。
- `textured-independent-final.json`记录实际源重开/UV/贴图证据；`av04-independent-final.json`记录实际二进制/PNG/GLB AO读回。
- v1审查结果仅作已修复历史证据，最终身份以上述冻结哈希为准。
