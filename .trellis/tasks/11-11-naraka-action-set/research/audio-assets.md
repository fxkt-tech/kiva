# Research: 永劫无间横刀与移动音效

- Query: 从指定本机客户端恢复横刀攻击、振刀及对应通用动作的原音效，验证媒体能否离线解码并供 Web 使用。
- Scope: internal；原始客户端只读，未运行、注入或修改游戏；规划研究，没有修改 Kiva 产品代码。
- Date: 2026-11-11

## Findings

### 结论和证据等级

音频解析路线已实际跑通，不是阻塞项。17 个原事件的 Event→PlayAction→Random/Switch/Sound→DIDX→WEM 链已恢复；每个事件至少一个原媒体实际解码 PCM 并转换 Vorbis/Ogg，原事件的其余媒体记录 ID/范围/SHA。横刀三段攻击、振刀、闪避、跑步、起跳、落地的**动作到声音连接来自原 proto_track_archive**，不是仅按英文命名推测。

这不等于原 Wwise 行为全部恢复：动态材质 switch、RTPC、随机音高/增益、衰减、混响总线和 proto 时间字段/组的语义仍有限制，见后文。未做浏览器 AudioContext 解码或主观听感验收。

### 原文件与可重复读取路径

- 客户端 `NarakaBladepoint_Data/StreamingAssets/Audio/GeneratedSoundBanks/Windows/` 是标准 `AKPK` PCK，不在 Unity AudioClip 中。`SoundActor1.pck` 434,934,120 B；`SoundActor2.pck` 237,043,210 B。目录还含语言、音乐、环境及其它分包，未全量展开。
- 原 `WwiseBanks` TextAsset：配置包 `f/a/fab72152d7112058`，PathID `-8685861610438291164`，12,731,643 B，原 SHA `b453b211dee64ef887e43d6bddfb3845ca500a26641bc418a5180d077a762a4a`。用已有版本限定 XOR 规则从原 `.bytes` **重新解码**，与已有 `association/WwiseBanks.dbbl` 逐字节相等；decoded SHA `0fbb905eb229ebe9e93a37cdfabd9d7e78755256fa0d2c5b5afec542d4b11f75`，见 [audio-dbbl-verification.json](audio-dbbl-verification.json)。
- 这张表用 UTF16 **字符串键+记录长度**索引，不是既有通用解析器的 `u32 ID + size`；新研究探针严格验证四个表全边界及记录自有键，共 Bank1125、Event51088、Localization13、SwitchGroup3。
- [audio_probe.py](audio_probe.py) 读取 PCK header 中每个 section 的 count、20 B entry、alignment×offset 与实际文件范围；银行名字按 Wwise 小写 FNV-1 算 ID，再与 BKHD/HIRC 实际对象交叉核对。所有已读 Bank 的 BKHD version=135。仅哈希 PCK 目录和选中原 Bank/媒体，**不宣称已计算整 PCK SHA**。
- 动作声音原表：`e/5/e50ef81ba77ae51a` 中 `assets/raw_resources/proto_bytes/proto_track_archive.bytes`，PathID `-5518007691462251984`，原 object30,107,448 B、SHA `707b99d88aaefb825b9c79a858c4a2dae9607977dbfd5b5f13d596f07b81671b`；payload30,107,418 B、SHA `93dcf8a2c161571fe190b2151e5247dc796235140087bdbf172d930abc1268f1`。只为此确证对象传 `SparseBundle(max_read=32MiB, read_budget=160MiB)`，实际块读取证据另存本地完整报告。
- `a/8/a89c86ec6df2a5d1` 原 `reference_sound_config.asset` 明确中文“拼架刀音效”→`gman_all_break_hurt`，以及“男性死亡音效”→`gman_dying_voice_male`。简要原树及对象 SHA 见 [audio-config-selected.json](audio-config-selected.json)。`statesetsoundconfig.asset` 仅 saw 蓄力和冰面滑动两项，不能用于证明横刀。

