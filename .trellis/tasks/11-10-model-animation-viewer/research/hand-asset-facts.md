# Research: 默认待机手部问题的动画与骨架来源事实

- Query: 默认李火旺＋伏浪锚截图实际使用什么动作；燕云与永劫角色手部／手指／扭转骨从导出到采样、重定向是否保留？
- Scope: internal；只读当前运行代码、随包二进制与转换元数据，不修改模型、导入器或运行行为。
- Date: 2026-11-11

代码链说明以调查开始时的`2f49537`为基线；后续用户已授权修复，由实现代理负责。本文仍只记录研究证据，不把修复前事实写成当前应遵守的行为。

## Findings

### 用户确认的默认状态不是燕云源片段

`Session::new`固定李火旺、伏浪锚、`game:idle`、time=0、暂停（`src/client/viewer/session.rs:142–151`）。第0帧的`RigPose::new`调用`poses::idle(...Melee)`，trace是`Sword_Idle`，source=None（`animation.rs:93–104`）。

`poses::idle_with_trace`先采样CC0 `Sword_Idle`，再叠加`hold_guard`（`animation/poses.rs:92–109`）。`advance_source`只为攻击、受击、防御、闪避和跳落等选择燕云源片段，Idle走None（`animation.rs:242–305`）；上一动作的源层会淡出，但这不影响刚打开页面的frame0（`:339–350`）。

因此这张已确认的默认截图可用来定位CC0持握／跨模型适配，不能据此认定燕云原动作导出损坏，也不能据此证明燕云原动作正确。

### 默认手部计算的实际顺序

1. CC0来自Quaternius骨骼片段，导入仅保留23个结构节点，包含双手腕但没有任何手指、前臂扭转辅助骨（`scripts/import-animations.py:1–7,17–23`）。
2. `guard(Melee)`使用固定骨盆相对握点`(0.10,0.25,-0.20)`、yaw165°、pitch−15°（`poses.rs:80–85`）。它只区分Melee／Ranged，不按锤／链斧选择独立待机。
3. `weapon_pose`在CC0骨架中计算握点，近战只对右臂执行IK；左臂保持CC0片段本身的姿态（`:321–341`）。`arm`以CC0肩／肘／腕距离解两段链，并强制腕部朝向（`:369–411`）。
4. `avatar::driver_pose`以CC0当前模型旋转乘参考模型旋转的逆得到结构增量，再沿目标角色自身anchors重建位置（`avatar.rs:395–409`）。默认没有`companion_hands`，不会走后面的目标角色臂长IK分支（`:450–452,555–560`）。
5. 每个实际蒙皮骨接收其driver的同一增量乘自身导入rest（`:565–570,627–653`）；蒙皮仍使用每部件原inverse bind（`:264–290`）。武器挂点是后续显示步骤，不负责独立修复身体手指。

上述只描述执行链，没有把其中任何一步先定为默认截图的几何错误根因。

### 李火旺导出保留了细骨与权重，播放控制只剩23个结构自由度

`import-naraka.py:21–33`定义23个`gMan`结构锚点。导出保留每个部件原skin joints、逐骨IBM与sourceRestModel（`:514–528,650–659`），并按最近明确结构祖先分配driver（`:171–182`）。元数据明确`source_animation_name_matching=false`，骨名带`naraka::`前缀（`assets/lihuowang/source-manifest.json:36–38`）。

实际运行二进制扫描结果：

| 李火旺的源骨类别（去掉部件前缀后去重） | 实际结构driver |
|---|---|
| 30个`gMan L/R Finger*`，每侧15个 | 每侧全部共用hand_l／hand_r |
| 每侧`gMan ... Forearm`与Twist1／2／3 | 每侧4个共用lowerarm_l／lowerarm_r |
| 每侧`gMan ... UpperArm`与Twist1／2／3／4 | 每侧5个共用upperarm_l／upperarm_r |

主身体`naraka::outfit::body`有1074个顶点含手指骨权重、350个含手骨权重、583个含前臂骨权重；衣装`body02`另有480／955／772个。这是“至少一个非零槽含此类别”的顶点数，同一顶点可重复计入不同类别，不是互斥数量。

