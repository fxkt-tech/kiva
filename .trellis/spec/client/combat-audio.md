# 英雄战斗音效

## 1. 范围与所有权

适用于英雄普攻、技能提交、死亡与重生音效、音频资源、Web 播放器、暂停设置及音频诊断。
`src/client/audio.rs` 唯一拥有事实到 cue 的转换；`web/audio.js` 唯一拥有 Web Audio 设备和混音。
不启用第二套 Bevy Audio 播放路径，不在动画、输入、HUD 或模拟层直接播放声音。
模拟继续只发布既有 `SimulationEvent`；声音可用性不得影响指令、伤害、冷却或对局推进。

音频与约 80ms 的完整 HUD 投影分离。模拟固定步、音频显示帧与完整 HUD 三种频率不能混用。
关联会话/捕获见 [character-selection.md](character-selection.md)，团队观察见
[champion-presentation.md](champion-presentation.md)，浏览器夹具见 [browser.md](browser.md)。

## 2. 接口与接线

- `CombatAudioPlugin` 初始化独立 `AudioRuntime`，使用自己的 `MessageCursor<SimulationEvent>`。
- `capture(&mut World)` 位于每次 `FixedUpdate` 的 `SimulationSet::Step` 之后、
  `GameplaySet::Outcome` 之前，必须早于后续固定步及玩法清理。
- `publish(&mut World)` 位于 `PostUpdate`、`camera::update` 之后，按显示帧发布。
- 同步经典脚本 `audio.js` 在 Rust/WASM 启动前安装 `kiva-audio` 监听器；不得依赖 defer HUD
  及时注册。本地 `assets/audio/` 随 Trunk 构建复制，不在游戏启动时导入或访问外部音源。
- 分类设置的声音面板在DOM就绪后绑定 `#audio-muted` 和 `#audio-volume`；后者范围0–100，默认65，控件各一份。
- `window.kivaAudio = Object.freeze({unlock,setMuted,setVolume})` 只有音频副作用。
  `window.__kivaAudio` 是冻结的只读诊断快照，不公开 AudioContext、节点或写 World 入口。

每路 voice 使用 `source → gain → pan → master gain → analyser → destination`，
共享单一 `AudioContext`、master 和 analyser。上下文使用 `latencyHint: "interactive"`，
页面生命周期内复用；暂停不重建或关闭设备。

## 3. 数据、时序与生命周期契约

`kiva-audio` 的 `CustomEvent.detail` 是 JSON 字符串：

```json
{
  "frame": 12,
  "session": "1:4294967000",
  "paused": false,
  "tick": 120,
  "catalog": null,
  "cues": [{
    "sequence": 1, "tick": 120,
    "source": "4294967000", "life": "0",
    "cue": "jarvan.w", "variant": 0,
    "gain": 0.85, "pan": 0.0, "priority": 2
  }]
}
```

| 字段 | 契约 |
|---|---|
| `frame` | 显示发布序号，跨重开持续单调递增；JS 拒绝旧/重复 frame |
| `session` | Active 时为 `MatchId:EntityBits` 字符串；无玩家为 null；死亡不改变会话 |
| `paused` / `tick` | 权威暂停和模拟 tick；不能用音频墙钟推导模拟状态 |
| `catalog` | 首次显示发布带 `{cues}`，以后 null；英雄/技能映射留在 Rust |
| `cues` | 当前已消费的瞬时事件，最多 48 条；不作为持久状态或延迟播放队列 |
| `sequence` | 当前会话单调序号；重复/旧序号不能再次播放 |
| `source` / `life` | EntityBits 和生命 epoch 均用字符串，避免 JS 数字精度丢失 |
| `cue` / `variant` | 已知 cue ID 和该 cue 本地文件数组下标 |
| `gain` / `pan` | Rust 给出的有限数值，Web 分别约束到 0–1 和 −1–1 |
| `priority` | Rust 队列裁剪使用；Web 从自身已安装的同一静态 cue 定义读取优先级 |

### 固定步事实捕获

