# Research: 手臂与手指重定向的参考基准审查

- Query: 如何在不改原rest／IBM、不加查看页偏移的前提下，修复默认手臂参考方向差异，并将燕云逐指运动可靠传到李火旺／白灵淼？
- Scope: internal；独立数学与调用链审查，只写本研究文件；生产实现由hand_repro负责。
- Date: 2026-11-11

## Findings

### 1. 当前方向偏差的数学原因成立

`avatar.rs:395–408`先计算模型空间运动增量

```text
q_old = q_donor_current × inverse(q_donor_rest)
p_target_child = p_target_parent + q_old_parent × own_rest_offset
```

它能保持目标段长，但当目标A姿态的原段方向与CC0 T姿态不同，原方向差也会随动画整体旋转。已有实际数据给出两侧上臂48.794°、前臂56.854°；这不是靠比较骨长或同一helper的结果能检出的错误。导入参考不同于bind时，换IBM或把rest强制抵消不是正确修复。

最小修正可以保留现有旋转中的轴向成分，用最短摆动对齐真正采样的解剖段方向：

```text
u = normalize(q_old × target_own_rest_child_offset)
v = normalize(donor_current_child_position - donor_current_parent_position)
q_corrected = shortest_arc(u, v) × q_old
```

对upperarm→elbow和elbow→wrist分别求值，再按目标自身段长进行FK。这样约束的是姿态方向，不要求不同身材的手腕绝对坐标相同，也不写死49°／57°数值。

若依赖固定rest校准，只在采样child局部平移始终是rest且无额外IK重写时与上式等价；直接用当前采样段方向覆盖范围更准确。修正必须在该driver的child位置递推之前生效，或在局部臂链内重新递推全部受影响子节点，不能仅转上臂而留下旧肘／腕位置。

`posed_drivers`的明确伴随握点IK（`:450–560`）可在上述基础姿态后继续执行，其最终握点与脚部约束仍优先。不要因修正默认近战而删除女性陡角度枪姿的锁骨／骨盆／双腿求解。

### 2. 手腕native轴和武器语义轴需要分开

`poses.rs:369–409`把CC0手腕模型旋转直接设置为持握Key旋转；`stickman.rs:22–32`把该语义frame的+Y×0.055当挂点偏移、+Z当近战柄轴校正前的方向。因此手腕的数值frame在这里已经承担握持语义，不只是一个任意native骨局部轴。

当前`attachment_pose`对右手保留donor参考frame，而实际皮肤使用`delta × target_rest`（`avatar.rs:627–653,704–752`）。它们即使在旧测试中数值相等，也不证明掌心、手指与杆轴的几何关系正确。

对实际目标腕骨模型姿态M和自身rest矩阵B，目标资产的实际语义握持参考frame记为G0，则一致的语义frame是：

```text
deformation = M × inverse(B)
G_current = deformation × G0
```

G0应由目标手掌的真实参考几何／骨点确定；不能直接把native腕骨四元数或另一角色的参考四元数当成解剖握点frame。可用腕→中指MCP作为掌长轴，用食指↔小指MCP的横向确定掌平面，并明确左右手的手性和+Z柄轴约定。其余滚转符号必须由真实手部与杆轴画面／向量对照确认，不宜先猜一组Euler角。

本次应先修正臂链，再复查掌心／握点。手掌frame校准是单独变量，不应把臂链、腕轴与武器偏移同时调到“看起来差不多”。现有+0.055m和柄轴约定可保留；若实际掌心需要不同的资产语义参考，改共用求值及相应独立几何断言。

### 3. 指链不是4个有权重骨对应3个骨

本地资产清单确认燕云每指为3个实际指节加一个无权重Nub端点，永劫gMan每指为3个实际指节；因此匹配同侧Finger0..4的前三个解剖关节是有证据的，不需要合并两根真实骨。

AV03运行palette只存name／driver／IBM／当前rest，没有完整native父级字段（`avatar.rs:264–290`）。新指链必须按明确语义链建立：腕→FingerX→FingerX1→FingerX2。不能用当前所有指骨都指向hand的结构driver关系，冒充指骨真实层级。

源层完整`source.model`与`rest_model`足够求父相对姿态：

```text
L_source_current(i) = inverse(S_current(parent)) × S_current(i)
L_source_rest(i)    = inverse(S_rest(parent)) × S_rest(i)
```

但直接复制上述局部四元数，或者仅用`inverse(targetParentRestRot) × sourceParentRestRot`作共轭，都不能保证在不同掌姿／骨轴下沿对应解剖弯曲平面转动。后一个公式主要对齐世界参考轴，还可能保留与手臂相同的目标初始弯曲偏差。

### 4. 推荐的指链校准与层级求值

先为源、目标各自构造静态解剖手掌frame：腕→中指根给掌长，食指与小指根给掌宽／掌平面；拇指按自身链平面独立校准。原生指骨轴与解剖轴的关系由各自真实rest确定，不能按数字索引或同名四元数直接套用。

对于源当前指节的长轴方向，先在源**当前**手掌frame中表达，再转入目标**当前**手掌frame；目标指节按该解剖方向定向，沿目标自身局部段长递推。可从首两段的叉积取得弯曲平面法线；近共线时使用掌平面作为明确fallback。最后一节可用源Nub方向；目标无Nub时，从前两节推导其native前向轴约定，并验证该推导符合实际资产。

