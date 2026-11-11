# 永劫无间音效离线提取设计

稀疏读取PCK目录/选定bank与媒体；严格解析版本化HIRC/事件action/音源容器，解析随机/switch条件而不是按序号猜。保留source manifest和原选中媒体作为冻结准备包，ffmpeg只导出选定Ogg。客户端依赖稳定语义cue，不暴露原数值ID给产品UI。

## 边界与兼容

新音效离线脚本，assets/prepared/naraka/音频准备包，assets/audio/naraka-*及manifest；不改web/audio.js或client/audio.rs。

依赖父任务audio-assets与动作轨道语义研究；可以与动作转换/三段模拟并行。表现任务依赖本任务最终cue目录。

完整数据流、三段/防御/音效及回滚约定以父任务design.md为准。离线源路径不得进入运行时，当前请求不包含提交与推送。未知原始语义不能补造。
