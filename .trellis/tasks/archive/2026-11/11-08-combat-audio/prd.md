# 英雄战斗音效

## 目标与需求
对应父任务 [需求](../11-08-battle-presentation/prd.md) 的 R3、R4 音效部分，沿用其用户授权、范围与验收；用户明确要求方案后立即实现，无需再次确认。

## 验收
Rust 每完整固定步后、玩法清理前消费独立 SimulationEvent 读者，保存当次位置、身份与观察；只读 cue 配置按 AbilityId，WebAudio 是唯一 Web 播放器。可信进入/恢复手势主动启用 AudioContext，预加载、并发上限、静音/音量、暂停清理和新局边界完整。语音与 SFX 分开命名；不宣称 VO 等于源计划 SFX。

功能保持既有输入、模拟和整局生命周期；提供真实行为测试与集成证据，不以仅能编译视为完成。

## 边界
新 src/client/audio/ 或 audio.rs 插件、web/audio.js、对应单元/浏览器测试、assets/audio/及导入脚本与来源清单；不要改 Cargo/client/mod/index，给主代理接线说明。
不提交/推送，不接管开发服务。
