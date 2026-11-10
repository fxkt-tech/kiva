# 伏浪锚：原武器资源准备

原物品ID 2310478。本目录仅供后续接入，未改变 Kiva 枪械、弓箭、动作、输入、WASM 或任何 runtime/include_bytes。原坐标/比例、原层级、UV、材质引用与源骨槽保留，未做 Kiva 握柄轴重定向或刚性烘焙。

models/按原配置明确部件分别提供GLB。原本静态的附件保持静态，不补造骨；有蒙皮的主体保留源骨序、逆绑定和规范化 glTF 蒙皮表示，以及原当前姿态。原导入姿态未必为bind pose。原配置部件ID为[3112349]；GLB源输入标签为main，共2个原蒙皮实例与0个原静态实例。

伏浪锚主体保留原四权重/四索引流；附属448顶点原为UInt32/dim1单索引、无独立权重通道，仅将原索引与implicit weight1窄展开为glTF四槽。原索引字节、布局、骨路径CRC与逐骨LBS数值核验均在source-evidence/additional及model-checks内，不把派生展开称原四权重。



textures/保留全部实际绑定的原decoded PNG尺寸/通道（含normal、packed、em、ID等）；GLB基础色是中性PBR预览，原特殊shader、未知packed通道/法线约定及动态效果并未恢复。原GPU压缩数据/CAB留在外部研究目录，未塞入本包。

portable_manifest.json用相对路径、SHA和字节数覆盖所有交付文件。source-evidence/原JSON中的绝对路径仅是历史来源定位，验证器不会访问原WSL或游戏目录；已包含小型原身份record.bin与源元数据。没有浏览器、截图或游戏执行验收。

在仓库根目录，用numpy/Pillow环境离线验证：

    python scripts/prepare-naraka-weapons.py --verify assets/prepared/naraka/fulangmao

从已冻结研究export重新准备：

    python scripts/prepare-naraka-weapons.py --source /path/to/naraka --target fulangmao
