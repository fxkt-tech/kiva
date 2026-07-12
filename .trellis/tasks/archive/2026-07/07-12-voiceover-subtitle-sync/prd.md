# 配音与字幕同步

## Goal

让最终成片中的主理人播报和所有玩家发言都有稳定、可辨识的配音，并由真实音频时间驱动画面、Galgame 对话框和字幕，使 Preview 与 Remotion 导出完全一致。

## Background

- 当前播放模型已区分主理人与玩家说话者，Remotion composition 也预留系统语音和玩家语音 cue。
- 当前主理人音频依赖静态 `voiceFile`，玩家正文没有配音；场景时长主要按文本长度估算。
- 当前玩家字幕按标点/字数切窗，再按场景总进度平均切换，整段文案可能提前显示，无法跟随真实发音。
- 当前 Game 是 `kivdb/games/<gameId>.json` 单文件，无法自然拥有二进制配音资产。
- 所有现有音频、旧 `voiceFile` 映射及兼容逻辑均作废；本任务不迁移、不兼容。

## Requirements

### Voice identities

- 每个角色定义拥有 Edge 音色配置：`voice`、`lang`、`pitch`、`rate`、`volume` 和 adapter 版本。
- 创建 Game 时把角色音色完整复制到对应 Player snapshot；角色库后续修改不得改变已有 Game。
- MVP 固定使用三种已验证音色：主理人 `zh-CN-YunjianNeural`、男玩家 `zh-CN-YunxiNeural`、女玩家 `zh-CN-XiaoxiaoNeural`；所有语音统一使用 `rate: "+20%"`（1.2 倍速），角色可保留不同 pitch/volume，但不使用其他 Edge voice。
- 主理人拥有固定音色和预制语音库，不按 Game 重新生成。

### Presenter voice library

- 主理人语音通过开发期构建命令使用 `node-edge-tts` 预制；Preview 和 Game 运行时只读构建产物。
- 主理人任何仅含一个座位变量的文案（点名、夜间目标、查验、用药、守护、猎人、单人结算等）必须分别预制 1–12 号的完整句子，一次只播放一个连续 MP3，不得把句子与座位号拼接。包含两个玩家或任意动态名单的文案继续通过预制片段组合，不枚举组合全集，也不朗读玩家姓名。
- 构建产物必须包含音频 manifest，并校验全部必需语义片段和合法座位号齐全。
- 主理人播报通常很短，不做字幕短句拆分；一条播报作为一个字幕 cue，覆盖其全部组合音频的起止区间。
- 未来可以用真人录音替换相同 manifest 契约下的音频，而不改变 Game 或播放模型。

### Player voice generation

- 玩家配音不得在 Editor 阶段自动生成；用户只能在 Preview 右栏显式创建“生成全部配音”任务。
- 生成对象是目标成片中所有已确认且 `transcriptSpeaker === "player"`、尚未成功生成的 event，包括公开发言、PK、遗言、狼人策略和狼人意见。
- 未确认 draft 只能无声预览，不生成配音。
- 每份玩家配音绑定稳定 `eventId`；成功产物不可覆盖或重新生成，失败和中断项可以重试。
- 批量生成使用服务端持久化 job 与低并发队列。页面刷新或离开不取消任务；服务重启后活动 job 标记为 `interrupted`，用户手动重试。
- Player 配音文件和生成记录归对应 Game 所有，不得放入全局 `kivdb/assets`。

### Adapter boundary

- MVP 服务端 adapter 使用固定版本的 `node-edge-tts`，不得由浏览器直接连接 Edge TTS。
- 标准配音领域请求保留作者文本，不把“删除括号动作说明”定义成通用规则。
- `node-edge-tts` adapter 可以把 `（停顿）`转换为边界/静音，并删除其他无法表达的动作说明；它必须返回实际合成的 `synthesizedText`。
- 原始 event 文本保持不变并记录哈希；字幕以 `synthesizedText` 和真实词级边界为准。
- Edge 词级 `{part,start,end}` 必须聚合成应用短句 cue；无法无损覆盖 `synthesizedText` 时该条生成失败，不发布半成品。
- 播放与 Remotion 层只消费统一音频/字幕契约，不依赖 `node-edge-tts` 字段。

