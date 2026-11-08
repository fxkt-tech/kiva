# 英雄战斗音效技术设计

继承[共同设计](../11-08-battle-presentation/design.md)。

Rust 每完整固定步后、玩法清理前消费独立 SimulationEvent 读者，保存当次位置、身份与观察；只读 cue 配置按 AbilityId，WebAudio 是唯一 Web 播放器。可信进入/恢复手势主动启用 AudioContext，预加载、并发上限、静音/音量、暂停清理和新局边界完整。语音与 SFX 分开命名；不宣称 VO 等于源计划 SFX。

## 变更边界
新 src/client/audio/ 或 audio.rs 插件、web/audio.js、对应单元/浏览器测试、assets/audio/及导入脚本与来源清单；不要改 Cargo/client/mod/index，给主代理接线说明。

## 协作
读取本任务 research/ 中的已验证样例；公共接线由主代理负责，文件重叠先沟通。实现完成只报告实际验证，不能静默省略需求或伪造资源来源。