### 动作到声音的原始连接

[audio-tracks-selected.json](audio-tracks-selected.json) 保留原 entry、group、event 的字节位置与 SHA、sound 参数及未知数值字段；[audio_tracks.py](audio_tracks.py) 可从原 payload 重建。完整向量子消息仅保留范围/哈希以缩小报告。

| 原动作 key | 原声音 event | 原 group数值 / time float |
| --- | --- | --- |
| `male_hengdao_attack_light_01` | `gman_hengdao_attack_light_01` | 1 / 0.1153846085 |
| `male_hengdao_attack_light_02` | `gman_hengdao_attack_light_02` | 1 / 0.2769230306 |
| `male_hengdao_attack_light_03` | `gman_hengdao_attack_light_03` | 1 / 0.1000000015 |
| `male_hengdao_attack_shock_light_01` | `gman_hengdao_attack_shock_light_01` | 1 / 0.1070402414，条件声音轨 |
| `male_hengdao_shock_01`、`shock_04` | `gman_all_weapon_touch_shock_katana` | 1 / 0.0505268276，条件声音轨 |
| `male_hengdao_dodge_front_01` | `gman_dodge_01` | 1 / 0.0289224703 |
| `male_hengdao_dodge_back/left/right_01` | `gman_dodge_01` | 1 / 0.0724637583 |
| `male_hengdao_jump_inplace_up_01`、`jump_forward_up_01` | `move_jump_takeoff_cloth_fast` | 1 / 0.3260869086 |
| `male_hengdao_jump_down_to_land_01` | `move_jump_land` | 0 / 0.0437397808 |
| `male_hengdao_run_01` | `move_run` | 0 / 0.3249999583、0.8249999881 |

直接声音是 track type10 / payload union110；条件声音是 type170 / union225。后者同时保留带 `_donghuang` 的另一分支，其额外 numeric field2=3611698；**此数字的类型/条件语义未恢复，不把它擅称皮肤 ID**。原表在这里没有 `touch_shock_hengdao` 专名，实际横刀轨引用的是 `...katana` 共享声音。

`attack_shock` 是后续反击动作声音，不能仅因名字里有 shock 就作为成功碰撞声。`shock_01` 本身的声音连接也不能证明 Kiva 某次防御实际成功；产品中成功/受击声必须仍由真实结算事件决定。

### 实际可解码媒体与体积

完整证据 [audio_sample_trace.json](audio_sample_trace.json)，精简测量 [audio-measurements.json](audio-measurements.json)，复现 [audio_sample_trace.py](audio_sample_trace.py)。表中时长/体积为**首个原变体**，不是事件总变体之和；所有媒体原样完整解码，无裁剪、拼接、归一化或变速。来源均为 codec `0xffff` Wwise Vorbis、44.1kHz；挥击/碰撞多为 stereo，脚步为 mono。

| 原 event | 媒体变体 / PlayAction数 | 首变体时长 | 首变体 Ogg |
| --- | --- | --- | --- |
| `gman_hengdao_attack_light_01` | 2 / 1 | 1.160s | 19,871 B |
| `gman_hengdao_attack_light_02` | 3 / 1 | 1.413s | 21,577 B |
| `gman_hengdao_attack_light_03` | 3 / 1 | 1.741s | 23,700 B |
| `gman_hengdao_attack_shock_light_01` | 1 / 1 | 2.611s | 38,628 B |
| `gman_all_weapon_touch_shock_katana` | 1 / 1 | 3.119s | 42,655 B |
| `gman_all_weapon_touch_bounce_katana` | 5 / 1 | 1.832s | 30,142 B |
| `gman_all_jump_start_pre_01` | 4 / 1 | 0.756s | 9,772 B |
| `gman_all_jump_down_recover_01` | 4 / 1 | 4.200s | 50,989 B |
| `gman_dodge_01` | 6 / 1 | 0.932s | 14,079 B |
| `impact_body`（原默认 switch） | 5 / 1 | 1.134s | 18,237 B |
| `move_run`、`move_sprint`（各原默认 switch） | 各8 / 1 | 各0.484s | 各8,390 B |
| `move_jump_takeoff_cloth_fast` | 4 / 1 | 0.660s | 12,579 B |
| `move_jump_land`（各层原默认 switch） | 16 / **2** | 首层0.589s | 首层9,103 B |
| `gman_all_break_hurt` | 1 / 1 | 2.842s | 39,843 B |
| `gman_all_die_drop` | 1 / 1 | 0.856s | 12,516 B |
| `gman_dying_voice_male` | 7 / 1 | 1.170s | 18,123 B |

