# 指定模型与随包格式

## Scope / Trigger

修改 `avatar`、`model_asset`、`weapon_models`、`weapons`、`companion`、女性握持 IK 或两个模型转换脚本时阅读。
运行时只依赖随包二进制，不引入 GLTF／JSON 解析器，也不读取游戏目录。规则与动作时间线仍由 simulation 决定。

## Signatures

- `Appearance::{Choulingke,Lihuowang,Bailingmiao}`：默认值为愁铃客；玩家选择默认李火旺，白灵淼仅用于 Companion。
- `AvatarAssets::{shared,cached}`／`AvatarAssetCache` 复用身体网格、PNG、材质变体与逆绑定资产。
- `WeaponModel::shared(world,pair,mode)` 以 pair＋WeaponMode 对应四个固定缓存槽，保存模型及 `[Vec3;2]` 双手握点；`weapon_models::spawn(world,parent,pair,mode,Mount::{Weapon,Spine})` 按挂接类型创建显示实体。
- `uv run --offline --with numpy --with pillow python scripts/import-prepared-models.py [--verify]`：从冻结的 prepared 包生成三组运行资源；verify 在临时目录重建并逐字节比较。
- `uv run --offline --with numpy --with pillow python scripts/import-melee-models.py [--verify]`：单独生成伏浪锚／谪星摧山，不改前一脚本及旧五组运行资源。

## Contracts

### 手臂、掌轴与源指轨重定向

`avatar::{driver_pose,source_driver_pose}` 对上臂／前臂的既有运动旋转作最小摆动校准：将旋转后的自身参考臂段对准实际采样解剖方向，再按目标自身段长FK。不能把 donor T姿态的全局增量直接作用到目标A姿态偏移，也不能只检查骨长而忽略约49°／57°的方向偏差。显式妹妹握点IK在此后求解；资产参考模式仍返回原sourceRestModel，不把原件或IBM改成donor姿态。

`avatar/hand.rs` 用实际手腕与食指／中指／小指MCP建立语义掌框。CC0读取其声明的手姿态框，源层读取当前源掌框，分别校准到目标自身掌参考。`attachment_pose` 从当前真实palette的腕／MCP重建握持坐标，语义scale固定ONE；+Y由腕伸向掌内，近战轴校正后的柄头朝食指／拇指侧，柄尾朝小指。保留既有+0.055m偏移，但方向须与实际掌心一致，不能保留旧错误手腕四元数自证正确，也不能只挪武器弥补错误皮肤。

燕云40个finger节点为30个加权指节和10个无权重Nub。李火旺／白灵淼各按名称映射前三节gMan指链，Nub仅用于源端节方向，不能压缩为目标末节或虚构目标骨长。源当前指节解剖框相对源当前掌框传到目标掌框，保留滚转并校准目标原生轴，不能直接复制不同骨架的local quat或继续叠加目标参考握拳角度。

源权重先在各级局部旋转混合，再从已混合实际手腕按自身局部偏移FK；全局关节位置插值会缩短指链。实际Naraka原生指节+X方向有真实父子偏移和末节LBS顶点证据；仅作方向，不构造不存在的Nub／长度。不完整单palette指链使用资产内真实同名rest并集补足计算祖先，保留当前palette已有rest，拒绝歧义，不创建额外渲染关节。

无源CC0保留原目标参考指型，参考模式完全绕过动画校准；愁铃客完整源palette仍直接保留原骨轨。旧愁铃客全局混合的中间掌形可能不刚性，挂点读取当前实际MCP，不能以腕四元数冒充真实掌框。本次未添加原作独立twist控制器：现有35片段没有相应twist动画轨，已知twist骨仍随臂段驱动，不宣称恢复未知分摊。

白灵淼衣装、莲母、女性基脸来自 `assets/prepared/naraka/bailingmiao`，运行输出为 `assets/bailingmiao/character.bin`（KIVAAV03）。
23 个结构锚点取自身女性 Lianmu 骨架；各部件保留各自原骨序、逆绑定和 sourceRestModel，不能套用男性比例／索引／脸。女性自身的结构参考驱动原有动作增量，双手 IK 保留自身上臂／前臂长。
陡角度枪姿可用自身锁骨最小转动与表现层骨盆高度调整适配握持，向上站立受双腿真实可达范围限制；双腿 IK 保持自身骨长和脚掌／脚趾落点、朝向，不移动显示根或权威射击源。跨越臂长极限时锁骨转动连续，不能用二值阈值直接跳到最大转角。武器瞄准在妹妹局部坐标中求旋转，避免世界空间最短弧在不同 yaw 下产生翻滚。
实际导入姿态可不同于 bind pose，不能把 sourceRestModel 替换为 inverse(IBM)，或把蒙皮结果强制抵消为单位矩阵。

引冰落／羿之影输出 `assets/{yinbingluo,yizhiying}/weapon.bin`（KIVAWP01）：magic8、u32 长度来源头、纹理与 AV03 材质表、6 个握点 f32、部件表。
部件为 name、u8 mount、primitive 数；每 primitive 为材质索引、顶点／索引数，顶点 8f32（位置／法线／UV），索引 u32。数字均小端，字符串为 u16 字节长度＋UTF8。
mount0（Mount::Weapon）在远程以发射源为局部原点、朝 -Z，在近战以握柄为局部原点、长轴 +Y；mount1（Mount::Spine）已转为女性 spine_03 相对坐标，不能再次挂在枪口下。具体字段与当前数值见任务模型合同和随包 conversion-report。

