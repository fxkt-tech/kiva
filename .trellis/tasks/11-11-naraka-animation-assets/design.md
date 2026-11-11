# 永劫无间动作离线提取设计

只读SparseBundle和原TypeTree；离线恢复肌肉与generic曲线，再用原Avatar转换为 donor本地SRT。原曲线（含missing-vs-zero mask/原默认指层证据）和转换后的可播放姿态分开保存。相同共享片段按实际controller/武器手部层上下文明确组合，不拿95维零数组补齐缺轨。运行时仅消费有界版本化binary，不带UnityPy/原数据库。普通verify只对冻结、经H0验证的locals重打包；完整原生rebake需要另列实际工具/环境。转换完整性未证明前不得将实验姿态标为原生Mecanim复刻或启用核心动作。

## 边界与兼容

新动作离线解析/导入脚本，assets/prepared/naraka/动作准备包，assets/naraka/运行时动画与来源清单；不改avatar/animation运行时代码。

依赖父任务的 animation-assets 与 humanoid-conversion 研究；不依赖战斗代码。表现子任务必须等本任务通过真实转换门槛，不能用占位片段先接入。

完整数据流、三段/防御/音效及回滚约定以父任务design.md为准。离线源路径不得进入运行时，当前请求不包含提交与推送。未知原始语义不能补造。
