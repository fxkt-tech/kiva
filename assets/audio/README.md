# 战斗声音

当前随包的 `attack.ogg`、`hit.ogg`、`deflect.ogg` 是 Kiva 自制的程序音效，分别跟随实际挥刀、受击和卸势反馈；原有枪声继续由 `web/audio.js` 合成。它们没有冒充燕云原始音效。来源与哈希见 `effects-manifest.json`，重建命令：

```sh
python3 scripts/generate-combat-audio.py
```

重建依赖 Python 标准库和 FFmpeg/libvorbis，不是游戏运行时依赖。Trunk 的 copy-dir 将整个目录复制到同源 `audio/`。

**愁铃客镇守原版 BGM 尚未接入。** 已从原关卡脚本确认 `bg_sound_no=5050619/5050620`，但这些配置编号与音频 bank 的准确映射未恢复。不能用尾名相似或时长相近的文件代替。`music.json` 的 `file` 因此保持 `null`；浏览器跳过音乐，不请求缺失文件。

音乐循环接口已具备：确认曲目并离线转成 Ogg 后，将随包文件名填入 `music.json.file`。只接受本目录简单 `.ogg` 文件名，可信进入操作后才解码，最多一条循环，暂停保存位置并释放节点。当前没有宣称已验证真实镇守 BGM 播放。

只读调查原件和证据保存在：

`/home/asuka/kiva-asset-research/choulingke/`

- `audio_research_report.md`：调查结论和未解决链路。
- `lua_audio.json`：关卡音频编号；`lua_names.json` / `audio_all_lua_event_names.json`：61,365个 Lua 的完整扫描与已证实事件。
- `audio_music_banks.json` / `audio_music_sources.json`：447个音乐 bank、412个不同 MusicTrack sourceID，其中325个定位到本地 WEM。
- `audio_music_duration_validation.json`：367个已定位剪辑的时长与 WEM 在1ms内相符。
- `extracted/audio/`：4个已解码的同名候选及哈希；含大量静音，未确认是目标音乐，未放入游戏。

下一步的具体障碍是 `HD/v2` 中 `sound_data` 的 codec6 字典变换；原版地图实例布局也依赖该配置库。容器边界、压缩帧和原始资源都已经保留，后续可从该环节继续。

本轮继续用原骨架Cue核对音效：129项注册中有11个声音Cue，包含`3200370/3200371`配置编号，以及通用蹲伏/落地事件；这未建立愁铃客挥刀/卸势和BGM的准确链路，不能将通用落地音改名冒充目标音效。已核对codec6容器边界并尝试有限的字典头/流变换，实际Zstd验证仍报Dictionary corrupted；没有将未解码数据接入运行时。该缺口也阻碍原技能槽与完整地图配置的恢复。
