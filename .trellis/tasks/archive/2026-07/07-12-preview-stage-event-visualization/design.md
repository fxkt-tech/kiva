# Preview 舞台事件结果可视化 — Technical Design

## 1. Problem and boundary

中央舞台目前是一个空的 grid slot。事件已经被编译成 `PlaybackItem`，但该结构只保留展示文案，丢失了可稳定驱动图形的事件语义。不能从中文 `title` / `text` 反向解析行动者、目标和结果。

本设计在播放编译阶段生成结构化 `stage` 数据，并让 HTML Preview 与 Remotion 继续共享 `HtmlPlaybackStage`。不修改 GameEvent 持久化 schema，不新增网络请求，也不在 React 渲染层读取原始事件。

## 2. Data flow

```text
GameEvent
  -> shouldIncludeEvent(audience)
  -> stagePresentationForEvent(event)
  -> PlaybackItem.stage
  -> createShotFrame / stage view model
  -> StageEventVisual
  -> HTML Preview and Remotion export
```

可见性过滤必须先于 `stagePresentationForEvent`。舞台数据只能来自已经允许进入当前 audience 的事件，避免渲染层自行判断隐私。

## 3. Playback contract

为 `PlaybackItem` 增加可选的结构化字段 `stage: StagePresentation | null`。建议使用判别联合，而不是开放字符串或 `details` 解析：

- `action`
  - `action`: `protect | attack | inspect | antidote | poison`
  - `actor`: 玩家引用或 `wolves` 团队引用
  - `target`: 玩家引用或 `none`
  - `result`: `selected | good | wolves | used | skipped`
- `night_result`
  - `deaths`: 玩家引用与可用的死亡原因
  - 空数组表示平安夜
- `vote_result`
  - `voteType`: `exile | pk`
  - `outcome`: `exiled | tied | no_exile`
  - 候选人汇总票数、弃票数、出局玩家
- `game_result`
  - `winner`: `good | wolves`
  - `reason`: 复用现有 `GameEndReason`

玩家引用只携带 `playerId`；头像、座位、名称、身份从同一 scene 的玩家快照解析，避免复制过期展示数据。狼人刀人使用合成的“狼队”行动者，因为 `wolf_vote_resolved` 是团队结论且没有单一 actor。

## 4. Event mapping

| Event | Stage variant | Core display |
|---|---|---|
| `guard_protect_selected` | action/protect | 守卫 → 守护 → 目标；结果“已守护” |
| `wolf_vote_resolved` | action/attack | 狼队 → 袭击 → 刀口；平票待裁定时目标为“未确定” |
| `seer_check_selected` | action/inspect | 预言家 → 查验 → 目标；结果“查验中” |
| `seer_check_result` | action/inspect | 同一结构；结果印章“好人/狼人” |
| `witch_antidote_decided` | action/antidote | 使用时指向获救玩家；未使用时目标为“未使用” |
| `witch_poison_decided` | action/poison | 使用时指向中毒玩家；未使用时目标为“未使用” |
| `night_resolved` | night_result | 最终死亡名单与原因；空数组为“平安夜” |
| `death_announced` | night_result | 公开视角的昨夜死讯；不展示私密死亡原因 |
| `exile_resolved` | vote_result | 放逐或 PK 结论、候选汇总票数、弃票数 |
| `game_ended` | game_result | 胜利阵营与获胜原因 |

`hunter_shot_decided` 暂不进入首批范围，结构上后续可复用 action 变体。

## 5. Audience and privacy

| Information | Director Preview | Public audience |
|---|---:|---:|
| 守卫、狼刀、查验、女巫选择 | 显示 | 不显示，对应事件已被过滤 |
| 夜间即时结算 | 显示 `night_resolved` | 不显示 |
| 次日公开死讯 | 显示 | 显示 |
| 放逐 / PK / 对局结算 | 显示 | 显示 |

当前 `shouldIncludeEvent` 无条件过滤 `night_resolved`。需调整为：仅 public 过滤，director 保留，以满足上帝视角在夜间结算环节显示最终结果。`wolf_vote_cast` 等逐票密票仍然无条件过滤，舞台只消费 `wolf_vote_resolved`。

## 6. Visual system

### Shared container

- 使用中央舞台现有区域，不侵占 Header、左右座位轨和底部 Galgame 字幕。
- 容器沿用 day/night palette 的 surface、border、shadow、accent。
- 信息层级：环节小标题 → 主视觉 → 结果印章 → 可选摘要。

### Action layout

- 左：行动者（玩家头像卡或狼队徽章）。
- 中：动作图标、动作名称、水平连线。
- 右：目标玩家头像卡或空状态卡。
- 下：结果印章（已守护、查验中、好人、狼人、已救、已毒、未使用、未确定）。

### Resolution layouts

- 夜间：一人死亡用单卡；多人死亡并排；无人死亡使用居中“平安夜”。
- 放逐/PK：有出局者时居中突出该玩家及票数；平票时并排候选及票数；弃票仅显示摘要。
- 对局结束：只显示胜方大标题和获胜原因，不重复全员身份。

## 7. Animation and timing

所有动画只使用 `shot.clock.sceneMs/progress`：

1. 0–350ms：容器 opacity 0→1，translateY 18→0。
2. 250–750ms：连线 scaleX 0→1，动作图标 scale 0.9→1。
3. 650–1050ms：结果印章 opacity/scale 进入。
4. 场景最后 350ms：整体轻微淡出，继续复用当前场景切换机制。

短于标准时长时按 scene progress 归一化；不得使用 CSS keyframes、setTimeout、随机数或 DOM 测量，以保证逐帧导出确定性。

## 8. Duration policy

先不增加新的场景，也不改变配音时序。舞台动画适配现有场景时长。若无配音的行动场景不足以完整展示，统一提高对应 `durationForEvent` 的最低时长，而不是在组件内延迟时间线。

## 9. Compatibility and migration

- `PlaybackItem.stage` 为可选字段，旧视频 composition 可解码并以空舞台降级。
- 不提升 video composition schema version，前提是 decoder 对缺失 `stage` 使用 `null`，并为新判别联合做显式验证。
- 不修改历史 `record.json` 或 GameEvent schema。
- 回滚只需停止生成/渲染 `stage`；原有标题、字幕、座位卡保持可用。

## 10. Test strategy

- Unit：每个事件到 `StagePresentation` 的映射，包括未用药、狼队平票、无人死亡、放逐平票、PK 平票、弃票、双方胜利原因。
- Privacy：public playback 中不存在夜间私密 `stage`；director 包含它们；密票永不进入舞台。
- Decoder：缺失 `stage` 可兼容；非法判别值拒绝。
- View model：玩家引用正确解析，缺失玩家安全降级为空状态而不抛错。
- Render：每个 variant 的静态 markup 快照/关键文本、昼夜 palette 与动画中间帧。
- Integration：相同 composition + frame 在 HTML Preview 和 Remotion 产生相同舞台结构。
