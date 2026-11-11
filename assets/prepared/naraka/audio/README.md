# 永劫无间音效冻结准备包

原件来自用户本机NARAKA BLADEPOINT，版本由PCK目录/已选bank/WEM与原配置SHA固定。
source-evidence保留原动作到声音轨、中文引用配置、原表重解验证与原HIRC媒体链。
banks只保留实际采用事件经过的原HIRC对象，并保留完整对象ID/type索引验证引用；
sources是未改写的完整Wwise Vorbis WEM；pcm是完整PCM16LE的gzip无损副本。

无归一化、裁剪、拼接、变速或变调；运行时gain只提供固定峰值余量。
每层独立选原变体，保留所有选中分支的随机权重。地表/impact选原default；
未恢复RTPC、原random模式、完整混响总线或proto轨time单位，不冒称完整Wwise复刻。
落地有两个原PlayAction；第一层有真实100ms DelayTime，第二层0ms。
该bank延迟与未知单位的proto轨float是两种不同数据。
远程新增原FK射击与弓射击事件。FK标准分支与Barrett条件分支区分，
不将未知条件数字称皮肤ID，不把无Sound轨的sniper片段说成已恢复声音连接。
枪/弓cue均为武器家族适配，不是引冰落/羿之影或角色专属音效。
成功闪避使用用户选定的move_soul_dodge_1p，保留三个原变体；这是Kiva用途适配，非原作PVP触发语义的确证。

普通import/--verify从本包无损PCM重建Ogg并验证完整媒体链、哈希与格式，
不需要客户端、Unity或vgmstream。需要Python3.11+和ffmpeg/ffprobe。
--redecode另从原WEM调用指定、固定SHA的vgmstream，核对PCM样本；
这不等于普通verify，也不恢复原Wwise运行时。

运行时只复制assets/audio/naraka-*，此准备包不编入游戏/WASM。
原attack/hit/deflect和BGM不属于本导入器，不会被写入。
