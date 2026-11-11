# 永劫无间动作音效接入与验收设计

双来源库+SourceRig的有限泛化，不改模拟依赖。RigPose读取stage/DefenseSnapshot；映射表独占clip/裁剪/持势/原音源时点，独立观察actualsim cue给WebAudio。共享avatar binding和手部解剖校准服务游戏与viewer，禁止查看页额外魔法偏移。

## 边界与兼容

src/client/source_animation.rs、animation.rs/preview.rs、avatar.rs/hand.rs、audio.rs、viewer模块与必要controls；web/audio.js及对应Node测试；不拥有离线脚本/模拟规则。

必须先有动作任务已验证的binary/骨架/映射、音效任务cue/媒体/时点和三段任务公开快照/事件。可以先评审接口，不能用假资产完成验收。

完整数据流、三段/防御/音效及回滚约定以父任务design.md为准。离线源路径不得进入运行时，当前请求不包含提交与推送。未知原始语义不能补造。