故不能说“李火旺导出时把手指骨／权重全删了”。确切限制是**骨与权重保留，但播放时指骨没有独立弯曲输入，多个扭转辅助骨也共用结构增量**。对同一hand driver下的指骨，`delta × own_rest`只整体搬动原有手形，不会恢复源动作的逐指变化。

### 燕云源文件确实保留独立手指轨，愁铃客与李火旺消费方式不同

扫描当前`assets/choulingke/animations.bin`得到986节点、35片段；其中40个名字含Finger（每手15节＋5个Nub端点）、2个Hand。命名分类中的ForeArm／UpperArm还含服饰辅助节点，不能把字符串计数误称为纯解剖骨数。

| 源clip | 总轨数 | Hand轨 | Finger轨 | Finger旋转确有变化的轨 |
|---|---:|---:|---:|---:|
| skill_ylk_1 | 82 | 2 | 40 | 40 |
| skill_huajie_01__shuangqiangt | 246 | 2 | 40 | 40 |

源导入器按真实骨名与明确父级接受T/R/S轨，不针对手指做特殊删除（`scripts/import-choulingke-animations.py:117–143`）。运行采样从完整rest初始化，逐track采T/R/S，缺通道回原rest（`source_animation.rs:270–294`）。

愁铃客的每个palette骨可以按原骨名找到完整源节点（`avatar.rs:276–280`）；source存在时`joint_pose`优先使用该节点的source.model（`:640–648`）。实际愁铃客body palette有30个Finger骨，全部名称存在于源库；CC0 fallback时它们共用hand driver，**源动画模式则可获得独立逐指轨**。

李火旺的每个palette骨刻意不按燕云名字匹配（`:277–280`），源层经过`source_driver_pose`从23个明确燕云锚点取增量（`:178–202,584–610`），再按23个driver作用于本角色原rest（`:613–653`）。因此即使启用`source:skill_ylk_1`，李火旺也没有消费40个独立燕云Finger轨；这是重定向层的保真度边界，与“源文件是否有指轨”是两件事。

`game:attack`会触发共用Melee cue（`animation/preview.rs:228–230`）；锤／斧／原刀当前用`skill_ylk_1`的0.9–1.8秒裁剪，分别按0.6／0.42秒视觉时长压缩，并与CC0混合（`animation.rs:253–263,332–335`）。它不同于默认Idle，也不同于直接源片段对照。

## Files found / 可重复读数

- `src/client/animation.rs`、`animation/poses.rs`、`animation/preview.rs`：真实状态／采样和CC0持握。
- `src/client/avatar.rs`：23驱动、角色自身rest／IBM、完整源骨／结构源骨两条消费分支。
- `scripts/import-animations.py`、`import-naraka.py`、`import-choulingke.py`、`import-choulingke-animations.py`：导入、骨名映射和轨道保留。
- `assets/{choulingke,lihuowang}/character.bin`、`assets/choulingke/animations.bin`：本次只读扫描对象。
- [hand-asset-counts.py](hand-asset-counts.py)：仅用Python标准库读取上述二进制，完整消费字节并按真实骨名／driver／非零权重统计，运行不修改产品资产。
- [hand-asset-counts.json](hand-asset-counts.json)：扫描结果，含每部件映射、权重影响统计与两个代表clip的逐骨键数量。
- [hand-finger-samples.py](hand-finger-samples.py)／[JSON](hand-finger-samples.json)：源手指局部四元数采样、目标完整骨名和twist位置、CC0参考与目标结构段方向。
- [hand-finger-correspondence.py](hand-finger-correspondence.py)／[JSON](hand-finger-correspondence.json)：每条指链的首／末关节位置、节长、实际权重槽数、source Nub无权重和白灵淼原GLB父子链核验。
- [hand-native-axes.py](hand-native-axes.py)／[JSON](hand-native-axes.json)：用完整sourceRestModel逆矩阵核实指节原生方向，再以真实四权重LBS后的末节顶点云验证无Nub末端方向。

