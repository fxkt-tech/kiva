# 实施边界

用户已批准父方案与四子任务。本切片负责从原PCK读取已确认事件，生成可复核原WEM/无损PCM准备包，再确定性转换为运行时Ogg及分层cue目录。

最小行为缺口在离线资源层：当前游戏只有旧共享attack/hit/deflect文件，没有新横刀三段和通用运动音源。本任务不修改模拟、客户端或Web桥；这些由同伴接入。

预计修改：scripts/naraka_audio.py（有界格式/媒体工具）、scripts/import-naraka-audio.py（提取/打包/校验入口）、必要解析/损坏回归，assets/prepared/naraka/audio（冻结原件/PCM/证据）与assets/audio/naraka-*（新声音和目录）。旧音频文件、原安装和外部研究工具只读。

已补充原事实：move_jump_land第一个PlayAction有v135属性0x0F=100，即100ms DelayTime；第二层无延迟。运行时cue的layers.delay明确保留该延迟，与尚未恢复单位的proto轨time分开。已通知主会话。普通verify不需安装和vgmstream；redecode才从原WEM重跑指定decoder。
