# 谪星·摧山：原武器资源准备

原物品ID 2311638。本目录仅供后续接入，未改变 Kiva 枪械、弓箭、动作、输入、WASM 或任何 runtime/include_bytes。原坐标/比例、原层级、UV、材质引用与源骨槽保留，未做 Kiva 握柄轴重定向或刚性烘焙。

models/区分唯一配置主部件与受原控制器引用的独立刀刃/特效输入；各GLB单独保存，不把全部输入叠加成一份默认外观，也不算新增WeaponTemp。原本静态的附件保持静态，不补造骨；有蒙皮的主体保留源骨序、逆绑定和规范化 glTF 蒙皮表示，以及原当前姿态。原导入姿态未必为bind pose。原配置部件ID为[3112012]；GLB源输入标签为main、source_variant_input_01、source_variant_input_02、source_variant_input_03、source_variant_input_04、source_fx_input_01、source_fx_input_02，共1个原蒙皮实例与6个原静态实例。

原物品属于谪星：RareSkin1022引用颜色、刃形、神威、凶魂溯四类词条。随包保留原记录、规则及实际材质/数值输入；原随机星格选择、刀刃变式、溶解与灵光行为未执行。基础几何本身不证明已还原默认完整刀形或任一随机星格结果，未知通道未猜成颜色或形变算法。



textures/保留全部实际绑定的原decoded PNG尺寸/通道（含normal、packed、em、ID等）；GLB基础色是中性PBR预览，原特殊shader、未知packed通道/法线约定及动态效果并未恢复。完整CAB及普通纹理GPU压缩数据留在外部研究目录；本目标必要的形变数值及非2D纹理原数据作为明确例外随包保存。

标记display_only的Float/形变PNG仅供显示，含明确clamp等预览转换；原数值以随包raw/npy与布局/SHA证据为准，未用8bitPNG冒充原Float/signed精度。

原绑定Cubemap等非普通2D纹理没有伪展平为PNG；完整原GPU/对象/face与mip布局证据随additional_source_evidence保存，尚未执行其原反射/FX材质行为。

portable_manifest.json用相对路径、SHA和字节数覆盖所有交付文件。source-evidence/原JSON中的绝对路径仅是历史来源定位，验证器不会访问原WSL或游戏目录；已包含小型原身份record.bin与源元数据。没有浏览器、截图或游戏执行验收。

在仓库根目录，用numpy/Pillow环境离线验证：

    python scripts/prepare-naraka-weapons.py --verify assets/prepared/naraka/zhexingcuishan

从已冻结研究export重新准备：

    python scripts/prepare-naraka-weapons.py --source /path/to/naraka --target zhexingcuishan
