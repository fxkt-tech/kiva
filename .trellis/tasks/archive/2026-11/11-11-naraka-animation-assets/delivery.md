# 动作资产交付

## 产物

- `assets/naraka/animations.bin`：KIVASR02，6,281,901B，44个原片段，原63骨独立rig身份 `naraka/l22_male_skeleton`。高精度f32秒、自适应非均匀时间、常量通道一key。没有覆盖燕云数据。
- `assets/naraka/animations-manifest.json`：逐clip原SHA、实际默认指层组合、原生版本、坐标归一与H0摘要身份。
- `assets/prepared/naraka/animations`：原对象CAS、原TypeTree、完整Avatar/rest/human信息、authored mask、oracle scalar输入、真正native capture、verified locals、独立offset参考、1116实际手部顶点/原权重/IBM、所有清单和完整验证说明。
- `scripts/naraka_animation.py`：有界原streamed cubic/dense/constant与绑定解析、缺失/显式零分离、KV2打包/读取、原生capture读取。
- `scripts/naraka-native/NativeSampler.cs`：原Clip/Avatar→Unity native PlayableGraph→locals/world，无近似Humanoid数学。
- `scripts/run-naraka-native.py`、`validate-naraka-interpolation.py`、`bake-naraka-native.py`：原生运行、独立参考验证、冻结打包。
- `scripts/naraka_animation_validation.py`：纯stdlib runtime采样/FK/原实际权重LBS逐时刻重算；普通验证不需要Unity/Windows/UnityPy/NumPy。

## H0与采样

原资源serialized版本2019.4.41f2，独立原生评估器为官方2019.4.41f1 Mono；差异明确写入产物。只运行任务自有播放器、标准设置与自编译脚本，不运行游戏或其脚本，无Editor/许可依赖。环境由父任务建立，见 `native-runtime-environment.json`；生产采样脚本已在仓库。

原攻95、idle75、fall55三种遮罩通过原Clip Graph对照。main身体/authored指链与独立新实例原Graph、缺失指链与原default独立Graph误差均0；正式逐key原生重放与自适应取样locals误差0。1116个真实源手部顶点的Native BakeMesh与独立Σw·world·IBM·p误差小于0.5微米。这里的原源实际顶点用于独立交叉绑定验证，不虚称恢复了原角色prefab的完整装配。

首次均匀60Hz虽然所有native keys正确，仍会在横击2近180°原生腕/前臂分支间插出错误90°中间姿态。另有高速通道和默认层循环边界误差，因此未停留在端点/非零/骨长自检。`temporal-validation.json` 保留60/120共3809个相同原生时刻的locals组件完全一致（非cache污染），及均匀60负面数据。

最终以60Hz为种子，对每区间原生1/4、1/2、3/4值进行角度和全骨世界位置检查并有界细分。6534个输出时刻、26610次评估。无法连续插值的源分支仅在邻接f32时刻停止，65个精度切点保留左右原值/原clip SHA；横击2真实腕分支约0.22718145与0.23481453秒。消费者不能把这些相邻时间分母钳为EPSILON。

独立137Hz+0.00123秒偏移不参与补点。纯stdlib从最终runtime二进制对其8670时刻重新采样，每时刻重算所有63骨与1116原手部权重点：

| 指标 | 最大误差 |
| --- | ---: |
| local quaternion | 0.078316° |
| local位置 | 0.227331mm |
| 全骨世界位置 | 0.283705mm |
| 实际手部LBS顶点 | 0.278271mm |
| 原生locals→native world独立FK矩阵元素 | 1.30e-6 |
| 打包→解包→native world矩阵元素 | 1.26e-6 |

门槛仍为0.1°/1mm。以上是覆盖明确时刻的实测，不是连续函数误差数学证明。冻结保留完整独立native参考，普通verify重新计算这些误差，不仅信任报告hash。

## 来源与上下文

- FK鸟铳族：引冰落原WeaponSkin记录同时含flintlock资产名与fk UI名；原 `female_fk_weapon_player` 与 `male_fk_weapon_player` 均将attack_501映射到同名 `male_fk_attack_01`，字段2/3相同。idle/hold/pre_aim/recover同样有直接映射，详见 `fk-animation-identity.json`。女性控制器共用男命名原clip有原证据。
- FK攻击/pre_aim/recover仅39维身体遮罩，不能直接当完整全身；已告知client采用原上身+模拟/CC0下身与实际握点适配。FK idle/hold55维原Pelvis约0.973m，攻击层Pelvis约-0.09m。
- 原缺指层上下文：横刀用原横刀default，弓用原弓default，FK用原male_unarmed作为Kiva握指适配。只有完全缺失的四DoF整指链复制原default的3节locals。权重1、按default时长循环是明确Kiva规则，未声称原controller权重/顺序已恢复。部分缺DoF指链会硬失败。
- original authored指链永不被默认层覆盖；原muscle值超[-1,1]保留。`HumanPose.Get→Set`实测有损，只在隔离clone诊断，未参与烘焙。
- 原轴→Kiva为Z反射+实测缩放，floor只加根。实际既有Li模型22个driver rest点与源FK严格匹配；原末指轴-X已用原子骨偏移和真实权重顶点证实。

## 检查

`python3 scripts/prepare-naraka-animations.py --verify` 已通过全部44原对象/Avatar/scalar/oracle、native来源、逐字节runtime重建、全部8670独立时刻采样/FK/LBS及65个cut。纯stdlib全流程约22.5秒。

`--repackage` 同样完整验证，通过后才重新生成runtime与清单；已实跑成功，输出与原二进制逐字节一致。原生重新烘焙是独立命令流程，前置/命令见准备包README。

`python3 scripts/test-naraka-animation.py` 覆盖实际源曲线/95/75/55与FK39遮罩、原超范围值、坏stream/dense/binding/flags/长度、非有限/非单位quat/父环/时间碰撞、真实65切点，以及实际均匀60Hz包必须被独立原生采样门槛拒绝的回归。Python语法编译与 `git diff --check` 通过。Rust/浏览器集成由父任务和表现worker验收。

## 限制

未恢复原controller默认层权重/顺序、原脚本状态机或原游戏逐帧时序；模拟位移、妹妹局部层、武器对位、时间映射和弹反成功反击是Kiva侧有明确标签的适配。保留sniper候选用于溯源而不把它称为鸟铳专用。没有改用户安装、提交或推送。
