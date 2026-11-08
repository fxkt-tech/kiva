# 模型命中与操作目标

## 1. 范围与职责
`client/picking.rs`拥有模型部件、CSS屏幕几何及有序候选；`controls.rs`按当前操作选择合法目标，模拟仍复核攻击/技能规则。当前用户要求只检测模型，血条与名字不提供命中区域。
几何不只保留敌人，不判断射程或具体技能；存活、当前实体及操作者团队视野仍为公共可见性门控。未来友方技能使用同一候选并接入自己的资格谓词，不复制几何。

## 2. 接口
- `picking::candidates(world: &mut World, point: Vec2) -> Vec<Entity>`：point为CSS坐标，返回去重且稳定排序的实际模型候选。
- `PickGeometry`标记可选模型子树；刚性Mesh3d部件使用Aabb和GlobalTransform，蒙皮部件使用Mesh保留的JointAabb及当前`joint GlobalTransform * inverse_bindpose`。技能特效/范围/弹丸不得放进该子树，UnitGroundBase装饰不参与；基地平台属于实际模型。
- `Pickable {offset,half_size}`为根profile，只在模型Mesh边界尚未就绪/无渲染夹具时fallback，不替代已存在的实际部件。
- `PointerState.candidates`保存公共候选，`hovered`为当前悬停结果；完整HudView增加`hovered:string|null`，既有持久`selected`不变。
- 逐帧`kiva-camera`增加`enemy_hovered:boolean`，保留center/footprint。pointer.js只据此改变`#game-cursor[data-target=enemy|none]`的箭头/剑SVG。

## 3. 坐标与命中契约
输入在固定步前读取最后已显示的Camera、GlobalTransform与Aabb，不维护第二份位置缓存，不用当前未插值UnitPose重算身体。
逐个模型部件投影8个角点并取屏幕凸区域，明确区域内命中优先；边缘最多6 CSS px容错，不能换成世界单位或再乘DPR。不能用整单位/长斜武器的大矩形吸住空白处。
每单位聚合最佳部件命中；按明确命中、屏幕距离、可见深度及稳定Entity身份排序，移出有限容错后返回空。
蒙皮资源以Dynamic bounds加载并保留MAIN_WORLD数据；逐关节投影沿用同一凸区域和6px容错，不用整个绑定网格框替代。
非有限或零比例隐藏的关节拒绝命中；导入需压缩每个可见primitive实际引用的顶点，避免共享accessor中的隐藏附件扩大边界，详见[导入模型](imported-models.md)。
controls默认悬停优先合法敌人；右键与Enemy技能从公共候选找敌方目标，然后沿用preview/模拟验证。普通点击保留选择语义，悬停不自动发命令或修改持久selected，超距攻击仍可追近。
A+左键命中合法敌方模型时发AttackTarget并更新selected，与右键锁同一候选；无合法敌人且有地面点时才发原AttackMove。不能把“点模型”降为点地后自动索敌，也不能把旧selected强行当作新空地点击目标。

## 4. 门控与错误矩阵
- 非有限位置、无有效相机/投影/viewport：空候选。
- 死亡、不可见、隐藏模型部件、已清理实体：拒绝；不能用profile fallback让被隐藏的模型重新可选。
- 击飞/骨骼动作：按真实部件位置，旧地面标记不可命中；标签仍锚定唯一显示根。
- 面板/小地图或UI手势捕获：先执行既有UI隔离，不发世界命令，光标保持箭头。
- 暂停/失焦/失锁/普通恢复/重选：pointer所有者清旧敌方形状，默认箭头；Rust逐帧重新复核生命/可见/阵营。
- 标签没有额外命中面积，不加data-ui、DOM命中桥或点击转发；标签处若没有实际模型在下方则不可命中。

## 5. 典型场景
正常：鼠标覆盖皇子动作伸出的枪/肢体显示剑，右键追击；同一公共候选可由友方资格谓词选择队友。默认：地面/友军/面板用箭头。错误：血条空白边缘被当成模型，或只有相机平移就能选到未侦察敌人。

## 6. 验证
native覆盖伸出部件/实际显示变换、近远镜头和DPR下6px边界、斜武器空白、候选排序与友敌过滤复用、隐藏/死亡/旧实体/非法投影，另保留真实英雄构造与击飞回归。
Node pointer专项覆盖逐帧敌方反馈只改形状不改位置、UI/手势和暂停/失焦/失锁复位。browser-picking用真实模型/剑形及标签负向、HUD隔离、射程外右键追近伤害、暂停/重选，检查800×600与DPR2。
native还覆盖A点远敌时不被另一近敌抢走、空地保留原自动索敌且不黏旧selected、友方/隐藏不转AttackTarget；browser区分显式模型指令order=Attacking与空地order=Attack move，不能只靠最终掉血证明锁对目标。
蒙皮回归移动实际关节后命中新显示位置、拒绝旧绑定位置，并验证零比例附件不产生空白命中区域。
按影响执行make check、WASM Clippy、smoke、pointer-response和原生healthbars；交互帧时钟不作为性能证据，不新增写World测试入口。

## 7. 错误与正确
错误：在picking里硬编码敌人/技能ID/普攻射程，或在JS扩大血条区域来伪造模型命中。正确：同一实际显示几何返回公共候选，操作层选择目标，模拟复核规则，pointer所有者只显示真实逐帧反馈。