### Timeline and subtitles

- 浏览器 Preview 与 Remotion 导出必须消费同一份已解析时间线。
- 场景时长以最终音频真实时长为主，不再以字数估时作为有声内容的权威值。
- 玩家发言时间线依次为：主理人点名、250ms 空白、玩家正文、350ms 收尾；主理人动态片段间隔为 80ms。
- 同一时间最多播放一个语音 segment；主理人与玩家、相邻玩家之间不得发生声音重叠。
- 主理人 segment 显示主理人身份；玩家 segment 才切换玩家高亮和 Galgame 对话框。
- 主理人与玩家复用现有下方对话框。主理人整条短播报完整显示；玩家长发言按真实短句 cue 瞬时替换完整短句，不使用打字机、逐字高亮或跨句累积。
- 玩家姓名和头像在整段发言期间保持稳定；未播到的后文不得提前显示。

### Availability and export gate

- Preview 必须展示 `待生成 / 排队中 / 生成中 / 已完成 / 失败 / 已中断` 状态和逐项结果。
- 配音不完整时 Preview 仍可播放；未生成或失败的玩家 event 使用无声画面和完整静态文本降级。
- 正式导出只在全部目标玩家配音成功且全部必需主理人音频存在时允许启动，并指出阻塞项。
- 删除 Game 时必须一并删除其玩家配音与 voice job，不影响主理人语音库或其他 Game。

### Process isolation

- Next.js 进程只创建、查询和取消持久化任务，不执行 Edge TTS、Remotion 或 FFmpeg。
- 玩家配音与视频导出分别由独立的单并发 Worker 轮询领取，重任务不得阻塞页面 SSR、API 或热更新。
- 尚未领取的 `queued` 任务在任一进程重启后继续排队；只有已经开始执行的任务才转为 `interrupted`。
- 跨进程取消以持久化 job 状态为准，不依赖 API 进程内存。

## Acceptance Criteria

- [ ] 新建 Game 的每个 Player 都含有来自角色定义的不可变音色快照。
- [ ] 主理人构建命令生成并校验完整 manifest；运行时不调用 TTS 生成主理人语音。
- [ ] Preview 右栏一次创建包含全部缺失玩家配音的持久化 job，离开页面后仍继续执行。
- [ ] 服务重启将活动 job 标记为 `interrupted`；重试跳过已成功 event，且成功音频永不被覆盖。
- [ ] 所有进入成片的玩家说话 event（含狼人私聊）均受相同的生成和导出门禁约束。
- [ ] 每份玩家产物可由 `eventId` 解析到音频、真实时长、实际合成文本和短句 cue。
- [ ] Edge 动作说明适配不会修改 event 原文；字幕与实际读出的文本一致。
- [ ] 主理人点名、250ms 间隔、玩家正文和 350ms 收尾严格串行；动态主理人片段间隔为 80ms。
- [ ] 玩家 Galgame 对话框按真实短句 cue 切换，不按场景进度平均分配，不提前展示后文。
- [ ] Preview 与 Remotion 对同一 composition input 产生相同的场景边界、音频起止点和字幕窗口。
- [ ] 配音不完整时 Preview 可用但正式导出被阻止，并明确列出阻塞项。
- [ ] 删除 Game 会删除 Game 目录内的全部配音和 voice job。
- [ ] 删除所有旧音频和旧映射后，新方案仍可独立完成预制、生成、Preview 与导出。
- [ ] 仅运行 Web 进程时任务保持 `queued`；启动对应 Worker 后才执行，生成期间主页和 Preview 仍可响应。

## Out of Scope

- 自定义声音克隆。
- 多家 TTS 供应商同时可配置；只保留可替换 adapter 边界。
- 背景音乐、环境音、音效混音、唇形同步。
- 逐字卡拉 OK、打字机效果、动作说明的可视化演出。
- 旧 Game、旧音频和旧 `voiceFile` 数据迁移。
- 生产 SLA；`node-edge-tts` 是低成本 MVP 通道，正式商用前另行核查底层服务条款。