复现命令：`python3 .trellis/tasks/11-10-model-animation-viewer/research/hand-asset-counts.py`。该脚本是**来源／能力清单**，不是“手姿正确”的回归测试，也不能替代主会话的实际蒙皮／关节不变量与画面对照。

### 后续补充：准确参考姿态和实际局部手指运动

`Driver.reference`取`AnimationLibrary.model_pose(all Bone.rest)`，不是Sword_Idle第0帧（`avatar.rs:162–168,221`）。Bone.rest来自Quaternius GLB原节点rest，导入时原样写入（`import-animations.py:132–140`），model_pose另加Y转180°、全身scale1.1（`skeleton.rs:135–148`）。

李火旺结构anchor取原Xinsu hair的当前l22源骨模型矩阵，经实际转换／归一化后得到（`import-naraka.py:437–451`）；白灵淼取自身Lianmu当前l22_female_skeletonAvatar（`import-prepared-models.py:198–215`）；愁铃客的anchor则取原皮肤inverse IBM的逆矩阵位置（`import-choulingke.py:179–185`）。Li/Bai主体实际AV03 UpperArm／Forearm／Hand rest位置与选定anchors吻合，最大差分别6.73e−7m／7.74e−7m，未发现这六个关节有明显hair／outfit位置偏差。

在同一游戏坐标中，CC0原参考段方向与Li/Bai自身anchor段方向确有大角度差：两侧上臂约48.794°、前臂约56.854°。这些是rest方向差的事实；是否被当前重定向正确校准要由运行几何回归判断，不能仅凭角度大宣判解码错误。

本地采样使用与`Keys::sample`相同的时间夹取／最短弧slerp规则，根heading归一化不会影响这些局部手指quat。`skill_ylk_1`在0、0.2、0.9、1.35、1.8秒的实测如下（相对对应指节原local rest的四元数最短角，单位度）：

| 指节 | 0 | 0.2 | 0.9 | 1.35 | 1.8 |
|---|---:|---:|---:|---:|---:|
| Bip001 L Finger11 | 95.69 | 89.30 | 64.95 | 28.17 | 24.62 |
| Bip001 R Finger12 | 41.62 | 47.21 | 54.54 | 55.15 | 55.75 |

所以源数据含有实际非零、会变化的关节姿态，不只是40条空轨。角差本身不是已解剖校准的“屈曲角”，仍需目标手指自身rest轴／父级校准，不能原样复制local quaternion。

### 后续补充：source四节点与目标三节点的对应

每侧每指编号d=0…4：源链是`Bip001 {side} Finger{d}`→`Finger{d}1`→`Finger{d}2`→`Finger{d}Nub`。源前3个节点都有真实皮肤权重；所有10个Nub均无皮肤权重且不进入愁铃客palette。目标Li/Bai实际有同编号前三个`gMan`节点，全部带权重；白灵淼Lianmu和outfit原GLB确认Hand→base→middle→distal父子顺序，且没有Nub节点。

有证据支持的是对应前三节名称匹配，保留目标自身骨长／轴；这不是把40个有权重源关节压成30个，也不能跳过源base或把Nub错配到目标最后一节。编号到Thumb／Index等英文解剖标签并未在当前冻结元数据明文标注，配置应以已核实的编号链为准，不把习惯称呼当作额外源证据。

右侧指d0…4的源三段长度（包含至Nub末端）分别为26.22/27.34/23.66、37.87/19.56/19.02、38.68/22.99/20.62、35.20/20.48/20.28、24.34/16.90/17.30mm；Li目标可由前三个关节测出的两段分别为38.26/31.94、37.70/26.65、37.89/27.26、30.15/26.61、23.12/18.40mm，Bai略有自身比例差。末端无Nub时不能凭缺失端点强行补同一长度。

### 后续补充：twist节点位置和权重事实

实际右前臂twist沿自身肘→腕线段的位置比例：

| 角色 | Twist1 | Twist2 | Twist3 |
|---|---:|---:|---:|
| 李火旺 | 0.250002 | 0.500002 | 0.750001 |
| 白灵淼 | 0.307608 | 0.575064 | 0.818121 |

