# Research: Naraka Humanoid 肌肉曲线到原生骨架的离线转换

- Query: 已取到真实 Avatar 和95条 Humanoid muscle 后，是否可以在没有 Unity Editor 的当前环境可靠烘焙身体与手指？
- Scope: mixed；本机原始探针、隔离数值 spike、公开的一手项目代码与 Unity 文档。
- Date: 2026-11-11

## Findings

### 结论与可交付边界

**能正确取得95条源 muscle，能按真实 Avatar 的 pre/post/limit/sign 构造独立身体与30指节的候选骨骼运动，但目前不能证明其等同 Mecanim 真实求值。** 解析通道和把通道变回骨骼是两个独立关口。当前研究不能把“曲线非零”“手指会动”“骨长保持”当作恢复正确的证明。

建议最终计划把首个执行关口限定为：一条真实横刀攻击＋原53节点 Humanoid骨架＋真实手指＋root/body solve，先取得可信对照并通过数值/画面检查，才把整个动作集设为游戏默认。既有源动画保持可回滚。尚无对照时，候选输出只能标注为 research approximation，不进入默认动作包冒称原作已恢复。

### 本机输入与身份

| 资源 | 实际来源/校验 |
|---|---|
| Avatar | `5/8/589302f25ec7b3dd`，`CAB-735972d8c50e4d130075dba921e34762`，PathID `-5575226590203376231`，名 `l22_male_skeletonAvatar`，原对象30012字节，SHA256 `32e43c425e6c9f40326f300b4ee473b1c1ef8a1636e1c597ea4f50467cc0212e`。 |
| Clip | `assets/res/animclipadapter/01/male_hengdao_attack_light_01.anim`，`d/6/d653f1688755a556`，PathID `5670820053815679417`，原对象20588字节，SHA256 `90cb9647d45f0d975749d973393d660be324cba0ee6f4efcd3ce977711790f96`。 |
| Unity序列化版本 | 两者均 `2019.4.41f2`，按源 TypeTree 读取；外层包版本不代替此版本。 |
| Avatar数据 | full Avatar skeleton63节点，native human skeleton53节点（含一个MotionRoot），52组axes，52项human description，30个真实手指骨节；human scale `1.0782519578933716`。 |

原对象探针由主会话写入 `humanoid-avatar.probe.json`、`male_hengdao_attack_light_01.anim.probe.json`。本研究没有覆盖它们，也没有修改客户端资源。

### 已实证的索引规则

1. 不能把 `m_HumanBoneIndex` 数组下标当作 C# `HumanBodyBones` enum。Native身体枚举把 UpperChest 插在Chest后、Neck前；C# API枚举中UpperChest在末尾。`m_HumanBoneIndex` 本例25项只描述身体槽，并不是52个有效骨骼的完整列表。
2. 本例可靠路径是 `m_HumanSkeletonIndexArray[human_node]` → full skeleton index → `m_SkeletonNameIDArray[index]`。后者是**叶骨名**的CRC32，不是完整路径hash。原 `m_TOS` 的路径键则是另一套完整路径映射。
3. 用原TOS每个叶名的CRC32反查，全部53个human节点唯一还原；再按原 `m_HumanDescription.m_Human` 的 boneName→humanName 绑定具体语义。正反index数组逐项一致。不能通过“轴数组第n项应该是哪个骨”来硬猜。
4. 真实完整顺序为 `MotionRoot→gMan Pelvis→Spine→Spine1→Spine2→Neck→Head`，然后左臂和15指、右臂和15指、右腿、左腿。每个节点的 `m_AxesId` 决定轴表项，必须单独读。

### 已实证的曲线解码边界