只处理实际 `AttackReleased`、`AbilityCommitted`、`Died`、`Respawned`；拒绝施法、瞄准、
输入按下、动作起播或命中动画不能触发同等音效。必须在当次固定步保存源位置和生命引用，
不能等显示帧再从当前已重生/已清理实体补读。独立 cursor 保证每个提交消费一次，不影响其他读者。

当前只为带已配置 `CharacterIdentity` 的英雄发声。英雄攻击/死亡/重生 cue 来自角色映射；
技能 cue 按已提交的 `AbilityId`，不能按 QWER 槽位或角色默认技能推断，混合装配仍正确。
寒冰强化普攻从实际 `UnitMechanics::empowered()` 选择独立 cue，不补算层数或强化时长。
随机变体使用客户端独立 PRNG，多个变体不连续重复，不消耗模拟随机状态。

源在事件发生固定步必须通过当前玩家团队的 `team_can_see`；隐藏敌人的 cue 和位置不能进入 Web 包。
镜头平移不给视野。显示发布时再按 `CameraRig.center` 衰减听距：30 世界单位外丢弃，
范围内 gain 乘 `(1-distance/30)^2`，声像沿屏幕右轴 `(2,-1)` 计算并限制到 ±0.8。
当前玩家自己的死亡/重生通知不受镜头听距影响。

Rust 暂存上限 48，优先裁剪最旧的最低优先级 cue；低优先级攻击不能挤掉更高优先级生命通知。
Rust 和 Web 都丢弃超过 15 tick 的 cue，长显示停顿不能在重生后补播旧死亡。
队列清空与消费不依赖成功播放。

### 解锁、暂停、焦点与重开

`audio.js` 的 document capture/passive `pointerdown`、`keydown` 监听器只接受 `isTrusted`，
同步创建/调用 `AudioContext.resume()`，再等待其 Promise。不能先 await 资源或另开定时器再解锁，
不能 preventDefault、消费或改写游戏输入。音频解锁成功也不代表游戏已进入：播放仍要求当前会话、
未暂停、文档聚焦，且`#game`实际Pointer Lock或设置打开。

文件可提前 fetch；上下文存在后异步 decode。未解锁、未解码、静音、音量 0 或暂停的 cue 直接丢弃，
资源后来就绪/恢复音量不补播。设备激活失败不阻断进入游戏，后续可信手势可再次尝试 resume。

暂停、`data-paused=true`、非设置丢失捕获、blur、document hidden都停止并断开全部active voice；运行中设置允许未捕获时继续新cue和已有voice。
Rust 暂停也清空待发布 cue。普通继续复用当前会话和 AudioContext，不重播旧事实。
session 改变时两端清理待播/活动声音，Web 同时清空生命表和 sequence；Rust 显示 `frame` 不归零。
同源出现新生命 epoch 时，停止该源旧生命仍在播放的节点，保留其他源声音。
整场重开/返回选角不能通过 tick 归零重新接受旧包或保留旧局节点。

Web 并发上限 16；满载优先替换较低优先级最旧 voice，高优先级声音不被低优先级攻击挤掉。
自然结束释放 source/gain/pan；手动停止移除 ended 回调并断开同一组节点。
音量默认 0.65，静音和音量用 `kiva-audio-muted`、`kiva-audio-volume` 持久化；存储不可用仍正常工作。

## 4. 校验与错误矩阵

| 情形 | 行为 |
|---|---|
| checked-in catalog 字段/映射/文件路径/增益/优先级无效 | Rust 启动校验失败，作为内容开发错误修复；不能静默猜测 cue |
| 未知角色/未映射技能或非英雄事件 | 不产生 cue，不改变模拟 |
| 敌方实际施法但没有团队视野 | Rust 内过滤，不向 JS 泄露音频或坐标 |
| 非法 JSON、旧 frame（含旧局迟到包）、重复 sequence、过期 tick | 不播放；旧显示包不能恢复旧局 |
| 设备不可用、resume 拒绝、HTTP 或 decode/playback 失败 | 保持游戏可用，错误记录到只读诊断，无待解锁音效积压 |
| 资源 fetch/decode 尚未完成 | 丢弃该次 cue；decode 完成只更新资源状态 |
| 静音/音量 0 | 当前声音停止；新 cue 消费后丢弃；恢复不补播 |
| 暂停/失焦/丢捕获/重新选择 | active 节点归零，按相应会话边界清理，无旧声音残留 |
| native 测试运行 | 只生成/消费 cue，不要求音频设备、ALSA 或浏览器上下文 |

