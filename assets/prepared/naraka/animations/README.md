# 永劫动作准备包

运行时只读取 `assets/naraka/animations.bin`，本目录用于溯源、离线验证和重打包。冻结44个原片段、原男性 Avatar 的63骨完整层级/53骨 human 映射、原95维 authored mask、原始对象字节、原生 locals 与独立原生参考。8个默认指层是辅助层；鸟铳FK攻击为39维局部身体层，不能当作完整全身姿态。

## 验证与重打包

只需 Python 3.11+ 标准库；不读取客户端、不需要 Unity/Windows/UnityPy/NumPy：

```sh
python3 scripts/prepare-naraka-animations.py --verify
python3 scripts/test-naraka-animation.py
```

`--verify` 校验全部冻结文件SHA、原对象身份、scalar端点、原Avatar层级、H0记录、runtime逐字节重建；再从真正runtime二进制采样，对8670个独立137Hz+0.00123秒偏移原生时刻重算角差、63骨FK和每帧1116个原手部权重点LBS。验证65个f32邻接切点两侧的原值。角度门槛0.1°，位置/真实手部顶点门槛1mm。

需要重新生成runtime文件时显式执行：

```sh
python3 scripts/prepare-naraka-animations.py --repackage
```

此模式完整验证成功后才写 `assets/naraka/animations.bin` 与清单。它不重新解析原游戏，也不重新运行原生采样器。

## 原生重新烘焙

以下是另一个流程，需要原客户端、外部容器工具（`naraka_sparse.py` / `naraka_container.py`，UnityPy 1.25.4）和已有的隔离Unity原生环境。不会启动或写入用户游戏。环境版本/官方安装包SHA见任务 `native-runtime-environment.json`。原生采样使用官方2019.4.41f1独立Mono播放器与自编译 `NativeSampler.cs`；原资源序列化版本是2019.4.41f2，保留这一补丁版本差异，不宣称原f2逐位等价或已恢复原控制器。

1. 用冻结的 `selection.json` 从只读安装中提取原对象，并在仓库和游戏之外生成原生可加载的UnityFS输入：

```sh
python scripts/prepare-naraka-animations.py --extract \
  --source "$NARAKA_STREAMING_ASSETS" --tools "$NARAKA_RESEARCH_TOOLS" \
  --native-bundles "$NARAKA_TASK_CACHE/bundles"
```

2. 运行自有原生fixture，自适应采样直接来自原Clip+原Avatar的 `AnimationClipPlayable`。原生foot/playable IK关闭、applyRootMotion关闭；身体/twist/COM与原 authored 手指由原生求值，不使用近似muscle数学。默认手指只补完整缺失的四DoF指链：

```sh
python3 scripts/run-naraka-native.py --environment "$NARAKA_ORACLE" \
  --bundle-manifest "$NARAKA_TASK_CACHE/bundles/native-input-bundles.json" \
  --with-real-skin --adaptive --output-dir "$NARAKA_TASK_CACHE/adaptive"
python3 scripts/run-naraka-native.py --environment "$NARAKA_ORACLE" \
  --bundle-manifest "$NARAKA_TASK_CACHE/bundles/native-input-bundles.json" \
  --with-real-skin --fps 137 --time-offset 0.00123 \
  --output-dir "$NARAKA_TASK_CACHE/reference"
```

自适应以60Hz为种子，对每区间1/4、1/2、3/4原生值检查；局部角≤0.075°、local位置与全骨世界位置≤0.25mm。未满足则有界细分；只在f32相邻时刻停止无法连续插值的原生分支，保留两侧原值、时间与来源。运行时不可把这些相邻时间的分母钳为EPSILON。当前6534个输出时刻共执行26610次原生求值，原生重复评估误差为0。

3. 用含NumPy的离线Python环境验证独立非格点参考后打包：

```sh
python scripts/validate-naraka-interpolation.py \
  --coarse "$NARAKA_TASK_CACHE/adaptive/native-poses.bin" \
  --reference "$NARAKA_TASK_CACHE/reference/native-poses.bin" \
  --real-hand assets/prepared/naraka/animations/source/real-hand-lbs.json \
  --output "$NARAKA_TASK_CACHE/interpolation.json"
python scripts/bake-naraka-native.py \
  --capture "$NARAKA_TASK_CACHE/adaptive/native-poses.bin" \
  --native-report "$NARAKA_TASK_CACHE/adaptive/native-report.json" \
  --reference "$NARAKA_TASK_CACHE/reference/native-poses.bin" \
  --interpolation-report "$NARAKA_TASK_CACHE/interpolation.json" \
  --real-hand assets/prepared/naraka/animations/source/real-hand-lbs.json
python3 scripts/prepare-naraka-animations.py --verify
```

## 来源与适配边界

- 原生Clip→Graph是真正Humanoid求值；`HumanPose.Get→Set`实测有损，仅在隔离clone做失败诊断，不写回正式骨架。原始scalar不当作归一化HumanPose参数。
- 原片段 authored 身体和指链保持原生locals。缺指链的3节locals复制对应原默认指层的独立原生值，权重1、按源时长循环是Kiva适配；没有恢复原controller权重/顺序的声明。横刀/弓各用同族原默认；FK用原male_unarmed默认作为明确适配。完整逐clip说明见runtime清单 `composition`。
- 引冰落原皮肤记录同时含flintlock模型名和fk UI名；原男女FK override均指同名 `male_fk_*`，已证该族和片段身份。sniper候选仍保留用于溯源，不声称它们是原鸟铳专用。
- 原Unity→Kiva只反射Z，平移乘实测缩放，地板偏移只加骨架根。实际Li模型22个driver rest锚点已核对；原生末节指骨轴为-X。原动作垂直位移可能在Pelvis，模拟位移/上身局部层组合是客户端另一层适配。
- 保存的采样误差是明确参考时刻的实测结果，未声称连续函数的数学误差上界。负面60Hz插值结果、GetSet结果和FK来源链保留在任务研究记录。

## KIVASR02

全部little-endian：8字节magic、u16 UTF-8长度+rig_id、u16骨数/片段数/MotionRoot索引/flags(0)。每骨为原名、i16父索引、rest的T3/Qxyzw4/S3 f32。每片段为原名、f32秒时长、u16轨数；每轨u16骨索引，依次T/Q/S，每通道u32 key数，随后f32秒时间+3/4/3个f32值。常量通道只有一key；动态为自适应非均匀时间。rig_id固定 `naraka/l22_male_skeleton`，不得混用燕云骨索引。
