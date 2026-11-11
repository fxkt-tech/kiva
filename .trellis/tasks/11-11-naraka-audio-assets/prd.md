# 永劫无间音效离线提取

## 目标

恢复横刀挥击/碰撞及通用运动的原音效事件到媒体链，交付少量可播放同源Ogg及可重复校验清单。

## 来源和依赖

父任务11-11-naraka-action-set/prd.md是需求源；具体产品取舍见父design.md。

依赖父任务audio-assets与动作轨道语义研究；可以与动作转换/三段模拟并行。表现任务依赖本任务最终cue目录。

## 验收标准

- 三段横击有原动作轨→Wwise event→PlayAction→Random/Sound→DIDX/WEM证明。
- 振刀/防御成功音与反击挥刀音分清；共享金属事件如被采用明确标为原共享适配。
- 起跳/落地/闪避/脚步/命中逐项记录媒体、switch条件及未恢复范围。
- 转码保留自然音高，记录原/目标SHA、时长、声道、采样率、增益和全部采用变体；默认不整包解出。
- --verify独立于原安装，精简试听样本可播且转码可重复；不覆盖BGM、燕云原attack或既有程序声生成器拥有的文件。

## 范围

新音效离线脚本，assets/prepared/naraka/音频准备包，assets/audio/naraka-*及manifest；不改web/audio.js或client/audio.rs。

父任务整体方案及四子任务已获用户明确批准，按依赖启动实施，不重复请求规划批准。