17 个首变体 Ogg 合计378,594 B，不能当作保留所有变体/分层的最终包体。保留两个小试听证据：`audio-sample-gman_hengdao_attack_light_01.ogg` 和 `audio-sample-gman_all_weapon_touch_shock_katana.ogg`。其它中间 WEM/WAV/Ogg 在测量后删除，脚本可重新提取。

离线解码使用 `/home/asuka/kiva-asset-research/vgmstream-r2168/vgmstream-cli`，可执行 SHA 记录在 JSON；转换命令 `ffmpeg -v error -y -i full-source.wav -c:a libvorbis -q:a 4 output.ogg`，再由 ffprobe 核对通道/采样率/时长。完整 PCM 的样本数、峰值、RMS、WAV/Ogg SHA 均已保存。例：三段挥击峰值0.7380/0.7486/0.6722、RMS约−25.87/−25.05/−26.34dBFS；shock共享金属声原峰值到1.0、RMS−19.51dBFS。不得因为源电平高就暗中改写原数据。

### Switch 和分层边界

- `impact_body` 的 group ID2553674422 与原 `impact_body` 名 FNV 一致；默认 switch2454616260→random443331604→5个媒体。原还有 switch3071189824、1719082493两支。**分支值的“肉体/护甲/强弱”等名称未恢复**，现阶段只可称“原默认分支”，不能称特定护甲或强打击。
- `move_run/sprint` group1936126619 对应原 `material_general`，各21个材质分支；默认2195636714。两个默认分支的8个 source ID完全相同，不能把同媒体冒称两套独立原脚步。步频/使用状态差异应由真实运动决定。
- `move_jump_land` 的 Event 有2个 PlayAction，两层各自经过 material switch和随机组，默认共16个终端 Sound。生产 cue schema 应是 `layers: [{variants:[...]}, {variants:[...]}]`；每次每层各选一个原变体，**只取第一个 Sound 不等于还原此事件**。两层一起参与全局声部和峰值预算。
- 已恢复随机组的唯一有界 children/playlist尾部，保留每项原 weight。未解释的 NodeBase/RTPC/随机前缀仍不赋予虚假含义。可以在 Kiva 明确采用确定性轮换/按原权重选择，不能宣称复刻全部 Wwise random模式。

### 运行时接缝与推荐方案