`__kivaAudio` 包含 session、paused、volume、muted、active、played、dropped、context_state、
loaded_files、expected_files、ready、peak、played_by_cue、errors；错误列表最多 8 条。
`peak` 来自实际 analyser 样本，不能用计数增长替代解码后输出证据。

## 5. 正常、默认与错误示例

- 正常：皇子 W 真正提交后一次 `jarvan.w`；寒冰混装该技能仍播放 `jarvan.w`。
- 正常：两次显示发布之间发生真实死亡和重生，固定步快照保留各自位置/epoch；过期死亡不补播。
- 默认：选角无 session，允许预加载和可信手势初始化设备，仍不播放战斗音效。
- 默认：默认音量 65%，两英雄当前实际资源为源计划寒冰与原皮皇子；皇子重生单独使用已注明来源的通用音效。
- 错误：技能按钮按下就播放、隐藏敌人跨图可听、重选后旧节点仍响，或音频 await 阻塞开始按钮。

资源契约在 [assets/audio/README.md](../../../assets/audio/README.md)：本地 OGG、catalog、
`sources.json` 的 event/WEM/输出 SHA 和 `riot-sources.json` 的原始版本/分块 SHA 一起维护。
离线导入工具只在主动更新资产时运行。不能将 VO 标成武器 SFX、旧版本样本标成当前皮肤，
也不能把选定媒体导出宣称为完整复刻 Wwise RTPC/材质切换/音高图。

## 6. 必要验证与证据边界

- `cargo test --locked --lib client::audio`：真实命令驱动拒绝/一次提交、混装、4 次普攻后 Q 强化攻击、
  两显示帧间死亡重生、隐藏/可见敌人、暂停/新局、积压上限/通知优先级、衰减和变体。
  不用伪造 `SimulationEvent` 代替实际提交证明。
- `node --test scripts/audio-unit.test.cjs`：生产 JS 所有者的设备唯一性、未解码不补播、暂停/失焦/重选、
  旧 frame/生命、16 路优先级、设置和真实加载失败分支。VM 中设备是测试替身，不证明浏览器真实输出。
- `python3 scripts/import-audio.py --verify`：全部本地输出 SHA；资产更新还应完整解码并从空缓存重建比较。
- `node --check web/audio.js`、`make check` 和 WASM Clippy 保持通过；native 不加设备依赖来绕过 Web 解锁。
- 构建后串行运行 `node scripts/browser-audio.cjs`：可信选择/进入、实际文件全部 decode、真实皇子/寒冰技能提交、
  `played_by_cue` 与 analyser 非零输出、暂停 active=0、静音不补播、音量 range、重选切换会话。
  临时静态 fixture 在 finally 清理，不接管用户开发服务，不暴露写 World 后门。

浏览器帧时钟只归一模拟/显示交互，不改变 AudioContext 的真实音频时钟；不能把虚拟 tick 当作音频时长。
原生适配、VM 节点测试、哈希/ffmpeg 解码、真实浏览器输出分别证明不同边界。
浏览器脚本准备完毕、纯逻辑测试通过，都不能写成已经通过真实浏览器听觉/输出回归。

## 7. 错误与正确

错误：在完整 HUD 快照中观察“攻击中”后反复播放；等到显示帧才读取死者的当前重生位置；
decode 完成后批量补播积压音效；每次继续创建一个新的 AudioContext。

正确：固定步独立 cursor 读取真正提交，保存当次源位置和生命、先过滤视野；显示帧只发布有时效的 cue；
单一已解锁设备立即尝试播放，缺少任何就绪条件即丢弃；暂停/换局断开全部节点并清理暂存。
