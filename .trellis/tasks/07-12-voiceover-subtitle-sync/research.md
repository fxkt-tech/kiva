# 玩家配音供应商研究

研究日期：2026-07-12

## 当前结论

基于成本决策，MVP 改用 `node-edge-tts`。它通过 Edge 在线 TTS 服务生成音频，无需 API key；包本身支持代理、MP3 输出和带毫秒起止点的词级字幕 JSON。

领域层仍只依赖统一结果：规范文本、音频、真实时长、句级字幕 cue 和生成器标识。包的文件写入与字幕格式隔离在服务端 adapter 内。

上游资料：

- https://github.com/SchneeHertz/node-edge-tts
- https://learn.microsoft.com/azure/ai-services/speech-service/language-support?tabs=tts

## `node-edge-tts` 方案要点

- 固定包版本，不追随 `latest`；只允许服务端调用。
- 每个角色保存 `voice`、`lang`、`pitch`、`rate`、`volume` 与 adapter 版本，并快照进 Game Player。
- adapter 先写 Game 目录内临时文件，校验音频和字幕后原子改名；结构化记录最后提交。
- 包输出词级 `{part,start,end}`；应用先对规范文本分短句，再将词边界聚合为句级 cue。
- 字幕边界无法无损覆盖规范文本时，该条生成失败且不发布音频。
- 批量生成采用串行或极低并发，降低非官方服务限流风险。

## `node-edge-tts` 风险

- 它调用 Edge 在线服务，不是带 SLA 的正式 Azure Speech API；协议、访问限制和可用音色可能变化，因此只定位为低成本 MVP。
- Edge 只提供预设音色，不能做角色音色克隆；MVP 的“角色独立音色”是不同 voice/参数组合。
- Game 必须快照 voice 配置；生成前验证 voice 仍可用，生成成功后以本地固化音频为准。
- MIT 只是 npm 包代码许可证，不等于底层语音服务的商用授权；商业发布前需要单独完成条款核查。
- 免费调用把直接费用转化为可用性、合规性和维护风险。

## 未来正式供应商候选：MiniMax

- 官方同步接口单次返回音频和 `audio_length`，适合 Preview 中用户手动触发的短发言批量任务。
- 原生支持句级字幕时间戳，与本任务已确定的短句级同步直接匹配，不需要额外 ASR 或强制对齐。
- `speech-2.8-hd` 支持情绪、语速、音高、音量、发音词典、停顿和非语言语气标签，适合区分狼人杀角色。
- 提供系统音色和音色复刻；角色定义只需要保存稳定 `voiceId` 与少量生成参数。
- 提供北京备用 API 地址，国内服务端接入路径直接。
- 同步接口支持少于 10,000 字符，远高于单条玩家发言需求。

官方资料：

- https://platform.minimaxi.com/docs/api-reference/speech-t2a-http
- https://platform.minimaxi.com/docs/guides/speech-t2a-async
- https://platform.minimaxi.com/docs/api-reference/voice-cloning-clone
- https://platform.minimaxi.com/docs/pricing/overview

## 候选对比

### 火山引擎豆包语音：第二选择

优势：中文、情感和国内可用性强；异步长文本接口原生支持句级、词级和音素级字幕；公开套餐价格有竞争力；支持秒级音色复刻。

不足：产品线和接口族较多，音色、集群、模型及计费组合的接入认知成本更高；官方长文本文档说明异步返回可能随负载波动，虽然单条短发言可改用在线接口，但首版需要额外确认具体模型与字幕响应的一致性。

官方资料：

- https://www.volcengine.com/docs/6561/1096680
- https://www.volcengine.com/docs/6561/133350
- https://www.volcengine.com/docs/6561/1167802

### ElevenLabs：适合国际化，不适合当前首版

优势：音色生态、表现力和克隆能力强；时间戳接口返回字符级 alignment，可聚合为短句；中文受 Multilingual 模型支持。

不足：国内网络与结算路径更复杂；商业使用需要付费计划；字符级结果仍需本地聚合；官方定价页面显示单位分钟成本明显高于国内厂商常见套餐。它更适合作为未来国际版或高预算精品音色 adapter。

官方资料：

- https://elevenlabs.io/docs/overview/capabilities/text-to-speech
- https://elevenlabs.io/docs/api-reference/text-to-speech/convert-with-timestamps
- https://elevenlabs.io/zh/pricing

### Azure Speech：企业治理优先时再考虑

优势：成熟 SDK、SSML、事件边界与企业级云治理；中文标准音色覆盖稳定。

不足：独特角色音色通常涉及 Custom Voice 的额外准入、训练和治理流程；对当前“小团队、12 个鲜明角色、Preview 手动批量生成”的最短闭环不如 MiniMax。

官方资料：

- https://learn.microsoft.com/azure/ai-services/speech-service/text-to-speech

### 自建开源 TTS：当前不推荐

优势：数据和推理完全可控，长期规模化后可能降低边际成本。

不足：需要 GPU 推理、模型与音色授权审查、中文韵律调优、时间对齐、运维和质量监控。当前核心问题是可靠地产出配音与字幕时间轴，自建会把任务扩大成语音基础设施项目。

## 风险与验证

- 供应商输出具有非确定性，所以成功音频必须固化到 Game，不能按预览重新生成。
- 角色 voice ID 的长期可用性、系统音色商业使用条件和音色复刻授权需在正式商用前逐项确认。
- 在全面接入前应做 12 角色试听样本：相同测试文本、对白文本、数字/座位号、多音字、情绪句和长句各一组。
- 必须验证 MiniMax 返回的句级文本能否无损映射到提交的规范文本；若供应商切句不适合 Galgame 对话框，则由应用预先分句并在文本中插入受控边界，再校验返回 cue。
- 价格会变化，实施前以控制台实时价格为准；架构不把单价写入业务逻辑。