这比只保留原目标rest姿态的角度增量更接近“传递实际源手形”：目标原指节已经弯约50–75°，如果只叠加源增量，可能继续保留错误握持偏差。若首版选择只传增量，应明确这个限制，并测试弯曲平面、旋转符号与目标手形，不能仅以非零变化作为正确性依据。

无论采用哪一种校准，目标每个child的位置都必须从**已经更新的目标parent**和目标rest-local偏移计算。给每指独立旋转但保留原全局joint位置，会断开骨链，实际LBS手指仍可能扭曲。

### 5. 淡入淡出必须先混局部，再FK

当前`joint_pose`最后用全局Transform的translation lerp和rotation slerp混合fallback与source（`:650–653`）。即便两个端点的链长正确，直接混全局关节位置也可能在weight0.5缩短弯曲骨链。

新指链应从已求出的实际混合手腕开始，先混合指节**局部旋转**，保留目标自身局部平移／尺度，再逐级FK：

```text
q_local(i,w) = slerp(q_target_rest_local(i), q_calibrated_source_local(i), w)
M_target(i,w) = M_target(parent,w) × TRS(own_rest_local_translation,
                                      q_local(i,w), own_rest_local_scale)
```

这样weight0回到原有目标手形，weight1接收校准源动作，中间权重不因全局位置插值缩短指节。参考模式仍在最前面直接返回原B，不走校准；原资产rest和IBM不可改。若给上／前臂混合增加同类约束，也应采用局部转角／段长递推，不能把旧全局混合问题带入新手部实现。

### 6. 扭转骨与最小范围

目标前臂Twist1/2/3共用lowerarm驱动是事实，但不应凭名字把源／目标扭转骨按序号硬配。源、目标数量不同；应检查各骨沿实际段轴的位置和作用权重。

若臂链修正后仍出现前臂“糖纸扭曲”，可将前臂与手掌间的**轴向**差分旋转作swing/twist分解，再按目标扭转骨沿段长的真实位置分配roll，保持这些骨的轴向位置不动。不能把腕部屈伸swing也分给整条前臂；不得声称这种几何分摊是恢复了原作专属twist曲线。本研究没有验证该扩展的画面必要性，因此不建议把它作为臂链修复的先决大重构。

### 7. 独立回归断言

- 双侧实际目标上／前臂方向跟随实际采样方向，同时保留各自段长；至少覆盖默认持握、原CC0、源动作，不只断言finite。
- 有明确companion握点时，实际palette腕骨抵达握点；脚、脚趾、显示根与权威射击源保持原约束。
- 给实际源指节已知>1rad的变化，实际目标对应指节在正确解剖平面响应；验证多个指／多个时刻，不能只验证“某个骨动了”。
- 指链在source权重0、0.5、1保持自身段长，parent→child连续，且weight0与纯fallback一致、reference与原rest一致。
- 实际被指骨加权的网格顶点相对手腕改变形状，避免只改marker或抽象RigPose；纯腕骨加权顶点仍刚性跟腕。
- 从实际皮肤腕骨变形与目标参考握点算出的掌心／柄轴，应与共用attachment输出一致；不要测试两边调用同一个错helper的恒等式。
- 愁铃客全源同名palette路径保持原样；未知名称或不完整指链应安全保留原fallback，不能串用另一角色骨索引。

## Files Found / Code Patterns

- `src/client/avatar.rs:162–223`：CC0原rest参考及目标anchors。
- `src/client/avatar.rs:264–290`：palette原name／rest／IBM读取，Naraka没有直接同名源索引。
- `src/client/avatar.rs:395–409`：当前世界旋转增量与自身anchor递推。
- `src/client/avatar.rs:414–560`：保目标段长的局部臂／腿IK及真实远程握点。
- `src/client/avatar.rs:584–653`：23源锚点传递与实际皮肤混合。
- `src/client/avatar.rs:704–752`：实际语义挂点与旧右手特殊分支。
- `src/client/animation/poses.rs:321–409`：默认近战右臂IK和显式腕部持握旋转。
- `src/client/stickman.rs:22–32`：固定握点偏移与武器轴约定。
- [hand-diagnostic.md](hand-diagnostic.md)、[hand-asset-facts.md](hand-asset-facts.md)、[hand-finger-samples.json](hand-finger-samples.json)：本次已建立的真实数据与红测依据。

## Related Specs / External References

- `.trellis/spec/client/model-assets.md`：原rest／IBM、女性自身骨长与实际palette验证。
- `.trellis/spec/client/model-viewer.md`：游戏与验收页共用求值，禁止查看页专属纠偏。
- `.trellis/workflow.md`：研究只写任务research，实施／检查各自持有生产职责。
- 未查询外部资料；矩阵与四元数结论由当前代码的数据表示直接推导，当前依赖Bevy0.20.1。

## Caveats / Not Found

- 本文是数学与接口审查，不是代码修复或画面通过记录。文件行号为本研究读取时的状态，实施会移动行号。
- 解剖手掌frame的左右手符号、最后指节native前向轴和拇指弯曲平面必须由真实骨点／画面验证；没有证据时不能补魔法角度。
- 指节T/R/S中的非均匀／带符号尺度不能随意丢弃。指链应保留目标rest尺度；若源有反射或有意stretch，应单独处理并记录，不能把矩阵分解出的任意四元数当作已校准的关节旋转。
- 没有改生产代码、资产、规范、Git、服务或任务状态。