1. 当前 `src/client/audio.rs:49` 按 sequence消费 `CombatFeedback`，暂停也前移；`audio.rs:58` 将所有近战归到attack、所有三种防御成功归到deflect。需按不可变 action_id/stage和实际结算区分，不能从“当前动画名”反推旧事件阶段。
2. 三段挥击使用阶段开始权威 cue，再由客户端单次调度偏移；**声音自然音高/时长保持不变**，动画变速不应连带变调。取消动作/暂停/死亡/换武器时清掉未发出的 cue；没有音频解锁或缓冲未就绪时丢弃，恢复不补播。
3. **原 time float 单位以及 group0/1/2含义尚未恢复。** 虽然数值像相位，不能直接标成秒或把各组全合并。保留原数值作证据；首版若使用 Kiva 自有偏移，manifest明确写适配值和依据，不冒称原帧时序已还原。
4. `Deflect/Block/Parry` 的声音只在真实对应结算时触发。可将原“拼架刀”用于普通成功拆/挡、共享shock金属声用于成功弹反，作为**Kiva强弱区分**记录；这不是已证明的原游戏强弱映射。振刀尝试是否配预备声需产品规则明确，不能每帧发碰撞。
5. 起跳/落地/闪避用实际状态边缘；脚步只在 grounded且实际碰撞后位移达到步距时推进，或在共享姿态周期越过已选Kiva标记时触发。空中/原地顶墙/暂停不发脚步。不能用输入held或每帧update发声。传送/复活清理基线，落地一次即可，不按接触抖动连发。
6. `web/audio.js:34` 已有可信手势后异步fetch/decode一次；`audio.js:62` 全局4个短音效声部，BGM单独循环，gain0.18；`audio.js:181` 桥API。扩展简单文件名白名单与变体/分层cue，保留BGM和原音频解锁、暂停、失焦、旧context清理契约。落地双层仍计入4声部；成功金属/命中优先于脚步，低优先级不能截掉刚发生的关键声音。音量不按网络下载完成自动补播。
7. `web/audio.js:3` 查看页现在直接退出；若需在模型页试听，应复用同一个解锁/播放引擎并由显式试听按钮激活，暂停/seek不补播经过的全部声音；不得启用游戏pointer-lock依赖。原生依旧空操作，不新增音频平台依赖。

## Files found / References / Related specs

- `/home/asuka/kiva-asset-research/naraka/identity/{appres_index.py,dbbl_tables.py}`：本机资源索引/已验证的版本限定表解码。
- `/home/asuka/kiva-asset-research/naraka/containers/naraka_sparse.py`：按原CAB对象与压缩块范围读取。
- `/home/asuka/kiva-asset-research/choulingke/wwiser_wparser_reference.py.txt`：之前保存的 bnnm/wwiser 原实现参考，`CAkBankMgr__LoadSource`、`CAkAction__SetInitialValues`、`CAkEvent__SetInitialValues`、`CAkRanSeqCntr__SetInitialValues`、`CAkSwitchCntr__SetInitialValues`；本次不下载第三方二进制、不执行游戏。
- `.trellis/spec/client/index.md:44`、`:118`：可信解锁、并发/暂停/音源归属；`.trellis/spec/client/combat-presentation.md`：客户端仅消费已结算事实；`.trellis/spec/client/model-viewer.md`：检查页独立App与共享表现。
- [audio-prefab-source.json](audio-prefab-source.json)：额外核对原横刀light01/shock04打击prefab只有PlayerColliderCell/CollideCellAgent，`HitEventID`为空，不能从这里补造成功音频身份。

## Caveats / Not Found

- 未恢复 proto 原时间单位和组语义、impact强弱/护甲switch名、21种地表switch的具体名称和完整 RTPC/衰减/总线；首版选默认分支或Kiva规则必须如实标注。
- 未逐一主观听完全部随机变体；17事件的首变体已真实完整解码/测量/转换，其它变体只有源链和字节哈希。实施验收应试听最终选入的全部文件并在浏览器验证解码。
- `gman_dying_voice_male` 是原通用男性死亡声音，不能称李火旺专属配音。当前BGM、妹妹远程音效和敌人原音效不应随本次横刀导入器被覆盖。
- 完整临时源 payload/debug留在本机研究缓存即可，不应随任务文档提交几十MB原配置；保留精简证据、工具及小试听样本。待root搬迁：`proto_track_archive.bytes.payload.bin`、`audio-proto-track-source.json`、`audio_probe.json`、`audio-tracks-selected-debug.json`、`audio-configs-source.json`、`statesetsoundconfig.asset.probe.json`。这些文件在本轮仍供其它研究复用，未擅自删除。
