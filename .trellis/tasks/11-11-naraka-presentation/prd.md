# 永劫无间动作音效接入与验收

## 目标

将已验证动作/音效和权威阶段接入真实游戏及共享模型检查路径，保护刚修复的手掌/手指/武器握持。

## 来源和依赖

父任务11-11-naraka-action-set/prd.md是需求源；具体产品取舍见父design.md。

必须先有动作任务已验证的binary/骨架/映射、音效任务cue/媒体/时点和三段任务公开快照/事件。可以先评审接口，不能用假资产完成验收。

## 验收标准

- 哥哥三段及右键映射使用真实片段；三段由权威clock驱动，弹反0.18秒判定与0.60秒可打断表现手势分开；Boss燕云、妹妹Q/Tab和实际握点保持。
- SourcePose/clip显式rig；不同拓扑不混合裸数组；各目标每来源binding缓存复用真实rest/IBM。
- 两来源×三外观实际臂/指/掌框与武器方向回归；角色切换不重建多份Mesh/Image或改变物理。
- 真实音效按起手/源轨偏移/命中/防御/运动cue去重；挥空无impact，无成功结算无成功声；4声部优先级/峰值、BGM及失锁丢弃不补播有效。
- 查看页同一frame/皮肤/挂点，支持新游戏场景/源目录和trace；保持独立静音App，seek不发音。
- make check/WASM Clippy/临时Trunk与有界面浏览器输入/渲染通过；硬件未验收如实记录。

## 范围

src/client/source_animation.rs、animation.rs/preview.rs、avatar.rs/hand.rs、audio.rs、viewer模块与必要controls；web/audio.js及对应Node测试；不拥有离线脚本/模拟规则。

父任务整体方案及四子任务已获用户明确批准，按依赖启动实施，不重复请求规划批准。
