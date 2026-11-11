# 音效离线交付

已完成15个原事件→16个Kiva cue，48个独立原WEM全部实际解码，完整PCM无损冻结；48个Vorbis/Ogg合计945182B。准备包约8.21MB。原42个媒体逐字节保留，另增枪/弓6个原媒体。原attack/hit/deflect、BGM及原清单与HEAD逐字节一致；未写客户端、Web、模拟或原安装。

## 文件

- scripts/naraka_audio.py：有界PCK/135HIRC/Play/Random/default-Switch/嵌入Vorbis读取、媒体布局、完整PCM与确定性编码、准备包路径/哈希校验。
- scripts/import-naraka-audio.py：只读原安装prepare、冻结PCM→运行时import、完整--verify及可选--redecode。
- scripts/test-naraka-audio.py：10个真实源身份、两层延迟、非法属性、容器循环、PCK布局、RIFF/PCM损坏和路径边界回归。
- assets/prepared/naraka/audio：原WEM、gzip无损PCM、选中原HIRC对象/完整对象索引、原轨/配置证据与准备包清单；目录内* -text防换行转换破坏SHA。
- assets/audio/naraka-<source_id>.ogg、naraka-cues.json、naraka-manifest.json：同源运行时媒体、稳定语义目录与来源/信号/适配记录。root已为该prefix添加Git属性保护。

## 使用

普通导入：

    python3 scripts/import-naraka-audio.py --source assets/prepared/naraka/audio

普通离线验证（本轮已执行，并用审计hook禁止任何/mnt/c和vgmstream读取/调用，仍通过）：

    python3 scripts/import-naraka-audio.py --verify assets/prepared/naraka/audio
    python3 scripts/test-naraka-audio.py

重提原件（需要原安装和固定SHA的vgmstream；可用已冻结证据作为版本限定recipe）：

    python3 scripts/import-naraka-audio.py --prepare '<NARAKA BLADEPOINT安装目录>' --evidence assets/prepared/naraka/audio/source-evidence --decoder '<vgmstream-cli>' --prepared-output '<新准备包目录>'

从原WEM再次求解PCM的独立检查：

    python3 scripts/import-naraka-audio.py --verify assets/prepared/naraka/audio --redecode --decoder '<vgmstream-cli>'

普通verify只需Python3.11+、ffmpeg/ffprobe，不需要客户端/Unity/vgmstream；按记录的FFmpeg版本和参数重建。完整PCM与Ogg没有裁剪、归一化、拼接或变调。prepare阶段48原媒体均已解码；本轮没有再次执行--redecode重复全部解码。

## 稳定运行时合同

naraka-cues.json版本1，cues为对象。键：naraka_attack_1/2/3、naraka_riposte、naraka_hit、naraka_deflect、naraka_block、naraka_parry、naraka_jump、naraka_land、naraka_dodge、naraka_step、naraka_death、naraka_death_voice、naraka_gun、naraka_bow。

每项为{priority,offset,layers}。priority越大越重要（脚步0、运动1、攻击/命中2、防御成功3），offset当前0，明确是Kiva适配，不能把原proto未知单位float当秒。layers每项为{delay,variants}，delay单位秒；variants每项为{file,weight,gain}。weight为原随机分支权重正规化后的相对权重；每层选择一个变体，Kiva weighted选择不冒称原Wwise random/sequence模式。文件名只允许naraka-数字.ogg。

三段挥击原变体2/3/3。落地保留两个独立PlayAction，默认分支各8个Sound、实际共用同一组8个WEM：第一层原0x0F DelayTime=100ms→delay0.1，第二层0。必须按两个声源播放并分别计入4声部；未来start节点也计入预算，暂停stop取消，不能setTimeout跨暂停补播。

gain=min(1,0.85/完整Vorbis浮点解码peak)，只在播放时应用，源PCM/Ogg不改。最小约0.750，最大原Vorbis浮点peak约1.133；按现有全局effect gain0.18应用一次时，4声部effect信号最坏上界0.612（不含BGM）。不要再次重复乘全局gain或忽略variant gain。

naraka_deflect/block复用原“拼架刀”共享事件，naraka_parry使用原横刀轨实际引用的共享katana金属声，均为Kiva结算映射；空振不发成功金属声。naraka_riposte为原振刀反击挥刀，不能当成功碰撞。naraka_death_voice是可选原通用男性死亡声音，非李火旺专属，不能用于妹妹；naraka_death本身只用原共享倒地/drop声。脚步按真实接地位移，不能输入held每帧播放。

## 校验与限制

validation.json记录本轮真实检查。15个采用事件的首变体WAV SHA/帧数/声道/采样率还与此前独立研究全部一致。完整portable verify已从冻结PCM重新编码48个Ogg并逐字节核对目录/manifest。

原proto轨time单位/组语义、材质/护甲switch名称、RTPC/原bus/混响与random模式尚未恢复，采用default和Kiva规则均明示。浏览器AudioContext解码、实际游戏生命周期和主观听感由主会话集成验收；本切片未冒称已通过。新声音不影响现有BGM或妹妹原程序枪声。

## 远程补齐（对应音效要求）

原proto_track直接证实male_bow_attack_01→gman_bow_attack_01；male_fk_attack_01的条件声音轨同时列gman_fk_attack_01_barrett与标准gman_fk_attack_01。选标准事件作Kiva鸟铳武器族音效，未把未知条件数字称皮肤ID。原引冰落WeaponSkin同条原记录含“鸟铳”、flintlock presentation及FK UI字段；源动作同伴进一步证实female_fk_weapon_player和male_fk_weapon_player均attack_501→male_fk_attack_01，字段2/3同名（其独立身份报告归动作子任务）。

所查male_sniper普通/ADS和bow aim entry没有Sound轨，female_sniper同名key未在该archive发现；不能把它们的声音绑定伪称已恢复。FK与弓均为家族原声音适配，不是引冰落/羿之影或角色专属。

naraka_gun/naraka_bow都是priority2、offset0、单层delay0，各保留3个等权原变体；不加一次额外SniperShot发声。新文件：naraka-384264562.ogg、naraka-471162968.ogg、naraka-477445358.ogg、naraka-585387678.ogg、naraka-771871815.ogg、naraka-938553406.ogg。主会话只需补这6件浏览器解码和真实Shot分流检查，无需重跑已通过的42件浏览器解码。

当前48媒体完整portable verify已通过，审计同样禁止原安装/vgmstream访问，10个定向测试通过。源研究入口research/probe-ranged-audio.py可复核轨道、WwiseBanks→PCK/HIRC链及两族首变体实际解码；ranged-source-evidence供本版prepare使用，冻结准备包已包含相同证据。
