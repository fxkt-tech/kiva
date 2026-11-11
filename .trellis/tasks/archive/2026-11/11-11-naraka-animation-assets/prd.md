# 永劫无间动作离线提取

## 目标

从原客户端读取横刀三段横击、振刀/振刀成功及对应通用动作，形成含原骨架与完整身体/指轨的可重复离线产物。

## 来源和依赖

父任务11-11-naraka-action-set/prd.md是需求源；具体产品取舍见父design.md。

依赖父任务的 animation-assets 与 humanoid-conversion 研究；不依赖战斗代码。表现子任务必须等本任务通过真实转换门槛，不能用占位片段先接入。

## 验收标准

- 核心5类动作有原状态/override/clip关系和源对象/哈希；通用动作逐项映射，缺失明确列出。
- H0必须有原生/独立可信完整转换器逐帧对照，记录源serialized与实际求值器版本（包括补丁差异），覆盖身体/30指/root和相关twist/IK，并记录角差/腕指位置/LBS误差；非零运动/有限quat/骨长保持不能代替此关口。
- 保留missing-vs-zero authored mask，保留原默认左右手层及其证据；原控制器次序/权重未知时明确采用主已写指链优先的Kiva组合，不冒称原规则恢复；H0涵盖全95攻击、75待机、55跳落循环，不能只验全通道片段。
- streamed/dense/constant按真实绑定采样；原Avatar pre/post/sign/limits与必要root/twist/IK处理有证据；超范围原muscle不擅自clamp。
- 冻结准备包保留原名、父序、rest、mask/默认层、经验证baked locals、误差与来源；--verify独立重打包不需要Windows安装，原生--rebake的外部工具/版本另列，二者不混称。
- 新运行时动画数据版本、长度/边界/有限性/末帧/重建SHA校验通过；旧燕云文件不覆盖。

## 范围

新动作离线解析/导入脚本，assets/prepared/naraka/动作准备包，assets/naraka/运行时动画与来源清单；不改avatar/animation运行时代码。

父任务整体方案及四子任务已获用户明确批准，按依赖启动实施，不重复请求规划批准。
