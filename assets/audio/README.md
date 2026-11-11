# 战斗声音

当前哥哥普通战斗、运动与妹妹枪弓使用永劫原音效：16个cue、48个Ogg，保留三段挥击变体、真实防御成功事件、落地双层与原100ms延迟。完整Event→Action→容器→媒体来源、SHA和增益见[naraka-manifest.json](naraka-manifest.json)，运行时配置见[naraka-cues.json](naraka-cues.json)，冻结输入与离线复核见[准备包](../prepared/naraka/audio/README.md)。声源自然音高，variant峰值增益再乘全局0.18；四短声部按优先级整组预留。空挥没有命中声，空右键没有成功声。查看页静音，游戏需可信点击解锁；未就绪／暂停的声音丢弃，恢复不补播。

```sh
python3 scripts/import-naraka-audio.py --verify assets/prepared/naraka/audio
python3 scripts/test-naraka-audio.py
```

原鸟铳与弓媒体按实际Shot（actor=Companion）的不可变pair播放，伴随狙击特效事件不重复。原Wwise总线／完整switch混音和原动作音轨时间单位未完全恢复，未证明的起手时点采用明确Kiva偏移0；不能把这些适配称为完整原控制器。通用男性死亡语音只保留可选cue，不自动加到所有角色。

以下记录燕云／旧兼容声音与BGM，相关文件仍保留。

`attack.ogg` 已替换为燕云原作的真实长兵器挥声，作为Kiva当前长刀的共享适配。原common骨架的Cue → `char_skill_qiang_end_swing_fast_1p` → Event/Action/RandomContainer → Sound → `Sounds/Media/145056080.wem` 已逐段核对；来源偏移与SHA见 [effects-manifest.json](effects-manifest.json)。选择一条完整原变体，保留48kHz立体声/.71525秒源录音，仅施加恒定增益使源PCM峰值到.85，未人为截音或改变音高/速度。不称为愁铃客专属刀声，也没有恢复原随机变体、Cue时间或Wwise混音。

```sh
python3 scripts/import-choulingke-effects.py \
  --source /home/asuka/kiva-asset-research/choulingke \
  --vgmstream /home/asuka/kiva-asset-research/vgmstream-r2168/vgmstream-cli
```

`hit.ogg`、`deflect.ogg` 仍为 Kiva 自制程序音效；原版命中和成功卸势音的准确身份尚未恢复。旧程序枪声在 `web/audio.js` 保留兼容入口，实际妹妹枪弓已使用上述永劫媒体。旧三类短音效仅用于未迁移的敌人／技能兼容反馈，来源逐项记录，不把共用挥声或程序声混称为愁铃客原声全集。

```sh
python3 scripts/generate-combat-audio.py
```

该生成器现在只重建自身负责的hit/deflect，并核对、保留已导入attack及来源记录，不会将原挥声覆盖回程序占位。Python标准库、FFmpeg/libvorbis和vgmstream仅用于离线导入；运行时Trunk复制同源 `audio/`，没有新增原生音频依赖。

已随包接入官方原声《镇守_大雾幽篁_愁铃客_惊惧一夜》（78.666667秒），由官方艺人“燕云十六声”发行于《江南问情》。来源为[官方曲目](https://music.163.com/song?id=3429659434)的公开完整发行版，核对艺人、专辑、源文件大小/MD5/SHA后离线转成 `choulingke-bgm.ogg`；身份和转码记录见 [music-manifest.json](music-manifest.json)。不在游戏运行时连接音乐站点。

```sh
python3 scripts/import-choulingke-music.py \
  --source /home/asuka/kiva-asset-research/choulingke/audio_reference
```

导入要求本地原MP3及保存的官方metadata，并校验固定SHA；FFmpeg/libvorbis仅用于离线转码。播放器可信点击解锁，音乐至多一条循环，暂停保存位置并释放节点。真实浏览器已核对完整解码、播放、暂停/恢复及AudioContext挂起后的手势恢复。

当前循环的是完整发行录音。原关卡 `bg_sound_no=5050619/5050620` 与本地音频bank的准确映射、原循环区间、分轨及两阶段切换尚未恢复；不把发行版称为提取出的WEM。普通Sound对象也可能播放音乐，不能仅因bank没有MusicTrack便排除其BGM身份。调查方案和边界见[音频恢复](../../docs/plans/choulingke-audio-recovery.md)。

只读调查原件和证据保存在：

`/home/asuka/kiva-asset-research/choulingke/`

- `audio_research_report.md`：调查结论和未解决链路。
- `lua_audio.json`：关卡音频编号；`lua_names.json` / `audio_all_lua_event_names.json`：61,365个 Lua 的完整扫描与已证实事件。
- `audio_music_banks.json` / `audio_music_sources.json`：447个音乐 bank、412个不同 MusicTrack sourceID，其中325个定位到本地 WEM。
- `audio_music_duration_validation.json`：367个已定位剪辑的时长与 WEM 在1ms内相符。
- `extracted/audio/`：4个已解码的同名候选及哈希；含大量静音，未确认是目标音乐，未放入游戏。

本地原事件配置的具体障碍是 `HD/v2` 中 `sound_data` 的 codec6 字典变换；原版地图实例布局也依赖该配置库。容器边界、压缩帧和原始资源都已经保留，后续可从该环节继续。

上一轮用愁铃客骨架Cue核对音效：129项注册中有11个声音Cue，包含`3200370/3200371`配置编号，以及通用蹲伏/落地事件；这未建立愁铃客挥刀/卸势和BGM的准确链路，不能将通用落地音改名冒充目标音效。已核对codec6容器边界并尝试有限的字典头/流变换，实际Zstd验证仍报Dictionary corrupted；没有将未解码数据接入运行时。该缺口也阻碍原技能槽与完整地图配置的恢复。

本轮新增调查/验证文件在同一研究目录：

- `audio_followup/mesh_soundbanks_info_report.json` / `mesh_soundbanks.sqlite`：完整158379 bank、158393 event及189249媒体目录记录，原名被剥离。
- `audio_followup/mesh_verified_swing_media.json`：6条原挥声的全链、WEM/WAV SHA和信号统计；`audio_fx_followup/pre_swing_manifest.json`记录新增同族，未当成刀/卸势身份。
- `audio_reference/browser-full-game-audio.json` / `browser-real-audio.json`：完整游戏真实挥刀发声、BGM播放、失锁清理/续播及AudioContext挂起恢复。工具不支持音频输入，未宣称人耳主观试听通过。
- `audio_hd_followup/README.md`：53份codec6字典及定向候选真实解压仍未成功。
- 项目外 `../rea-research/REPORT.md`：用户建议的[REA](https://github.com/morluto/rea)已实际运行静态检查，但未从受保护磁盘镜像恢复HD装载逻辑。

官方安卓包的分段读取也未找到可用的明文HD实现；版本/索引/保护证据见研究目录 `audio_mobile_followup/`。该入口未解出专属刀声或成功卸势，TODO保留未完成。
