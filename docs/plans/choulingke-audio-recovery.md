# 愁铃客原声音频接入

承接已完成的画面修正，用户要求继续解决原版声音，并建议尝试REA。不创建Trellis任务，游戏目录保持只读。

## 可交付链路

原关卡脚本的bg_sound_no=5050619/5050620已确证，sound_data到Wwise事件映射仍受HD/v2 codec6字典阻塞。本轮不再把此映射作为所有声音接入的前置条件：网易云音乐官方艺人“燕云十六声”发行的《江南问情》明确包含“镇守_大雾幽篁_愁铃客_惊惧一夜”（song3429659434，78.666秒）和“第十三声至”（song3429659386，53.793秒）。从公开免费播放接口核对艺人、专辑、原文件大小/MD5和SHA后，使用前者作为Kiva的愁铃客BGM，离线转Ogg随包；发行版本不能冒称为游戏内原始WEM、精确原循环区间或原两阶段切换。第二首只作为身份调查参考，不随包加载未使用资源。

播放器继续由client/audio.rs只读消费CombatFeedback、同步SimulationActive，浏览器可信点击解锁、最多一条音乐及四条短音效，暂停保留音乐位置，恢复不补播旧短音效。原生仍为空操作；不得为声音反向驱动模拟。攻击已采用已验证的原作长兵器挥声作为Kiva共享长刀适配；命中/成功卸势仍为自制占位，只有取得明确源事件链才替换，不能把语音/通用落地声改名。

## 并行调查与边界

- Sounds/SoundbanksInfo.bin：严格解析完整.MESSIAH v1文件，158379条bank记录及189249媒体目录记录；事件/媒体引用可用，但原始事件名已剥离。
- 325个已定位MusicTrack媒体与两首官方OST做有界时间偏移谱特征对比，另从73605个global媒体ID定位并解析61141个WEM头，选30–180秒的1713个媒体全量解码对比，0解码错误。当前无足以确证的匹配；正控制（同一官方曲目转码）30秒余弦相似度.996857，候选最高.630028。不能据此排除分轨/编排/剪辑差异，普通Sound也可能承载音乐，不能因HIRC类型排除。
- 继续从已确认的原动作Cue、完整媒体目录和普通Sound追查原挥刀/命中/卸势。
- 用户建议的REA已在项目外本地安装并运行静态objdump/PE导入检查。主EXE常规代码段raw size为0、高熵保护段，未定位HD代码；不把安装工具本身称为解码成功。HD字典候选必须通过真实Zstd解压，不能用猜测字典填数据。

## 验收

资源清单记录发行来源、来源核验、原/随包哈希、转码命令与适配边界，离线导入脚本可重跑且不改游戏原件。通过完整Ogg解码、实际WebAudio解码/播放/暂停/恢复验证；保留短音效一次消费/声部释放/并发上限测试。make check、WASM Clippy与临时目录Trunk构建按实际代码变化执行，不启动或接管用户常驻服务。


## 已验证的原挥击链路

原common骨架 `skill_02_01_cq__shuangqiangc` 注册的Cue明确命名 `char_skill_qiang_end_swing_fast_1p`；完整VFS键、bankID、Event→Action→RandomContainer→三个Sound及SoundbanksInfo媒体集合相互吻合。三条fast媒体均为48kHz立体声、.71525秒，按确定性选择source145056080；不恢复原随机权重/RTPC/空间混音。作为Kiva当前长刀的共享挥声适配，不能称为愁铃客专属或成功卸势碰撞声。

所选完整PCM原峰值约.207916，导入时只施加恒定线性增益至.85，再转Vorbis；保留原时间、音高与动态，不截取别的素材。技术核验为事件身份链、样本数、起音/峰值及完整解码，当前工具不支持音频输入，未宣称主观试听已完成。实际WebAudio能播放的验证与人耳听感判断分开记录。


## 最终实现验收

- `make check`再次通过：188 Rust库测试、9物理集成测试、49 Node测试；现有audio桥14个专项测试全部通过，Rust/JS运行时接口未变。
- 临时目录Trunk构建通过，Cargo.lock没有变化；游戏真实WebGPU运行无JS/shader错误。
- `audio_reference/browser-full-game-audio.json`：实际游戏进入演武后78.666667秒BGM解码/循环；用户挥刀的权威反馈调用attack并实际启动原音频BufferSource；退出鼠标捕获后音乐/短声全部停止，重新进入仅一条音乐从.928798秒续播。
- `audio_reference/browser-real-audio.json`另验AudioContext挂起/可信手势恢复、缓存和短音效停止。原始WEM完整解成34332帧；本次Vorbis输出granule时长仍.71525秒，FFmpeg浮点导出34204帧（少128帧），Chrome44100Hz解码为31424帧/.712562秒；这是实际编码/解码首包priming观测，未人为裁源，不宣称解码样本数完全一致。
- 官方BGM/原挥声导入器重跑产物哈希稳定，坏艺人metadata、坏attack哈希均在写入前被拒绝。两个声音生成器在外部staging按 importer→generator→generator→importer 顺序复跑，原attack与其来源保持不变，程序hit/deflect音频PCM不变。

已交付官方愁铃客发行BGM和原作共享挥声。原长刀专属挥声、命中、成功卸势及bg_sound_no/原分轨/阶段映射仍未确证；TODO父项因此继续未完成，不能把现有可发声与原音频全集恢复混为一谈。


## 后续静态调查收束

同游戏已核EZST key的156个AES/RC4/ChaCha小集合检验均未匹配已知字典前8字节流，已知向量控制通过，无候选被当成字典成功；复现见项目外 `audio_hd_followup/fixed_key_results.md`。

另沿官网HTML→官方NieDownload模块→网易CDN的真实安卓下载链，使用HTTP Range读取ZIP中央目录与小成员，未下载约1.99GB整包、未运行游戏。原包身份为release-v1.15.1.3 / r64464 / publish_mobile、2026-10-26；不能将其直接视为PC相同配置版本。libGame.so首4MiB压缩片段可解出约21MB AArch64 ELF前缀，但入口指向追加R-X段；原包配置明确启用so保护等机制，前缀没有可定位的HD/Zstd实现。故没有继续下载54MB完整成员或安装Ghidra盲目分析受保护内容。Resources.mpkinfo的34044条索引也未匹配PC已知HD manifest/data/index或sound_data精确键；未取得直接sound_data地址。

官方来源、Range账本、CRC/版本/ELF前缀证据在项目外 `audio_mobile_followup/`，REA实际CLI/静态结论在 `../rea-research/REPORT.md`。这些结果说明本轮静态调查尚未恢复字典和专属声音配置，不证明它们不可恢复，也不能用工具安装成功代替解码成功。交付范围仍以本方案上述真实播放与来源记录为准。