- 此clip总宽 `179 = streamed7 + dense113 + constant59`。130项genericBinding按Transform维度展开后恰为179，不能把130直接当curve总宽。
- `typeID=95/customType=8` 的attribute42..136恰对应95个Humanoid muscle；本例全部存在。每项实际列号与 `m_IndexArray[attribute]` 独立相符。
- Dense是25帧×113列、30Hz，原clip区间 `[0,0.7666667103767395]`。全部muscle落在dense或constant区间；此数值spike不需要处理streamed muscle，但其它clip不能假设相同布局。
- `m_DeltaPose.m_DoFArray`（55）＋左右手DoFArray（20+20）在本例是**起点减终点**，不是缺轨的初始pose。对照原dense首末帧最大误差 `2.384185791015625e-7`。把它当默认起始muscle会整段歪掉。
- 实际muscle范围为 `[-5.16339635848999, 5.30141019821167]`，2375个样本中300个超过[-1,1]。解析不能擅自clamp；是否传入native评估时也允许越界应由native对照确认。

### 有证据的局部角度候选模型

ShaderMotion作者的 HumanAxes 源码给出校准框关系，使用swing（YZ旋转向量）再twist（X）组合。本文只把它作为候选数学模型，**不是 Unity 原生源码**。[HumanAxes.cs](https://gitlab.com/lox9973/ShaderMotion/-/raw/master/Script/Common/HumanAxes.cs)

设 `m` 是muscle值，轴的负/正范围来自原Avatar的radianLimit。研究模型为：先按m正负乘相应limit幅度，再乘原m_Sgn；组合局部旋转 `preQ · swingYZ · twistX · inverse(postQ)`。这是研究假设，需验证非对称limit、native axis-mapping及twist分摊；不能把muscle直接当Euler角。

源Avatar的pre/post属于neutral（motorcycle）pose，而不是模型导入/T姿态。数值比较本例的 `preQ·inverse(postQ)` 和原 human skeleton pose：

| 骨骼 | 零muscle neutral相对原导入local旋转差 |
|---|---:|
| 左/右上臂 | 48.648° |
| 左/右前臂 | 79.993° |
| 右拇指近节 | 20.050° |
| 右食指近节 | 34.497° |
| 右食指末节 | 36.526° |
| 左大腿 | 30.005° |
| 右小腿 | 79.993° |

因此不能把“模型原local quaternion×muscle增量”当作native pose；这会再次出现手臂与握拳参考姿态偏差。

### 本次实际数学spike与输出

文件：

- `probe-humanoid-muscle-math.py`：只写本任务research，依赖已有本机NumPy环境；不修改产品或原探针。
- `humanoid-muscle-math.probe.json`：输入SHA、索引/曲线检查、逐骨neutral差、候选运动span、5个实际源时刻的53骨位置和local quat。

运行：

```sh
/home/asuka/kiva-asset-research/naraka/.venv/bin/python \
  .trellis/tasks/11-11-naraka-action-set/research/probe-humanoid-muscle-math.py
```

结果：

| 检查 | 实际结果 | 证明范围 |
|---|---:|---|
| 身体骨local旋转变化>1° | 21个 | 候选模型确实读取身体muscle，不是静止替身。 |
| 手指骨local旋转变化>1° | 30/30 | 全部真实三节指链独立读取源muscle。 |
| FK骨长最大误差 | `1.6653e-16` normalized unit | 此候选FK不缩短骨骼；不证明方向正确。 |
| quaternion单位长度最大误差 | `2.2204e-16` | 数学有效；不证明与Unity等效。 |
| 右手相对首帧最大移动 | 1.1712 normalized unit | 存在大幅摆臂，可用于后续渲染对照。 |
| 右前臂local相对首帧span | 129.881° | 不能只用“有变化”验收，需要真实方向/时刻对照。 |

spike刻意保持Hips参考，以隔离可验证的肢体数学，不伪造body COM/root solve；也没有套generic helper transforms。故此输出不能用作完整横刀动画。human skeleton的平移是normalized单位，最终写入原模型前还需验证human scale与完整native层级的局部变换。

### 尚缺的原生行为

1. **axis与muscle归一的独立真值。** 本研究的逐骨XYZ muscle索引显式列在脚本里，依据已知语义/校准轴推导；Unity公开的 `HumanTrait.MuscleFromBone` 和limit API都转调native，C#参考源不提供完整表或求解算法。必须导出真实同版本API结果或与独立可信converter对照。[Unity HumanTrait bindings](https://raw.githubusercontent.com/Unity-Technologies/UnityCsReference/master/Modules/Animation/ScriptBindings/HumanTrait.bindings.cs)
2. **twist distribution。** 原Avatar `ArmTwist/ForeArmTwist/UpperLegTwist/LegTwist` 均0.5。只对对应local轴乘角无法证明与分摊后的真实骨架相同；不能以手指解析成功推定上臂、前臂滚转也准确。
3. **Body Transform不是Hips。** Unity文档明确Body Transform是身体质心，姿态还含相对它的muscle/手脚IK；把RootT/Q直接赋Pelvis不是准确转换。X/Z根移动剥除与质心/骨盆求解是不同步骤。[Unity 2019.4 Root Motion](https://docs.unity3d.com/2019.4/Documentation/Manual/RootMotion.html)
4. **goal IK、stretch、root flags。** 即使某clip的DeltaPose goal权重为0，也不能推定全部时刻/其它片段的foot IK均禁用；需要状态/Animator FootIK配置与真实采样结果。原Avatar armStretch0.05、legStretch0.002，body曲线与原generic helper/root还需按绑定分别处理。
5. **neutral/import/bind与坐标basis。** 保留原AvatarSkeletonPose、DefaultPose、humanSkeletonPose的身份；三个不是任意互换。按完整层级还原rawUnity后，只做一次约定反射/正面转换；最终runtime原模型自己的rest/IBM不能改写成donor neutral。

ShaderMotion本身在README的骨骼旋转说明中也限定muscle匹配是近似，明确提到Mecanim twist分摊的复杂性。因此不能把该项目公式的正确引用当作原生保真验收。[作者项目说明](https://github.com/CuteWaterBeary/ShaderMotion#bone-rotation)

### 优先执行的验证路径

**路径A：同版本Unity离线oracle，推荐。**

- 使用已验证的Avatar层级/原人骨描述在单独离线工具工程中创建有效Avatar，或恢复原序列化Avatar；不运行/注入游戏，不改用户安装。
- 用原AnimationClip通过Animator/Playable采样，导出full skeleton locals、world matrices、human pose和可选native肌肉轴表。`HumanPoseHandler.SetHumanPose` 是可把human pose应用到handled骨架的正式API，但仅调用它仍不等于采样完整clip（generic曲线、clip root flags及Animator IK需同时考虑）。[Unity SetHumanPose](https://docs.unity3d.com/2019.4/Documentation/ScriptReference/HumanPoseHandler.SetHumanPose.html)
- 实际取attack01、振刀、成功振刀、jump/land各代表片段，以及neutral/single-muscle正负脉冲。保存Unity版本、Avatar和clip源SHA及工具版本。
- 当前主会话已检查常用安装位置、PATH和Windows卸载注册表，未发现Unity Editor/Hub；这不代表穷尽所有便携/手工安装位置，范围详见 `research/environment.md`。本研究没有安装软件、激活许可或假定native转换已可运行。获得可用同版本offline exporter是实际技术依赖。

**路径B：独立完整Humanoid解码器。**

- 需要能证明95通道映射、native axis limit/pre/post、twist、质心到骨盆、generic curve与IK等行为的实现，且有同版本测试向量。
- AssetStudio/uTinyRipper等能把muscle曲线恢复为Unity的命名float tracks，不代表可直接导出原生骨骼Quaternion。uTinyRipper的 `AddAnimatorMuscleCurve` 保留标量曲线，未在该步骤转换骨架，因此不能用“工具导出了anim”越过此关口。[uTinyRipper AnimationClipConverter](https://raw.githubusercontent.com/mafaca/UtinyRipper/master/uTinyRipperCore/Converters/Classes/AnimationClip/AnimationClipConverter.cs)

**路径C：同动作的完整Generic骨轨原件。**

- 如果客户端还保留非adapter、直接带完整body+finger Transform SRT的同动作版本，可以优先走准确曲线→原骨FK，不需逆向Mecanim。必须核对动作身份和真实受影响骨数。
- 仅有MotionRoot/body_dummy/helper的Generic通道不是此替代方案；本横刀样本核心body仍是95muscle。资源研究代理已定向核查完整AppRes：横刀attack_light01/02/shock01只有animclipadapter `.anim`，没有同名FBX；横刀主目录FBX仅特殊PVE特效蛇和武器水下姿态，无可用body通用副本。实际采样的19个攻击/振刀/移动/跳跃/四向闪避clip也均以95条Humanoid muscle为核心。因此当前没有证明可用的完整Generic替代件，不能据其它helper曲线宣称绕过问题。

### 建议的数值验收阈值与防错项

以下是建议的工程阈值，不是当前已达到的等效结果：

- 相同源rig、无显示重定向时，对oracle逐帧local quaternion角差≤0.1°，腕/手指位置≤1mm，真实加权LBS顶点最大差≤1mm；若native浮点差或外部烘焙降采样放宽阈值，必须记录理由与峰值。
- 双手五指每一级、pre/post非同一参考的上臂/前臂、扭转>90°片段、越[-1,1] muscle、足底和body root分别验收，不能只取一帧/一根手骨。
- 曲线布局全宽一致；human正反映射一致；显式Native→C# bone枚举；不把DeltaPose当起始pose；不clamp越界muscle；不把Euler替代swing-twist。
- 先在53骨rawSource诊断画面核对真实姿态，再接已有三角色目标重定向与腕/MCP握点；来源错误不能由目标IK强行掩盖。
- 游戏root-motion剥除、空中Y、防御持势、attack阶段时间映射都在源真实性验收之后单独验证。伤害和声音仍由simulation真实marker/反馈驱动。

## Files found / Related specs

- `research/probe-humanoid-avatar.py`、`humanoid-avatar.probe.json`：本机原Avatar完整TypeTree与读取证据。
- `research/male_hengdao_attack_light_01.anim.probe.json`：本机原clip及来源完整TypeTree。
- `research/check-animation-curves.py`、`animation-curves-validation.json`：另一研究代理已独立验证stream cubic+dense+constant采样，179条曲线在0时与m_ValueArrayDelta.m_Start精确一致，末时最大误差约3.1e-13；这证明标量读取，仍不是Quaternion求解oracle。
- `research/probe-humanoid-muscle-math.py`、`humanoid-muscle-math.probe.json`：本研究隔离候选数学与结果。
- `research/runtime-combat-integration.md`：双来源Rig/缓存binding、权威连段及手指保护。
- `.trellis/spec/client/model-assets.md:18`：arm/palm/finger校准；`:33`原import rest不同于bind，不可互换。
- `src/client/source_animation.rs:172`：旧源root硬编码Bip001；不能将本机Naraka blob直接替换进去。
- `scripts/import-naraka.py:440`：已glTF→gameplay正面转换，rawUnity不能再重复/省略反射。

## Caveats / Not Found

- 尚未取得native Unity或原游戏逐帧骨架oracle；本研究只证明源曲线与Avatar信息足够构造候选运动，不能证明完整保真。
- candidate曲线采样25帧中末帧可能超出clip stop区间；本spike使用全部dense帧检查DeltaPose和运动范围，产品烘焙必须严格按原start/stop裁界并保留末帧定义。
- 没有执行产品构建、替换动作、改状态或提交git；所有新增文件只在本任务research目录。