这些节点偏离该线均小于1e−6m，且都带非零皮肤权重。Li三节点权重255总和为33294／51270／50412，Bai为8734／15801／62458。上臂Twist1…4也不完全按同一比例分布：Li约0/.2/.4/.6，Bai约0/.238/.431/.605。因此后续若处理扭转分配，应使用目标实际rest位置与轴的证据，不能用骨名编号写死一套比例；本报告没有宣称已恢复原作twist控制算法。

### 后续补充：目标末节原生+X方向的实测依据

目标没有Nub，不能虚构末端节点或给末节随意补长度。本次通过两个独立事实检查末节方向约定：

1. 对每个实际palette中存在的FingerBase→Middle和Middle→Distal对，用父骨完整sourceRestModel的逆乘子骨rest位置。所有可用对的局部方向都几乎精确为+X：Li共26个palette对，最小cos(+X)=0.999999999959；Bai共28对，最小0.999999999944。不是仅按骨名猜轴。
2. 读取实际POSITION、4个JOINT槽与4个weights255，计算每顶点完整`Σweight × sourceRestModel × IBM × POSITION`，再把末节骨有权重的顶点变换到该骨原生坐标。主体十指的末节加权中心均在+X一侧；Li native X约11.8–18.2mm，84.8–94.8%末节权重位于X>0；Bai约9.9–18.5mm、85.9–97.3%。只保留末节权重≥128的顶点时，加权中心方向与+X的cos仍分别≥0.9599／0.9727。

末节rest矩阵没有镜像轴歧义：Li三个basis列长约0.912853（范围0.9128523–0.9128544），det3约+0.760682；Bai约0.916810（0.9168090–0.9168102），det3约+0.770615。它们是正的近均匀缩放，所以归一化矩阵X列与分解后proper rotation×+X一致。现有资产中可把它作为末节方向依据；不应把顶点云中心的长度当成新增骨长，也不应据此声称所有未来角色骨架都用+X。

原生顶点云并不轴对称，尤其拇指有横向偏移；几何中心仅用于确认方向半空间，不应替代骨轴。完整逐骨scale／det、局部bounds、加权与主导权重中心在JSON中保留。

另有部分palette不完整的实据：Li主体body缺少`gMan L Finger0`，该左拇指base由body02 palette保留；整个外观合并后仍有30个指骨。Bai主体body本身包含完整30个。实现不能假定每个独立skin都拥有完整三节链，也不能因中间部件隐藏就丢掉主身体需要的参考关系。

## 诊断对照的变量边界

待主会话红色复现建立后，可保持同一李火旺、时间、镜头，比较`game:idle`与`cc0:Sword_Idle`，只改变是否叠加Kiva持握；再比较相同CC0片段在愁铃客／李火旺的实际皮肤。参考姿态和隐藏武器能分别隔离源网格自身rest与武器遮挡，但不能靠只挪武器去证明身体已修复。

源轨问题应另用`source:skill_ylk_1`在愁铃客完整源分支与李火旺23-driver分支中对照。默认截图没有进入这条源轨路径，当前不据此判断其动作采样／解码是否正确。

## Related specs / External references

- `.trellis/spec/client/model-assets.md`：自身rest／IBM、原件冻结、模型与显示适配的边界。
- `.trellis/spec/client/model-viewer.md`：默认配置、源／游戏模式区分和共用求值要求。
- `.trellis/workflow.md:369–388`：研究产物写回任务research目录。
- 未使用外部资料或互联网；所有结论来自当前本地代码和实际随包字节。

## Caveats / Not Found

- 用户已确认截图来自默认状态；这消除了模式／片段／暂停时间的不确定性。
- 这里只核实来源、映射与能力，没有证明缺少逐指／分段扭转就是本图右腕异常的唯一或主要根因；实际手臂／皮肤几何红测由`hand_repro`负责。
- 指轨存在及变化不等于原动作已逐帧正确恢复；稀疏采样、坐标和原作着装／握持仍需各自验证。
- 没有改产品代码、模型、权重、导入器或任何任务状态，也未运行Git操作、服务或完整构建。