武器先求真实源层级／全部 skin 槽的当前 LBS，再烘焙为刚性显示网格；引冰落保留原 joint1 的 135 顶点，羿之影保留九个有效槽的变形。它们原本有骨骼，静态烘焙不等于原模型无骨骼。
quiver／arrowbag 的位置是 Kiva 身体挂接适配；独立 related 箭几何缺少原材质和 Actor 装配，不能称为第四件已配置部件。武器握点／尺寸为适配，不能通过挪动 simulation 射击源补偿。

伏浪锚／谪星摧山分别为 `assets/{fulangmao,zhexingcuishan}/weapon.bin`，同用 KIVAWP01；两件近战都以握柄0／长轴 +Y 接入原近战手腕校正，显示总长适配为 1.10m，不改变攻击范围。
伏浪锚完整保留主体原四权重和附属 448 顶点的单索引／implicit weight1 LBS。谪星由原 main＋四份互补非 FX 部件组成，源 controller／规则把它们关联为刀背、刀刃、人头、刀柄；准备包的 source_variant_input 名称不等于四把互斥整刀。
这些原静态输入集中在原点，未知 Float／shader 变形不能据预览 PNG 猜测。本次采用逐部件有记录的 Kiva 固定装配，含来自 HeadPosition 的参考设计锚点；不能把该字段称为原 Actor 装配矩阵，也不声称恢复源默认／随机星格、完整刃形算法、溶解、灵光或 cubemap 效果。
手持与 FlyingAxe 复用完全相同的近战网格／材质句柄。绳索只有共享线段几何，起点在本帧 avatar／stickman 附件同步后由 PlayerView 根变换与实际握柄局部变换合成；不读取滞后一帧的 GlobalTransform，不用 Body＋固定高度替代手腕，不写物理状态。

`model_asset` 共用 AV03／KIVAWP01 的材质和 PNG 读入；基础色使用 sRGB，已证明的 normal／surface 使用线性空间，未知 packed／ID／SH 图不猜接颜色或 ORM。
白灵淼女性基脸无原绑定贴图，运行时中性皮肤／眼睛／睫毛色为明确的 PBR 适配；不宣称专属捏脸、妆容、染色／发丝／布料 shader 或原作武器动画已恢复。
prepared 原件与清单冻结，更新转换只写新运行目录。来源、哈希、LBS 误差、变换／尺寸和未恢复项随各运行资源一起记录。

## Validation & Error Matrix

| 情况 | 行为 |
|---|---|
| 数量超过剩余字节／错误格式／非有限值／非法索引／多余尾字节 | 有界读入断言失败，不能截断后继续造几何 |
| 源清单哈希变化 | 离线导出／verify 失败，不静默接受新源 |
| F3／暂停换装 | 仅替换哥哥 AvatarPart，妹妹固定白灵淼，敌人固定愁铃客 |
| 切组／出场结束／逻辑妹妹失效 | 原资产复用；隐藏或回收相应显示树，不遗留附件 |

## Good / Base / Bad Cases

Good：双手握点由实际武器变换得到，女性 palette 的手腕抵达握点，源枪口仍落在权威 source_position。
Base：默认李火旺，Q 召唤白灵淼持引冰落；换组后持羿之影并显示背部附件。
Bad：仅移动抽象 RigPose 手坐标而 palette 仍垂手，或只读取武器 joint0 导致其他源骨骼几何错位。

## Tests Required

手部回归须先用真实源指轨／实际palette抓到原“约91.6°→0°”丢失，再检查双Naraka／双手／五指的实际child或已证明末节+X方向与源child／Nub在当前掌框中的方向一致，不能只断言非零旋转。源权重0/.25/.5/.75/1检查自身指节长度、连续性、实际LBS响应和握点轴；柄头方向须用signed轴·(indexMCP−littleMCP)>0，不能取abs掩盖倒握。CC0声明掌轴、原参考／IBM、愁铃客完整源矩阵、妹妹真实握点与游戏／查看一致性均回归。

真实 GLB→二进制验证包含原 LBS、量化误差、权重／索引／法线／UV、female own rest、PNG 和可复现字节；不只测构造的单位绑定。
World 回归检查实际 female palette 握点／臂长／几何变形、陡角度时脚掌与腿长、枪口0和 -Z、不同 yaw／pitch 下的武器上轴、两条真实三发技能与暂停、F2／死亡／F3 隔离，以及反复施放的实体／网格／纹理／材质数量稳定。
近战额外验证两套哥哥身体的实际手腕轴、握柄与 +Y、四槽缓存和 source mesh 数量、链斧手持／飞行句柄一致、绳索本帧端点，以及普通 Q／完整拉近联动／取消／死亡／暂停回收且资产数量稳定。
运行 make check、WASM Clippy；入口或嵌入资源改变后运行临时目录 Trunk 构建。真实画面观察单独记录，数值检查不替代画面验收。

## Wrong vs Correct

错误：直接复制李火旺骨索引／手位置、把原 ID 当发色、把准备包说明中的“未接入”改写成精确原作效果已还原。
正确：独立女性参考和部件原 rest，原件保持冻结，运行说明列清实际接入与材质／动画近似。
